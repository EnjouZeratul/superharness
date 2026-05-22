//! 集成测试 - LLM 真实调用 + CLI 端到端 + Git + MCP
//!
//! 运行: cargo test -p sh-layer4 --test integration_llm -- --nocapture
//! 需要 .env.test 文件中的真实 API 密钥（LLM 测试可选）

mod common;

use common::test_config::{get_api_key, get_base_url, get_model, is_api_available, load_env};

use continuum_cli as cli;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

// ===== 1. LLM 真实调用 =====

#[cfg(test)]
mod llm_tests {
    use super::*;

    #[test]
    fn test_env_loaded() {
        load_env();
        let key = get_api_key();
        let url = get_base_url();
        let model = get_model();

        println!("API Key present: {}", key.is_some());
        println!("Base URL: {}", url);
        println!("Model: {}", model);

        assert!(!url.is_empty());
        assert!(!model.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires CONTINUUM_API_KEY in .env.test"]
    async fn test_real_chat_request() {
        load_env();

        use sh_layer1::llm_client::{
            LlmClient, LlmClientTrait, LlmProvider, LlmRequestConfig, Message, MessageRole,
        };

        let api_key = get_api_key().unwrap();
        let base_url = get_base_url();

        // 根据 base_url 推断 provider，但保留自定义 URL
        let provider = if base_url.contains("openai") {
            LlmProvider::OpenAI
        } else if base_url.contains("gemini") || base_url.contains("google") {
            LlmProvider::Gemini
        } else {
            LlmProvider::Anthropic // 使用 Anthropic 格式的 API
        };

        // 创建客户端并用自定义 base_url 覆盖默认值
        let client = LlmClient::new(provider, api_key).with_base_url(base_url);

        let config = LlmRequestConfig {
            model: get_model(),
            max_tokens: 50,
            temperature: 0.0,
            ..Default::default()
        };

        let messages = vec![Message {
            role: MessageRole::User,
            content: "Say exactly 'INTEGRATION_TEST_OK' and nothing else.".to_string(),
        }];

        let result = client.send(messages, &config).await;

        match result {
            Ok(response) => {
                println!("LLM response: {}", response.content);
                assert!(!response.content.is_empty(), "Response should not be empty");
                assert!(response.usage.input_tokens > 0, "Should have input tokens");
                assert!(
                    response.usage.output_tokens > 0,
                    "Should have output tokens"
                );
                // 验证内容包含预期的关键词
                let content_lower = response.content.to_lowercase();
                assert!(
                    content_lower.contains("integration") || content_lower.contains("ok"),
                    "Response should contain 'INTEGRATION' or 'OK', got: {}",
                    response.content
                );
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                if error_msg.contains("not found") || error_msg.contains("404") {
                    panic!(
                        "SKIP: Model or endpoint not found - API endpoint may be incompatible. \
                         Error: {}",
                        e
                    );
                } else {
                    panic!("LLM API error: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires CONTINUUM_API_KEY in .env.test"]
    async fn test_real_tool_call() {
        load_env();

        use sh_layer1::llm_client::{
            LlmClient, LlmClientTrait, LlmProvider, LlmRequestConfig, Message, MessageRole,
        };

        let api_key = get_api_key().unwrap();
        let base_url = get_base_url();

        let provider = if base_url.contains("openai") {
            LlmProvider::OpenAI
        } else if base_url.contains("gemini") || base_url.contains("google") {
            LlmProvider::Gemini
        } else {
            LlmProvider::Anthropic
        };

        let client = LlmClient::new(provider, api_key).with_base_url(base_url);

        let config = LlmRequestConfig {
            model: get_model(),
            max_tokens: 100,
            temperature: 0.0,
            ..Default::default()
        };

        // 模拟工具调用场景：让 LLM 生成一个 bash 命令
        let messages = vec![Message {
            role: MessageRole::User,
            content: "I need to list all .rs files in the current directory. What bash command should I use? Reply with ONLY the command, no explanation.".to_string(),
        }];

        let result = client.send(messages, &config).await;

        match result {
            Ok(response) => {
                println!("LLM tool call response: {}", response.content);
                assert!(
                    !response.content.is_empty(),
                    "Tool call response should not be empty"
                );
                assert!(
                    response.usage.output_tokens > 0,
                    "Should have output tokens"
                );
                let content_lower = response.content.to_lowercase();
                let is_command_like = content_lower.contains("find")
                    || content_lower.contains("ls")
                    || content_lower.contains("dir")
                    || content_lower.contains("glob")
                    || content_lower.contains("*.rs");
                assert!(
                    is_command_like,
                    "Response should suggest a file listing command, got: {}",
                    response.content
                );
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                if error_msg.contains("not found") || error_msg.contains("404") {
                    panic!(
                        "SKIP: Model or endpoint not found - API endpoint may be incompatible. \
                         Error: {}",
                        e
                    );
                } else {
                    panic!("LLM API error: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires CONTINUUM_API_KEY in .env.test"]
    async fn test_real_long_response() {
        load_env();

        use sh_layer1::llm_client::{
            LlmClient, LlmClientTrait, LlmProvider, LlmRequestConfig, Message, MessageRole,
        };

        let api_key = get_api_key().unwrap();
        let base_url = get_base_url();

        let provider = if base_url.contains("openai") {
            LlmProvider::OpenAI
        } else if base_url.contains("gemini") || base_url.contains("google") {
            LlmProvider::Gemini
        } else {
            LlmProvider::Anthropic
        };

        let client = LlmClient::new(provider, api_key).with_base_url(base_url);

        // 请求较长输出以验证流式/长响应处理
        let config = LlmRequestConfig {
            model: get_model(),
            max_tokens: 500,
            temperature: 0.3,
            ..Default::default()
        };

        let messages = vec![Message {
            role: MessageRole::User,
            content: "Explain in 3 short paragraphs: what is the Rust programming language and why is it useful for systems programming?".to_string(),
        }];

        let result = client.send(messages, &config).await;

        match result {
            Ok(response) => {
                println!(
                    "LLM long response ({} chars): {}...",
                    response.content.len(),
                    &response.content[..response.content.len().min(200)]
                );
                assert!(
                    response.content.len() > 100,
                    "Long response should be >100 chars, got {} chars",
                    response.content.len()
                );
                assert!(response.usage.input_tokens > 0, "Should have input tokens");
                assert!(
                    response.usage.output_tokens > 20,
                    "Should have significant output tokens for long response, got {}",
                    response.usage.output_tokens
                );
                // 验证内容包含 Rust 相关关键词
                let content_lower = response.content.to_lowercase();
                assert!(
                    content_lower.contains("rust")
                        || content_lower.contains("memory")
                        || content_lower.contains("safety"),
                    "Long response should discuss Rust/memory/safety, got: {}...",
                    &response.content[..response.content.len().min(100)]
                );
            }
            Err(e) => {
                panic!("LLM API error: {}", e);
            }
        }
    }

    #[tokio::test]
    #[ignore = "requires CONTINUUM_API_KEY in .env.test"]
    async fn test_real_multi_turn() {
        load_env();

        use sh_layer1::llm_client::{
            LlmClient, LlmClientTrait, LlmProvider, LlmRequestConfig, Message, MessageRole,
        };

        let api_key = get_api_key().unwrap();
        let base_url = get_base_url();

        let provider = if base_url.contains("openai") {
            LlmProvider::OpenAI
        } else if base_url.contains("gemini") || base_url.contains("google") {
            LlmProvider::Gemini
        } else {
            LlmProvider::Anthropic
        };

        let client = LlmClient::new(provider, api_key).with_base_url(base_url);

        let config = LlmRequestConfig {
            model: get_model(),
            max_tokens: 50,
            temperature: 0.0,
            ..Default::default()
        };

        // 多轮对话：验证 LLM 记住上下文
        let messages = vec![
            Message {
                role: MessageRole::User,
                content: "My secret code word is 'pineapple'.".to_string(),
            },
            Message {
                role: MessageRole::Assistant,
                content: "Got it, I'll remember your code word.".to_string(),
            },
            Message {
                role: MessageRole::User,
                content: "What is my secret code word? Reply with ONLY the word.".to_string(),
            },
        ];

        let result = client.send(messages, &config).await;

        match result {
            Ok(response) => {
                println!("LLM multi-turn response: {}", response.content);
                assert!(
                    !response.content.is_empty(),
                    "Multi-turn response should not be empty"
                );
                let content_lower = response.content.to_lowercase();
                assert!(
                    content_lower.contains("pineapple"),
                    "LLM should remember the code word 'pineapple' from context, got: {}",
                    response.content
                );
            }
            Err(e) => {
                panic!("LLM API error: {}", e);
            }
        }
    }
}

// ===== 2. CLI 端到端 =====

#[cfg(test)]
mod cli_e2e_tests {
    use super::*;

    #[test]
    fn test_cli_bash_command() {
        let result =
            cli::commands::tool_exec::execute_bash("echo hello_world", None, 10, false).unwrap();
        assert!(result.stdout.contains("hello_world"));
        assert_eq!(result.exit_code, 0);
        assert!(!result.timed_out);

        let result = cli::commands::tool_exec::execute_bash("exit 42", None, 10, false).unwrap();
        assert_eq!(result.exit_code, 42);
    }

    #[test]
    fn test_cli_read_command() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("read_test.txt");

        // 写入多行文件
        let content: Vec<String> = (1..=20).map(|i| format!("Line {}", i)).collect();
        fs::write(&file_path, content.join("\n")).unwrap();

        // 读取完整文件
        let result =
            cli::commands::tool_exec::execute_read(file_path.to_str().unwrap(), None, None, false)
                .unwrap();
        assert!(result.contains("Line 1"));
        assert!(result.contains("Line 20"));

        // 读取部分行
        let result = cli::commands::tool_exec::execute_read(
            file_path.to_str().unwrap(),
            Some(5),
            Some(3),
            false,
        )
        .unwrap();
        assert!(result.contains("Line 6"));
        assert!(!result.contains("Line 1"));

        // 读取不存在的文件
        let result =
            cli::commands::tool_exec::execute_read("/nonexistent/file.txt", None, None, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_write_command() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("write_test.txt");

        // 首次写入
        let result = cli::commands::tool_exec::execute_write(
            file_path.to_str().unwrap(),
            Some("first write"),
            false,
            false,
        )
        .unwrap();
        assert!(result.contains("bytes"));

        // 追加写入
        let result = cli::commands::tool_exec::execute_write(
            file_path.to_str().unwrap(),
            Some("appended"),
            true,
            false,
        )
        .unwrap();
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("first write"));
        assert!(content.contains("appended"));

        // 备份写入
        let result = cli::commands::tool_exec::execute_write(
            file_path.to_str().unwrap(),
            Some("with backup"),
            false,
            true,
        )
        .unwrap();
        let backup_path = dir.path().join("write_test.txt.bak");
        assert!(backup_path.exists());
    }

    #[test]
    fn test_cli_edit_command() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("edit_test.txt");
        fs::write(&file_path, "foo bar foo baz foo").unwrap();

        // 替换第一个
        let result = cli::commands::tool_exec::execute_edit(
            file_path.to_str().unwrap(),
            "foo",
            "QUX",
            false,
        )
        .unwrap();
        assert!(result.contains("1 occurrence"));
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "QUX bar foo baz foo");

        // 替换所有
        let result =
            cli::commands::tool_exec::execute_edit(file_path.to_str().unwrap(), "foo", "QUX", true)
                .unwrap();
        assert!(result.contains("2 occurrence"));
        let content = fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "QUX bar QUX baz QUX");
    }

    #[test]
    fn test_cli_grep_command() {
        let dir = TempDir::new().unwrap();

        fs::write(
            dir.path().join("app.rs"),
            "fn main() {\n    println!(\"hello\");\n}\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("lib.rs"),
            "pub fn greet() {\n    \"hello\"\n}\n",
        )
        .unwrap();

        // 搜索所有文件
        let results = cli::commands::tool_exec::execute_grep(
            "hello",
            dir.path().to_str().unwrap(),
            None,
            false,
            true,
            None,
        )
        .unwrap();
        assert_eq!(results.len(), 2);

        // 用 glob 过滤
        let results = cli::commands::tool_exec::execute_grep(
            "hello",
            dir.path().to_str().unwrap(),
            Some("*.rs"),
            false,
            true,
            None,
        )
        .unwrap();
        assert_eq!(results.len(), 2);

        // 不存在的模式
        let results = cli::commands::tool_exec::execute_grep(
            "nonexistent_pattern_xyz",
            dir.path().to_str().unwrap(),
            None,
            false,
            true,
            None,
        )
        .unwrap();
        assert!(results.is_empty());
    }
}

// ===== 3. Git 集成 =====

#[cfg(test)]
mod git_tests {
    use super::*;
    use cli::git::{branch, commit, diff, status};

    fn init_git_repo(dir: &std::path::Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "--allow-empty", "-m", "init"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn test_git_status_real() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());

        // 创建未跟踪文件
        fs::write(dir.path().join("untracked.txt"), "content").unwrap();

        let status = status::get_status(dir.path()).unwrap();
        assert!(status.has_changes());
        assert!(!status.untracked_files().is_empty());

        let rendered = status.render();
        assert!(!rendered.is_empty());
        assert!(rendered.contains("untracked"));

        // git add 后验证 staged
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        let status = status::get_status(dir.path()).unwrap();
        assert!(!status.staged_files().is_empty());
    }

    #[test]
    fn test_git_diff_real() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());

        // 创建并提交文件
        fs::write(dir.path().join("file.txt"), "initial content\n").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(dir.path())
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "add file"])
            .current_dir(dir.path())
            .output()
            .unwrap();

        // 修改文件
        fs::write(dir.path().join("file.txt"), "modified content\n").unwrap();

        // 工作区 diff
        let diff = diff::get_diff(dir.path(), diff::DiffType::Working, &[]).unwrap();
        assert!(diff.files_changed > 0);
        assert!(!diff.entries.is_empty());
        assert!(diff.total_additions > 0 || diff.total_deletions > 0);

        // stat
        let stat = diff.stat();
        assert!(!stat.is_empty());
        assert!(stat.contains("1 file") || stat.contains("changed"));

        // 暂存区 diff 应为空（未 add）
        let staged_diff = diff::get_diff(dir.path(), diff::DiffType::Staged, &[]).unwrap();
        assert_eq!(staged_diff.files_changed, 0);
    }

    #[test]
    fn test_git_commit_flow() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());

        // 创建文件
        fs::write(dir.path().join("feature.txt"), "new feature\n").unwrap();

        // git add
        commit::add_all(dir.path()).unwrap();

        // 验证 staged
        let status = status::get_status(dir.path()).unwrap();
        assert!(!status.staged_files().is_empty());

        // git commit
        let result = commit::commit(dir.path(), "feat: add feature", false).unwrap();
        assert!(result.contains("feat: add feature") || result.contains("commit"));

        // 提交后应无 staged 文件
        let status = status::get_status(dir.path()).unwrap();
        assert!(status.staged_files().is_empty());

        // 修改并测试 add 指定路径
        fs::write(dir.path().join("feature.txt"), "updated feature\n").unwrap();
        fs::write(dir.path().join("another.txt"), "another file\n").unwrap();
        commit::add(dir.path(), &["feature.txt"]).unwrap();

        let status = status::get_status(dir.path()).unwrap();
        // feature.txt 应 staged，another.txt 应 untracked
        let staged: Vec<_> = status.staged_files().iter().map(|e| &e.path).collect();
        assert!(staged.iter().any(|p| p.contains("feature")));
    }

    #[test]
    fn test_git_branch() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());

        let manager = branch::BranchManager::new(dir.path());

        let current = manager.current().unwrap();
        assert!(!current.is_empty());

        manager.create("test-branch").unwrap();
        let branches = manager.list(false).unwrap();
        assert!(branches.iter().any(|b| b.name == "test-branch"));

        manager.create_and_switch("feature-branch").unwrap();
        assert_eq!(manager.current().unwrap(), "feature-branch");

        manager.switch(&current).unwrap();
        assert_eq!(manager.current().unwrap(), current);

        manager.delete("feature-branch", false).unwrap();
        let branches = manager.list(false).unwrap();
        assert!(!branches.iter().any(|b| b.name == "feature-branch"));
    }
}

// ===== 4. MCP 端到端 =====

#[cfg(test)]
mod mcp_tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_memory_transport() {
        use sh_layer4::mcp_bridge::protocol::{McpMessage, McpRequest, RequestId};
        use sh_layer4::mcp_bridge::transport::{McpTransport, MemoryTransport};

        let transport = MemoryTransport::new();

        // 发送请求
        let msg = McpMessage::Request(McpRequest {
            id: RequestId::Number(1),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({"protocol_version": "2024-11-05"})),
        });
        transport.send(&msg).await.unwrap();

        // 接收请求
        let received = transport.receive().await.unwrap();
        assert!(received.is_some());
        if let Some(McpMessage::Request(req)) = received {
            assert_eq!(req.method, "initialize");
        } else {
            panic!("Expected Request message");
        }

        // 关闭
        transport.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_mcp_tool_call_real() {
        use sh_layer4::mcp_bridge::handler::{
            DefaultHandler, McpHandler, SimpleToolExecutor, ToolExecutor,
        };
        use sh_layer4::mcp_bridge::protocol::{
            ContentBlock, McpRequest, RequestId, ToolDefinition, ToolResult,
        };
        use std::sync::Arc;

        let handler = DefaultHandler::new("test-server", "1.0.0");

        // 注册 echo 工具
        handler.register_tool(
            ToolDefinition {
                name: "echo".to_string(),
                description: Some("Echo input text".to_string()),
                input_schema: None,
            },
            Arc::new(SimpleToolExecutor(|_name, args| {
                let text = args.as_str().unwrap_or("empty").to_string();
                Ok(ToolResult {
                    is_error: false,
                    content: vec![ContentBlock::Text { text }],
                })
            })),
        );

        // 注册 add 工具
        handler.register_tool(
            ToolDefinition {
                name: "add".to_string(),
                description: Some("Add two numbers".to_string()),
                input_schema: None,
            },
            Arc::new(SimpleToolExecutor(|_name, args| {
                let a = args.get("a").and_then(|v| v.as_i64()).unwrap_or(0);
                let b = args.get("b").and_then(|v| v.as_i64()).unwrap_or(0);
                Ok(ToolResult {
                    is_error: false,
                    content: vec![ContentBlock::Text {
                        text: format!("{}", a + b),
                    }],
                })
            })),
        );

        // 验证工具列表
        let list_request = McpRequest {
            id: RequestId::Number(1),
            method: "tools/list".to_string(),
            params: None,
        };
        let response = handler.handle(&list_request).await.unwrap();
        assert!(response.error.is_none());
        let result = response.result.unwrap();
        let tools = result.get("tools").and_then(|t| t.as_array()).unwrap();
        assert_eq!(tools.len(), 2);

        // 调用 echo 工具
        let echo_request = McpRequest {
            id: RequestId::Number(2),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({"name": "echo", "arguments": "hello"})),
        };
        let response = handler.handle(&echo_request).await.unwrap();
        assert!(response.error.is_none());
        let result: ToolResult = serde_json::from_value(response.result.unwrap()).unwrap();
        assert!(!result.is_error);
        if let ContentBlock::Text { text } = &result.content[0] {
            assert_eq!(text, "hello");
        } else {
            panic!("Expected text content block");
        }

        // 调用 add 工具
        let add_request = McpRequest {
            id: RequestId::Number(3),
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({"name": "add", "arguments": {"a": 3, "b": 5}})),
        };
        let response = handler.handle(&add_request).await.unwrap();
        assert!(response.error.is_none());
        let result: ToolResult = serde_json::from_value(response.result.unwrap()).unwrap();
        assert!(!result.is_error);
        if let ContentBlock::Text { text } = &result.content[0] {
            assert_eq!(text, "8");
        } else {
            panic!("Expected text content block");
        }
    }

    #[tokio::test]
    async fn test_mcp_client_manager() {
        use sh_layer4::mcp_bridge::client::McpClientManager;
        use sh_layer4::mcp_bridge::transport::McpTransportType;

        let manager = McpClientManager::new();

        let config = sh_layer4::mcp_bridge::client::McpServerConfig {
            name: "test-server".to_string(),
            transport: McpTransportType::Stdio {
                command: "echo".to_string(),
                args: vec![],
            },
            auto_reconnect: false,
            reconnect_interval_ms: 1000,
        };

        manager.add_server(config).await.unwrap();

        let servers = manager.list_servers();
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].0, "test-server");

        let status = manager.render_status();
        assert!(status.contains("test-server"));
    }

    #[tokio::test]
    async fn test_mcp_handler() {
        use sh_layer4::mcp_bridge::handler::{DefaultHandler, McpHandler, SimpleToolExecutor};
        use sh_layer4::mcp_bridge::protocol::{
            ContentBlock, McpRequest, RequestId, ToolDefinition, ToolResult,
        };
        use std::sync::Arc;

        let handler = DefaultHandler::new("test-server", "1.0.0");

        handler.register_tool(
            ToolDefinition {
                name: "greet".to_string(),
                description: Some("Greet someone".to_string()),
                input_schema: None,
            },
            Arc::new(SimpleToolExecutor(|_name, args| {
                let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("world");
                Ok(ToolResult {
                    is_error: false,
                    content: vec![ContentBlock::Text {
                        text: format!("Hello, {}!", name),
                    }],
                })
            })),
        );

        // 请求工具列表
        let request = McpRequest {
            id: RequestId::Number(1),
            method: "tools/list".to_string(),
            params: None,
        };

        let response = handler.handle(&request).await.unwrap();
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }
}

// ===== 5. 错误恢复 =====

#[cfg(test)]
mod recovery_tests {
    use super::*;

    #[test]
    fn test_error_recovery_category() {
        use sh_layer2::checkpoint_system::ErrorCategory;

        assert_eq!(
            ErrorCategory::from_error_message("network timeout"),
            ErrorCategory::Transient
        );
        assert_eq!(
            ErrorCategory::from_error_message("api key invalid"),
            ErrorCategory::Configuration
        );
        assert_eq!(
            ErrorCategory::from_error_message("invalid parameter"),
            ErrorCategory::Logic
        );

        assert!(ErrorCategory::Transient.is_retryable());
        assert!(ErrorCategory::Resource.is_retryable());
        assert!(!ErrorCategory::Configuration.is_retryable());
        assert!(!ErrorCategory::Logic.is_retryable());
    }

    #[test]
    fn test_retry_policy() {
        use sh_layer2::checkpoint_system::RetryPolicy;

        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 3);

        let d0 = policy.delay_for_attempt(0);
        let d1 = policy.delay_for_attempt(1);
        assert!(d1 > d0, "Delay should increase with attempts");
    }

    #[tokio::test]
    async fn test_error_recovery_stats() {
        use sh_layer2::checkpoint_system::{ErrorRecovery, FallbackStrategy};

        let recovery = ErrorRecovery::new().with_fallback(FallbackStrategy::Skip);

        let stats = recovery.get_stats().await;
        assert_eq!(stats.total_errors, 0);
    }

    #[test]
    fn test_session_recovery() {
        use sh_layer2::checkpoint_system::SessionRecovery;

        let dir = TempDir::new().unwrap();
        let recovery = SessionRecovery::new(dir.path());

        let interrupted = recovery.detect_interrupted_sessions().unwrap();
        assert!(interrupted.is_empty());

        let rendered = recovery.render_interrupted();
        assert!(rendered.contains("No interrupted"));
    }

    #[test]
    fn test_checkpoint_real_save_load() {
        use chrono::Utc;
        use sh_layer2::checkpoint_system::{
            CheckpointData, CheckpointSystemTrait, CheckpointWriter,
        };
        use sh_layer2::types::{CheckpointId, SessionId};

        let temp_dir = TempDir::new().unwrap();
        let writer = CheckpointWriter::new(temp_dir.path());

        let data = CheckpointData {
            checkpoint_id: CheckpointId::new(),
            session_id: SessionId::new(),
            created_at: Utc::now(),
            trigger: "test".to_string(),
            iteration: 1,
            messages: vec![serde_json::json!({"role": "user", "content": "test"})],
            tool_calls_pending: Vec::new(),
            tool_results: serde_json::Value::Null,
            tokens_used: 100,
            cost_estimate: 0.01,
            resume_hint: None,
        };

        let saved_id = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(writer.save(&data))
            .expect("Save should succeed");

        let session_id = data.session_id.clone();
        let loaded = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(writer.load(&session_id, None))
            .expect("Load should succeed");

        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.checkpoint_id, saved_id);
        assert_eq!(loaded.iteration, 1);
    }
}
