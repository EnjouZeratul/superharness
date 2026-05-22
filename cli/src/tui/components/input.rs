//! TUI 输入组件

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// 最大历史记录数量
const MAX_HISTORY: usize = 100;

/// 输入组件
pub struct InputComponent {
    /// 输入内容（可能多行）
    input: String,
    /// 光标位置
    cursor_position: usize,
    /// 是否处于输入模式
    is_focused: bool,
    /// 占位符文本
    placeholder: String,
    /// 输入历史记录
    history: Vec<String>,
    /// 当前历史索引（0表示最新，history.len()表示不在历史中）
    history_index: usize,
    /// 临时输入（在浏览历史时保存当前输入）
    temp_input: Option<String>,
    /// 多行模式
    multiline_mode: bool,
    /// 滚动偏移（多行时使用）
    scroll_offset: u16,
    /// 最大输入长度
    max_length: usize,
}

impl InputComponent {
    /// 创建新的输入组件
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor_position: 0,
            is_focused: true,
            placeholder: "Type your message... (Alt+Enter for newline)".to_string(),
            history: Vec::new(),
            history_index: 0,
            temp_input: None,
            multiline_mode: false,
            scroll_offset: 0,
            max_length: 10000,
        }
    }

    /// 添加字符
    pub fn push_char(&mut self, c: char) {
        if self.input.len() < self.max_length {
            self.input.insert(self.cursor_position, c);
            self.cursor_position += c.len_utf8();
        }
    }

    /// 插入换行符（多行模式）
    pub fn insert_newline(&mut self) {
        if self.input.len() < self.max_length {
            self.input.insert(self.cursor_position, '\n');
            self.cursor_position += 1;
            self.multiline_mode = true;
        }
    }

    /// 删除字符
    pub fn delete_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input.remove(self.cursor_position);
        }
    }

    /// 删除前一个单词
    pub fn delete_word(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        // 找到前一个单词的起始位置
        let mut pos = self.cursor_position;
        // 跳过空格
        while pos > 0
            && self
                .input
                .chars()
                .nth(pos - 1)
                .map(|c| c.is_whitespace())
                .unwrap_or(false)
        {
            pos -= 1;
        }
        // 删除单词字符
        while pos > 0
            && self
                .input
                .chars()
                .nth(pos - 1)
                .map(|c| !c.is_whitespace())
                .unwrap_or(false)
        {
            pos -= 1;
        }

        self.input.drain(pos..self.cursor_position);
        self.cursor_position = pos;
    }

    /// 删除到行首
    pub fn delete_to_start(&mut self) {
        // 找到当前行的起始位置
        let line_start = self.input[..self.cursor_position]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        self.input.drain(line_start..self.cursor_position);
        self.cursor_position = line_start;
    }

    /// 移动光标到左边
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.clamp_scroll_offset();
        }
    }

    /// 移动光标到右边
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
            self.clamp_scroll_offset();
        }
    }

    /// 移动光标到开始
    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
        self.scroll_offset = 0;
    }

    /// 移动光标到结束
    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.input.len();
        self.clamp_scroll_offset();
    }

    /// 移动到上一行
    pub fn move_cursor_up(&mut self) {
        if !self.multiline_mode && !self.input.contains('\n') {
            // 单行模式下，向上是历史记录
            self.history_prev();
            return;
        }

        // 找到当前行的起始和长度
        let line_start = self.input[..self.cursor_position]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        let _line_end = self.input[self.cursor_position..]
            .find('\n')
            .map(|pos| self.cursor_position + pos)
            .unwrap_or(self.input.len());
        let column = self.cursor_position - line_start;

        // 找到上一行
        if line_start > 0 {
            let prev_line_end = line_start - 1; // 跳过换行符
            let prev_line_start = self.input[..prev_line_end]
                .rfind('\n')
                .map(|pos| pos + 1)
                .unwrap_or(0);
            let prev_line_len = prev_line_end - prev_line_start;
            self.cursor_position = prev_line_start + column.min(prev_line_len);
            self.clamp_scroll_offset();
        }
    }

    /// 移动到下一行
    pub fn move_cursor_down(&mut self) {
        if !self.multiline_mode && !self.input.contains('\n') {
            // 单行模式下，向下是历史记录
            self.history_next();
            return;
        }

        // 找到当前行的起始
        let line_start = self.input[..self.cursor_position]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        let line_end = self.input[self.cursor_position..]
            .find('\n')
            .map(|pos| self.cursor_position + pos)
            .unwrap_or(self.input.len());
        let column = self.cursor_position - line_start;

        // 找到下一行
        if line_end < self.input.len() {
            let next_line_start = line_end + 1; // 跳过换行符
            let next_line_end = self.input[next_line_start..]
                .find('\n')
                .map(|pos| next_line_start + pos)
                .unwrap_or(self.input.len());
            let next_line_len = next_line_end - next_line_start;
            self.cursor_position = next_line_start + column.min(next_line_len);
            self.clamp_scroll_offset();
        }
    }

    /// 移动到单词开始
    pub fn move_word_left(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        let mut pos = self.cursor_position;
        // 跳过空格
        while pos > 0
            && self
                .input
                .chars()
                .nth(pos - 1)
                .map(|c| c.is_whitespace())
                .unwrap_or(false)
        {
            pos -= 1;
        }
        // 移动到单词开始
        while pos > 0
            && self
                .input
                .chars()
                .nth(pos - 1)
                .map(|c| !c.is_whitespace())
                .unwrap_or(false)
        {
            pos -= 1;
        }
        self.cursor_position = pos;
    }

    /// 移动到单词结束
    pub fn move_word_right(&mut self) {
        if self.cursor_position >= self.input.len() {
            return;
        }

        let mut pos = self.cursor_position;
        // 跳过当前单词
        while pos < self.input.len()
            && self
                .input
                .chars()
                .nth(pos)
                .map(|c| !c.is_whitespace())
                .unwrap_or(false)
        {
            pos += 1;
        }
        // 跳过空格
        while pos < self.input.len()
            && self
                .input
                .chars()
                .nth(pos)
                .map(|c| c.is_whitespace())
                .unwrap_or(false)
        {
            pos += 1;
        }
        self.cursor_position = pos;
    }

    /// 获取输入内容
    pub fn get_input(&self) -> &str {
        &self.input
    }

    /// 清空输入
    pub fn clear(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
        self.scroll_offset = 0;
        self.multiline_mode = false;
    }

    /// 设置焦点
    pub fn set_focused(&mut self, focused: bool) {
        self.is_focused = focused;
    }

    /// 是否聚焦
    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    /// 设置占位符
    pub fn set_placeholder(&mut self, placeholder: &str) {
        self.placeholder = placeholder.to_string();
    }

    /// 添加到历史记录
    pub fn add_to_history(&mut self, input: &str) {
        if input.is_empty() {
            return;
        }

        // 如果与最后一条历史相同，不重复添加
        if self.history.last().map(|h| h.as_str()) == Some(input) {
            return;
        }

        self.history.push(input.to_string());

        // 限制历史数量
        if self.history.len() > MAX_HISTORY {
            self.history.remove(0);
        }

        // 重置历史索引
        self.history_index = self.history.len();
        self.temp_input = None;
    }

    /// 上一个历史记录
    pub fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }

        // 第一次进入历史时保存当前输入
        if self.history_index == self.history.len() {
            self.temp_input = Some(self.input.clone());
        }

        if self.history_index > 0 {
            self.history_index -= 1;
            self.input = self.history[self.history_index].clone();
            self.cursor_position = self.input.len();
            self.multiline_mode = self.input.contains('\n');
            self.clamp_scroll_offset();
        }
    }

    /// 下一个历史记录
    pub fn history_next(&mut self) {
        if self.history.is_empty() || self.history_index >= self.history.len() {
            return;
        }

        self.history_index += 1;
        if self.history_index == self.history.len() {
            // 恢复临时输入
            if let Some(temp) = self.temp_input.take() {
                self.input = temp;
            } else {
                self.input.clear();
            }
        } else {
            self.input = self.history[self.history_index].clone();
        }
        self.cursor_position = self.input.len();
        self.multiline_mode = self.input.contains('\n');
        self.clamp_scroll_offset();
    }

    /// 获取历史记录数量
    pub fn history_count(&self) -> usize {
        self.history.len()
    }

    /// 切换多行模式
    pub fn toggle_multiline(&mut self) {
        self.multiline_mode = !self.multiline_mode;
    }

    /// 是否为多行模式
    pub fn is_multiline(&self) -> bool {
        self.multiline_mode || self.input.contains('\n')
    }

    /// 获取字符数量
    pub fn char_count(&self) -> usize {
        self.input.chars().count()
    }

    /// 获取字节数量
    pub fn byte_count(&self) -> usize {
        self.input.len()
    }

    /// 获取行数
    pub fn line_count(&self) -> usize {
        if self.input.is_empty() {
            0
        } else {
            self.input.lines().count()
        }
    }

    /// 获取单词数
    pub fn word_count(&self) -> usize {
        self.input.split_whitespace().count()
    }

    /// 设置最大输入长度
    pub fn set_max_length(&mut self, max: usize) {
        self.max_length = max;
    }

    /// 调整滚动偏移
    fn clamp_scroll_offset(&mut self) {
        // 简单实现：确保光标可见
        // 在多行模式下，需要更复杂的逻辑
    }

    /// 渲染组件
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let border_color = if self.is_focused {
            Color::Green
        } else {
            Color::Gray
        };

        // 构建标题（包含统计信息）
        let title = if self.input.is_empty() {
            " Input ".to_string()
        } else {
            let lines = self.line_count();
            let chars = self.char_count();
            if lines > 1 {
                format!(" Input ({} lines, {} chars) ", lines, chars)
            } else {
                format!(" Input ({} chars) ", chars)
            }
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        // 构建显示内容
        let display_lines: Vec<Line> = if self.input.is_empty() {
            vec![Line::from(Span::styled(
                &self.placeholder,
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            // 多行显示
            self.input
                .lines()
                .skip(self.scroll_offset as usize)
                .map(|line| Line::from(Span::raw(line)))
                .collect()
        };

        let paragraph = Paragraph::new(display_lines).block(block);

        f.render_widget(paragraph, area);

        // 设置光标位置
        if self.is_focused && !self.input.is_empty() {
            // 计算光标位置
            let (cursor_line, cursor_col) = self.get_cursor_line_col();

            // 检查是否在可见区域内
            if cursor_line >= self.scroll_offset as usize {
                let visible_line = cursor_line - self.scroll_offset as usize;
                let cursor_x = area.x + cursor_col as u16 + 1; // +1 for border
                let cursor_y = area.y + visible_line as u16 + 1; // +1 for border

                // 确保光标在区域内
                if cursor_y < area.y + area.height {
                    f.set_cursor_position((cursor_x, cursor_y));
                }
            }
        } else if self.is_focused {
            // 空输入时，光标在输入框开始
            let cursor_x = area.x + 1;
            let cursor_y = area.y + 1;
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }

    /// 获取光标所在行和列
    fn get_cursor_line_col(&self) -> (usize, usize) {
        let before_cursor = &self.input[..self.cursor_position];
        let line = before_cursor.matches('\n').count();
        let col = before_cursor
            .rfind('\n')
            .map(|pos| self.cursor_position - pos - 1)
            .unwrap_or(self.cursor_position);
        (line, col)
    }

    /// 设置输入内容
    pub fn set_input(&mut self, input: String) {
        self.input = input;
        self.cursor_position = self.input.len();
        self.multiline_mode = self.input.contains('\n');
        self.clamp_scroll_offset();
    }

    /// 撤销（从历史恢复）
    pub fn undo(&mut self) {
        // 简单实现：恢复临时输入
        if let Some(temp) = self.temp_input.take() {
            self.input = temp;
            self.cursor_position = self.input.len();
        }
    }
}

impl Default for InputComponent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_component_creation() {
        let input = InputComponent::new();
        assert!(input.get_input().is_empty());
        assert!(input.is_focused());
        assert_eq!(input.history_count(), 0);
    }

    #[test]
    fn test_push_char() {
        let mut input = InputComponent::new();
        input.push_char('a');
        assert_eq!(input.get_input(), "a");
        input.push_char('b');
        assert_eq!(input.get_input(), "ab");
    }

    #[test]
    fn test_delete_char() {
        let mut input = InputComponent::new();
        input.push_char('a');
        input.push_char('b');
        input.delete_char();
        assert_eq!(input.get_input(), "a");
    }

    #[test]
    fn test_cursor_movement() {
        let mut input = InputComponent::new();
        input.push_char('a');
        input.push_char('b');
        input.push_char('c');
        input.move_cursor_left();
        assert_eq!(input.cursor_position, 2);
        input.move_cursor_right();
        assert_eq!(input.cursor_position, 3);
        input.move_cursor_to_start();
        assert_eq!(input.cursor_position, 0);
        input.move_cursor_to_end();
        assert_eq!(input.cursor_position, 3);
    }

    #[test]
    fn test_clear() {
        let mut input = InputComponent::new();
        input.push_char('a');
        input.clear();
        assert!(input.get_input().is_empty());
    }

    #[test]
    fn test_multiline_insert() {
        let mut input = InputComponent::new();
        input.push_char('a');
        input.insert_newline();
        input.push_char('b');
        assert_eq!(input.get_input(), "a\nb");
        assert!(input.is_multiline());
        assert_eq!(input.line_count(), 2);
    }

    #[test]
    fn test_char_count() {
        let mut input = InputComponent::new();
        input.push_char('你');
        input.push_char('好');
        assert_eq!(input.char_count(), 2);
        assert_eq!(input.byte_count(), 6);
    }

    #[test]
    fn test_word_count() {
        let mut input = InputComponent::new();
        input.set_input("hello world test".to_string());
        assert_eq!(input.word_count(), 3);
    }

    #[test]
    fn test_line_count() {
        let mut input = InputComponent::new();
        input.set_input("line1\nline2\nline3".to_string());
        assert_eq!(input.line_count(), 3);
    }

    #[test]
    fn test_history_add() {
        let mut input = InputComponent::new();
        input.add_to_history("first");
        input.add_to_history("second");
        assert_eq!(input.history_count(), 2);
    }

    #[test]
    fn test_history_navigation() {
        let mut input = InputComponent::new();
        input.add_to_history("first");
        input.add_to_history("second");

        // 向上浏览历史
        input.history_prev();
        assert_eq!(input.get_input(), "second");

        input.history_prev();
        assert_eq!(input.get_input(), "first");

        // 向下浏览历史
        input.history_next();
        assert_eq!(input.get_input(), "second");

        // 到达最新的历史后，恢复空输入
        input.history_next();
        assert!(input.get_input().is_empty());
    }

    #[test]
    fn test_history_temp_input() {
        let mut input = InputComponent::new();
        input.set_input("current typing".to_string());
        input.add_to_history("history");

        // 保存当前输入
        input.history_prev();
        assert_eq!(input.get_input(), "history");

        // 恢复当前输入
        input.history_next();
        assert_eq!(input.get_input(), "current typing");
    }

    #[test]
    fn test_history_no_duplicate() {
        let mut input = InputComponent::new();
        input.add_to_history("same");
        input.add_to_history("same");
        assert_eq!(input.history_count(), 1);
    }

    #[test]
    fn test_delete_word() {
        let mut input = InputComponent::new();
        input.set_input("hello world test".to_string());
        input.move_cursor_to_end();
        input.delete_word();
        assert_eq!(input.get_input(), "hello world ");
    }

    #[test]
    fn test_move_word() {
        let mut input = InputComponent::new();
        input.set_input("hello world test".to_string());
        input.move_cursor_to_start();
        input.move_word_right();
        // 应该移动到 "world" 的开始
        assert_eq!(input.cursor_position, 6);
    }

    #[test]
    fn test_multiline_cursor_movement() {
        let mut input = InputComponent::new();
        input.set_input("line1\nline2\nline3".to_string());
        // "line1\nline2\nline3" length is 17
        // positions: 0-5: line1, 6: newline, 7-12: line2, 13: newline, 14-19: line3
        // Actually: line1(5) + \n(1) + line2(5) + \n(1) + line3(5) = 17

        input.move_cursor_to_end();
        assert_eq!(input.cursor_position, 17);

        // 向上移动 - 从line3到line2
        input.move_cursor_up();
        // 应该移动到第二行（cursor_position应该减小）
        assert!(input.cursor_position < 17);

        // 再向上 - 从line2到line1
        input.move_cursor_up();
        // 应该移动到第一行
        assert!(input.cursor_position < 12);
    }

    #[test]
    fn test_max_length() {
        let mut input = InputComponent::new();
        input.set_max_length(5);

        // 添加超过限制的字符
        for c in "abcdefghij".chars() {
            input.push_char(c);
        }

        assert_eq!(input.char_count(), 5);
    }

    #[test]
    fn test_default() {
        let input = InputComponent::default();
        assert!(input.get_input().is_empty());
    }

    #[test]
    fn test_set_input() {
        let mut input = InputComponent::new();
        input.set_input("test input".to_string());
        assert_eq!(input.get_input(), "test input");
        assert_eq!(input.cursor_position, 10);
    }

    #[test]
    fn test_multiline_navigation_with_history() {
        let mut input = InputComponent::new();
        input.add_to_history("history item");

        // 单行模式，上箭头应该是历史
        input.history_prev();
        assert_eq!(input.get_input(), "history item");
    }
}
