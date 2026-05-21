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

    #[pyclass(name = "CheckpointSystem")]
    pub struct PyCheckpointSystem {
        inner: std::sync::Arc<tokio::sync::Mutex<Option<sh_layer2::CheckpointWriter>>>,
    }

    #[pymethods]
    impl PyCheckpointSystem {
        #[new]
        fn new() -> Self {
            Self {
                inner: std::sync::Arc::new(tokio::sync::Mutex::new(None)),
            }
        }
    }

    /// Agent Python 绑定
    #[pyclass(name = "Agent")]
    pub struct PyAgent {
        id: String,
        state: std::sync::Arc<tokio::sync::Mutex<AgentState>>,
    }

    #[derive(Clone)]
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
                state: std::sync::Arc::new(tokio::sync::Mutex::new(AgentState::Idle)),
            }
        }

        fn id(&self) -> &str {
            &self.id
        }

        fn state(&self) -> String {
            match &*self.state.blocking_lock() {
                AgentState::Idle => "idle".to_string(),
                AgentState::Running => "running".to_string(),
                AgentState::Paused => "paused".to_string(),
                AgentState::Error => "error".to_string(),
            }
        }

        fn start(&self) -> PyResult<()> {
            let mut state = self.state.blocking_lock();
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
            let mut state = self.state.blocking_lock();
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
            let mut state = self.state.blocking_lock();
            *state = AgentState::Idle;
        }

        fn execute(&self, task: &str) -> PyResult<String> {
            let state = self.state.blocking_lock();
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

        fn id(&self) -> &str {
            &self.id
        }

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

    #[pyclass(name = "ToolExecutor")]
    pub struct PyToolExecutor;

    #[pymethods]
    impl PyToolExecutor {
        #[new]
        fn new() -> Self {
            Self
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
