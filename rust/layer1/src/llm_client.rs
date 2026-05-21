//! LLM 客户端模块
//!
//! 统一的 LLM API 客户端，支持多提供商。

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LLM 提供商类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmProvider {
    Anthropic,
    OpenAI,
    Gemini,
    Custom(String),
}

/// LLM 请求配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmRequestConfig {
    /// 模型名称
    pub model: String,
    /// 最大 token 数
    pub max_tokens: u32,
    /// 温度参数
    pub temperature: f32,
    /// 系统提示
    pub system_prompt: Option<String>,
    /// 停止词
    pub stop_sequences: Vec<String>,
}

impl Default for LlmRequestConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-6".to_string(),
            max_tokens: 4096,
            temperature: 0.7,
            system_prompt: None,
            stop_sequences: vec!["\n\n\n".to_string()],
        }
    }
}

/// LLM 响应
#[derive(Debug, Serialize, Deserialize)]
pub struct LlmResponse {
    /// 响应内容
    pub content: String,
    /// Token 使用情况
    pub usage: TokenUsage,
    /// 模型名称
    pub model: String,
    /// 响应 ID
    pub response_id: String,
}

/// Token 使用情况
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenUsage {
    /// 输入 token 数
    pub input_tokens: u32,
    /// 输出 token 数
    pub output_tokens: u32,
}

/// LLM 客户端 trait
#[async_trait]
pub trait LlmClientTrait {
    /// 发送请求并获取响应
    async fn send(&self, messages: Vec<Message>, config: &LlmRequestConfig) -> Result<LlmResponse>;

    /// 发送请求并流式获取响应
    async fn send_stream(
        &self,
        messages: Vec<Message>,
        config: &LlmRequestConfig,
    ) -> Result<impl futures::Stream<Item = Result<String>>>;
}

/// 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// LLM 客户端实现
pub struct LlmClient {
    /// HTTP 客户端
    client: Client,
    /// API 密钥
    api_key: String,
    /// 提供商
    provider: LlmProvider,
    /// API 基础 URL
    base_url: String,
}

impl LlmClient {
    pub fn new(provider: LlmProvider, api_key: String) -> Self {
        let base_url = match &provider {
            LlmProvider::Anthropic => "https://api.anthropic.com/v1".to_string(),
            LlmProvider::OpenAI => "https://api.openai.com/v1".to_string(),
            LlmProvider::Gemini => "https://generativelanguage.googleapis.com/v1".to_string(),
            LlmProvider::Custom(url) => url.clone(),
        };

        Self {
            client: Client::new(),
            api_key,
            provider,
            base_url,
        }
    }

    /// 创建客户端并指定自定义 base_url（覆盖 provider 默认值）
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }
}

#[async_trait]
impl LlmClientTrait for LlmClient {
    async fn send(&self, messages: Vec<Message>, config: &LlmRequestConfig) -> Result<LlmResponse> {
        // 根据提供商构建请求
        match self.provider {
            LlmProvider::Anthropic => self.send_anthropic(messages, config).await,
            LlmProvider::OpenAI => self.send_openai(messages, config).await,
            LlmProvider::Gemini => self.send_gemini(messages, config).await,
            LlmProvider::Custom(_) => {
                Err(anyhow!("Custom provider requires custom implementation"))
            }
        }
    }

    async fn send_stream(
        &self,
        _messages: Vec<Message>,
        _config: &LlmRequestConfig,
    ) -> Result<impl futures::Stream<Item = Result<String>>> {
        // TODO: 实现流式响应
        Ok(futures::stream::empty())
    }
}

impl LlmClient {
    async fn send_anthropic(
        &self,
        messages: Vec<Message>,
        config: &LlmRequestConfig,
    ) -> Result<LlmResponse> {
        let url = if self.base_url.ends_with("/v1") || self.base_url.ends_with("/v1/") {
            format!("{}/messages", self.base_url.trim_end_matches('/'))
        } else {
            format!("{}/v1/messages", self.base_url.trim_end_matches('/'))
        };

        let request_body = AnthropicRequest {
            model: config.model.clone(),
            max_tokens: config.max_tokens,
            messages: messages
                .into_iter()
                .map(|m| AnthropicMessage {
                    role: match m.role {
                        MessageRole::User => "user",
                        MessageRole::Assistant => "assistant",
                        MessageRole::System => "system",
                    },
                    content: AnthropicContent::Text(m.content),
                })
                .collect(),
            system: config.system_prompt.clone(),
            temperature: config.temperature,
        };

        let response = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request_body)
            .send()
            .await?;

        let response_text = response.text().await?;
        tracing::debug!("Anthropic API response: {}", response_text);

        let response_body: AnthropicResponse = serde_json::from_str(&response_text)?;

        Ok(LlmResponse {
            content: response_body
                .content
                .first()
                .map(|c| c.text.clone())
                .unwrap_or_default(),
            usage: TokenUsage {
                input_tokens: response_body.usage.input_tokens,
                output_tokens: response_body.usage.output_tokens,
            },
            model: response_body.model,
            response_id: response_body.id,
        })
    }

    async fn send_openai(
        &self,
        messages: Vec<Message>,
        config: &LlmRequestConfig,
    ) -> Result<LlmResponse> {
        let url = format!("{}/chat/completions", self.base_url);

        // 构建 OpenAI 格式的消息
        let mut openai_messages: Vec<OpenAiMessage> = Vec::new();

        // 添加系统提示
        if let Some(ref system) = config.system_prompt {
            openai_messages.push(OpenAiMessage {
                role: "system",
                content: system.clone(),
            });
        }

        // 添加用户/助手消息
        for m in messages {
            openai_messages.push(OpenAiMessage {
                role: match m.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => "system",
                },
                content: m.content,
            });
        }

        let request_body = OpenAiRequest {
            model: config.model.clone(),
            messages: openai_messages,
            max_tokens: Some(config.max_tokens),
            temperature: Some(config.temperature),
            stop: if config.stop_sequences.is_empty() {
                None
            } else {
                Some(config.stop_sequences.clone())
            },
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await?;

        let response_body: OpenAiResponse = response.json().await?;

        let choice = response_body
            .choices
            .first()
            .ok_or_else(|| anyhow!("No response choices"))?;

        Ok(LlmResponse {
            content: choice.message.content.clone(),
            usage: TokenUsage {
                input_tokens: response_body.usage.prompt_tokens,
                output_tokens: response_body.usage.completion_tokens,
            },
            model: response_body.model,
            response_id: response_body.id,
        })
    }

    async fn send_gemini(
        &self,
        messages: Vec<Message>,
        config: &LlmRequestConfig,
    ) -> Result<LlmResponse> {
        let url = format!(
            "{}/models/{}:generateContent?key={}",
            self.base_url, config.model, self.api_key
        );

        // 构建 Gemini 格式的消息
        let mut contents: Vec<GeminiContent> = Vec::new();

        // 添加系统提示作为单独的请求参数
        let system_instruction = config.system_prompt.clone();

        // 添加用户/助手消息
        for m in messages {
            contents.push(GeminiContent {
                role: match m.role {
                    MessageRole::User => "user".to_string(),
                    MessageRole::Assistant => "model".to_string(),
                    MessageRole::System => "user".to_string(), // Gemini 没有 system role，用 user 替代
                },
                parts: vec![GeminiPart { text: m.content }],
            });
        }

        let request_body = GeminiRequest {
            contents,
            generation_config: Some(GeminiGenerationConfig {
                max_output_tokens: Some(config.max_tokens),
                temperature: Some(config.temperature),
                stop_sequences: if config.stop_sequences.is_empty() {
                    None
                } else {
                    Some(config.stop_sequences.clone())
                },
            }),
            system_instruction: system_instruction.map(|s| GeminiSystemInstruction {
                parts: vec![GeminiPart { text: s }],
            }),
        };

        let response = self.client.post(&url).json(&request_body).send().await?;

        let response_body: GeminiResponse = response.json().await?;

        let candidate = response_body
            .candidates
            .first()
            .ok_or_else(|| anyhow!("No response candidates"))?;

        let content = candidate
            .content
            .parts
            .first()
            .map(|p| p.text.clone())
            .unwrap_or_default();

        Ok(LlmResponse {
            content,
            usage: TokenUsage {
                input_tokens: response_body.usage_metadata.prompt_token_count.unwrap_or(0),
                output_tokens: response_body
                    .usage_metadata
                    .candidates_token_count
                    .unwrap_or(0),
            },
            model: config.model.clone(),
            response_id: "".to_string(), // Gemini 不返回 response ID
        })
    }
}

// Anthropic API 结构
#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    system: Option<String>,
    temperature: f32,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: &'static str,
    content: AnthropicContent,
}

// Anthropic content - 可以是字符串或数组
#[derive(Serialize)]
#[serde(untagged)]
enum AnthropicContent {
    Text(String),
    Blocks(Vec<AnthropicContentBlock>),
}

#[derive(Serialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    content_type: String,  // "text"
    text: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    #[serde(default)]
    id: String,
    #[serde(default)]
    model: String,
    #[serde(default)]
    content: Vec<AnthropicContentResponse>,
    #[serde(default)]
    usage: AnthropicUsage,
    #[serde(default)]
    #[serde(rename = "type")]
    response_type: Option<String>,
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    stop_reason: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicContentResponse {
    #[serde(rename = "type", default)]
    content_type: String,
    #[serde(default)]
    text: String,
}

#[derive(Deserialize, Default)]
struct AnthropicUsage {
    #[serde(default)]
    input_tokens: u32,
    #[serde(default)]
    output_tokens: u32,
}

// OpenAI API 结构
#[derive(Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Serialize)]
struct OpenAiMessage {
    role: &'static str,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiResponse {
    id: String,
    model: String,
    choices: Vec<OpenAiChoice>,
    usage: OpenAiUsage,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
    finish_reason: String,
}

#[derive(Deserialize)]
struct OpenAiResponseMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

// Gemini API 结构
#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiSystemInstruction>,
}

#[derive(Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Serialize)]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
    usage_metadata: GeminiUsageMetadata,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResponse,
    finish_reason: String,
}

#[derive(Deserialize)]
struct GeminiContentResponse {
    parts: Vec<GeminiPartResponse>,
    role: String,
}

#[derive(Deserialize)]
struct GeminiPartResponse {
    text: String,
}

#[derive(Deserialize)]
struct GeminiUsageMetadata {
    prompt_token_count: Option<u32>,
    candidates_token_count: Option<u32>,
    total_token_count: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LlmRequestConfig::default();
        assert_eq!(config.model, "claude-sonnet-4-6");
        assert_eq!(config.max_tokens, 4096);
    }

    #[test]
    fn test_client_creation() {
        let client = LlmClient::new(LlmProvider::Anthropic, "test_key".to_string());
        assert_eq!(client.base_url, "https://api.anthropic.com/v1");
    }

    #[test]
    fn test_openai_client_creation() {
        let client = LlmClient::new(LlmProvider::OpenAI, "test_key".to_string());
        assert_eq!(client.base_url, "https://api.openai.com/v1");
    }

    #[test]
    fn test_gemini_client_creation() {
        let client = LlmClient::new(LlmProvider::Gemini, "test_key".to_string());
        assert_eq!(
            client.base_url,
            "https://generativelanguage.googleapis.com/v1"
        );
    }

    #[test]
    fn test_custom_provider() {
        let client = LlmClient::new(
            LlmProvider::Custom("https://custom.api.com/v1".to_string()),
            "test_key".to_string(),
        );
        assert_eq!(client.base_url, "https://custom.api.com/v1");
    }

    #[test]
    fn test_message_creation() {
        let message = Message {
            role: MessageRole::User,
            content: "Hello".to_string(),
        };
        assert_eq!(message.content, "Hello");
    }

    #[test]
    fn test_config_with_system_prompt() {
        let config = LlmRequestConfig {
            model: "gpt-4".to_string(),
            max_tokens: 8192,
            temperature: 0.5,
            system_prompt: Some("You are a helpful assistant".to_string()),
            stop_sequences: vec![],
        };
        assert_eq!(config.model, "gpt-4");
        assert!(config.system_prompt.is_some());
    }

    #[test]
    fn test_llm_response_creation() {
        let response = LlmResponse {
            content: "Hello".to_string(),
            usage: TokenUsage {
                input_tokens: 10,
                output_tokens: 5,
            },
            model: "gpt-4".to_string(),
            response_id: "resp_123".to_string(),
        };
        assert_eq!(response.content, "Hello");
        assert_eq!(response.usage.input_tokens, 10);
    }

    #[test]
    fn test_provider_serialization() {
        let provider = LlmProvider::Anthropic;
        let json = serde_json::to_string(&provider).unwrap();
        assert!(json.contains("Anthropic"));
    }

    #[test]
    fn test_message_role_serialization() {
        let role = MessageRole::User;
        let json = serde_json::to_string(&role).unwrap();
        assert!(json.contains("User"));
    }
}
