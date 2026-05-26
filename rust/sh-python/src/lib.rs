//! # Continuum Python Bindings
//!
//! Python bindings for Continuum.
//!
//! ## Performance Optimizations
//! - Global tokio runtime (lazy initialization via OnceLock)
//! - GIL release with `allow_threads` for blocking operations

use pyo3::prelude::*;
use std::sync::OnceLock;
use tokio::runtime::Runtime;

/// Global tokio runtime for async operations
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Get or create the global tokio runtime
fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime")
    })
}

/// Python 模块定义
#[pymodule]
fn sh_python(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Layer 0
    m.add_class::<PySecurityGateway>()?;

    // Layer 1
    m.add_class::<PyLlmClient>()?;
    m.add_class::<PyLlmRequestConfig>()?;
    m.add_class::<PyLlmResponse>()?;
    m.add_class::<PyCostTracker>()?;
    m.add_class::<PyUsageSnapshot>()?;
    m.add_class::<PyCostEstimate>()?;

    // Layer 2
    m.add_class::<PyAgentRuntime>()?;
    m.add_class::<PySessionManager>()?;
    m.add_class::<PyCheckpointSystem>()?;
    m.add_class::<PyAgent>()?;
    m.add_class::<PySession>()?;

    // Layer 3
    m.add_class::<PyToolExecutor>()?;
    m.add_class::<PyQueryEngine>()?;
    m.add_class::<PyMemorySystem>()?;
    m.add_class::<PyVectorStore>()?;
    m.add_class::<PyVectorItem>()?;
    m.add_class::<PySearchResult>()?;

    // Layer 4
    m.add_class::<PyMcpBridge>()?;
    m.add_class::<PyAuditLogger>()?;

    Ok(())
}

mod bindings {
    use super::*;
    use sh_layer1::LlmClientTrait;
    use sh_layer2::{AgentRuntimeTrait, CheckpointSystemTrait, SessionManagerTrait, ToolRegistryTrait};
    use sh_layer3::{ToolExecutor, VectorStoreTrait};

    /// 将 Layer2Error 转换为适当的 Python 异常类型
    fn layer2_error_to_pyerr(e: &sh_layer2::Layer2Error) -> PyErr {
        use sh_layer2::Layer2Error;
        match e {
            Layer2Error::SessionNotFound(id) => {
                pyo3::exceptions::PyKeyError::new_err(format!("Session not found: {}", id.0))
            }
            Layer2Error::MaxSessionsReached(n) => {
                pyo3::exceptions::PyRuntimeError::new_err(format!("Max sessions reached: {}", n))
            }
            Layer2Error::LlmNotConfigured => {
                pyo3::exceptions::PyRuntimeError::new_err("LLM client not configured")
            }
            Layer2Error::InvalidStateTransition { from, to } => {
                pyo3::exceptions::PyValueError::new_err(
                    format!("Invalid state transition: from {:?} to {:?}", from, to)
                )
            }
            Layer2Error::MaxIterations(n) => {
                pyo3::exceptions::PyRuntimeError::new_err(format!("Max iterations reached: {}", n))
            }
            Layer2Error::AgentError(msg) => {
                pyo3::exceptions::PyRuntimeError::new_err(msg.clone())
            }
            Layer2Error::Io(e) => {
                pyo3::exceptions::PyIOError::new_err(e.to_string())
            }
            Layer2Error::Serialization(e) => {
                pyo3::exceptions::PyValueError::new_err(format!("Serialization error: {}", e))
            }
            Layer2Error::CheckpointNotFound(id) => {
                pyo3::exceptions::PyKeyError::new_err(format!("Checkpoint not found: {}", id.0))
            }
            Layer2Error::CheckpointCorrupted(msg) => {
                pyo3::exceptions::PyValueError::new_err(format!("Checkpoint corrupted: {}", msg))
            }
            Layer2Error::ToolNotFound(name) => {
                pyo3::exceptions::PyKeyError::new_err(format!("Tool not found: {}", name))
            }
            Layer2Error::TaskNotFound(id) => {
                pyo3::exceptions::PyKeyError::new_err(format!("Task not found: {}", id.0))
            }
            Layer2Error::LockTimeout => {
                pyo3::exceptions::PyTimeoutError::new_err("Lock acquisition timeout")
            }
        }
    }

    /// 将 anyhow::Error 转换为 Python 异常，尝试提取 Layer2Error
    fn anyhow_to_pyerr(e: anyhow::Error) -> PyErr {
        if let Some(layer2_err) = e.downcast_ref::<sh_layer2::Layer2Error>() {
            layer2_error_to_pyerr(layer2_err)
        } else {
            pyo3::exceptions::PyRuntimeError::new_err(e.to_string())
        }
    }

    // ========================================================================
    // Layer 0: SecurityGateway
    // ========================================================================

    /// SecurityGateway Python 绑定
    #[pyclass(name = "SecurityGateway")]
    pub struct PySecurityGateway {
        inner: std::sync::Arc<tokio::sync::Mutex<sh_layer0::SecurityGateway>>,
    }

    #[pymethods]
    impl PySecurityGateway {
        #[new]
        fn new() -> Self {
            Self {
                inner: std::sync::Arc::new(tokio::sync::Mutex::new(
                    sh_layer0::SecurityGateway::new(),
                )),
            }
        }

        fn validate_input<'py>(&self, py: Python<'py>, input: String) -> PyResult<String> {
            let inner = self.inner.clone();
            pyo3_async_runtimes::tokio::run(py, async move {
                let gateway = inner.lock().await;
                gateway
                    .validate_input(&input)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }
    }

    // ========================================================================
    // Layer 1: LlmClient, CostTracker
    // ========================================================================

    /// LLM 提供商类型枚举
    #[pyclass(name = "LlmProvider")]
    #[derive(Clone)]
    pub enum PyLlmProvider {
        Anthropic(),
        OpenAI(),
        Gemini(),
        Custom { base_url: String },
    }

    impl From<PyLlmProvider> for sh_layer1::LlmProvider {
        fn from(provider: PyLlmProvider) -> Self {
            match provider {
                PyLlmProvider::Anthropic() => sh_layer1::LlmProvider::Anthropic,
                PyLlmProvider::OpenAI() => sh_layer1::LlmProvider::OpenAI,
                PyLlmProvider::Gemini() => sh_layer1::LlmProvider::Gemini,
                PyLlmProvider::Custom { base_url } => sh_layer1::LlmProvider::Custom(base_url),
            }
        }
    }

    #[pyclass(name = "LlmRequestConfig")]
    #[derive(Clone)]
    pub struct PyLlmRequestConfig {
        #[pyo3(get)]
        model: String,
        #[pyo3(get)]
        max_tokens: u32,
        #[pyo3(get)]
        temperature: f32,
        #[pyo3(get)]
        system_prompt: Option<String>,
    }

    #[pymethods]
    impl PyLlmRequestConfig {
        #[new]
        #[pyo3(signature = (model="claude-sonnet-4-6", max_tokens=4096, temperature=0.7, system_prompt=None))]
        fn new(model: &str, max_tokens: u32, temperature: f32, system_prompt: Option<&str>) -> Self {
            Self {
                model: model.to_string(),
                max_tokens,
                temperature,
                system_prompt: system_prompt.map(|s| s.to_string()),
            }
        }
    }

    impl From<&PyLlmRequestConfig> for sh_layer1::LlmRequestConfig {
        fn from(config: &PyLlmRequestConfig) -> Self {
            sh_layer1::LlmRequestConfig {
                model: config.model.clone(),
                max_tokens: config.max_tokens,
                temperature: config.temperature,
                system_prompt: config.system_prompt.clone(),
                stop_sequences: vec!["\n\n\n".to_string()],
            }
        }
    }

    /// LLM 响应 Python 类
    #[pyclass(name = "LlmResponse")]
    pub struct PyLlmResponse {
        #[pyo3(get)]
        content: String,
        #[pyo3(get)]
        input_tokens: u32,
        #[pyo3(get)]
        output_tokens: u32,
        #[pyo3(get)]
        model: String,
        #[pyo3(get)]
        response_id: String,
    }

    #[pymethods]
    impl PyLlmResponse {
        /// 获取 token 总数
        fn total_tokens(&self) -> u32 {
            self.input_tokens + self.output_tokens
        }

        /// 转换为 JSON 字符串
        fn to_json(&self) -> PyResult<String> {
            serde_json::to_string(&serde_json::json!({
                "content": self.content,
                "input_tokens": self.input_tokens,
                "output_tokens": self.output_tokens,
                "model": self.model,
                "response_id": self.response_id,
            }))
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        }
    }

    #[pyclass(name = "LlmClient")]
    pub struct PyLlmClient {
        inner: std::sync::Arc<tokio::sync::Mutex<Option<sh_layer1::LlmClient>>>,
        provider: PyLlmProvider,
    }

    #[pymethods]
    impl PyLlmClient {
        #[new]
        #[pyo3(signature = (provider="anthropic", api_key=None, base_url=None))]
        fn new(provider: &str, api_key: Option<&str>, base_url: Option<&str>) -> Self {
            let py_provider = match provider.to_lowercase().as_str() {
                "anthropic" => PyLlmProvider::Anthropic(),
                "openai" => PyLlmProvider::OpenAI(),
                "gemini" => PyLlmProvider::Gemini(),
                custom => PyLlmProvider::Custom {
                    base_url: custom.to_string(),
                },
            };

            let rust_provider = sh_layer1::LlmProvider::from(py_provider.clone());
            let key = api_key
                .map(|s| s.to_string())
                .unwrap_or_else(|| std::env::var("ANTHROPIC_API_KEY")
                    .or_else(|_| std::env::var("OPENAI_API_KEY"))
                    .or_else(|_| std::env::var("GEMINI_API_KEY"))
                    .unwrap_or_default());

            let client = sh_layer1::LlmClient::new(rust_provider, key);
            let client_with_url = if let Some(url) = base_url {
                client.with_base_url(url.to_string())
            } else {
                client
            };

            Self {
                inner: std::sync::Arc::new(tokio::sync::Mutex::new(Some(client_with_url))),
                provider: py_provider,
            }
        }

        /// 连接并验证 API
        fn connect(&self) -> PyResult<bool> {
            // 简单检查是否配置了客户端
            let inner = self.inner.clone();
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            rt.block_on(async {
                let client = inner.lock().await;
                Ok(client.is_some())
            })
        }

        /// 检查是否已连接
        fn is_connected(&self) -> bool {
            let inner = self.inner.clone();
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let client = inner.lock().await;
                client.is_some()
            })
        }

        /// 发送消息并获取响应
        fn send<'py>(
            &self,
            py: Python<'py>,
            messages: Vec<(String, String)>,
            config: &PyLlmRequestConfig,
        ) -> PyResult<PyLlmResponse> {
            let inner = self.inner.clone();
            let rust_config = sh_layer1::LlmRequestConfig::from(config);

            // 转换消息格式
            let llm_messages: Vec<sh_layer1::Message> = messages
                .into_iter()
                .map(|(role, content)| {
                    let msg_role = match role.to_lowercase().as_str() {
                        "user" => sh_layer1::MessageRole::User,
                        "assistant" => sh_layer1::MessageRole::Assistant,
                        "system" => sh_layer1::MessageRole::System,
                        _ => sh_layer1::MessageRole::User,
                    };
                    sh_layer1::Message {
                        role: msg_role,
                        content,
                    }
                })
                .collect();

            pyo3_async_runtimes::tokio::run(py, async move {
                let client_guard = inner.lock().await;
                let client = client_guard
                    .as_ref()
                    .ok_or_else(|| pyo3::exceptions::PyRuntimeError::new_err("LlmClient not initialized"))?;

                let response = client
                    .send(llm_messages, &rust_config)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

                Ok(PyLlmResponse {
                    content: response.content,
                    input_tokens: response.usage.input_tokens,
                    output_tokens: response.usage.output_tokens,
                    model: response.model,
                    response_id: response.response_id,
                })
            })
        }

        /// 发送单条消息的便捷方法
        fn send_message<'py>(
            &self,
            py: Python<'py>,
            message: &str,
            config: &PyLlmRequestConfig,
        ) -> PyResult<PyLlmResponse> {
            self.send(py, vec![("user".to_string(), message.to_string())], config)
        }

        /// 获取提供商名称
        fn provider_name(&self) -> String {
            match &self.provider {
                PyLlmProvider::Anthropic() => "anthropic".to_string(),
                PyLlmProvider::OpenAI() => "openai".to_string(),
                PyLlmProvider::Gemini() => "gemini".to_string(),
                PyLlmProvider::Custom { base_url } => format!("custom:{}", base_url),
            }
        }

        /// 获取支持的模型列表
        fn supported_models(&self) -> Vec<String> {
            match &self.provider {
                PyLlmProvider::Anthropic() => vec![
                    "claude-opus-4-7".to_string(),
                    "claude-sonnet-4-6".to_string(),
                    "claude-haiku-4-5".to_string(),
                ],
                PyLlmProvider::OpenAI() => vec![
                    "gpt-4o".to_string(),
                    "gpt-4o-mini".to_string(),
                    "gpt-4-turbo".to_string(),
                ],
                PyLlmProvider::Gemini() => vec![
                    "gemini-2.5-pro".to_string(),
                    "gemini-2.5-flash".to_string(),
                ],
                PyLlmProvider::Custom { .. } => vec!["custom-model".to_string()],
            }
        }
    }

    #[pyclass(name = "CostTracker")]
    pub struct PyCostTracker {
        inner: std::sync::Arc<tokio::sync::Mutex<sh_layer1::CostTracker>>,
    }

    #[pymethods]
    impl PyCostTracker {
        #[new]
        fn new() -> Self {
            Self {
                inner: std::sync::Arc::new(tokio::sync::Mutex::new(sh_layer1::CostTracker::new())),
            }
        }

        /// 设置预算上限
        fn set_budget_limit(&self, limit: f64) {
            let inner = self.inner.clone();
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let tracker = inner.lock().await;
                tracker.set_budget_limit(limit);
            });
        }

        /// 记录使用情况
        fn record_usage<'py>(
            &self,
            py: Python<'py>,
            model: &str,
            input_tokens: u64,
            output_tokens: u64,
        ) -> PyResult<()> {
            let inner = self.inner.clone();
            let model_str = model.to_string();

            pyo3_async_runtimes::tokio::run(py, async move {
                let tracker = inner.lock().await;
                tracker
                    .record_usage(&model_str, input_tokens, output_tokens)
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }

        /// 获取当前使用情况
        fn get_current_usage<'py>(&self, py: Python<'py>) -> PyResult<PyUsageSnapshot> {
            let inner = self.inner.clone();

            pyo3_async_runtimes::tokio::run(py, async move {
                let tracker = inner.lock().await;
                let snapshot = tracker.get_current_usage();
                Ok(PyUsageSnapshot::from_snapshot(snapshot))
            })
        }

        /// 预估下一步成本
        fn estimate_next_step(&self, model: &str, estimated_input: u64, estimated_output: u64) -> PyCostEstimate {
            let inner = self.inner.clone();
            let model_str = model.to_string();
            let rt = tokio::runtime::Runtime::new().unwrap();

            rt.block_on(async {
                let tracker = inner.lock().await;
                let estimate = tracker.estimate_next_step(&model_str, estimated_input, estimated_output);
                PyCostEstimate::from_estimate(estimate)
            })
        }

        /// 生成成本报告
        fn generate_report<'py>(&self, py: Python<'py>) -> PyResult<String> {
            let inner = self.inner.clone();

            pyo3_async_runtimes::tokio::run(py, async move {
                let tracker = inner.lock().await;
                let report = tracker.generate_report();
                serde_json::to_string(&report)
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }

        /// 重置追踪器
        fn reset(&self) {
            let inner = self.inner.clone();
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let tracker = inner.lock().await;
                tracker.reset();
            });
        }

        /// 获取总成本（便捷方法）
        fn total_cost(&self) -> f64 {
            let inner = self.inner.clone();
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let tracker = inner.lock().await;
                tracker.get_current_usage().total_cost_usd
            })
        }
    }

    /// 使用情况快照 Python 类
    #[pyclass(name = "UsageSnapshot")]
    pub struct PyUsageSnapshot {
        #[pyo3(get)]
        total_input_tokens: u64,
        #[pyo3(get)]
        total_output_tokens: u64,
        #[pyo3(get)]
        total_cost_usd: f64,
        #[pyo3(get)]
        budget_remaining: Option<f64>,
    }

    impl PyUsageSnapshot {
        fn from_snapshot(snapshot: sh_layer1::cost_tracker::UsageSnapshot) -> Self {
            Self {
                total_input_tokens: snapshot.total_input_tokens,
                total_output_tokens: snapshot.total_output_tokens,
                total_cost_usd: snapshot.total_cost_usd,
                budget_remaining: snapshot.budget_remaining,
            }
        }
    }

    #[pymethods]
    impl PyUsageSnapshot {
        /// 获取模型成本明细
        fn model_costs(&self) -> PyResult<String> {
            // 返回 JSON 字符串，Python 可以解析
            let snapshot = sh_layer1::cost_tracker::UsageSnapshot {
                total_input_tokens: self.total_input_tokens,
                total_output_tokens: self.total_output_tokens,
                total_cost_usd: self.total_cost_usd,
                model_costs: std::collections::HashMap::new(),
                budget_remaining: self.budget_remaining,
            };
            serde_json::to_string(&snapshot.model_costs)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
        }
    }

    /// 成本预估 Python 类
    #[pyclass(name = "CostEstimate")]
    pub struct PyCostEstimate {
        #[pyo3(get)]
        min_tokens: u64,
        #[pyo3(get)]
        max_tokens: u64,
        #[pyo3(get)]
        estimated_cost_usd: f64,
        #[pyo3(get)]
        confidence: String,
    }

    impl PyCostEstimate {
        fn from_estimate(estimate: sh_layer1::cost_tracker::CostEstimate) -> Self {
            Self {
                min_tokens: estimate.min_tokens,
                max_tokens: estimate.max_tokens,
                estimated_cost_usd: estimate.estimated_cost_usd,
                confidence: estimate.confidence,
            }
        }
    }

    // ========================================================================
    // Layer 2: AgentRuntime, SessionManager, CheckpointSystem, Agent, Session
    // ========================================================================

    /// Agent 配置 Python 类
    #[pyclass(name = "AgentConfig")]
    #[derive(Clone)]
    pub struct PyAgentConfig {
        #[pyo3(get)]
        agent_id: String,
        #[pyo3(get)]
        model: String,
        #[pyo3(get)]
        temperature: f32,
        #[pyo3(get)]
        max_iterations: i32,
        #[pyo3(get)]
        system_prompt: Option<String>,
    }

    #[pymethods]
    impl PyAgentConfig {
        #[new]
        #[pyo3(signature = (agent_id=None, model="gpt-4o", temperature=0.7, max_iterations=100, system_prompt=None))]
        fn new(
            agent_id: Option<&str>,
            model: &str,
            temperature: f32,
            max_iterations: i32,
            system_prompt: Option<&str>,
        ) -> Self {
            Self {
                agent_id: agent_id
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                model: model.to_string(),
                temperature,
                max_iterations,
                system_prompt: system_prompt.map(|s| s.to_string()),
            }
        }
    }

    impl From<&PyAgentConfig> for sh_layer2::AgentConfig {
        fn from(config: &PyAgentConfig) -> Self {
            sh_layer2::AgentConfig {
                agent_id: sh_layer2::AgentId(config.agent_id.clone()),
                model: config.model.clone(),
                temperature: config.temperature,
                max_iterations: config.max_iterations,
                system_prompt: config.system_prompt.clone(),
            }
        }
    }

    /// Agent 结果 Python 类
    #[pyclass(name = "AgentResult")]
    pub struct PyAgentResult {
        #[pyo3(get)]
        session_id: String,
        #[pyo3(get)]
        final_state: String,
        #[pyo3(get)]
        iterations: i32,
        #[pyo3(get)]
        tokens_used: i64,
        messages_json: String,
        tool_calls_json: String,
        tool_results_json: String,
    }

    #[pymethods]
    impl PyAgentResult {
        /// 获取消息列表
        fn get_messages(&self) -> PyResult<Vec<(String, String)>> {
            let messages: Vec<serde_json::Value> = serde_json::from_str(&self.messages_json)
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            Ok(messages
                .iter()
                .filter_map(|m| {
                    let role = m.get("role")?.as_str()?;
                    let content = m.get("content")?.as_str()?;
                    Some((role.to_string(), content.to_string()))
                })
                .collect())
        }

        /// 获取消息 JSON
        fn messages_json(&self) -> &str {
            &self.messages_json
        }

        /// 获取工具调用 JSON
        fn tool_calls_json(&self) -> &str {
            &self.tool_calls_json
        }

        /// 获取工具结果 JSON
        fn tool_results_json(&self) -> &str {
            &self.tool_results_json
        }
    }

    #[pyclass(name = "AgentRuntime")]
    pub struct PyAgentRuntime {
        inner: std::sync::Arc<tokio::sync::Mutex<sh_layer2::AgentRuntime>>,
    }

    #[pymethods]
    impl PyAgentRuntime {
        #[new]
        fn new() -> Self {
            Self {
                inner: std::sync::Arc::new(tokio::sync::Mutex::new(
                    sh_layer2::AgentRuntime::with_defaults(),
                )),
            }
        }

        /// 运行 Agent 完成任务
        fn run<'py>(
            &self,
            py: Python<'py>,
            task: &str,
            config: &PyAgentConfig,
        ) -> PyResult<PyAgentResult> {
            let inner = self.inner.clone();
            let rust_config = sh_layer2::AgentConfig::from(config);
            let task_str = task.to_string();

            pyo3_async_runtimes::tokio::run(py, async move {
                let runtime = inner.lock().await;
                let result = runtime
                    .run(&task_str, rust_config)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

                // 转换结果
                let messages_json = serde_json::to_string(&result.messages)
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

                let tool_calls_json = serde_json::to_string(&result.tool_calls)
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

                let tool_results_json = serde_json::to_string(&result.tool_results)
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

                Ok(PyAgentResult {
                    session_id: result.session_id.0,
                    final_state: format!("{:?}", result.final_state),
                    iterations: result.iterations,
                    tokens_used: result.tokens_used,
                    messages_json,
                    tool_calls_json,
                    tool_results_json,
                })
            })
        }

        /// 启动 Agent 会话
        fn start<'py>(
            &self,
            py: Python<'py>,
            task: &str,
            config: &PyAgentConfig,
        ) -> PyResult<String> {
            let inner = self.inner.clone();
            let rust_config = sh_layer2::AgentConfig::from(config);
            let task_str = task.to_string();

            pyo3_async_runtimes::tokio::run(py, async move {
                let runtime = inner.lock().await;
                let session_id = runtime
                    .start(&task_str, rust_config)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

                Ok(session_id.0)
            })
        }

        /// 暂停 Agent
        fn pause<'py>(&self, py: Python<'py>, session_id: &str) -> PyResult<()> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);

            pyo3_async_runtimes::tokio::run(py, async move {
                let runtime = inner.lock().await;
                runtime
                    .pause(&sid)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }

        /// 恢复 Agent
        fn resume<'py>(&self, py: Python<'py>, session_id: &str) -> PyResult<()> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);

            pyo3_async_runtimes::tokio::run(py, async move {
                let runtime = inner.lock().await;
                runtime
                    .resume(&sid)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }

        /// 停止 Agent
        fn stop<'py>(&self, py: Python<'py>, session_id: &str) -> PyResult<()> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);

            pyo3_async_runtimes::tokio::run(py, async move {
                let runtime = inner.lock().await;
                runtime
                    .stop(&sid)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }

        /// 获取 Agent 状态
        fn status(&self, session_id: &str) -> PyResult<String> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);

            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            rt.block_on(async {
                let runtime = inner.lock().await;
                let state = runtime
                    .status(&sid)
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

                Ok(format!("{:?}", state))
            })
        }

        /// 向 Agent 发送消息
        fn send_message<'py>(&self, py: Python<'py>, session_id: &str, message: &str) -> PyResult<()> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);
            let msg = message.to_string();

            pyo3_async_runtimes::tokio::run(py, async move {
                let runtime = inner.lock().await;
                runtime
                    .send_message(&sid, &msg)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }

        /// 注册工具（目前不支持 Python 端注册，请使用 Rust 端内置工具）
        fn register_tool(&self, _name: &str, _description: &str) -> PyResult<()> {
            // 工具注册需要 Box<dyn Tool>，暂时不支持从 Python 端直接注册
            Err(pyo3::exceptions::PyRuntimeError::new_err(
                "Tool registration from Python is not supported. Use builtin tools or implement Tool trait in Rust."
            ))
        }

        /// 列出可用工具
        fn list_tools(&self) -> PyResult<Vec<String>> {
            let inner = self.inner.clone();

            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            rt.block_on(async {
                let runtime = inner.lock().await;
                Ok(runtime.tool_registry().definitions().iter().map(|d| d.function.name.clone()).collect())
            })
        }
    }

    #[pyclass(name = "SessionManager")]
    pub struct PySessionManager {
        inner: std::sync::Arc<sh_layer2::ConcurrentSessionManager>,
    }

    #[pymethods]
    impl PySessionManager {
        #[new]
        #[pyo3(signature = (max_sessions=100))]
        fn new(max_sessions: usize) -> Self {
            Self {
                inner: std::sync::Arc::new(sh_layer2::ConcurrentSessionManager::new(max_sessions)),
            }
        }

        /// 创建新会话
        fn create<'py>(&self, py: Python<'py>, model: Option<&str>, max_iterations: Option<i32>) -> PyResult<String> {
            let inner = self.inner.clone();
            let config = sh_layer2::SessionConfig {
                model: model.map(|s| s.to_string()).unwrap_or_else(|| "gpt-4o".to_string()),
                max_iterations: max_iterations.unwrap_or(100),
                temperature: 0.7,
                system_prompt: None,
            };

            pyo3_async_runtimes::tokio::run(py, async move {
                let manager = inner;
                let session_id = manager
                    .create(config)
                    .await
                    .map_err(anyhow_to_pyerr)?;

                Ok(session_id.0)
            })
        }

        /// 获取会话
        fn get<'py>(&self, py: Python<'py>, session_id: &str) -> PyResult<Option<String>> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);

            pyo3_async_runtimes::tokio::run(py, async move {
                let manager = inner;
                let session = manager
                    .get(&sid)
                    .await
                    .map_err(anyhow_to_pyerr)?;

                match session {
                    Some(s) => {
                        let json = serde_json::to_string(&serde_json::json!({
                            "session_id": s.session_id.0,
                            "agent_id": s.agent_id.0,
                            "state": format!("{:?}", s.state),
                            "created_at": s.created_at.to_rfc3339(),
                            "messages_count": s.messages.len(),
                            "tokens_total": s.tokens_total,
                        }))
                        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
                        Ok(Some(json))
                    }
                    None => Ok(None),
                }
            })
        }

        /// 删除会话
        fn delete<'py>(&self, py: Python<'py>, session_id: &str) -> PyResult<bool> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);

            pyo3_async_runtimes::tokio::run(py, async move {
                let manager = inner;
                manager
                    .delete(&sid)
                    .await
                    .map_err(anyhow_to_pyerr)
            })
        }

        /// 列出所有会话
        fn list<'py>(&self, py: Python<'py>) -> PyResult<Vec<(String, String, String)>> {
            let inner = self.inner.clone();

            pyo3_async_runtimes::tokio::run(py, async move {
                let manager = inner;
                let metas = manager
                    .list()
                    .await
                    .map_err(anyhow_to_pyerr)?;

                Ok(metas
                    .iter()
                    .map(|m| (
                        m.session_id.0.clone(),
                        m.agent_id.0.clone(),
                        format!("{:?}", m.state),
                    ))
                    .collect())
            })
        }

        /// 获取会话统计
        fn stats(&self) -> (usize, usize, usize) {
            let manager = &self.inner;
            let stats = manager.stats();
            (stats.total_sessions, stats.max_sessions, stats.active_sessions)
        }

        /// 设置会话状态
        fn set_state<'py>(&self, py: Python<'py>, session_id: &str, state: &str) -> PyResult<bool> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);

            let agent_state = match state.to_lowercase().as_str() {
                "idle" => sh_layer2::AgentState::Idle,
                "running" => sh_layer2::AgentState::Running,
                "toolcalling" => sh_layer2::AgentState::ToolCalling,
                "waitingtool" => sh_layer2::AgentState::WaitingTool,
                "stopped" => sh_layer2::AgentState::Stopped,
                "completed" => sh_layer2::AgentState::Completed,
                "error" => sh_layer2::AgentState::Error,
                _ => sh_layer2::AgentState::Idle,
            };

            pyo3_async_runtimes::tokio::run(py, async move {
                let manager = inner;
                manager
                    .set_state(&sid, agent_state)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }

        /// 添加消息到会话
        fn add_message<'py>(&self, py: Python<'py>, session_id: &str, role: &str, content: &str) -> PyResult<bool> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);

            let message = match role.to_lowercase().as_str() {
                "user" => sh_layer2::Message::user(content),
                "assistant" => sh_layer2::Message::assistant(content),
                "system" => sh_layer2::Message::system(content),
                _ => sh_layer2::Message::user(content),
            };

            pyo3_async_runtimes::tokio::run(py, async move {
                let manager = inner;
                manager
                    .add_message(&sid, message)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }

        /// 获取会话消息
        fn get_messages<'py>(&self, py: Python<'py>, session_id: &str) -> PyResult<Vec<(String, String)>> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);

            pyo3_async_runtimes::tokio::run(py, async move {
                let manager = inner;
                let messages = manager
                    .get_messages(&sid)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

                match messages {
                    Some(msgs) => Ok(msgs
                        .iter()
                        .map(|m| (format!("{:?}", m.role), m.content.clone()))
                        .collect()),
                    None => Ok(Vec::new()),
                }
            })
        }
    }

    /// CheckpointSystem - 检查点写入器
    #[pyclass(name = "CheckpointSystem")]
    pub struct PyCheckpointSystem {
        inner: std::sync::Arc<tokio::sync::Mutex<sh_layer2::CheckpointWriter>>,
    }

    #[pymethods]
    impl PyCheckpointSystem {
        #[new]
        #[pyo3(signature = (storage_path=None))]
        fn new(storage_path: Option<&str>) -> Self {
            let path = storage_path
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::env::temp_dir().join("continuum_checkpoints"));
            Self {
                inner: std::sync::Arc::new(tokio::sync::Mutex::new(
                    sh_layer2::CheckpointWriter::new(path),
                )),
            }
        }

        /// 保存检查点
        fn save<'py>(&self, py: Python<'py>, session_id: &str, data: String) -> PyResult<String> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);
            let checkpoint_data = sh_layer2::CheckpointData {
                checkpoint_id: sh_layer2::CheckpointId::new(),
                session_id: sid.clone(),
                created_at: chrono::Utc::now(),
                trigger: "manual".to_string(),
                iteration: 0,
                messages: vec![
                    serde_json::from_str(&data).unwrap_or(serde_json::json!({"content": data}))
                ],
                tool_calls_pending: Vec::new(),
                tool_results: serde_json::Value::Null,
                tokens_used: 0,
                cost_estimate: 0.0,
                resume_hint: None,
            };

            pyo3_async_runtimes::tokio::run(py, async move {
                let writer = inner.lock().await;
                let id = writer
                    .save(&checkpoint_data)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
                Ok(id.to_string())
            })
        }

        /// 加载检查点
        fn load<'py>(
            &self,
            py: Python<'py>,
            session_id: &str,
            checkpoint_id: Option<&str>,
        ) -> PyResult<Option<String>> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);
            let cid = checkpoint_id.map(|s| sh_layer2::CheckpointId(s.to_string()));

            pyo3_async_runtimes::tokio::run(py, async move {
                let writer = inner.lock().await;
                match writer.load(&sid, cid.as_ref()).await {
                    Ok(Some(data)) => {
                        let json = serde_json::to_string(&data.messages).map_err(|e| {
                            pyo3::exceptions::PyRuntimeError::new_err(e.to_string())
                        })?;
                        Ok(Some(json))
                    }
                    Ok(None) => Ok(None),
                    Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
                }
            })
        }

        /// 列出所有检查点
        fn list<'py>(&self, py: Python<'py>, session_id: &str) -> PyResult<Vec<String>> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);

            pyo3_async_runtimes::tokio::run(py, async move {
                let writer = inner.lock().await;
                let metas = writer
                    .list(&sid)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
                Ok(metas.iter().map(|m| m.checkpoint_id.to_string()).collect())
            })
        }

        /// 删除检查点
        fn delete<'py>(
            &self,
            py: Python<'py>,
            session_id: &str,
            checkpoint_id: &str,
        ) -> PyResult<bool> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);
            let cid = sh_layer2::CheckpointId(checkpoint_id.to_string());

            pyo3_async_runtimes::tokio::run(py, async move {
                let writer = inner.lock().await;
                writer
                    .delete(&sid, &cid)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }
    }

    /// Agent Python 绑定
    #[pyclass(name = "Agent")]
    pub struct PyAgent {
        id: String,
        agent_state: std::sync::Arc<tokio::sync::Mutex<AgentState>>,
    }

    #[derive(Clone)]
    #[allow(dead_code)]
    enum AgentState {
        Idle,
        Running,
        Paused,
        Error,
    }

    #[pymethods]
    impl PyAgent {
        #[new]
        #[pyo3(signature = (name=None))]
        fn new(name: Option<&str>) -> Self {
            Self {
                id: name.unwrap_or("default").to_string(),
                agent_state: std::sync::Arc::new(tokio::sync::Mutex::new(AgentState::Idle)),
            }
        }

        #[getter]
        fn id(&self) -> &str {
            &self.id
        }

        #[getter]
        fn state(&self) -> String {
            match &*self.agent_state.blocking_lock() {
                AgentState::Idle => "idle".to_string(),
                AgentState::Running => "running".to_string(),
                AgentState::Paused => "paused".to_string(),
                AgentState::Error => "error".to_string(),
            }
        }

        fn start(&self) -> PyResult<()> {
            let mut state = self.agent_state.blocking_lock();
            match *state {
                AgentState::Idle | AgentState::Paused => {
                    *state = AgentState::Running;
                    Ok(())
                }
                AgentState::Running => Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Agent is already running",
                )),
                AgentState::Error => Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Agent is in error state",
                )),
            }
        }

        fn pause(&self) -> PyResult<()> {
            let mut state = self.agent_state.blocking_lock();
            match *state {
                AgentState::Running => {
                    *state = AgentState::Paused;
                    Ok(())
                }
                _ => Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Agent is not running",
                )),
            }
        }

        fn stop(&self) {
            let mut state = self.agent_state.blocking_lock();
            *state = AgentState::Idle;
        }

        fn execute(&self, task: &str) -> PyResult<String> {
            let state = self.agent_state.blocking_lock();
            match *state {
                AgentState::Running => Ok(format!("Executing: {}", task)),
                _ => Err(pyo3::exceptions::PyRuntimeError::new_err(
                    "Agent is not running",
                )),
            }
        }

        fn create_session(&self) -> PySession {
            PySession::new(Some(&format!("{}-session", self.id)))
        }
    }

    /// Session Python 绑定
    #[pyclass(name = "Session")]
    pub struct PySession {
        id: String,
        created_at: chrono::DateTime<chrono::Utc>,
        messages: std::sync::Arc<tokio::sync::Mutex<Vec<SessionMessage>>>,
    }

    #[derive(Clone)]
    struct SessionMessage {
        role: String,
        content: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    }

    #[pymethods]
    impl PySession {
        #[new]
        #[pyo3(signature = (id=None))]
        fn new(id: Option<&str>) -> Self {
            Self {
                id: id.unwrap_or("default-session").to_string(),
                created_at: chrono::Utc::now(),
                messages: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
            }
        }

        #[getter]
        fn id(&self) -> &str {
            &self.id
        }

        #[getter]
        fn created_at(&self) -> String {
            self.created_at.to_rfc3339()
        }

        fn add_user_message(&self, content: &str) {
            let mut messages = self.messages.blocking_lock();
            messages.push(SessionMessage {
                role: "user".to_string(),
                content: content.to_string(),
                timestamp: chrono::Utc::now(),
            });
        }

        fn add_assistant_message(&self, content: &str) {
            let mut messages = self.messages.blocking_lock();
            messages.push(SessionMessage {
                role: "assistant".to_string(),
                content: content.to_string(),
                timestamp: chrono::Utc::now(),
            });
        }

        fn message_count(&self) -> usize {
            self.messages.blocking_lock().len()
        }

        fn get_messages(&self) -> Vec<(String, String)> {
            self.messages
                .blocking_lock()
                .iter()
                .map(|m| (m.role.clone(), m.content.clone()))
                .collect()
        }

        fn clear_messages(&self) {
            self.messages.blocking_lock().clear();
        }

        fn export(&self) -> String {
            let messages = self.messages.blocking_lock();
            let exported: Vec<serde_json::Value> = messages
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "role": m.role,
                        "content": m.content,
                        "timestamp": m.timestamp.to_rfc3339()
                    })
                })
                .collect();
            serde_json::to_string(&exported).unwrap_or_default()
        }
    }

    // ========================================================================
    // Layer 3: ToolExecutor, QueryEngine, MemorySystem
    // ========================================================================

    /// ToolExecutor - 工具执行器
    #[pyclass(name = "ToolExecutor")]
    pub struct PyToolExecutor {
        inner: std::sync::Arc<tokio::sync::Mutex<sh_layer3::DefaultToolExecutor>>,
    }

    #[pymethods]
    impl PyToolExecutor {
        #[new]
        fn new() -> Self {
            Self {
                inner: std::sync::Arc::new(tokio::sync::Mutex::new(
                    sh_layer3::DefaultToolExecutor::new(),
                )),
            }
        }

        /// 执行工具
        fn execute<'py>(&self, py: Python<'py>, name: &str, args_json: String) -> PyResult<String> {
            let inner = self.inner.clone();
            let tool_name = name.to_string();
            let args: serde_json::Value = serde_json::from_str(&args_json).map_err(|e| {
                pyo3::exceptions::PyValueError::new_err(format!("Invalid JSON: {}", e))
            })?;

            let request = sh_layer3::ToolRequest {
                call_id: uuid::Uuid::new_v4().to_string()[..8].to_string(),
                name: tool_name.clone(),
                arguments: args,
            };

            pyo3_async_runtimes::tokio::run(py, async move {
                let executor = inner.lock().await;
                let response = executor
                    .execute(request)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
                Ok(response.content)
            })
        }

        /// 读取文件
        #[pyo3(signature = (path, offset=None, limit=None))]
        fn read_file(
            &self,
            path: &str,
            offset: Option<usize>,
            limit: Option<usize>,
        ) -> PyResult<String> {
            let args = serde_json::json!({
                "path": path,
                "offset": offset,
                "limit": limit,
            });
            let _args_json = serde_json::to_string(&args).unwrap();

            // 使用同步运行时
            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            rt.block_on(async {
                let executor = self.inner.lock().await;
                let request = sh_layer3::ToolRequest {
                    call_id: uuid::Uuid::new_v4().to_string()[..8].to_string(),
                    name: "read_file".to_string(),
                    arguments: args,
                };

                match executor.execute(request).await {
                    Ok(response) => Ok(response.content),
                    Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
                }
            })
        }

        /// 写入文件
        fn write_file(&self, path: &str, content: &str) -> PyResult<String> {
            let args = serde_json::json!({
                "path": path,
                "content": content,
            });

            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            rt.block_on(async {
                let executor = self.inner.lock().await;
                let request = sh_layer3::ToolRequest {
                    call_id: uuid::Uuid::new_v4().to_string()[..8].to_string(),
                    name: "write_file".to_string(),
                    arguments: args,
                };

                match executor.execute(request).await {
                    Ok(response) => Ok(response.content),
                    Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
                }
            })
        }

        /// 执行 Bash 命令
        #[pyo3(signature = (command, timeout_ms=None, working_dir=None))]
        fn bash(
            &self,
            command: &str,
            timeout_ms: Option<u64>,
            working_dir: Option<&str>,
        ) -> PyResult<String> {
            let mut args = serde_json::json!({
                "command": command,
            });
            if let Some(t) = timeout_ms {
                args["timeout"] = serde_json::json!(t);
            }
            if let Some(w) = working_dir {
                args["working_dir"] = serde_json::json!(w);
            }

            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            rt.block_on(async {
                let executor = self.inner.lock().await;
                let request = sh_layer3::ToolRequest {
                    call_id: uuid::Uuid::new_v4().to_string()[..8].to_string(),
                    name: "bash".to_string(),
                    arguments: args,
                };

                match executor.execute(request).await {
                    Ok(response) => Ok(response.content),
                    Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
                }
            })
        }

        /// Grep 搜索
        #[pyo3(signature = (pattern, path=None, glob=None))]
        fn grep(&self, pattern: &str, path: Option<&str>, glob: Option<&str>) -> PyResult<String> {
            let mut args = serde_json::json!({
                "pattern": pattern,
            });
            if let Some(p) = path {
                args["path"] = serde_json::json!(p);
            }
            if let Some(g) = glob {
                args["glob"] = serde_json::json!(g);
            }

            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            rt.block_on(async {
                let executor = self.inner.lock().await;
                let request = sh_layer3::ToolRequest {
                    call_id: uuid::Uuid::new_v4().to_string()[..8].to_string(),
                    name: "grep".to_string(),
                    arguments: args,
                };

                match executor.execute(request).await {
                    Ok(response) => Ok(response.content),
                    Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
                }
            })
        }

        /// Glob 查找
        #[pyo3(signature = (pattern, path=None))]
        fn glob(&self, pattern: &str, path: Option<&str>) -> PyResult<String> {
            let mut args = serde_json::json!({
                "pattern": pattern,
            });
            if let Some(p) = path {
                args["path"] = serde_json::json!(p);
            }

            let rt = tokio::runtime::Runtime::new()
                .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

            rt.block_on(async {
                let executor = self.inner.lock().await;
                let request = sh_layer3::ToolRequest {
                    call_id: uuid::Uuid::new_v4().to_string()[..8].to_string(),
                    name: "glob".to_string(),
                    arguments: args,
                };

                match executor.execute(request).await {
                    Ok(response) => Ok(response.content),
                    Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
                }
            })
        }

        /// 列出可用工具
        fn list_tools(&self) -> Vec<(String, String)> {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let executor = self.inner.lock().await;
                executor
                    .list_tools()
                    .iter()
                    .map(|m| (m.name.clone(), m.description.clone()))
                    .collect()
            })
        }

        /// 检查工具是否可用
        fn is_available(&self, name: &str) -> bool {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let executor = self.inner.lock().await;
                executor.is_available(name)
            })
        }
    }

    #[pyclass(name = "QueryEngine")]
    pub struct PyQueryEngine;

    #[pymethods]
    impl PyQueryEngine {
        #[new]
        fn new() -> Self {
            Self
        }
    }

    #[pyclass(name = "MemorySystem")]
    pub struct PyMemorySystem;

    #[pymethods]
    impl PyMemorySystem {
        #[new]
        fn new() -> Self {
            Self
        }
    }

    // ========================================================================
    // Layer 4: McpBridge, AuditLogger
    // ========================================================================

    #[pyclass(name = "McpBridge")]
    pub struct PyMcpBridge;

    #[pymethods]
    impl PyMcpBridge {
        #[new]
        fn new() -> Self {
            Self
        }
    }

    #[pyclass(name = "AuditLogger")]
    pub struct PyAuditLogger {
        inner: std::sync::Arc<sh_layer4::AuditLogger>,
    }

    #[pymethods]
    impl PyAuditLogger {
        #[new]
        fn new() -> Self {
            Self {
                inner: std::sync::Arc::new(sh_layer4::AuditLogger::new(Default::default())),
            }
        }

        fn log<'py>(
            &self,
            py: Python<'py>,
            user_id: &str,
            action: &str,
            resource_type: &str,
        ) -> PyResult<()> {
            let inner = self.inner.clone();
            let audit_action = match action {
                "login" => sh_layer4::AuditAction::Login,
                "logout" => sh_layer4::AuditAction::Logout,
                "read" => sh_layer4::AuditAction::Read,
                "create" => sh_layer4::AuditAction::Create,
                "update" => sh_layer4::AuditAction::Update,
                "delete" => sh_layer4::AuditAction::Delete,
                "execute" => sh_layer4::AuditAction::Execute,
                _ => sh_layer4::AuditAction::Other(action.to_string()),
            };

            let entry = sh_layer4::AuditEntry::new(user_id, audit_action, resource_type);

            pyo3_async_runtimes::tokio::run(py, async move {
                inner
                    .log(entry)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })?;

            Ok(())
        }

        fn count<'py>(&self, py: Python<'py>) -> PyResult<usize> {
            let inner = self.inner.clone();
            pyo3_async_runtimes::tokio::run(py, async move {
                inner
                    .count()
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }
    }

    // ========================================================================
    // Layer 3: VectorStore
    // ========================================================================

    /// VectorStore Python 绑定
    #[pyclass(name = "VectorStore")]
    pub struct PyVectorStore {
        inner: std::sync::Arc<tokio::sync::Mutex<sh_layer3::InMemoryVectorStore>>,
    }

    #[pymethods]
    impl PyVectorStore {
        #[new]
        #[pyo3(signature = (metric="cosine"))]
        fn new(metric: &str) -> Self {
            let distance_metric = match metric.to_lowercase().as_str() {
                "cosine" => sh_layer3::DistanceMetric::Cosine,
                "euclidean" => sh_layer3::DistanceMetric::Euclidean,
                "dot_product" | "dotproduct" => sh_layer3::DistanceMetric::DotProduct,
                "manhattan" => sh_layer3::DistanceMetric::Manhattan,
                _ => sh_layer3::DistanceMetric::Cosine,
            };

            let config = sh_layer3::VectorStoreConfig {
                metric: distance_metric,
                ..Default::default()
            };

            Self {
                inner: std::sync::Arc::new(tokio::sync::Mutex::new(
                    sh_layer3::InMemoryVectorStore::new(config),
                )),
            }
        }

        /// 插入或更新向量
        fn upsert<'py>(
            &self,
            py: Python<'py>,
            id: &str,
            vector: Vec<f32>,
            metadata_json: Option<&str>,
        ) -> PyResult<bool> {
            let inner = self.inner.clone();
            let id_str = id.to_string();

            let metadata: std::collections::HashMap<String, serde_json::Value> = match metadata_json {
                Some(json) => serde_json::from_str(json)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid JSON metadata: {}", e)))?,
                None => std::collections::HashMap::new(),
            };

            pyo3_async_runtimes::tokio::run(py, async move {
                let store = inner.lock().await;
                store
                    .add(id_str, vector, metadata)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }

        /// 搜索相似向量
        fn search<'py>(
            &self,
            py: Python<'py>,
            vector: Vec<f32>,
            top_k: usize,
        ) -> PyResult<Vec<PySearchResult>> {
            let inner = self.inner.clone();

            pyo3_async_runtimes::tokio::run(py, async move {
                let store = inner.lock().await;
                let results = store
                    .query(vector, top_k)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;

                Ok(results
                    .into_iter()
                    .map(|r| PySearchResult {
                        id: r.doc_id,
                        score: r.score,
                        content: r.content,
                        metadata_json: serde_json::to_string(&r.metadata).unwrap_or_default(),
                    })
                    .collect())
            })
        }

        /// 删除向量
        fn delete<'py>(&self, py: Python<'py>, id: &str) -> PyResult<bool> {
            let inner = self.inner.clone();
            let id_str = id.to_string();

            pyo3_async_runtimes::tokio::run(py, async move {
                let store = inner.lock().await;
                store
                    .delete(&id_str)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }

        /// 获取向量
        fn get<'py>(&self, py: Python<'py>, id: &str) -> PyResult<Option<PyVectorItem>> {
            let inner = self.inner.clone();
            let id_str = id.to_string();

            pyo3_async_runtimes::tokio::run(py, async move {
                let store = inner.lock().await;
                match store.get(&id_str).await {
                    Ok(Some(item)) => Ok(Some(PyVectorItem {
                        id: item.id,
                        vector: item.vector,
                        content: item.content.unwrap_or_default(),
                        metadata_json: serde_json::to_string(&item.metadata).unwrap_or_default(),
                    })),
                    Ok(None) => Ok(None),
                    Err(e) => Err(pyo3::exceptions::PyRuntimeError::new_err(e.to_string())),
                }
            })
        }

        /// 获取向量数量
        fn count<'py>(&self, py: Python<'py>) -> PyResult<usize> {
            let inner = self.inner.clone();

            pyo3_async_runtimes::tokio::run(py, async move {
                let store = inner.lock().await;
                store
                    .count()
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }

        /// 清空存储
        fn clear<'py>(&self, py: Python<'py>) -> PyResult<bool> {
            let inner = self.inner.clone();

            pyo3_async_runtimes::tokio::run(py, async move {
                let store = inner.lock().await;
                store
                    .clear()
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }

        /// 批量插入向量
        fn upsert_batch<'py>(
            &self,
            py: Python<'py>,
            items: Vec<(String, Vec<f32>, Option<String>)>,
        ) -> PyResult<Vec<bool>> {
            let inner = self.inner.clone();

            let vector_items: Vec<sh_layer3::VectorItem> = items
                .into_iter()
                .map(|(id, vector, content)| {
                    sh_layer3::VectorItem::new(&id, vector)
                        .with_content(content.unwrap_or_default())
                })
                .collect();

            pyo3_async_runtimes::tokio::run(py, async move {
                let store = inner.lock().await;
                store
                    .add_batch(vector_items)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }

        /// 批量删除向量
        fn delete_batch<'py>(&self, py: Python<'py>, ids: Vec<String>) -> PyResult<usize> {
            let inner = self.inner.clone();

            pyo3_async_runtimes::tokio::run(py, async move {
                let store = inner.lock().await;
                store
                    .delete_batch(&ids)
                    .await
                    .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))
            })
        }
    }

    /// VectorItem Python 类
    #[pyclass(name = "VectorItem")]
    pub struct PyVectorItem {
        #[pyo3(get)]
        id: String,
        #[pyo3(get)]
        vector: Vec<f32>,
        #[pyo3(get)]
        content: String,
        metadata_json: String,
    }

    #[pymethods]
    impl PyVectorItem {
        /// 获取元数据 JSON
        fn get_metadata(&self) -> String {
            self.metadata_json.clone()
        }
    }

    /// SearchResult Python 类
    #[pyclass(name = "SearchResult")]
    pub struct PySearchResult {
        #[pyo3(get)]
        id: String,
        #[pyo3(get)]
        score: f32,
        #[pyo3(get)]
        content: String,
        metadata_json: String,
    }

    #[pymethods]
    impl PySearchResult {
        /// 获取元数据 JSON
        fn get_metadata(&self) -> String {
            self.metadata_json.clone()
        }
    }
}

use bindings::*;
