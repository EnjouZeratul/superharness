//! # LSP Client
//!
//! Language Server Protocol 客户端。

use crate::query_engine::{SymbolInfo, SymbolKind};
use crate::types::{CodeLocation, CodeRange, Layer3Result};
use async_trait::async_trait;
use std::path::PathBuf;

/// LSP 客户端 trait
///
/// 与 Language Server 通信的接口。
#[async_trait]
pub trait LspClient: Send + Sync {
    /// 初始化连接
    async fn initialize(&self, root_uri: String) -> Layer3Result<LspCapabilities>;

    /// 关闭连接
    async fn shutdown(&self) -> Layer3Result<bool>;

    /// 打开文档
    async fn open_document(
        &self,
        uri: String,
        language_id: String,
        content: String,
    ) -> Layer3Result<bool>;

    /// 关闭文档
    async fn close_document(&self, uri: String) -> Layer3Result<bool>;

    /// 更新文档内容
    async fn change_document(&self, uri: String, changes: Vec<TextChange>) -> Layer3Result<bool>;

    /// 保存文档
    async fn save_document(&self, uri: String) -> Layer3Result<bool>;

    /// 检查是否就绪
    fn is_ready(&self) -> bool;

    /// 获取支持的语言
    fn supported_languages(&self) -> Vec<String>;
}

/// 文本变更
#[derive(Debug, Clone)]
pub struct TextChange {
    /// 变更范围
    pub range: CodeRange,
    /// 变更范围长度（旧文本长度）
    pub range_length: u32,
    /// 新文本
    pub text: String,
}

/// LSP 能力描述
#[derive(Debug, Clone)]
pub struct LspCapabilities {
    /// 支持文本同步
    pub text_document_sync: bool,
    /// 支持悬停
    pub hover_provider: bool,
    /// 支持定义跳转
    pub definition_provider: bool,
    /// 支持引用查找
    pub references_provider: bool,
    /// 支持实现查找
    pub implementation_provider: bool,
    /// 支持类型定义
    pub type_definition_provider: bool,
    /// 支持文档符号
    pub document_symbol_provider: bool,
    /// 支持工作区符号
    pub workspace_symbol_provider: bool,
    /// 支持重命名
    pub rename_provider: bool,
    /// 支持代码补全
    pub completion_provider: Option<CompletionOptions>,
    /// 支持签名帮助
    pub signature_help_provider: Option<SignatureHelpOptions>,
}

impl Default for LspCapabilities {
    fn default() -> Self {
        Self {
            text_document_sync: true,
            hover_provider: true,
            definition_provider: true,
            references_provider: true,
            implementation_provider: false,
            type_definition_provider: false,
            document_symbol_provider: true,
            workspace_symbol_provider: false,
            rename_provider: false,
            completion_provider: None,
            signature_help_provider: None,
        }
    }
}

/// 补全选项
#[derive(Debug, Clone)]
pub struct CompletionOptions {
    /// 触发字符
    pub trigger_characters: Vec<String>,
    /// 是否支持所有触发字符
    pub all_commit_characters: Vec<String>,
}

/// 签名帮助选项
#[derive(Debug, Clone)]
pub struct SignatureHelpOptions {
    /// 触发字符
    pub trigger_characters: Vec<String>,
    /// 重触发字符
    pub retrigger_characters: Vec<String>,
}

/// LSP 请求 trait
///
/// 发送 LSP 请求的具体方法。
#[async_trait]
pub trait LspRequester: LspClient {
    /// 发送定义请求
    async fn request_definition(
        &self,
        uri: String,
        position: Position,
    ) -> Layer3Result<Option<LocationLink>>;

    /// 发送引用请求
    async fn request_references(
        &self,
        uri: String,
        position: Position,
        include_declaration: bool,
    ) -> Layer3Result<Vec<LocationLink>>;

    /// 发送悬停请求
    async fn request_hover(
        &self,
        uri: String,
        position: Position,
    ) -> Layer3Result<Option<HoverResult>>;

    /// 发送文档符号请求
    async fn request_document_symbols(&self, uri: String) -> Layer3Result<Vec<DocumentSymbol>>;

    /// 发送工作区符号请求
    async fn request_workspace_symbols(&self, query: String) -> Layer3Result<Vec<SymbolInfo>>;
}

/// LSP 位置（行号从 0 开始）
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl From<CodeLocation> for Position {
    fn from(loc: CodeLocation) -> Self {
        Self {
            line: loc.line - 1, // LSP 行号从 0 开始
            character: loc.column - 1,
        }
    }
}

/// 位置链接
#[derive(Debug, Clone)]
pub struct LocationLink {
    /// 目标 URI
    pub target_uri: String,
    /// 目标范围
    pub target_range: Range,
    /// 目标选择范围
    pub target_selection_range: Range,
}

/// LSP 范围
#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// 悬停结果
#[derive(Debug, Clone)]
pub struct HoverResult {
    /// 内容
    pub contents: MarkupContent,
    /// 范围（可选）
    pub range: Option<Range>,
}

/// 标记内容
#[derive(Debug, Clone)]
pub struct MarkupContent {
    /// 内容类型
    pub kind: MarkupKind,
    /// 内容文本
    pub value: String,
}

/// 标记类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupKind {
    PlainText,
    Markdown,
}

/// 文档符号
#[derive(Debug, Clone)]
pub struct DocumentSymbol {
    /// 符号名称
    pub name: String,
    /// 详细信息
    pub detail: Option<String>,
    /// 符号类型
    pub kind: SymbolKind,
    /// 是否deprecated
    pub deprecated: bool,
    /// 范围
    pub range: Range,
    /// 选择范围
    pub selection_range: Range,
    /// 子符号
    pub children: Vec<DocumentSymbol>,
}

/// LSP 服务器管理器 trait
#[async_trait]
pub trait LspServerManager: Send + Sync {
    /// 启动指定语言的 LSP 服务器
    async fn start_server(&self, language: String, root_path: PathBuf) -> Layer3Result<String>;

    /// 停止 LSP 服务器
    async fn stop_server(&self, server_id: &str) -> Layer3Result<bool>;

    /// 获取服务器的客户端
    fn get_client(&self, server_id: &str) -> Option<&dyn LspClient>;

    /// 列出活跃服务器
    fn list_servers(&self) -> Vec<(String, String)>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_from_code_location() {
        let loc = CodeLocation {
            file: PathBuf::from("test.rs"),
            line: 10,
            column: 5,
        };
        let pos: Position = loc.into();
        assert_eq!(pos.line, 9);
        assert_eq!(pos.character, 4);
    }

    #[test]
    fn test_lsp_capabilities_default() {
        let caps = LspCapabilities::default();
        assert!(caps.hover_provider);
        assert!(caps.definition_provider);
    }
}
