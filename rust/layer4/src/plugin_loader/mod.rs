//! # Plugin Loader
//!
//! 插件动态加载和管理系统。

use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::types::Layer4Result;

/// 插件接口
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 插件名称
    fn name(&self) -> &str;

    /// 插件版本
    fn version(&self) -> &str;

    /// 插件描述
    fn description(&self) -> &str {
        ""
    }

    /// 依赖列表
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }

    /// 初始化插件
    async fn initialize(&self, context: &PluginContext) -> Layer4Result<()>;

    /// 执行插件
    async fn execute(&self, input: &serde_json::Value) -> Layer4Result<serde_json::Value>;

    /// 关闭插件
    async fn shutdown(&self) -> Layer4Result<()> {
        Ok(())
    }
}

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub entry_point: String,
}

impl Default for PluginMeta {
    fn default() -> Self {
        Self {
            name: "unknown".to_string(),
            version: "0.1.0".to_string(),
            author: "unknown".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
            entry_point: "main".to_string(),
        }
    }
}

/// 插件状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    Unloaded,
    Loaded,
    Initialized,
    Running,
    Error,
    Shutdown,
}

/// 插件信息
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub meta: PluginMeta,
    pub state: PluginState,
    pub path: std::path::PathBuf,
    pub loaded_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 插件上下文
#[derive(Debug, Clone)]
pub struct PluginContext {
    pub plugin_name: String,
    pub config: serde_json::Value,
    pub data_dir: std::path::PathBuf,
}

impl PluginContext {
    pub fn new(plugin_name: impl Into<String>, data_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            plugin_name: plugin_name.into(),
            config: serde_json::Value::Null,
            data_dir: data_dir.into(),
        }
    }

    pub fn with_config(mut self, config: serde_json::Value) -> Self {
        self.config = config;
        self
    }
}

/// 插件注册表
pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, PluginInfo>>,
    instances: RwLock<HashMap<String, Box<dyn Plugin>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            instances: RwLock::new(HashMap::new()),
        }
    }

    /// 注册插件
    pub fn register(&self, plugin: Box<dyn Plugin>, path: &Path) -> Layer4Result<()> {
        let name = plugin.name().to_string();
        let meta = PluginMeta {
            name: name.clone(),
            version: plugin.version().to_string(),
            description: plugin.description().to_string(),
            dependencies: plugin.dependencies().iter().map(|s| s.to_string()).collect(),
            ..Default::default()
        };

        let info = PluginInfo {
            meta,
            state: PluginState::Loaded,
            path: path.to_path_buf(),
            loaded_at: Some(chrono::Utc::now()),
        };

        self.plugins.write().insert(name.clone(), info);
        self.instances.write().insert(name, plugin);

        Ok(())
    }

    /// 注销插件
    pub fn unregister(&self, name: &str) -> Layer4Result<bool> {
        self.plugins.write().remove(name);
        Ok(self.instances.write().remove(name).is_some())
    }

    /// 获取插件信息
    pub fn get_info(&self, name: &str) -> Option<PluginInfo> {
        self.plugins.read().get(name).cloned()
    }

    /// 获取插件实例
    pub fn get(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        // 由于 Box 不能直接共享，这里返回 Option
        // 实际实现需要使用 Arc
        None
    }

    /// 列出所有插件
    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins.read().values().cloned().collect()
    }

    /// 更新插件状态
    pub fn update_state(&self, name: &str, state: PluginState) {
        if let Some(info) = self.plugins.write().get_mut(name) {
            info.state = state;
        }
    }

    /// 插件数量
    pub fn count(&self) -> usize {
        self.plugins.read().len()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 插件加载器
pub struct PluginLoader {
    registry: PluginRegistry,
    plugin_dir: std::path::PathBuf,
}

impl PluginLoader {
    /// 创建新的插件加载器
    pub fn new(plugin_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            registry: PluginRegistry::new(),
            plugin_dir: plugin_dir.into(),
        }
    }

    /// 使用默认目录创建
    pub fn with_default_dir() -> Self {
        Self::new("~/.superharness/plugins")
    }

    /// 加载单个插件（占位实现）
    pub async fn load(&self, path: &Path) -> Layer4Result<String> {
        // 实际实现需要动态加载 (wasm/dylib)
        // 这里是占位实现

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        tracing::info!("Plugin loaded (placeholder): {} from {:?}", name, path);

        Ok(name)
    }

    /// 加载目录中的所有插件
    pub async fn load_dir(&self) -> Layer4Result<Vec<String>> {
        let mut loaded = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&self.plugin_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() || path.extension().map(|e| e == "wasm").unwrap_or(false) {
                    if let Ok(name) = self.load(&path).await {
                        loaded.push(name);
                    }
                }
            }
        }

        Ok(loaded)
    }

    /// 获取插件
    pub fn get(&self, name: &str) -> Option<PluginInfo> {
        self.registry.get_info(name)
    }

    /// 初始化插件
    pub async fn initialize(&self, name: &str, context: &PluginContext) -> Layer4Result<()> {
        self.registry.update_state(name, PluginState::Initialized);
        Ok(())
    }

    /// 重新加载插件
    pub async fn reload(&self, name: &str) -> Layer4Result<()> {
        self.registry.update_state(name, PluginState::Loaded);
        Ok(())
    }

    /// 卸载插件
    pub async fn unload(&self, name: &str) -> Layer4Result<()> {
        self.registry.update_state(name, PluginState::Shutdown);
        self.registry.unregister(name)?;
        Ok(())
    }

    /// 列出所有插件
    pub fn list(&self) -> Vec<PluginInfo> {
        self.registry.list()
    }

    /// 插件数量
    pub fn count(&self) -> usize {
        self.registry.count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_registry_creation() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.count(), 0);
    }

    #[test]
    fn test_plugin_context_creation() {
        let ctx = PluginContext::new("test-plugin", "/tmp/plugins");
        assert_eq!(ctx.plugin_name, "test-plugin");
    }

    #[test]
    fn test_plugin_loader_creation() {
        let loader = PluginLoader::with_default_dir();
        assert_eq!(loader.count(), 0);
    }

    #[test]
    fn test_plugin_meta_default() {
        let meta = PluginMeta::default();
        assert_eq!(meta.name, "unknown");
        assert_eq!(meta.version, "0.1.0");
    }
}