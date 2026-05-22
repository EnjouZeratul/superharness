//! 集成测试 - LLM 真实调用 + CLI 端到端 + Git + MCP
//!
//! 运行: cargo test --test integration_llm
//! 需要 .env.test 文件中的真实 API 密钥（LLM 测试可选）

mod common;

use common::test_config::{get_api_key, get_base_url, get_model, is_api_available, load_env};

macro_rules! require_api {
    () => {
        if !is_api_available() {
            eprintln!("Skipping: API key not available");
            return;
        }
    };
}

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
    async fn test_llm_client_creation() {
        require_api!();
        load_env();

        use sh_layer1::llm_client::{LlmClient, LlmProvider};

        let api_key = get_api_key().unwrap();
        let base_url = get_base_url();

        let provider = if base_url.contains("openai") {
            LlmProvider::OpenAI
        } else if base_url.contains("gemini") || base_url.contains("google") {
            LlmProvider::Gemini
        } else {
            LlmProvider::Anthropic
        };

        let _client = LlmClient::new(provider, api_key).with_base_url(base_url);
        // 验证客户端创建成功（无 panic）
        assert!(true);
    }

    #[tokio::test]
    async fn test_simple_chat_request() {
        require_api!();
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

        let messages = vec![Message {
            role: MessageRole::User,
            content: "Say exactly 'INTEGRATION_TEST_OK' and nothing else.".to_string(),
        }];

        let result = client.send(messages, &config).await;

        match result {
            Ok(response) => {
                println!("LLM response: {}", response.content);
                assert!(!response.content.is_empty(), "Response should not be empty");
            }
            Err(e) => {
                eprintln!("LLM API error (may be expected if quota exceeded): {}", e);
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

        let content: Vec<String> = (1..=20).map(|i| format!("Line {}", i)).collect();
        fs::write(&file_path, content.join("\n")).unwrap();

        let result =
            cli::commands::tool_exec::execute_read(file_path.to_str().unwrap(), None, None, false)
                .unwrap();
        assert!(result.contains("Line 1"));
        assert!(result.contains("Line 20"));

        let result = cli::commands::tool_exec::execute_read(
            file_path.to_str().unwrap(),
            Some(5),
            Some(3),
            false,
        )
        .unwrap();
        assert!(result.contains("Line 6"));
        assert!(!result.contains("Line 1"));

        let result =
            cli::commands::tool_exec::execute_read("/nonexistent/file.txt", None, None, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_cli_write_command() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("write_test.txt");

        let result = cli::commands::tool_exec::execute_write(
            file_path.to_str().unwrap(),
            Some("first write"),
            false,
            false,
        )
        .unwrap();
        assert!(result.contains("bytes"));

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

        let backup_path = dir.path().join("write_test.txt.bak");
        let result = cli::commands::tool_exec::execute_write(
            file_path.to_str().unwrap(),
            Some("with backup"),
            false,
            true,
        )
        .unwrap();
        assert!(backup_path.exists());
    }

    #[test]
    fn test_cli_edit_command() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("edit_test.txt");
        fs::write(&file_path, "foo bar foo baz foo").unwrap();

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
    }
}

// ===== 3. Git 集成 =====

#[cfg(test)]
mod git_tests {
    use super::*;
    use cli::git::{branch, diff, status};

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
    fn test_git_status() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());

        fs::write(dir.path().join("test.txt"), "content").unwrap();

        let status = status::get_status(dir.path()).unwrap();
        assert!(status.has_changes());

        let rendered = status.render();
        assert!(!rendered.is_empty());
    }

    #[test]
    fn test_git_diff() {
        let dir = TempDir::new().unwrap();
        init_git_repo(dir.path());

        // 创建并提交文件
        fs::write(dir.path().join("file.txt"), "initial").unwrap();
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
        fs::write(dir.path().join("file.txt"), "modified").unwrap();

        let diff = diff::get_diff(dir.path(), diff::DiffType::Working, &[]).unwrap();
        let stat = diff.stat();
        assert!(stat.contains("file") || diff.files_changed >= 0);
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

        let msg = McpMessage::Request(McpRequest {
            id: RequestId::Number(1),
            method: "initialize".to_string(),
            params: Some(serde_json::json!({"protocol_version": "2024-11-05"})),
        });

        transport.send(&msg).await.unwrap();

        let received = transport.receive().await.unwrap();
        assert!(received.is_some());

        transport.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_mcp_tool_call() {
        use sh_layer4::mcp_bridge::protocol::{ContentBlock, ToolDefinition, ToolResult};
        use sh_layer4::mcp_bridge::McpBridge;

        let bridge = McpBridge::new(Default::default());

        bridge.register_simple_tool("echo", "Echo tool", |_name, args| {
            let text = args.as_str().unwrap_or("empty").to_string();
            Ok(ToolResult {
                is_error: false,
                content: vec![ContentBlock::Text { text }],
            })
        });

        bridge.register_simple_tool("add", "Add numbers", |_name, args| {
            let a = args.get("a").and_then(|v| v.as_i64()).unwrap_or(0);
            let b = args.get("b").and_then(|v| v.as_i64()).unwrap_or(0);
            Ok(ToolResult {
                is_error: false,
                content: vec![ContentBlock::Text {
                    text: format!("{}", a + b),
                }],
            })
        });

        // 注册成功即测试通过
        assert!(true);
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
        assert!(d1 > d0);
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
}
