//! # Network Tools
//!
//! 网络请求工具集。

use crate::builtin_tools::BuiltinTool;
use crate::types::{Layer3Result, ToolCategory};
use async_trait::async_trait;
use std::time::Duration;

/// HTTP Request Tool
pub struct HttpRequestTool;

#[async_trait]
impl BuiltinTool for HttpRequestTool {
    fn name(&self) -> &str {
        "http_request"
    }

    fn description(&self) -> &str {
        "Make an HTTP request to a URL."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to request"
                },
                "method": {
                    "type": "string",
                    "enum": ["GET", "POST", "PUT", "DELETE", "HEAD", "PATCH"],
                    "description": "HTTP method (default: GET)"
                },
                "headers": {
                    "type": "object",
                    "description": "Optional: request headers as key-value pairs"
                },
                "body": {
                    "type": "string",
                    "description": "Optional: request body (for POST/PUT/PATCH)"
                },
                "timeout": {
                    "type": "integer",
                    "description": "Optional: timeout in seconds (default: 30)"
                }
            },
            "required": ["url"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Network
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        let url = args["url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing url parameter"))?;

        let method = args["method"].as_str().unwrap_or("GET").to_uppercase();
        let timeout_secs = args["timeout"].as_u64().unwrap_or(30);

        // Build client
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent("Continuum/1.0")
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

        // Build request
        let mut request = match method.as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "HEAD" => client.head(url),
            "PATCH" => client.patch(url),
            _ => client.get(url),
        };

        // Add headers
        if let Some(headers) = args["headers"].as_object() {
            for (key, value) in headers {
                if let Some(val_str) = value.as_str() {
                    request = request.header(key, val_str);
                }
            }
        }

        // Add body
        if let Some(body) = args["body"].as_str() {
            request = request.body(body.to_string());
        }

        // Execute
        let response = request
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

        let status = response.status();
        let headers = response.headers().clone();

        // Get body (or empty for HEAD)
        let body = if method == "HEAD" {
            String::new()
        } else {
            response
                .text()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?
        };

        // Format result
        let mut result = format!(
            "Status: {} {}\n",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        );
        result.push_str("Headers:\n");
        for (name, value) in headers.iter() {
            result.push_str(&format!(
                "  {}: {}\n",
                name,
                value.to_str().unwrap_or("<binary>")
            ));
        }
        if !body.is_empty() {
            result.push_str("\nBody:\n");
            // Limit body display
            if body.len() > 5000 {
                result.push_str(&format!(
                    "{}...\n(truncated, {} bytes total)",
                    &body[..5000],
                    body.len()
                ));
            } else {
                result.push_str(&body);
            }
        }

        Ok(result)
    }
}

/// Web Fetch Tool - 简化的网页抓取
pub struct WebFetchTool;

#[async_trait]
impl BuiltinTool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }

    fn description(&self) -> &str {
        "Fetch and extract text content from a webpage."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch"
                },
                "selector": {
                    "type": "string",
                    "description": "Optional: CSS selector to extract specific content"
                }
            },
            "required": ["url"]
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Network
    }

    async fn execute(&self, args: serde_json::Value) -> Layer3Result<String> {
        let url = args["url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing url parameter"))?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Continuum/1.0")
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

        let response = client
            .get(url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("HTTP error: {}", response.status()));
        }

        let body = response
            .text()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

        // Simple HTML to text extraction (strip tags)
        let text = extract_text_from_html(&body);

        // Limit output
        if text.len() > 10000 {
            Ok(format!(
                "{}...\n\n(truncated, {} chars total)",
                &text[..10000],
                text.len()
            ))
        } else {
            Ok(text)
        }
    }
}

/// 简单的 HTML 文本提取
fn extract_text_from_html(html: &str) -> String {
    // 移除 script 和 style 标签内容
    let mut result = html.to_string();

    // 移除 script 标签
    while let Some(start) = result.find("<script") {
        if let Some(end) = result.find("</script>").map(|e| e + 9) {
            if end > start {
                result.replace_range(start..end, "");
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // 移除 style 标签
    while let Some(start) = result.find("<style") {
        if let Some(end) = result.find("</style>").map(|e| e + 8) {
            if end > start {
                result.replace_range(start..end, "");
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // 移除所有 HTML 标签
    let mut text = String::new();
    let mut in_tag = false;
    for c in result.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            text.push(c);
        }
    }

    // 清理多余空白
    text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_http_tool_category() {
        let tool = HttpRequestTool;
        assert_eq!(tool.category(), ToolCategory::Network);
    }

    #[test]
    fn test_web_fetch_tool_category() {
        let tool = WebFetchTool;
        assert_eq!(tool.category(), ToolCategory::Network);
    }

    #[test]
    fn test_extract_text_from_html() {
        let html = "<html><body><h1>Title</h1><p>Content here</p></body></html>";
        let text = extract_text_from_html(html);
        assert!(text.contains("Title"));
        assert!(text.contains("Content"));
    }

    #[tokio::test]
    async fn test_http_request_missing_url() {
        let tool = HttpRequestTool;
        let result = tool.execute(json!({})).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing url"));
    }
}
