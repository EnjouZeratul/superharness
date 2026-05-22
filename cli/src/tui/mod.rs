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
pub mod ui;

use crate::agent::{AgentClient, AgentError};
use app::App;
use components::{ChatComponent, InputComponent, StatusComponent, ToolDisplayComponent};

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

    // 添加欢迎消息
    chat.add_message(app::Message {
        role: app::Role::System,
        content: "Welcome to Continuum TUI! Initializing Agent...".to_string(),
    });
    status.set_message_count(chat.message_count());

    // 主循环
    let res = run_app(
        &mut terminal,
        &mut app,
        &mut chat,
        &mut input,
        &mut status,
        &mut tools,
        agent,
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
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    chat: &mut ChatComponent,
    input: &mut InputComponent,
    status: &mut StatusComponent,
    tools: &mut ToolDisplayComponent,
    agent: Arc<RwLock<AgentClient>>,
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

    loop {
        // 绘制界面
        terminal.draw(|f| {
            ui::render(f, app, chat, input, status, tools, show_tools);
        })?;

        // 处理事件（带超时）
        if crossterm_event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = crossterm_event::read()? {
                // 如果正在处理，只处理 Ctrl+C
                if processing {
                    if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                        app.running = false;
                        break;
                    }
                    continue;
                }

                match handle_key_event(key, app, chat, input, status, tools, &mut show_tools) {
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
    /// 清屏
    ClearScreen,
    /// 保存会话
    SaveSession,
    /// 新建会话
    NewSession,
    /// 切换工具面板
    ToggleTools,
}

/// 处理键盘事件
fn handle_key_event(
    key: KeyEvent,
    app: &mut App,
    chat: &mut ChatComponent,
    input: &mut InputComponent,
    status: &mut StatusComponent,
    _tools: &mut ToolDisplayComponent,
    _show_tools: &mut bool,
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

        // Tab: 自动补全（占位）
        (KeyModifiers::NONE, KeyCode::Tab) => Ok(KeyAction::None),

        // Enter: 发送消息（在多行模式下可能需要特殊处理）
        (KeyModifiers::NONE, KeyCode::Enter) => {
            // 如果是空输入，不做任何事
            if input.get_input().is_empty() {
                return Ok(KeyAction::None);
            }

            let content = input.get_input().to_string();
            chat.add_message(app::Message {
                role: app::Role::User,
                content: content.clone(),
            });
            // 添加到历史记录
            input.add_to_history(&content);
            input.clear();
            status.set_message_count(chat.message_count());
            Ok(KeyAction::SendMessage(content))
        }

        // Esc: 取消/清空输入
        (KeyModifiers::NONE, KeyCode::Esc) => {
            input.clear();
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

        // 左箭头: 移动光标左（或单词）
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

        // 上箭头: 向上滚动（聊天）或历史记录（输入）
        (KeyModifiers::NONE, KeyCode::Up) => {
            // 如果输入为空，滚动聊天；否则浏览历史
            if input.get_input().is_empty() {
                chat.scroll_up(1);
            } else {
                input.history_prev();
            }
            Ok(KeyAction::None)
        }

        // 下箭头: 向下滚动（聊天）或历史记录（输入）
        (KeyModifiers::NONE, KeyCode::Down) => {
            // 如果输入为空，滚动聊天；否则浏览历史
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
            Ok(KeyAction::None)
        }

        // 忽略其他组合键
        _ => Ok(KeyAction::None),
    }
}
