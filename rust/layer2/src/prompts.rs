//! # Prompt Manager
//!
//! 提示词管理，支持模板化和动态生成。

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::Layer2Result;

/// 提示词模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    pub template: String,
    pub description: String,
    pub variables: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl PromptTemplate {
    pub fn new(name: impl Into<String>, template: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            template: template.into(),
            description: String::new(),
            variables: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// 渲染模板
    pub fn render(&self, context: &HashMap<String, String>) -> String {
        let mut result = self.template.clone();

        for var in &self.variables {
            if let Some(value) = context.get(var) {
                result = result.replace(&format!("{{{{{}}}}}", var), value);
            }
        }

        result
    }

    /// 提取模板中的变量
    pub fn extract_variables(&mut self) {
        use regex::Regex;

        let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
        self.variables = re
            .captures_iter(&self.template)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();
    }
}

/// 提示词管理器接口
pub trait PromptManagerTrait: Send + Sync {
    /// 注册模板
    fn register(&self, template: PromptTemplate) -> Layer2Result<()>;

    /// 注销模板
    fn unregister(&self, name: &str) -> Layer2Result<bool>;

    /// 获取模板
    fn get(&self, name: &str) -> Option<PromptTemplate>;

    /// 渲染提示词
    fn render(&self, name: &str, context: &HashMap<String, String>) -> Layer2Result<String>;

    /// 列出所有模板名称
    fn list(&self) -> Vec<String>;

    /// 模板数量
    fn count(&self) -> usize;
}

/// 提示词管理器实现
pub struct PromptManager {
    templates: RwLock<HashMap<String, PromptTemplate>>,
}

impl PromptManager {
    pub fn new() -> Self {
        Self {
            templates: RwLock::new(HashMap::new()),
        }
    }

    /// 创建带有默认模板的管理器
    pub fn with_defaults() -> Self {
        let manager = Self::new();

        // 添加默认模板
        manager.register_default_templates();

        manager
    }

    fn register_default_templates(&self) {
        let system = PromptTemplate::new(
            "system",
            "You are a helpful AI assistant. Be concise and accurate."
        )
        .with_description("Default system prompt");

        let code_review = PromptTemplate::new(
            "code_review",
            "Review the following code and provide feedback:\n\n{{code}}\n\nFocus on: {{focus_areas}}"
        )
        .with_description("Code review prompt template");

        let task = PromptTemplate::new(
            "task",
            "Task: {{task_description}}\n\nContext: {{context}}\n\nPlease complete this task."
        )
        .with_description("General task prompt template");

        let _ = self.register(system);
        let _ = self.register(code_review);
        let _ = self.register(task);
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl PromptManagerTrait for PromptManager {
    fn register(&self, template: PromptTemplate) -> Layer2Result<()> {
        let name = template.name.clone();
        self.templates.write().insert(name, template);
        Ok(())
    }

    fn unregister(&self, name: &str) -> Layer2Result<bool> {
        Ok(self.templates.write().remove(name).is_some())
    }

    fn get(&self, name: &str) -> Option<PromptTemplate> {
        self.templates.read().get(name).cloned()
    }

    fn render(&self, name: &str, context: &HashMap<String, String>) -> Layer2Result<String> {
        let templates = self.templates.read();

        let template = templates
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", name))?;

        Ok(template.render(context))
    }

    fn list(&self) -> Vec<String> {
        self.templates.read().keys().cloned().collect()
    }

    fn count(&self) -> usize {
        self.templates.read().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_template() {
        let mut template = PromptTemplate::new("test", "Hello {{name}}!");
        template.extract_variables();

        let mut context = HashMap::new();
        context.insert("name".to_string(), "World".to_string());

        let result = template.render(&context);
        assert_eq!(result, "Hello World!");
    }

    #[test]
    fn test_prompt_manager() {
        let manager = PromptManager::new();

        let template = PromptTemplate::new("test", "Test template");
        manager.register(template).unwrap();

        assert_eq!(manager.count(), 1);
        assert!(manager.get("test").is_some());
    }

    #[test]
    fn test_prompt_manager_defaults() {
        let manager = PromptManager::with_defaults();

        assert!(manager.get("system").is_some());
        assert!(manager.get("code_review").is_some());
        assert!(manager.get("task").is_some());
    }

    #[test]
    fn test_render_template() {
        let manager = PromptManager::with_defaults();

        let mut context = HashMap::new();
        context.insert("name".to_string(), "World".to_string());

        // 测试默认模板可以获取
        let templates = manager.list();
        assert!(!templates.is_empty());
    }
}
