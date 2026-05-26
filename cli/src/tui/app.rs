//! TUI 应用状态

use std::time::Instant;

/// TUI 应用状态
pub struct App {
    /// 应用是否运行
    pub running: bool,
    /// 当前输入
    pub input: String,
    /// 消息历史
    pub messages: Vec<Message>,
    /// 会话 ID
    pub session_id: Option<String>,
    /// 创建时间
    pub created_at: Instant,
    /// 调试模式
    pub debug_mode: bool,
}

/// 消息
#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// 消息角色
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
    System,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            input: String::new(),
            messages: Vec::new(),
            session_id: None,
            created_at: Instant::now(),
            debug_mode: false,
        }
    }

    /// 设置会话 ID
    pub fn set_session_id(&mut self, session_id: String) {
        self.session_id = Some(session_id);
    }

    /// 获取会话 ID
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// 添加消息
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// 获取运行时长
    pub fn elapsed(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// 清空消息
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    /// 切换调试模式
    pub fn toggle_debug_mode(&mut self) -> bool {
        self.debug_mode = !self.debug_mode;
        self.debug_mode
    }

    /// 设置调试模式
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert!(app.running);
        assert!(app.session_id.is_none());
        assert!(app.messages.is_empty());
    }

    #[test]
    fn test_set_session_id() {
        let mut app = App::new();
        app.set_session_id("test-session".to_string());
        assert!(app.session_id.is_some());
    }

    #[test]
    fn test_add_message() {
        let mut app = App::new();
        app.add_message(Message {
            role: Role::User,
            content: "Hello".to_string(),
        });
        assert_eq!(app.messages.len(), 1);
    }

    #[test]
    fn test_clear_messages() {
        let mut app = App::new();
        app.add_message(Message {
            role: Role::User,
            content: "Hello".to_string(),
        });
        app.clear_messages();
        assert!(app.messages.is_empty());
    }

    #[test]
    fn test_debug_mode_initial_state() {
        let app = App::new();
        assert!(!app.debug_mode);
    }

    #[test]
    fn test_toggle_debug_mode() {
        let mut app = App::new();
        let new_state = app.toggle_debug_mode();
        assert!(new_state);
        assert!(app.debug_mode);
        let new_state = app.toggle_debug_mode();
        assert!(!new_state);
        assert!(!app.debug_mode);
    }

    #[test]
    fn test_set_debug_mode() {
        let mut app = App::new();
        app.set_debug_mode(true);
        assert!(app.debug_mode);
        app.set_debug_mode(false);
        assert!(!app.debug_mode);
    }
}
