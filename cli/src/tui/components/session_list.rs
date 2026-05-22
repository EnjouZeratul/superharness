//! 会话列表组件
//!
//! 支持会话显示、搜索、排序、筛选等功能。

use chrono::{DateTime, Local};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};
use serde::{Deserialize, Serialize};

/// 会话列表组件
pub struct SessionListComponent {
    /// 会话列表
    sessions: Vec<SessionInfo>,
    /// 当前选中的索引
    selected_idx: usize,
    /// 滚动偏移
    scroll_offset: usize,
    /// 搜索词
    search_term: String,
    /// 筛选后的列表
    filtered_sessions: Vec<usize>, // 原始列表的索引
    /// 排序方式
    sort_by: SortBy,
    /// 筛选状态
    filter_status: Option<SessionStatus>,
}

/// 会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// 会话 ID
    pub id: String,
    /// 会话名称
    pub name: String,
    /// 创建时间
    pub created_at: DateTime<Local>,
    /// 更新时间
    pub updated_at: DateTime<Local>,
    /// 会话状态
    pub status: SessionStatus,
    /// 消息数量
    pub message_count: usize,
    /// Token 使用量
    pub tokens_used: u64,
    /// 标签
    pub tags: Vec<String>,
    /// 最后一条消息摘要
    pub last_message: Option<String>,
}

/// 会话状态
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Archived,
    Completed,
    Error,
}

impl SessionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            SessionStatus::Active => "active",
            SessionStatus::Archived => "archived",
            SessionStatus::Completed => "completed",
            SessionStatus::Error => "error",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            SessionStatus::Active => Color::Green,
            SessionStatus::Archived => Color::Gray,
            SessionStatus::Completed => Color::Blue,
            SessionStatus::Error => Color::Red,
        }
    }
}

/// 排序方式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortBy {
    UpdatedTime,
    CreatedTime,
    Name,
    MessageCount,
    TokensUsed,
}

impl SessionListComponent {
    /// 创建新的会话列表组件
    pub fn new() -> Self {
        Self {
            sessions: Vec::new(),
            selected_idx: 0,
            scroll_offset: 0,
            search_term: String::new(),
            filtered_sessions: Vec::new(),
            sort_by: SortBy::UpdatedTime,
            filter_status: None,
        }
    }

    /// 设置会话列表
    pub fn set_sessions(&mut self, sessions: Vec<SessionInfo>) {
        self.sessions = sessions;
        self.apply_filters();
    }

    /// 添加会话
    pub fn add_session(&mut self, session: SessionInfo) {
        self.sessions.push(session);
        self.apply_filters();
    }

    /// 移除会话
    pub fn remove_session(&mut self, id: &str) {
        self.sessions.retain(|s| s.id != id);
        self.apply_filters();
        if self.selected_idx >= self.filtered_sessions.len() && self.selected_idx > 0 {
            self.selected_idx -= 1;
        }
    }

    /// 获取选中的会话
    pub fn get_selected(&self) -> Option<&SessionInfo> {
        self.filtered_sessions
            .get(self.selected_idx)
            .and_then(|&idx| self.sessions.get(idx))
    }

    /// 获取选中会话 ID
    pub fn get_selected_id(&self) -> Option<&str> {
        self.get_selected().map(|s| s.id.as_str())
    }

    /// 选择上一项
    pub fn select_previous(&mut self) {
        if self.selected_idx > 0 {
            self.selected_idx -= 1;
        }
        self.ensure_selection_visible();
    }

    /// 选择下一项
    pub fn select_next(&mut self) {
        if self.selected_idx + 1 < self.filtered_sessions.len() {
            self.selected_idx += 1;
        }
        self.ensure_selection_visible();
    }

    /// 选择第一项
    pub fn select_first(&mut self) {
        self.selected_idx = 0;
        self.scroll_offset = 0;
    }

    /// 选择最后一项
    pub fn select_last(&mut self) {
        if !self.filtered_sessions.is_empty() {
            self.selected_idx = self.filtered_sessions.len() - 1;
        }
    }

    /// 确保选中项可见
    fn ensure_selection_visible(&mut self) {
        let visible_count = 10; // 假设可见项数
        if self.selected_idx < self.scroll_offset {
            self.scroll_offset = self.selected_idx;
        } else if self.selected_idx >= self.scroll_offset + visible_count {
            self.scroll_offset = self.selected_idx - visible_count + 1;
        }
    }

    /// 设置搜索词
    pub fn set_search(&mut self, term: String) {
        self.search_term = term;
        self.apply_filters();
        if !self.filtered_sessions.is_empty() {
            self.selected_idx = 0;
        }
    }

    /// 设置排序方式
    pub fn set_sort(&mut self, sort_by: SortBy) {
        self.sort_by = sort_by;
        self.apply_filters();
    }

    /// 设置状态筛选
    pub fn set_filter_status(&mut self, status: Option<SessionStatus>) {
        self.filter_status = status;
        self.apply_filters();
    }

    /// 应用筛选和排序
    fn apply_filters(&mut self) {
        // 筛选
        let mut indices: Vec<usize> = self
            .sessions
            .iter()
            .enumerate()
            .filter(|(_idx, session)| {
                // 搜索词匹配
                let matches_search = self.search_term.is_empty()
                    || session
                        .name
                        .to_lowercase()
                        .contains(&self.search_term.to_lowercase())
                    || session
                        .id
                        .to_lowercase()
                        .contains(&self.search_term.to_lowercase())
                    || session
                        .tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&self.search_term.to_lowercase()));

                // 状态匹配
                let matches_status =
                    self.filter_status.is_none() || session.status == self.filter_status.unwrap();

                matches_search && matches_status
            })
            .map(|(idx, _)| idx)
            .collect();

        // 排序
        indices.sort_by(|&a, &b| {
            let session_a = &self.sessions[a];
            let session_b = &self.sessions[b];

            match self.sort_by {
                SortBy::UpdatedTime => session_b.updated_at.cmp(&session_a.updated_at),
                SortBy::CreatedTime => session_b.created_at.cmp(&session_a.created_at),
                SortBy::Name => session_a.name.cmp(&session_b.name),
                SortBy::MessageCount => session_b.message_count.cmp(&session_a.message_count),
                SortBy::TokensUsed => session_b.tokens_used.cmp(&session_a.tokens_used),
            }
        });

        self.filtered_sessions = indices;
    }

    /// 会话总数
    pub fn total_count(&self) -> usize {
        self.sessions.len()
    }

    /// 筛选后数量
    pub fn filtered_count(&self) -> usize {
        self.filtered_sessions.len()
    }

    /// 渲染组件
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let title = format!(
            " Sessions ({}/{}) {} ",
            self.filtered_count(),
            self.total_count(),
            if self.search_term.is_empty() {
                String::new()
            } else {
                format!("[search: {}]", self.search_term)
            }
        );

        let block = Block::default().title(title).borders(Borders::ALL);

        let inner = block.inner(area);
        f.render_widget(block, area);

        let visible_height = inner.height as usize;
        let items: Vec<ListItem> = self
            .filtered_sessions
            .iter()
            .skip(self.scroll_offset)
            .take(visible_height)
            .enumerate()
            .map(|(display_idx, &session_idx)| {
                let session = &self.sessions[session_idx];
                let actual_idx = self.scroll_offset + display_idx;
                let is_selected = actual_idx == self.selected_idx;

                let status_color = session.status.color();
                let style = if is_selected {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                // 格式化时间
                let time_str = session.updated_at.format("%m/%d %H:%M").to_string();

                // 构建行
                let spans = vec![
                    Span::styled("● ".to_string(), Style::default().fg(status_color)),
                    Span::styled(
                        format!("{:20}", session.name.chars().take(20).collect::<String>()),
                        style,
                    ),
                    Span::styled(
                        format!(" {} ", time_str),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!(
                            "({} msg, {} tok)",
                            session.message_count, session.tokens_used
                        ),
                        Style::default().fg(Color::Gray),
                    ),
                ];

                ListItem::new(Line::from(spans))
            })
            .collect();

        let list = List::new(items);
        f.render_widget(list, inner);

        // 渲染滚动条
        if self.filtered_sessions.len() > visible_height {
            let scrollbar = Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight);

            let mut scrollbar_state = ScrollbarState::new(self.filtered_sessions.len())
                .position(self.scroll_offset)
                .viewport_content_length(visible_height);

            f.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
    }

    /// 通过 ID 选择会话
    pub fn select_by_id(&mut self, id: &str) {
        for (idx, &session_idx) in self.filtered_sessions.iter().enumerate() {
            if self.sessions[session_idx].id == id {
                self.selected_idx = idx;
                self.ensure_selection_visible();
                break;
            }
        }
    }

    /// 清空列表
    pub fn clear(&mut self) {
        self.sessions.clear();
        self.filtered_sessions.clear();
        self.selected_idx = 0;
        self.scroll_offset = 0;
    }

    /// 更新会话信息
    pub fn update_session(&mut self, session: SessionInfo) {
        if let Some(existing) = self.sessions.iter_mut().find(|s| s.id == session.id) {
            *existing = session;
            self.apply_filters();
        }
    }
}

impl Default for SessionListComponent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_session(id: &str, name: &str) -> SessionInfo {
        SessionInfo {
            id: id.to_string(),
            name: name.to_string(),
            created_at: Local::now(),
            updated_at: Local::now(),
            status: SessionStatus::Active,
            message_count: 10,
            tokens_used: 1000,
            tags: vec![],
            last_message: Some("test".to_string()),
        }
    }

    #[test]
    fn test_session_list_creation() {
        let list = SessionListComponent::new();
        assert!(list.sessions.is_empty());
        assert_eq!(list.selected_idx, 0);
    }

    #[test]
    fn test_add_session() {
        let mut list = SessionListComponent::new();
        list.add_session(create_test_session("1", "test"));
        assert_eq!(list.total_count(), 1);
    }

    #[test]
    fn test_selection() {
        let mut list = SessionListComponent::new();
        list.add_session(create_test_session("1", "a"));
        list.add_session(create_test_session("2", "b"));

        list.select_next();
        assert_eq!(list.selected_idx, 1);

        list.select_previous();
        assert_eq!(list.selected_idx, 0);
    }

    #[test]
    fn test_search() {
        let mut list = SessionListComponent::new();
        list.add_session(create_test_session("1", "alpha"));
        list.add_session(create_test_session("2", "beta"));
        list.add_session(create_test_session("3", "alpha2"));

        list.set_search("alpha".to_string());
        assert_eq!(list.filtered_count(), 2);
    }

    #[test]
    fn test_remove_session() {
        let mut list = SessionListComponent::new();
        list.add_session(create_test_session("1", "a"));
        list.add_session(create_test_session("2", "b"));

        list.remove_session("1");
        assert_eq!(list.total_count(), 1);
    }
}
