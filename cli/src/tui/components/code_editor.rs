//! 代码编辑器组件
//!
//! 支持代码编辑、撤销/重做、复制粘贴等功能。

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::VecDeque;

/// 代码编辑器组件
pub struct CodeEditorComponent {
    /// 编辑器内容（按行）
    lines: Vec<String>,
    /// 当前光标行
    cursor_line: usize,
    /// 当前光标列
    cursor_col: usize,
    /// 选区起始行
    selection_start_line: Option<usize>,
    /// 选区起始列
    selection_start_col: Option<usize>,
    /// 是否在选区模式
    selecting: bool,
    /// 撤销栈
    undo_stack: VecDeque<EditorState>,
    /// 重做栈
    redo_stack: VecDeque<EditorState>,
    /// 最大撤销数
    max_undo: usize,
    /// 剪贴板
    clipboard: String,
    /// 滚动偏移
    scroll_offset: usize,
    /// 编辑器模式
    mode: EditorMode,
    /// 缩进字符串
    indent_str: String,
    /// 文件路径
    file_path: Option<String>,
    /// 是否已修改
    modified: bool,
}

/// 编辑器模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorMode {
    /// 普通模式
    Normal,
    /// 插入模式
    Insert,
    /// 可视模式（选择）
    Visual,
    /// 命令模式
    Command,
}

/// 编辑器状态（用于撤销/重做）
#[derive(Debug, Clone)]
struct EditorState {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
}

impl CodeEditorComponent {
    /// 创建新的代码编辑器
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            selection_start_line: None,
            selection_start_col: None,
            selecting: false,
            undo_stack: VecDeque::with_capacity(100),
            redo_stack: VecDeque::with_capacity(100),
            max_undo: 100,
            clipboard: String::new(),
            scroll_offset: 0,
            mode: EditorMode::Normal,
            indent_str: "    ".to_string(), // 4空格缩进
            file_path: None,
            modified: false,
        }
    }

    /// 从内容创建编辑器
    pub fn from_content(content: &str) -> Self {
        let mut editor = Self::new();
        editor.lines = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };
        editor
    }

    /// 加载文件
    pub fn load_file(&mut self, path: &str, content: &str) {
        self.file_path = Some(path.to_string());
        self.lines = if content.is_empty() {
            vec![String::new()]
        } else {
            content.lines().map(|s| s.to_string()).collect()
        };
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
        self.modified = false;
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// 获取全部内容
    pub fn get_content(&self) -> String {
        self.lines.join("\n")
    }

    /// 保存状态（用于撤销）
    fn save_state(&mut self) {
        let state = EditorState {
            lines: self.lines.clone(),
            cursor_line: self.cursor_line,
            cursor_col: self.cursor_col,
        };

        self.undo_stack.push_back(state);
        if self.undo_stack.len() > self.max_undo {
            self.undo_stack.pop_front();
        }
        self.redo_stack.clear();
    }

    /// 撤销
    pub fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop_back() {
            self.redo_stack.push_back(EditorState {
                lines: self.lines.clone(),
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
            });
            self.lines = state.lines;
            self.cursor_line = state.cursor_line;
            self.cursor_col = state.cursor_col;
        }
    }

    /// 重做
    pub fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop_back() {
            self.undo_stack.push_back(EditorState {
                lines: self.lines.clone(),
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
            });
            self.lines = state.lines;
            self.cursor_line = state.cursor_line;
            self.cursor_col = state.cursor_col;
        }
    }

    /// 插入字符
    pub fn insert_char(&mut self, c: char) {
        self.save_state();
        self.modified = true;

        if c == '\n' {
            self.insert_newline();
        } else if let Some(line) = self.lines.get_mut(self.cursor_line) {
            line.insert(self.cursor_col, c);
            self.cursor_col += 1;
        }
    }

    /// 插入换行
    fn insert_newline(&mut self) {
        let current_line = self
            .lines
            .get(self.cursor_line)
            .cloned()
            .unwrap_or_default();
        let before_cursor: String = current_line.chars().take(self.cursor_col).collect();
        let after_cursor: String = current_line.chars().skip(self.cursor_col).collect();

        // 计算缩进
        let indent = self.get_line_indent(&before_cursor);

        self.lines[self.cursor_line] = before_cursor;
        self.lines
            .insert(self.cursor_line + 1, format!("{}{}", indent, after_cursor));
        self.cursor_line += 1;
        self.cursor_col = indent.len();
    }

    /// 获取行缩进
    fn get_line_indent(&self, line: &str) -> String {
        line.chars().take_while(|c| c.is_whitespace()).collect()
    }

    /// 删除字符
    pub fn delete_char(&mut self) {
        self.save_state();
        self.modified = true;

        if self.cursor_col > 0 {
            if let Some(line) = self.lines.get_mut(self.cursor_line) {
                line.remove(self.cursor_col - 1);
                self.cursor_col -= 1;
            }
        } else if self.cursor_line > 0 {
            // 删除换行，合并行
            let current_line = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            if let Some(prev_line) = self.lines.get_mut(self.cursor_line) {
                self.cursor_col = prev_line.len();
                prev_line.push_str(&current_line);
            }
        }
    }

    /// 删除光标后的字符
    pub fn delete_char_forward(&mut self) {
        self.save_state();
        self.modified = true;

        if let Some(line) = self.lines.get(self.cursor_line) {
            if self.cursor_col < line.len() {
                if let Some(line) = self.lines.get_mut(self.cursor_line) {
                    line.remove(self.cursor_col);
                }
            } else if self.cursor_line + 1 < self.lines.len() {
                let next_line = self.lines.remove(self.cursor_line + 1);
                if let Some(current) = self.lines.get_mut(self.cursor_line) {
                    current.push_str(&next_line);
                }
            }
        }
    }

    /// 删除整行
    pub fn delete_line(&mut self) {
        self.save_state();
        self.modified = true;

        if self.lines.len() > 1 {
            self.lines.remove(self.cursor_line);
            if self.cursor_line >= self.lines.len() {
                self.cursor_line = self.lines.len() - 1;
            }
            self.cursor_col = 0;
        } else {
            self.lines[0] = String::new();
            self.cursor_col = 0;
        }
    }

    /// 移动光标
    pub fn move_cursor(&mut self, direction: CursorDirection) {
        match direction {
            CursorDirection::Left => {
                if self.cursor_col > 0 {
                    self.cursor_col -= 1;
                } else if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_col = self
                        .lines
                        .get(self.cursor_line)
                        .map(|l| l.len())
                        .unwrap_or(0);
                }
            }
            CursorDirection::Right => {
                let line_len = self
                    .lines
                    .get(self.cursor_line)
                    .map(|l| l.len())
                    .unwrap_or(0);
                if self.cursor_col < line_len {
                    self.cursor_col += 1;
                } else if self.cursor_line + 1 < self.lines.len() {
                    self.cursor_line += 1;
                    self.cursor_col = 0;
                }
            }
            CursorDirection::Up => {
                if self.cursor_line > 0 {
                    self.cursor_line -= 1;
                    self.cursor_col = self.cursor_col.min(
                        self.lines
                            .get(self.cursor_line)
                            .map(|l| l.len())
                            .unwrap_or(0),
                    );
                }
            }
            CursorDirection::Down => {
                if self.cursor_line + 1 < self.lines.len() {
                    self.cursor_line += 1;
                    self.cursor_col = self.cursor_col.min(
                        self.lines
                            .get(self.cursor_line)
                            .map(|l| l.len())
                            .unwrap_or(0),
                    );
                }
            }
            CursorDirection::LineStart => {
                self.cursor_col = 0;
            }
            CursorDirection::LineEnd => {
                self.cursor_col = self
                    .lines
                    .get(self.cursor_line)
                    .map(|l| l.len())
                    .unwrap_or(0);
            }
            CursorDirection::FileStart => {
                self.cursor_line = 0;
                self.cursor_col = 0;
            }
            CursorDirection::FileEnd => {
                self.cursor_line = self.lines.len().saturating_sub(1);
                self.cursor_col = self
                    .lines
                    .get(self.cursor_line)
                    .map(|l| l.len())
                    .unwrap_or(0);
            }
            CursorDirection::WordForward => {
                self.move_word_forward();
            }
            CursorDirection::WordBackward => {
                self.move_word_backward();
            }
        }
        self.ensure_cursor_visible();
    }

    /// 向前移动一个单词
    fn move_word_forward(&mut self) {
        if let Some(line) = self.lines.get(self.cursor_line) {
            let mut col = self.cursor_col;
            let chars: Vec<char> = line.chars().collect();

            // 跳过当前单词
            while col < chars.len() && !chars[col].is_whitespace() {
                col += 1;
            }
            // 跳过空格
            while col < chars.len() && chars[col].is_whitespace() {
                col += 1;
            }

            if col < chars.len() {
                self.cursor_col = col;
            } else if self.cursor_line + 1 < self.lines.len() {
                self.cursor_line += 1;
                self.cursor_col = 0;
            }
        }
    }

    /// 向后移动一个单词
    fn move_word_backward(&mut self) {
        if self.cursor_col == 0 {
            if self.cursor_line > 0 {
                self.cursor_line -= 1;
                self.cursor_col = self
                    .lines
                    .get(self.cursor_line)
                    .map(|l| l.len())
                    .unwrap_or(0);
            }
            return;
        }

        if let Some(line) = self.lines.get(self.cursor_line) {
            let mut col = self.cursor_col;
            let chars: Vec<char> = line.chars().collect();

            // 跳过空格
            while col > 0 && chars[col - 1].is_whitespace() {
                col -= 1;
            }
            // 跳过单词
            while col > 0 && !chars[col - 1].is_whitespace() {
                col -= 1;
            }

            self.cursor_col = col;
        }
    }

    /// 确保光标可见
    fn ensure_cursor_visible(&mut self) {
        // 假设可见行数为20（实际应该从渲染参数获取）
        let visible_lines = 20;
        if self.cursor_line < self.scroll_offset {
            self.scroll_offset = self.cursor_line;
        } else if self.cursor_line >= self.scroll_offset + visible_lines {
            self.scroll_offset = self.cursor_line - visible_lines + 1;
        }
    }

    /// 缩进增加
    pub fn indent(&mut self) {
        self.save_state();
        self.modified = true;

        if let Some(line) = self.lines.get_mut(self.cursor_line) {
            line.insert_str(0, &self.indent_str);
            if self.cursor_col == 0 {
                self.cursor_col = self.indent_str.len();
            }
        }
    }

    /// 缩进减少
    pub fn unindent(&mut self) {
        self.save_state();
        self.modified = true;

        if let Some(line) = self.lines.get_mut(self.cursor_line) {
            let indent_len = self.indent_str.len();
            if line.starts_with(&self.indent_str) {
                line.drain(..indent_len);
                self.cursor_col = self.cursor_col.saturating_sub(indent_len);
            } else {
                // 删除开头的空格
                let spaces: usize = line.chars().take_while(|c| *c == ' ').count();
                if spaces > 0 {
                    line.drain(..spaces.min(indent_len));
                    self.cursor_col = self.cursor_col.saturating_sub(spaces.min(indent_len));
                }
            }
        }
    }

    /// 切换注释
    pub fn toggle_comment(&mut self, comment_prefix: &str) {
        self.save_state();
        self.modified = true;

        if let Some(line) = self.lines.get_mut(self.cursor_line) {
            let trimmed = line.trim_start();
            if trimmed.starts_with(comment_prefix) {
                // 取消注释
                let prefix_pos = line.find(comment_prefix).unwrap_or(0);
                line.replace_range(prefix_pos..prefix_pos + comment_prefix.len(), "");
                // 去除后面的空格
                if line[prefix_pos..].starts_with(' ') {
                    line.remove(prefix_pos);
                }
            } else {
                // 添加注释
                line.insert_str(0, &format!("{} ", comment_prefix));
            }
        }
    }

    /// 复制选中内容
    pub fn copy(&mut self) {
        if let (Some(start_line), Some(start_col)) =
            (self.selection_start_line, self.selection_start_col)
        {
            // 计算选区范围
            let (min_line, min_col, max_line, max_col) = if start_line < self.cursor_line
                || (start_line == self.cursor_line && start_col < self.cursor_col)
            {
                (start_line, start_col, self.cursor_line, self.cursor_col)
            } else {
                (self.cursor_line, self.cursor_col, start_line, start_col)
            };

            self.clipboard.clear();
            for line_num in min_line..=max_line {
                if let Some(line) = self.lines.get(line_num) {
                    if line_num == min_line && line_num == max_line {
                        self.clipboard.push_str(&line[min_col..max_col]);
                    } else if line_num == min_line {
                        self.clipboard.push_str(&line[min_col..]);
                        self.clipboard.push('\n');
                    } else if line_num == max_line {
                        self.clipboard.push_str(&line[..max_col]);
                    } else {
                        self.clipboard.push_str(line);
                        self.clipboard.push('\n');
                    }
                }
            }
        }
    }

    /// 粘贴
    pub fn paste(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }

        self.save_state();
        self.modified = true;

        let lines: Vec<&str> = self.clipboard.lines().collect();
        if lines.len() == 1 {
            // 单行粘贴
            if let Some(line) = self.lines.get_mut(self.cursor_line) {
                line.insert_str(self.cursor_col, lines[0]);
                self.cursor_col += lines[0].len();
            }
        } else {
            // 多行粘贴
            let current_line = self
                .lines
                .get(self.cursor_line)
                .cloned()
                .unwrap_or_default();
            let before: String = current_line.chars().take(self.cursor_col).collect();
            let after: String = current_line.chars().skip(self.cursor_col).collect();

            self.lines[self.cursor_line] = format!("{}{}", before, lines[0]);

            for (i, paste_line) in lines[1..].iter().enumerate() {
                self.lines
                    .insert(self.cursor_line + 1 + i, paste_line.to_string());
            }

            if let Some(last) = self.lines.get_mut(self.cursor_line + lines.len() - 1) {
                last.push_str(&after);
            }

            self.cursor_line += lines.len() - 1;
            self.cursor_col = self
                .lines
                .get(self.cursor_line)
                .map(|l| l.len())
                .unwrap_or(0);
        }
    }

    /// 开始选择
    pub fn start_selection(&mut self) {
        self.selection_start_line = Some(self.cursor_line);
        self.selection_start_col = Some(self.cursor_col);
        self.selecting = true;
    }

    /// 结束选择
    pub fn end_selection(&mut self) {
        self.selecting = false;
    }

    /// 清除选择
    pub fn clear_selection(&mut self) {
        self.selection_start_line = None;
        self.selection_start_col = None;
        self.selecting = false;
    }

    /// 删除选中内容
    pub fn delete_selection(&mut self) {
        if !self.selecting {
            return;
        }

        self.save_state();
        self.modified = true;

        if let (Some(start_line), Some(start_col)) =
            (self.selection_start_line, self.selection_start_col)
        {
            let (min_line, min_col, max_line, max_col) = if start_line < self.cursor_line
                || (start_line == self.cursor_line && start_col < self.cursor_col)
            {
                (start_line, start_col, self.cursor_line, self.cursor_col)
            } else {
                (self.cursor_line, self.cursor_col, start_line, start_col)
            };

            if min_line == max_line {
                if let Some(line) = self.lines.get_mut(min_line) {
                    line.replace_range(min_col..max_col, "");
                }
            } else {
                // 删除多行选区
                let first_part: String = self
                    .lines
                    .get(min_line)
                    .map(|l| l.chars().take(min_col).collect())
                    .unwrap_or_default();
                let last_part: String = self
                    .lines
                    .get(max_line)
                    .map(|l| l.chars().skip(max_col).collect())
                    .unwrap_or_default();

                self.lines[min_line] = format!("{}{}", first_part, last_part);
                self.lines.drain((min_line + 1)..=max_line);
            }

            self.cursor_line = min_line;
            self.cursor_col = min_col;
            self.clear_selection();
        }
    }

    /// 设置模式
    pub fn set_mode(&mut self, mode: EditorMode) {
        self.mode = mode;
    }

    /// 获取模式
    pub fn mode(&self) -> EditorMode {
        self.mode
    }

    /// 是否已修改
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// 获取文件路径
    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    /// 获取当前行号
    pub fn line_number(&self) -> usize {
        self.cursor_line + 1
    }

    /// 获取当前列号
    pub fn column_number(&self) -> usize {
        self.cursor_col + 1
    }

    /// 总行数
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }

    /// 渲染组件
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let mode_str = match self.mode {
            EditorMode::Normal => "NORMAL",
            EditorMode::Insert => "INSERT",
            EditorMode::Visual => "VISUAL",
            EditorMode::Command => "COMMAND",
        };

        let title = format!(
            " {} {} [{}] L{},C{} ",
            self.file_path.as_deref().unwrap_or("[No File]"),
            if self.modified { "[+]" } else { "" },
            mode_str,
            self.line_number(),
            self.column_number()
        );

        let block = Block::default().title(title).borders(Borders::ALL);

        let inner = block.inner(area);
        f.render_widget(block, area);

        // 计算可见行
        let visible_height = inner.height as usize;
        let end_line = (self.scroll_offset + visible_height).min(self.lines.len());

        let mut lines_to_render = Vec::new();

        for line_num in self.scroll_offset..end_line {
            if let Some(line) = self.lines.get(line_num) {
                let is_current_line = line_num == self.cursor_line;
                let line_num_str = format!("{:4} ", line_num + 1);

                let mut spans = vec![Span::styled(
                    line_num_str,
                    Style::default().fg(Color::DarkGray),
                )];

                // 渲染行内容
                for (col_idx, c) in line.chars().enumerate() {
                    let style = if is_current_line && col_idx == self.cursor_col {
                        Style::default().bg(Color::White).fg(Color::Black)
                    } else if is_current_line {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    };
                    spans.push(Span::styled(c.to_string(), style));
                }

                // 光标在行末
                if is_current_line && self.cursor_col >= line.len() {
                    spans.push(Span::styled(
                        " ",
                        Style::default().bg(Color::White).fg(Color::Black),
                    ));
                }

                lines_to_render.push(Line::from(spans));
            }
        }

        let paragraph = Paragraph::new(lines_to_render);
        f.render_widget(paragraph, inner);
    }
}

/// 光标移动方向
#[derive(Debug, Clone, Copy)]
pub enum CursorDirection {
    Left,
    Right,
    Up,
    Down,
    LineStart,
    LineEnd,
    FileStart,
    FileEnd,
    WordForward,
    WordBackward,
}

impl Default for CodeEditorComponent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_creation() {
        let editor = CodeEditorComponent::new();
        assert!(editor.lines.len() == 1);
        assert!(editor.lines[0].is_empty());
        assert_eq!(editor.cursor_line, 0);
        assert_eq!(editor.cursor_col, 0);
    }

    #[test]
    fn test_insert_char() {
        let mut editor = CodeEditorComponent::new();
        editor.insert_char('a');
        assert_eq!(editor.lines[0], "a");
        assert_eq!(editor.cursor_col, 1);
    }

    #[test]
    fn test_delete_char() {
        let mut editor = CodeEditorComponent::from_content("abc");
        editor.cursor_col = 2;
        editor.delete_char();
        assert_eq!(editor.lines[0], "ac");
    }

    #[test]
    fn test_insert_newline() {
        let mut editor = CodeEditorComponent::from_content("hello");
        editor.cursor_col = 3;
        editor.insert_char('\n');
        assert_eq!(editor.lines.len(), 2);
        assert_eq!(editor.lines[0], "hel");
        assert_eq!(editor.lines[1], "lo");
    }

    #[test]
    fn test_undo_redo() {
        let mut editor = CodeEditorComponent::new();
        editor.insert_char('a');
        editor.undo();
        assert!(editor.lines[0].is_empty());
        editor.redo();
        assert_eq!(editor.lines[0], "a");
    }

    #[test]
    fn test_cursor_movement() {
        let mut editor = CodeEditorComponent::from_content("hello\nworld");
        editor.move_cursor(CursorDirection::Down);
        assert_eq!(editor.cursor_line, 1);
        editor.move_cursor(CursorDirection::Up);
        assert_eq!(editor.cursor_line, 0);
    }
}
