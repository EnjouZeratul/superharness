//! TUI 聊天组件

use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::color_theme::ColorTheme;
use super::markdown_renderer::MarkdownRenderer;
use crate::tui::app::{Message, Role};

/// 聊天消息组件
pub struct ChatComponent {
    /// 消息列表
    messages: Vec<Message>,
    /// 滚动偏移
    scroll_offset: usize,
    /// 搜索关键词
    search_term: Option<String>,
    /// 搜索结果索引
    search_results: Vec<usize>,
    /// 当前搜索结果位置
    current_search_index: usize,
    /// Markdown 渲染器
    markdown_renderer: MarkdownRenderer,
    /// 当前主题
    theme: ColorTheme,
    /// 是否启用 Markdown 渲染
    enable_markdown: bool,
}

impl ChatComponent {
    /// 创建新的聊天组件
    pub fn new() -> Self {
        let theme = ColorTheme::dark();
        Self {
            messages: Vec::new(),
            scroll_offset: 0,
            search_term: None,
            search_results: Vec::new(),
            current_search_index: 0,
            markdown_renderer: MarkdownRenderer::new(theme.clone()),
            theme,
            enable_markdown: true,
        }
    }

    /// 设置主题
    pub fn set_theme(&mut self, theme: ColorTheme) {
        self.theme = theme.clone();
        self.markdown_renderer.set_theme(theme);
    }

    /// 切换 Markdown 渲染
    pub fn toggle_markdown(&mut self) {
        self.enable_markdown = !self.enable_markdown;
    }

    /// 设置 Markdown 渲染开关
    pub fn set_markdown_enabled(&mut self, enabled: bool) {
        self.enable_markdown = enabled;
    }

    /// 添加消息
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        // 如果有搜索词，重新搜索
        if self.search_term.is_some() {
            self.update_search_results();
        }
    }

    /// 向上滚动
    pub fn scroll_up(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    /// 向下滚动
    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(amount);
    }

    /// 滚动到底部
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    /// 设置搜索词
    pub fn set_search(&mut self, term: String) {
        self.search_term = Some(term);
        self.update_search_results();
        self.current_search_index = 0;
    }

    /// 清除搜索
    pub fn clear_search(&mut self) {
        self.search_term = None;
        self.search_results.clear();
        self.current_search_index = 0;
    }

    /// 更新搜索结果
    fn update_search_results(&mut self) {
        self.search_results.clear();
        if let Some(term) = &self.search_term {
            for (idx, msg) in self.messages.iter().enumerate() {
                if msg.content.to_lowercase().contains(&term.to_lowercase()) {
                    self.search_results.push(idx);
                }
            }
        }
    }

    /// 获取搜索结果数量
    pub fn search_count(&self) -> usize {
        self.search_results.len()
    }

    /// 下一个搜索结果
    pub fn next_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.current_search_index = (self.current_search_index + 1) % self.search_results.len();
        }
    }

    /// 上一个搜索结果
    pub fn prev_search_result(&mut self) {
        if !self.search_results.is_empty() {
            self.current_search_index = if self.current_search_index == 0 {
                self.search_results.len() - 1
            } else {
                self.current_search_index - 1
            };
        }
    }

    /// 导出消息为 JSON
    pub fn export_json(&self) -> String {
        let export_data: Vec<serde_json::Value> = self
            .messages
            .iter()
            .map(|m| {
                serde_json::json!({
                    "role": match m.role {
                        Role::User => "user",
                        Role::Assistant => "assistant",
                        Role::System => "system",
                    },
                    "content": m.content,
                })
            })
            .collect();
        serde_json::to_string_pretty(&export_data).unwrap_or_default()
    }

    /// 导出消息为纯文本
    pub fn export_text(&self) -> String {
        self.messages
            .iter()
            .map(|m| {
                let prefix = match m.role {
                    Role::User => "User",
                    Role::Assistant => "Assistant",
                    Role::System => "System",
                };
                format!("[{}] {}\n", prefix, m.content)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 获取消息内容（用于复制）
    pub fn get_all_content(&self) -> String {
        self.messages
            .iter()
            .map(|m| m.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// 渲染组件
    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let title = if let Some(term) = &self.search_term {
            format!(
                " Chat (search: \"{}\" - {} results) ",
                term,
                self.search_count()
            )
        } else {
            " Chat ".to_string()
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border));

        // 构建消息文本
        let mut lines: Vec<Line> = Vec::new();

        for (msg_idx, msg) in self.messages.iter().enumerate() {
            let (prefix, role_color) = match msg.role {
                Role::User => ("You", self.theme.user_message),
                Role::Assistant => ("Assistant", self.theme.assistant_message),
                Role::System => ("System", self.theme.system_message),
            };

            // 检查是否是当前搜索结果
            let is_current_search = self
                .search_results
                .get(self.current_search_index)
                .map(|idx| *idx == msg_idx)
                .unwrap_or(false);

            // 添加消息头
            let header_style = if is_current_search {
                Style::default().fg(self.theme.error_message)
            } else {
                Style::default()
                    .fg(role_color)
                    .add_modifier(ratatui::style::Modifier::BOLD)
            };
            lines.push(Line::from(vec![Span::styled(
                format!("[{}] ", prefix),
                header_style,
            )]));

            // 添加消息内容
            if self.enable_markdown && msg.role == Role::Assistant {
                // 使用 Markdown 渲染助手消息
                let md_lines = self.markdown_renderer.render(&msg.content);
                for md_line in md_lines {
                    lines.push(md_line);
                }
            } else {
                // 普通文本渲染，带搜索高亮
                for line in msg.content.lines() {
                    if let Some(term) = &self.search_term {
                        // 高亮搜索词
                        let lower_line = line.to_lowercase();
                        let lower_term = term.to_lowercase();
                        if lower_line.contains(&lower_term) {
                            let mut spans: Vec<Span> = Vec::new();
                            let mut remaining = line;
                            while let Some(pos) = remaining.to_lowercase().find(&lower_term) {
                                if pos > 0 {
                                    spans.push(Span::raw(format!("  {}", &remaining[..pos])));
                                }
                                let match_len = term.len();
                                spans.push(Span::styled(
                                    &remaining[pos..pos + match_len],
                                    Style::default().fg(self.theme.highlight),
                                ));
                                remaining = &remaining[pos + match_len..];
                            }
                            if !remaining.is_empty() {
                                spans.push(Span::raw(remaining.to_string()));
                            }
                            lines.push(Line::from(spans));
                        } else {
                            lines.push(Line::from(Span::styled(
                                format!("  {}", line),
                                Style::default().fg(self.theme.foreground),
                            )));
                        }
                    } else {
                        lines.push(Line::from(Span::styled(
                            format!("  {}", line),
                            Style::default().fg(self.theme.foreground),
                        )));
                    }
                }
            }

            lines.push(Line::from(Span::raw(""))); // 空行分隔
        }

        let text = Text::from(lines);

        let paragraph = Paragraph::new(text)
            .block(block)
            .scroll((self.scroll_offset as u16, 0));

        f.render_widget(paragraph, area);
    }

    /// 清空消息
    pub fn clear(&mut self) {
        self.messages.clear();
        self.scroll_offset = 0;
        self.search_term = None;
        self.search_results.clear();
        self.current_search_index = 0;
    }

    /// 获取消息数量
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// 获取消息列表
    pub fn get_messages(&self) -> &[Message] {
        &self.messages
    }

    /// 追加内容到最后一条消息
    pub fn append_to_last_message(&mut self, content: &str) {
        if let Some(last) = self.messages.last_mut() {
            last.content.push_str(content);
        }
    }

    /// 是否有搜索词
    pub fn has_search(&self) -> bool {
        self.search_term.is_some()
    }
}

impl Default for ChatComponent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_component_creation() {
        let chat = ChatComponent::new();
        assert_eq!(chat.message_count(), 0);
        assert!(!chat.has_search());
    }

    #[test]
    fn test_add_message() {
        let mut chat = ChatComponent::new();
        chat.add_message(Message {
            role: Role::User,
            content: "Hello".to_string(),
        });
        assert_eq!(chat.message_count(), 1);
    }

    #[test]
    fn test_clear_messages() {
        let mut chat = ChatComponent::new();
        chat.add_message(Message {
            role: Role::User,
            content: "Hello".to_string(),
        });
        chat.clear();
        assert_eq!(chat.message_count(), 0);
    }

    #[test]
    fn test_append_to_last_message() {
        let mut chat = ChatComponent::new();
        chat.add_message(Message {
            role: Role::Assistant,
            content: "Hello".to_string(),
        });
        chat.append_to_last_message(" World");
        assert_eq!(chat.message_count(), 1);
    }

    #[test]
    fn test_append_to_empty_chat() {
        let mut chat = ChatComponent::new();
        chat.append_to_last_message("test");
        assert_eq!(chat.message_count(), 0);
    }

    #[test]
    fn test_scroll_up() {
        let mut chat = ChatComponent::new();
        chat.scroll_up(5);
    }

    #[test]
    fn test_scroll_down() {
        let mut chat = ChatComponent::new();
        chat.scroll_down(5);
    }

    #[test]
    fn test_default() {
        let chat = ChatComponent::default();
        assert_eq!(chat.message_count(), 0);
    }

    #[test]
    fn test_set_search() {
        let mut chat = ChatComponent::new();
        chat.add_message(Message {
            role: Role::User,
            content: "Hello World".to_string(),
        });
        chat.set_search("Hello".to_string());
        assert!(chat.has_search());
        assert_eq!(chat.search_count(), 1);
    }

    #[test]
    fn test_clear_search() {
        let mut chat = ChatComponent::new();
        chat.set_search("test".to_string());
        chat.clear_search();
        assert!(!chat.has_search());
    }

    #[test]
    fn test_search_not_found() {
        let mut chat = ChatComponent::new();
        chat.add_message(Message {
            role: Role::User,
            content: "Hello".to_string(),
        });
        chat.set_search("xyz".to_string());
        assert_eq!(chat.search_count(), 0);
    }

    #[test]
    fn test_next_search_result() {
        let mut chat = ChatComponent::new();
        chat.add_message(Message {
            role: Role::User,
            content: "Hello World Hello".to_string(),
        });
        chat.set_search("hello".to_string());
        chat.next_search_result();
        // Should cycle
        assert!(chat.search_count() > 0);
    }

    #[test]
    fn test_prev_search_result() {
        let mut chat = ChatComponent::new();
        chat.add_message(Message {
            role: Role::User,
            content: "Hello World Hello".to_string(),
        });
        chat.set_search("hello".to_string());
        chat.prev_search_result();
        assert!(chat.search_count() > 0);
    }

    #[test]
    fn test_export_json() {
        let mut chat = ChatComponent::new();
        chat.add_message(Message {
            role: Role::User,
            content: "Hello".to_string(),
        });
        let json = chat.export_json();
        assert!(json.contains("Hello"));
        assert!(json.contains("user"));
    }

    #[test]
    fn test_export_text() {
        let mut chat = ChatComponent::new();
        chat.add_message(Message {
            role: Role::Assistant,
            content: "Response".to_string(),
        });
        let text = chat.export_text();
        assert!(text.contains("Assistant"));
        assert!(text.contains("Response"));
    }

    #[test]
    fn test_get_all_content() {
        let mut chat = ChatComponent::new();
        chat.add_message(Message {
            role: Role::User,
            content: "Hello".to_string(),
        });
        chat.add_message(Message {
            role: Role::Assistant,
            content: "Hi".to_string(),
        });
        let content = chat.get_all_content();
        assert!(content.contains("Hello"));
        assert!(content.contains("Hi"));
    }

    #[test]
    fn test_scroll_to_bottom() {
        let mut chat = ChatComponent::new();
        chat.scroll_down(10);
        chat.scroll_to_bottom();
        assert_eq!(chat.scroll_offset, 0);
    }

    #[test]
    fn test_set_theme() {
        let mut chat = ChatComponent::new();
        chat.set_theme(ColorTheme::light());
        // Theme should be changed (no direct assertion possible)
    }

    #[test]
    fn test_toggle_markdown() {
        let mut chat = ChatComponent::new();
        assert!(chat.enable_markdown);
        chat.toggle_markdown();
        assert!(!chat.enable_markdown);
        chat.toggle_markdown();
        assert!(chat.enable_markdown);
    }

    #[test]
    fn test_set_markdown_enabled() {
        let mut chat = ChatComponent::new();
        chat.set_markdown_enabled(false);
        assert!(!chat.enable_markdown);
        chat.set_markdown_enabled(true);
        assert!(chat.enable_markdown);
    }
}
