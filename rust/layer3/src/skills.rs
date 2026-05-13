//! # Skills
//!
//! Skills 模块：可复用的能力模块。

use crate::types::{Layer3Result, ToolRequest, ToolResponse};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Skill trait
///
/// 定义可复用能力模块的接口。
#[async_trait]
pub trait Skill: Send + Sync {
    /// Skill 名称
    fn name(&self) -> &str;

    /// Skill 描述
    fn description(&self) -> &str;

    /// Skill 版本
    fn version(&self) -> &str {
        "1.0.0"
    }

    /// 执行 Skill
    async fn execute(&self, input: SkillInput) -> Layer3Result<SkillOutput>;

    /// Skill 所需的工具列表
    fn required_tools(&self) -> Vec<String>;

    /// Skill 所需的权限
    fn required_permissions(&self) -> Vec<String>;

    /// 获取 Skill 配置 Schema
    fn config_schema(&self) -> serde_json::Value;
}

/// Skill 输入
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInput {
    /// 输入参数
    pub params: serde_json::Value,
    /// 上下文信息
    pub context: SkillContext,
}

/// Skill 输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillOutput {
    /// 输出结果
    pub result: serde_json::Value,
    /// 状态
    pub status: SkillStatus,
    /// 生成的工具调用（可选）
    pub tool_calls: Vec<ToolRequest>,
    /// 工具结果（可选）
    pub tool_results: Vec<ToolResponse>,
}

/// Skill 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillStatus {
    Success,
    Failed,
    Pending,
    NeedsApproval,
    ToolCalling,
}

/// Skill 执行上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillContext {
    /// 会话 ID
    pub session_id: String,
    /// 用户 ID
    pub user_id: Option<String>,
    /// 工作目录
    pub working_dir: String,
    /// 环境变量
    pub env_vars: std::collections::HashMap<String, String>,
}

impl Default for SkillContext {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            user_id: None,
            working_dir: ".".to_string(),
            env_vars: std::collections::HashMap::new(),
        }
    }
}

/// Skill 注册表
pub struct SkillRegistry {
    skills: std::collections::HashMap<String, Box<dyn Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, skill: Box<dyn Skill>) {
        self.skills.insert(skill.name().to_string(), skill);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Skill> {
        self.skills.get(name).map(|s| s.as_ref())
    }

    pub fn list(&self) -> Vec<&dyn Skill> {
        self.skills.values().map(|s| s.as_ref()).collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_context_default() {
        let ctx = SkillContext::default();
        assert_eq!(ctx.working_dir, ".");
    }

    #[test]
    fn test_skill_registry() {
        let registry = SkillRegistry::new();
        assert!(registry.list().is_empty());
    }
}
