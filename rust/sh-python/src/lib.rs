//! # Continuum Python Bindings
//!
//! Python bindings for Continuum.

use pyo3::prelude::*;

/// Python 模块定义
#[pymodule]
fn sh_python(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Layer 0
    m.add_class::<PySecurityGateway>()?;

    // Layer 1
    m.add_class::<PyLlmClient>()?;
    m.add_class::<PyCostTracker>()?;

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

    // Layer 4
    m.add_class::<PyMcpBridge>()?;
    m.add_class::<PyAuditLogger>()?;

    Ok(())
}

mod bindings {
    use super::*;
    use sh_layer3::ToolExecutor;
    use sh_layer2::CheckpointSystemTrait;

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

    #[pyclass(name = "LlmClient")]
    pub struct PyLlmClient {
        #[allow(dead_code)]
        inner: std::sync::Arc<tokio::sync::Mutex<Option<sh_layer1::LlmClient>>>,
    }

    #[pymethods]
    impl PyLlmClient {
        #[new]
        fn new() -> Self {
            Self {
                inner: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            }
        }

        fn is_connected(&self) -> bool {
            false
        }
    }

    #[pyclass(name = "CostTracker")]
    pub struct PyCostTracker {
        #[allow(dead_code)]
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

        fn total_cost(&self) -> f64 {
            0.0
        }
    }

    // ========================================================================
    // Layer 2: AgentRuntime, SessionManager, CheckpointSystem, Agent, Session
    // ========================================================================

    #[pyclass(name = "AgentRuntime")]
    pub struct PyAgentRuntime {
        #[allow(dead_code)]
        inner: std::sync::Arc<tokio::sync::Mutex<Option<sh_layer2::AgentRuntime>>>,
    }

    #[pymethods]
    impl PyAgentRuntime {
        #[new]
        fn new() -> Self {
            Self {
                inner: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            }
        }
    }

    #[pyclass(name = "SessionManager")]
    pub struct PySessionManager {
        #[allow(dead_code)]
        inner: std::sync::Arc<
            tokio::sync::Mutex<Option<sh_layer2::session_manager::ConcurrentSessionManager>>,
        >,
    }

    #[pymethods]
    impl PySessionManager {
        #[new]
        fn new() -> Self {
            Self {
                inner: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            }
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
                .map(|s| std::path::PathBuf::from(s))
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
                messages: vec![serde_json::from_str(&data).unwrap_or(serde_json::json!({"content": data}))],
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
        fn load<'py>(&self, py: Python<'py>, session_id: &str, checkpoint_id: Option<&str>) -> PyResult<Option<String>> {
            let inner = self.inner.clone();
            let sid = sh_layer2::SessionId::from(session_id);
            let cid = checkpoint_id.map(|s| sh_layer2::CheckpointId(s.to_string()));

            pyo3_async_runtimes::tokio::run(py, async move {
                let writer = inner.lock().await;
                match writer.load(&sid, cid.as_ref()).await {
                    Ok(Some(data)) => {
                        let json = serde_json::to_string(&data.messages)
                            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e.to_string()))?;
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
        fn delete<'py>(&self, py: Python<'py>, session_id: &str, checkpoint_id: &str) -> PyResult<bool> {
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
            let args: serde_json::Value = serde_json::from_str(&args_json)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid JSON: {}", e)))?;

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
        fn read_file(&self, path: &str, offset: Option<usize>, limit: Option<usize>) -> PyResult<String> {
            let args = serde_json::json!({
                "path": path,
                "offset": offset,
                "limit": limit,
            });
            let args_json = serde_json::to_string(&args).unwrap();

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
        fn bash(&self, command: &str, timeout_ms: Option<u64>, working_dir: Option<&str>) -> PyResult<String> {
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
}

use bindings::*;