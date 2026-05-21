//! 测试配置模块
//!
//! 支持的环境变量（按优先级）:
//! 1. CONTINUUM_API_KEY / CONTINUUM_BASE_URL（推荐）
//! 2. CONTINUUM_API_KEY / CONTINUUM_BASE_URL（兼容）
//! 3. ANTHROPIC_API_KEY / ANTHROPIC_BASE_URL（兼容）

use std::env;
use std::path::Path;

/// 从 env 文件加载配置（不覆盖已存在的环境变量）
fn load_env_file(filepath: &Path) {
    if !filepath.exists() {
        return;
    }
    if let Ok(content) = std::fs::read_to_string(filepath) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                if env::var(key).is_err() {
                    env::set_var(key, value);
                }
            }
        }
    }
}

/// 加载环境配置
pub fn load_env() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let project_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(manifest_dir);

    load_env_file(&project_root.join(".env"));
    load_env_file(&project_root.join(".env.test"));
}

/// 获取 API 密钥（按优先级）
pub fn get_api_key() -> Option<String> {
    load_env();
    env::var("CONTINUUM_API_KEY")
        .ok()
        .or_else(|| env::var("CONTINUUM_API_KEY").ok())
        .or_else(|| env::var("ANTHROPIC_API_KEY").ok())
}

/// 获取 API 基础 URL（按优先级）
pub fn get_base_url() -> String {
    load_env();
    env::var("CONTINUUM_BASE_URL")
        .ok()
        .or_else(|| env::var("CONTINUUM_BASE_URL").ok())
        .or_else(|| env::var("ANTHROPIC_BASE_URL").ok())
        .unwrap_or_else(|| "https://api.anthropic.com".to_string())
}

/// 获取模型名称
pub fn get_model() -> String {
    load_env();
    env::var("CONTINUUM_MODEL")
        .ok()
        .or_else(|| env::var("CONTINUUM_MODEL").ok())
        .or_else(|| env::var("ANTHROPIC_MODEL").ok())
        .unwrap_or_else(|| "claude-sonnet-4-6".to_string())
}

/// 检查 API 是否可用
pub fn is_api_available() -> bool {
    match get_api_key() {
        None => false,
        Some(k) => {
            let lower = k.to_lowercase();
            !lower.contains("your-api-key")
                && !lower.contains("sk-test")
                && !lower.contains("placeholder")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_functions() {
        load_env();
        let _ = get_base_url();
        let _ = get_model();
    }
}