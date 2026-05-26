//! TUI 界面模块

use anyhow::Result;
use crossterm::{
    event::{
        self as crossterm_event, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent,
        KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub mod app;
pub mod components;
mod event;
pub mod first_run;
pub mod slash_commands;
pub mod tutorial;
pub mod ui;

use crate::agent::{AgentClient, AgentError};
use app::App;
use components::{ChatComponent, ConfirmationDialog, InputComponent, KeyHintsComponent, PermissionManager, PermissionPopup, StatusComponent, ToolDisplayComponent};
use slash_commands::{CommandParser, CommandResult, ParsedCommand};

/// 运行 TUI 界面
pub fn run() -> Result<()> {
    run_with_session(None)
}

/// 运行 TUI 界面（带指定会话）
pub fn run_with_session(session: Option<String>) -> Result<()> {
    // 设置终端
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 创建应用和 Agent 客户端
    let mut app = App::new();
    let agent = Arc::new(RwLock::new(AgentClient::new()));

    // 如果有指定会话，设置会话 ID
    if let Some(session_id) = session {
        app.set_session_id(session_id);
    }

    // 创建组件
    let mut chat = ChatComponent::new();
    let mut input = InputComponent::new();
    let mut status = StatusComponent::new();
    let mut tools = ToolDisplayComponent::new();
    let mut confirmation = ConfirmationDialog::new();
    let mut permissions = PermissionManager::new();
    let mut permission_popup = PermissionPopup::new();
    let mut key_hints = KeyHintsComponent::new();
    let command_parser = CommandParser::new();

    // 检测首次启动状态
    let mut first_run_state = first_run::FirstRunState::new().unwrap_or_default();

    // 添加欢迎消息（根据首次启动状态定制）
    let welcome_msg = if first_run_state.is_first_run {
        first_run::FirstRunState::get_welcome_message()
    } else {
        "Welcome back! Agent initializing...".to_string()
    };
    chat.add_message(app::Message {
        role: app::Role::System,
        content: welcome_msg,
    });
    status.set_message_count(chat.message_count());

    // 首次启动时自动显示教程提示
    if first_run_state.is_first_run && !first_run_state.tutorial_completed {
        chat.add_message(app::Message {
            role: app::Role::System,
            content: first_run::FirstRunState::get_first_run_hint(),
        });
        status.set_message_count(chat.message_count());
    }

    // 主循环
    let res = run_app(
        &mut terminal,
        &mut app,
        &mut chat,
        &mut input,
        &mut status,
        &mut tools,
        &mut key_hints,
        &mut confirmation,
        &mut permissions,
        &mut permission_popup,
        &command_parser,
        agent,
        &mut first_run_state,
    );

    // 恢复终端
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}

/// 运行应用主循环
#[allow(clippy::too_many_arguments)]
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    chat: &mut ChatComponent,
    input: &mut InputComponent,
    status: &mut StatusComponent,
    tools: &mut ToolDisplayComponent,
    key_hints: &mut KeyHintsComponent,
    confirmation: &mut ConfirmationDialog,
    permissions: &mut PermissionManager,
    permission_popup: &mut PermissionPopup,
    command_parser: &CommandParser,
    agent: Arc<RwLock<AgentClient>>,
    first_run_state: &mut first_run::FirstRunState,
) -> Result<()> {
    // 创建 tokio runtime 用于异步操作
    let rt = tokio::runtime::Runtime::new()?;

    // 初始化 Agent
    let init_result = rt.block_on(async {
        let agent_guard = agent.read().await;
        agent_guard.init_from_config().await
    });

    match init_result {
        Ok(()) => {
            // 获取配置信息
            let config_info = rt.block_on(async {
                let agent_guard = agent.read().await;
                let provider = agent_guard.current_provider().await;
                let config = agent_guard.config_info().await;
                (provider, config)
            });

            chat.add_message(app::Message {
                role: app::Role::System,
                content:
                    "Agent initialized successfully. Press Enter to send a message, Ctrl+C to quit."
                        .to_string(),
            });

            // 设置状态栏信息
            status.set_connected(true);
            status.set_provider(Some(config_info.0));
            // 从 config_info 解析 model
            if config_info.1.contains("Model:") {
                let parts: Vec<&str> = config_info.1.split("Model:").collect();
                if parts.len() > 1 {
                    let model_part = parts[1].split("|").next().unwrap_or("").trim();
                    status.set_model(Some(model_part.to_string()));
                }
            }
        }
        Err(AgentError::ConfigError(msg)) => {
            chat.add_message(app::Message {
                role: app::Role::System,
                content: format!("Configuration error: {}\n\nPlease configure your API key:\n  continuum config add-provider anthropic --key YOUR_KEY\nOr set environment variable:\n  export CONTINUUM_API_KEY=YOUR_KEY", msg),
            });
            status.set_connected(false);
        }
        Err(e) => {
            chat.add_message(app::Message {
                role: app::Role::System,
                content: format!("Agent initialization failed: {}", e),
            });
            status.set_connected(false);
        }
    }
    status.set_message_count(chat.message_count());

    // 用于跟踪是否有正在进行的请求
    let mut processing = false;
    // 是否显示工具面板
    let mut show_tools = false;
    // 是否显示命令补全
    let mut show_completions = false;
    // 是否等待首次启动教程响应
    let mut waiting_tutorial_response = first_run_state.is_first_run && !first_run_state.tutorial_completed;

    // 设置初始快捷键提示上下文
    key_hints.set_context(components::HintContext::Normal);

    loop {
        // 更新快捷键提示上下文
        if processing {
            key_hints.set_context(components::HintContext::Processing);
        } else if show_tools {
            key_hints.set_context(components::HintContext::ToolsVisible);
        } else if !input.get_input().is_empty() {
            key_hints.set_context(components::HintContext::Input);
        } else {
            key_hints.set_context(components::HintContext::Normal);
        }

        // 绘制界面
        terminal.draw(|f| {
            ui::render(f, app, chat, input, status, tools, key_hints, show_tools);

            // 渲染权限确认弹窗（如果可见）
            if permission_popup.is_visible() {
                permission_popup.render(f, f.area());
            }
            // 渲染确认对话框（如果可见）
            else if confirmation.is_visible() {
                confirmation.render(f, f.area());
            }
        })?;

        // 处理事件（带超时）
        if crossterm_event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = crossterm_event::read()? {
                // 如果权限确认弹窗可见，优先处理权限弹窗的按键
                if permission_popup.is_visible() {
                    let action = permission_popup.handle_key(key.code);
                    match action {
                        components::PermissionAction::Allow => {
                            // 处理允许操作
                            if let Some(req) = permission_popup.get_request() {
                                let perm_key = PermissionManager::get_permission_key(&req.tool_name, &req.action);
                                permissions.grant_permission(&perm_key, false);
                                chat.add_message(app::Message {
                                    role: app::Role::System,
                                    content: format!("已允许: {} - {}", req.tool_name, req.action),
                                });
                                status.set_message_count(chat.message_count());
                            }
                            permission_popup.hide();
                        }
                        components::PermissionAction::Deny => {
                            // 处理拒绝操作
                            if let Some(req) = permission_popup.get_request() {
                                let perm_key = PermissionManager::get_permission_key(&req.tool_name, &req.action);
                                permissions.deny_permission(&perm_key);
                                chat.add_message(app::Message {
                                    role: app::Role::System,
                                    content: format!("已拒绝: {} - {}", req.tool_name, req.action),
                                });
                                status.set_message_count(chat.message_count());
                            }
                            permission_popup.hide();
                        }
                        components::PermissionAction::AlwaysAllow => {
                            // 处理始终允许操作
                            if let Some(req) = permission_popup.get_request() {
                                let perm_key = PermissionManager::get_permission_key(&req.tool_name, &req.action);
                                permissions.grant_permission(&perm_key, true);
                                chat.add_message(app::Message {
                                    role: app::Role::System,
                                    content: format!("已永久允许: {} - {}", req.tool_name, req.action),
                                });
                                status.set_message_count(chat.message_count());
                            }
                            permission_popup.hide();
                        }
                        components::PermissionAction::None => {}
                    }
                    continue;
                }

                // 如果确认对话框可见，优先处理确认对话框的按键
                if confirmation.is_visible() {
                    let action = confirmation.handle_key(key.code);
                    match action {
                        components::ConfirmAction::Confirmed => {
                            // 执行确认的命令
                            if let Some(cmd) = confirmation.get_pending_command() {
                                let result = execute_slash_command(
                                    cmd.clone(),
                                    chat,
                                    status,
                                    app,
                                    permissions,
                                );
                                handle_command_result(result, chat, status, app);
                            }
                            confirmation.hide();
                        }
                        components::ConfirmAction::Cancelled => {
                            chat.add_message(app::Message {
                                role: app::Role::System,
                                content: "操作已取消".to_string(),
                            });
                            status.set_message_count(chat.message_count());
                            confirmation.hide();
                        }
                        components::ConfirmAction::None => {}
                    }
                    continue;
                }

                // 如果等待教程响应，处理 Y/N
                if waiting_tutorial_response {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            waiting_tutorial_response = false;
                            first_run_state.mark_tutorial_completed().ok();
                            // 启动教程
                            let mut tutorial = tutorial::Tutorial::new();
                            if let Some(step) = tutorial.start() {
                                chat.add_message(app::Message {
                                    role: app::Role::System,
                                    content: tutorial::Tutorial::format_step(step),
                                });
                                status.set_message_count(chat.message_count());
                            }
                            continue;
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            waiting_tutorial_response = false;
                            first_run_state.mark_first_run_done().ok();
                            chat.add_message(app::Message {
                                role: app::Role::System,
                                content: "Tutorial skipped. Type /tutorial anytime to start.".to_string(),
                            });
                            status.set_message_count(chat.message_count());
                            continue;
                        }
                        _ => {}
                    }
                }

                // 如果正在处理，只处理 Ctrl+C
                if processing {
                    if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                        app.running = false;
                        break;
                    }
                    continue;
                }

                match handle_key_event(
                    key,
                    app,
                    chat,
                    input,
                    status,
                    tools,
                    key_hints,
                    &mut show_tools,
                    &mut show_completions,
                    command_parser,
                    confirmation,
                    permissions,
                ) {
                    Ok(action) => {
                        match action {
                            KeyAction::Exit => break,
                            KeyAction::SendMessage(content) => {
                                status.set_processing(true);

                                // 添加工具调用显示（模拟工具执行）
                                let tool_idx = tools.add_tool_call(
                                    "llm_request".to_string(),
                                    serde_json::to_string(&serde_json::json!({
                                        "prompt": content,
                                    }))
                                    .unwrap_or_default(),
                                );
                                tools.set_running(tool_idx);

                                // 在后台处理 Agent 调用
                                let agent_clone = agent.clone();
                                let response = rt.block_on(async {
                                    let agent_guard = agent_clone.read().await;
                                    if agent_guard.is_initialized().await {
                                        agent_guard.send_message(&content).await
                                    } else {
                                        Err(AgentError::ConfigError(
                                            "Agent not initialized".to_string(),
                                        ))
                                    }
                                });

                                match response {
                                    Ok(response_content) => {
                                        tools.complete_tool_call(
                                            tool_idx,
                                            "LLM response received".to_string(),
                                            false,
                                        );
                                        chat.add_message(app::Message {
                                            role: app::Role::Assistant,
                                            content: response_content,
                                        });
                                    }
                                    Err(e) => {
                                        tools.complete_tool_call(tool_idx, e.to_string(), true);
                                        chat.add_message(app::Message {
                                            role: app::Role::Assistant,
                                            content: format!("Error: {}", e),
                                        });
                                    }
                                }
                                status.set_message_count(chat.message_count());
                                status.set_processing(false);
                                processing = false;

                                // 自动显示工具面板当有工具调用时
                                show_tools = tools.count() > 0;
                            }
                            KeyAction::ClearScreen => {
                                chat.clear();
                                tools.clear();
                                show_tools = false;
                                status.set_message_count(0);
                            }
                            KeyAction::SaveSession => {
                                // 添加提示消息
                                chat.add_message(app::Message {
                                    role: app::Role::System,
                                    content: "Session saved (placeholder)".to_string(),
                                });
                                status.set_message_count(chat.message_count());
                            }
                            KeyAction::NewSession => {
                                // 清空当前会话并开始新会话
                                chat.clear();
                                tools.clear();
                                show_tools = false;
                                rt.block_on(async {
                                    let agent_guard = agent.read().await;
                                    agent_guard.clear_history().await;
                                });
                                chat.add_message(app::Message {
                                    role: app::Role::System,
                                    content: "New session started".to_string(),
                                });
                                status.set_message_count(chat.message_count());
                                // 生成新的会话 ID
                                let new_session_id = uuid::Uuid::new_v4().to_string();
                                app.set_session_id(new_session_id.clone());
                                status.set_session_id(Some(new_session_id));
                            }
                            KeyAction::ToggleTools => {
                                show_tools = !show_tools;
                            }
                            KeyAction::ShowHelp => {
                                let parser = CommandParser::new();
                                let help_text = parser.generate_help(None);
                                chat.add_message(app::Message {
                                    role: app::Role::System,
                                    content: help_text,
                                });
                                status.set_message_count(chat.message_count());
                            }
                            KeyAction::SlashCommand(cmd) => {
                                let result = execute_slash_command(cmd, chat, status, app, permissions);
                                handle_command_result(result, chat, status, app);
                            }
                            KeyAction::None => {}
                        }
                    }
                    Err(e) => {
                        tracing::error!("Key handling error: {}", e);
                    }
                }
            }
        }

        if !app.running {
            break;
        }
    }

    Ok(())
}

/// 键盘动作
enum KeyAction {
    /// 无动作
    None,
    /// 退出
    Exit,
    /// 发送消息
    SendMessage(String),
    /// 执行 Slash 命令
    SlashCommand(ParsedCommand),
    /// 清屏
    ClearScreen,
    /// 保存会话
    SaveSession,
    /// 新建会话
    NewSession,
    /// 切换工具面板
    ToggleTools,
    /// 显示帮助
    ShowHelp,
}

/// 执行 Slash 命令
fn execute_slash_command(
    cmd: ParsedCommand,
    chat: &mut ChatComponent,
    status: &mut StatusComponent,
    app: &mut App,
    _permissions: &mut PermissionManager,
) -> CommandResult {
    

    match cmd.command.name.as_str() {
        "help" => {
            let parser = CommandParser::new();
            let help_text = parser.generate_help(cmd.args.get("command").map(|s| s.as_str()));
            chat.add_message(app::Message {
                role: app::Role::System,
                content: help_text,
            });
            status.set_message_count(chat.message_count());
            CommandResult::Success("Help displayed".to_string())
        }
        "clear" => {
            chat.clear();
            status.set_message_count(0);
            CommandResult::Success("Screen cleared".to_string())
        }
        "new" => {
            chat.clear();
            status.set_message_count(0);
            let new_session_id = uuid::Uuid::new_v4().to_string();
            app.set_session_id(new_session_id.clone());
            status.set_session_id(Some(new_session_id));
            chat.add_message(app::Message {
                role: app::Role::System,
                content: "New session started".to_string(),
            });
            status.set_message_count(chat.message_count());
            CommandResult::Success("New session".to_string())
        }
        "exit" => {
            app.running = false;
            CommandResult::Exit
        }
        "tokens" => {
            chat.add_message(app::Message {
                role: app::Role::System,
                content: "Token usage statistics (placeholder)".to_string(),
            });
            status.set_message_count(chat.message_count());
            CommandResult::Success("Tokens info".to_string())
        }
        "debug" => {
            chat.add_message(app::Message {
                role: app::Role::System,
                content: "Debug mode toggled (placeholder)".to_string(),
            });
            status.set_message_count(chat.message_count());
            CommandResult::Success("Debug toggled".to_string())
        }
        "config" => {
            let config_info = if let Some(key) = cmd.args.get("key") {
                format!("Config {}: (placeholder)", key)
            } else {
                "Current config: (placeholder)".to_string()
            };
            chat.add_message(app::Message {
                role: app::Role::System,
                content: config_info,
            });
            status.set_message_count(chat.message_count());
            CommandResult::Success("Config shown".to_string())
        }
        "model" => {
            if let Some(model_name) = cmd.args.get("name") {
                chat.add_message(app::Message {
                    role: app::Role::System,
                    content: format!("Model switched to: {} (placeholder)", model_name),
                });
            } else {
                chat.add_message(app::Message {
                    role: app::Role::System,
                    content: "Current model: (placeholder)".to_string(),
                });
            }
            status.set_message_count(chat.message_count());
            CommandResult::Success("Model info".to_string())
        }
        "provider" => {
            if let Some(provider_name) = cmd.args.get("name") {
                chat.add_message(app::Message {
                    role: app::Role::System,
                    content: format!("Provider switched to: {} (placeholder)", provider_name),
                });
            } else {
                chat.add_message(app::Message {
                    role: app::Role::System,
                    content: "Current provider: (placeholder)".to_string(),
                });
            }
            status.set_message_count(chat.message_count());
            CommandResult::Success("Provider info".to_string())
        }
        "tools" => {
            chat.add_message(app::Message {
                role: app::Role::System,
                content: "Available tools: (placeholder - use 'continuum tools' command)".to_string(),
            });
            status.set_message_count(chat.message_count());
            CommandResult::Success("Tools listed".to_string())
        }
        "tutorial" => {
            let tutorial_text = if let Some(step_str) = cmd.args.get("step") {
                if let Ok(step_num) = step_str.parse::<usize>() {
                    let mut tutorial = tutorial::Tutorial::new();
                    if let Some(step) = tutorial.jump_to(step_num) {
                        tutorial::Tutorial::format_step(step)
                    } else {
                        format!("Invalid step number. Use /tutorial 1-5 or /tutorial for overview.")
                    }
                } else {
                    format!("Invalid step number. Use /tutorial 1-5 or /tutorial for overview.")
                }
            } else {
                tutorial::Tutorial::overview()
            };
            chat.add_message(app::Message {
                role: app::Role::System,
                content: tutorial_text,
            });
            status.set_message_count(chat.message_count());
            CommandResult::Success("Tutorial displayed".to_string())
        }
        "bash" | "write" | "edit" | "git" => {
            // 高风险命令需要确认
            CommandResult::NeedsConfirmation {
                command: cmd,
                message: "This operation may modify files or execute commands.".to_string(),
            }
        }
        _ => {
            CommandResult::Error(format!("Unknown command: /{}", cmd.command.name))
        }
    }
}

/// 处理命令执行结果
fn handle_command_result(
    result: CommandResult,
    chat: &mut ChatComponent,
    status: &mut StatusComponent,
    app: &mut App,
) {
    match result {
        CommandResult::Success(msg) => {
            tracing::debug!("Command success: {}", msg);
        }
        CommandResult::NeedsConfirmation { command: _, message } => {
            chat.add_message(app::Message {
                role: app::Role::System,
                content: format!("Waiting for confirmation: {}", message),
            });
            status.set_message_count(chat.message_count());
        }
        CommandResult::Error(msg) => {
            chat.add_message(app::Message {
                role: app::Role::System,
                content: format!("Error: {}", msg),
            });
            status.set_message_count(chat.message_count());
        }
        CommandResult::Exit => {
            app.running = false;
        }
        CommandResult::NoOp => {}
    }
}

/// 处理键盘事件
#[allow(clippy::too_many_arguments)]
fn handle_key_event(
    key: KeyEvent,
    app: &mut App,
    chat: &mut ChatComponent,
    input: &mut InputComponent,
    status: &mut StatusComponent,
    _tools: &mut ToolDisplayComponent,
    key_hints: &mut KeyHintsComponent,
    _show_tools: &mut bool,
    show_completions: &mut bool,
    command_parser: &CommandParser,
    confirmation: &mut ConfirmationDialog,
    permissions: &mut PermissionManager,
) -> Result<KeyAction> {
    match (key.modifiers, key.code) {
        // Ctrl+C: 退出
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            app.running = false;
            Ok(KeyAction::Exit)
        }

        // Ctrl+D: 退出
        (KeyModifiers::CONTROL, KeyCode::Char('d')) => {
            app.running = false;
            Ok(KeyAction::Exit)
        }

        // Ctrl+L: 清屏
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => Ok(KeyAction::ClearScreen),

        // Ctrl+S: 保存会话
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => Ok(KeyAction::SaveSession),

        // Ctrl+N: 新建会话
        (KeyModifiers::CONTROL, KeyCode::Char('n')) => Ok(KeyAction::NewSession),

        // Ctrl+T: 切换工具面板
        (KeyModifiers::CONTROL, KeyCode::Char('t')) => Ok(KeyAction::ToggleTools),

        // Ctrl+? 或 F1: 切换快捷键提示展开
        (KeyModifiers::CONTROL, KeyCode::Char('?')) | (KeyModifiers::NONE, KeyCode::F(1)) => {
            key_hints.toggle_expanded();
            Ok(KeyAction::None)
        }

        // Ctrl+H: 显示帮助
        (KeyModifiers::CONTROL, KeyCode::Char('h')) => Ok(KeyAction::ShowHelp),

        // Ctrl+W: 删除前一个单词
        (KeyModifiers::CONTROL, KeyCode::Char('w')) => {
            input.delete_word();
            Ok(KeyAction::None)
        }

        // Ctrl+A: 移动到行首
        (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
            input.move_cursor_to_start();
            Ok(KeyAction::None)
        }

        // Ctrl+E: 移动到行尾
        (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
            input.move_cursor_to_end();
            Ok(KeyAction::None)
        }

        // Alt+B: 后退一个单词
        (KeyModifiers::ALT, KeyCode::Char('b')) => {
            input.move_word_left();
            Ok(KeyAction::None)
        }

        // Alt+F: 前进一个单词
        (KeyModifiers::ALT, KeyCode::Char('f')) => {
            input.move_word_right();
            Ok(KeyAction::None)
        }

        // Alt+Enter 或 Shift+Enter: 插入换行
        (KeyModifiers::ALT, KeyCode::Enter) | (KeyModifiers::SHIFT, KeyCode::Enter) => {
            input.insert_newline();
            Ok(KeyAction::None)
        }

        // Tab: 命令补全
        (KeyModifiers::NONE, KeyCode::Tab) => {
            let current_input = input.get_input();
            if current_input.starts_with('/') {
                let completions = command_parser.get_completions(current_input);
                if !completions.is_empty() {
                    // 补全到第一个匹配项
                    if let Some(cmd) = completions.first() {
                        input.set_input(format!("/{} ", cmd.name));
                    }
                }
            }
            Ok(KeyAction::None)
        }

        // Enter: 发送消息或执行命令
        (KeyModifiers::NONE, KeyCode::Enter) => {
            if input.get_input().is_empty() {
                return Ok(KeyAction::None);
            }

            let content = input.get_input().to_string();

            // 检查是否是 Slash 命令
            if let Some(parsed) = command_parser.parse(&content) {
                // 添加到历史记录
                input.add_to_history(&content);
                input.clear();

                // 显示用户输入的命令
                chat.add_message(app::Message {
                    role: app::Role::User,
                    content: content.clone(),
                });
                status.set_message_count(chat.message_count());

                // 检查是否需要确认
                if parsed.command.requires_confirmation {
                    let details = vec![
                        format!("命令: /{}", parsed.command.name),
                        format!("描述: {}", parsed.command.description),
                    ];
                    confirmation.show(parsed, "执行命令", details);
                    return Ok(KeyAction::None);
                }

                // 执行命令
                let result = execute_slash_command(parsed, chat, status, app, permissions);
                handle_command_result(result, chat, status, app);
                Ok(KeyAction::None)
            } else {
                // 普通消息
                chat.add_message(app::Message {
                    role: app::Role::User,
                    content: content.clone(),
                });
                input.add_to_history(&content);
                input.clear();
                status.set_message_count(chat.message_count());
                Ok(KeyAction::SendMessage(content))
            }
        }

        // Esc: 取消/清空输入
        (KeyModifiers::NONE, KeyCode::Esc) => {
            input.clear();
            *show_completions = false;
            Ok(KeyAction::None)
        }

        // Backspace: 删除字符
        (KeyModifiers::NONE, KeyCode::Backspace) => {
            input.delete_char();
            Ok(KeyAction::None)
        }

        // Delete: 删除字符
        (KeyModifiers::NONE, KeyCode::Delete) => {
            input.delete_char();
            Ok(KeyAction::None)
        }

        // 左箭头: 移动光标左
        (KeyModifiers::NONE, KeyCode::Left) => {
            input.move_cursor_left();
            Ok(KeyAction::None)
        }

        // 右箭头: 移动光标右
        (KeyModifiers::NONE, KeyCode::Right) => {
            input.move_cursor_right();
            Ok(KeyAction::None)
        }

        // Home: 移动到开始
        (KeyModifiers::NONE, KeyCode::Home) => {
            input.move_cursor_to_start();
            Ok(KeyAction::None)
        }

        // End: 移动到结束
        (KeyModifiers::NONE, KeyCode::End) => {
            input.move_cursor_to_end();
            Ok(KeyAction::None)
        }

        // 上箭头: 向上滚动或历史记录
        (KeyModifiers::NONE, KeyCode::Up) => {
            if input.get_input().is_empty() {
                chat.scroll_up(1);
            } else {
                input.history_prev();
            }
            Ok(KeyAction::None)
        }

        // 下箭头: 向下滚动或历史记录
        (KeyModifiers::NONE, KeyCode::Down) => {
            if input.get_input().is_empty() {
                chat.scroll_down(1);
            } else {
                input.history_next();
            }
            Ok(KeyAction::None)
        }

        // Page Up: 向上滚动一页
        (KeyModifiers::NONE, KeyCode::PageUp) => {
            chat.scroll_up(10);
            Ok(KeyAction::None)
        }

        // Page Down: 向下滚动一页
        (KeyModifiers::NONE, KeyCode::PageDown) => {
            chat.scroll_down(10);
            Ok(KeyAction::None)
        }

        // 普通字符输入
        (KeyModifiers::NONE, KeyCode::Char(c)) => {
            input.push_char(c);

            // 检测是否输入了 / 开头，显示补全
            let current_input = input.get_input();
            *show_completions = current_input.starts_with('/') && current_input.len() > 1;

            Ok(KeyAction::None)
        }

        // 忽略其他组合键
        _ => Ok(KeyAction::None),
    }
}
