//! Token 统计组件
//!
//! 支持 Token 实时统计、历史图表、预算预警等功能。

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Sparkline},
    Frame,
};
use std::collections::VecDeque;

/// Token 统计组件
pub struct TokenStatsComponent {
    /// 总 Token 使用量
    total_tokens: u64,
    /// 输入 Token
    input_tokens: u64,
    /// 输出 Token
    output_tokens: u64,
    /// Token 预算
    budget: Option<u64>,
    /// 预算预警阈值（百分比）
    warning_threshold: u8,
    /// 预算超限阈值（百分比）
    critical_threshold: u8,
    /// Token 历史记录（用于图表）
    history: VecDeque<u64>,
    /// 历史记录最大长度
    max_history: usize,
    /// 当前会话开始时间
    session_start: std::time::Instant,
    /// 模型名称
    model_name: String,
    /// 每百万 Token 成本（美元）
    cost_per_million: f64,
    /// 是否显示成本
    show_cost: bool,
}

/// Token 使用快照
#[derive(Debug, Clone)]
pub struct TokenSnapshot {
    pub timestamp: std::time::Instant,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl TokenStatsComponent {
    /// 创建新的 Token 统计组件
    pub fn new() -> Self {
        Self {
            total_tokens: 0,
            input_tokens: 0,
            output_tokens: 0,
            budget: None,
            warning_threshold: 70,
            critical_threshold: 90,
            history: VecDeque::with_capacity(100),
            max_history: 100,
            session_start: std::time::Instant::now(),
            model_name: String::new(),
            cost_per_million: 0.0,
            show_cost: true,
        }
    }

    /// 设置预算
    pub fn set_budget(&mut self, budget: u64) {
        self.budget = Some(budget);
    }

    /// 清除预算
    pub fn clear_budget(&mut self) {
        self.budget = None;
    }

    /// 设置模型信息
    pub fn set_model(&mut self, name: &str, cost_per_million: f64) {
        self.model_name = name.to_string();
        self.cost_per_million = cost_per_million;
    }

    /// 添加 Token 使用
    pub fn add_tokens(&mut self, input: u64, output: u64) {
        self.input_tokens += input;
        self.output_tokens += output;
        self.total_tokens = self.input_tokens + self.output_tokens;

        // 记录历史
        self.history.push_back(output);
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    /// 重置统计
    pub fn reset(&mut self) {
        self.total_tokens = 0;
        self.input_tokens = 0;
        self.output_tokens = 0;
        self.history.clear();
        self.session_start = std::time::Instant::now();
    }

    /// 获取使用百分比
    pub fn usage_percentage(&self) -> Option<u8> {
        self.budget.map(|b| {
            if b == 0 {
                return 100;
            }
            ((self.total_tokens as f64 / b as f64) * 100.0).min(100.0) as u8
        })
    }

    /// 是否超预算警告
    pub fn is_warning(&self) -> bool {
        self.usage_percentage()
            .map(|p| p >= self.warning_threshold)
            .unwrap_or(false)
    }

    /// 是否超预算临界
    pub fn is_critical(&self) -> bool {
        self.usage_percentage()
            .map(|p| p >= self.critical_threshold)
            .unwrap_or(false)
    }

    /// 是否超预算
    pub fn is_over_budget(&self) -> bool {
        self.budget.map(|b| self.total_tokens > b).unwrap_or(false)
    }

    /// 计算成本
    pub fn calculate_cost(&self) -> f64 {
        (self.total_tokens as f64 / 1_000_000.0) * self.cost_per_million
    }

    /// 获取速率（Token/分钟）
    pub fn tokens_per_minute(&self) -> f64 {
        let elapsed = self.session_start.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            (self.total_tokens as f64) / (elapsed / 60.0)
        } else {
            0.0
        }
    }

    /// 预估剩余时间（分钟）
    pub fn estimated_remaining(&self) -> Option<f64> {
        self.budget.and_then(|b| {
            let remaining = b.saturating_sub(self.total_tokens);
            let rate = self.tokens_per_minute();
            if rate > 0.0 {
                Some((remaining as f64) / rate)
            } else {
                None
            }
        })
    }

    /// 获取总 Token
    pub fn total_tokens(&self) -> u64 {
        self.total_tokens
    }

    /// 获取输入 Token
    pub fn input_tokens(&self) -> u64 {
        self.input_tokens
    }

    /// 获取输出 Token
    pub fn output_tokens(&self) -> u64 {
        self.output_tokens
    }

    /// 渲染组件
    pub fn render(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // 进度条
                Constraint::Length(3), // 统计数字
                Constraint::Length(4), // 历史图表
            ])
            .split(area);

        self.render_progress(f, chunks[0]);
        self.render_stats(f, chunks[1]);
        self.render_history(f, chunks[2]);
    }

    /// 渲染进度条
    fn render_progress(&self, f: &mut Frame, area: Rect) {
        let (percentage, label) = if let Some(budget) = self.budget {
            let pct = self.usage_percentage().unwrap_or(0);
            let _remaining = budget.saturating_sub(self.total_tokens);
            (
                pct,
                format!("Budget: {}/{} ({pct}%)", self.total_tokens, budget),
            )
        } else {
            (0, format!("Tokens: {}", self.total_tokens))
        };

        let color = if self.is_over_budget() {
            Color::Red
        } else if self.is_critical() {
            Color::Magenta
        } else if self.is_warning() {
            Color::Yellow
        } else {
            Color::Green
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Token Usage"))
            .gauge_style(Style::default().fg(color))
            .percent(percentage as u16)
            .label(label);

        f.render_widget(gauge, area);
    }

    /// 渲染统计数字
    fn render_stats(&self, f: &mut Frame, area: Rect) {
        let cost = self.calculate_cost();
        let rate = self.tokens_per_minute();

        let mut spans = vec![
            Span::styled(
                format!(" In: {} ", self.input_tokens),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled(
                format!("Out: {} ", self.output_tokens),
                Style::default().fg(Color::Green),
            ),
        ];

        if self.show_cost && self.cost_per_million > 0.0 {
            spans.push(Span::styled(
                format!("Cost: ${:.4} ", cost),
                Style::default().fg(Color::Yellow),
            ));
        }

        if rate > 0.0 {
            spans.push(Span::styled(
                format!("Rate: {:.0}/min", rate),
                Style::default().fg(Color::Gray),
            ));
        }

        if let Some(remaining) = self.estimated_remaining() {
            spans.push(Span::styled(
                format!(" ETA: {:.0}min", remaining),
                Style::default().fg(Color::Magenta),
            ));
        }

        let paragraph =
            Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL));

        f.render_widget(paragraph, area);
    }

    /// 渲染历史图表
    fn render_history(&self, f: &mut Frame, area: Rect) {
        let data: Vec<u64> = self.history.iter().copied().collect();

        let sparkline = Sparkline::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("History ({} samples)", data.len())),
            )
            .data(&data)
            .style(Style::default().fg(Color::Cyan));

        f.render_widget(sparkline, area);
    }

    /// 渲染迷你版本（用于状态栏）
    pub fn render_mini(&self, f: &mut Frame, area: Rect) {
        let color = if self.is_over_budget() {
            Color::Red
        } else if self.is_critical() {
            Color::Magenta
        } else if self.is_warning() {
            Color::Yellow
        } else {
            Color::Green
        };

        let text = if let Some(pct) = self.usage_percentage() {
            format!(" Tok:{} {}% ", self.total_tokens, pct)
        } else {
            format!(" Tok:{} ", self.total_tokens)
        };

        let paragraph = Paragraph::new(Line::from(vec![Span::styled(
            text,
            Style::default().fg(color),
        )]));

        f.render_widget(paragraph, area);
    }

    /// 导出报告
    pub fn export_report(&self) -> TokenReport {
        TokenReport {
            total_tokens: self.total_tokens,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            budget: self.budget,
            usage_percentage: self.usage_percentage(),
            cost: self.calculate_cost(),
            tokens_per_minute: self.tokens_per_minute(),
            estimated_remaining_minutes: self.estimated_remaining(),
            model_name: self.model_name.clone(),
            duration_seconds: self.session_start.elapsed().as_secs(),
        }
    }
}

/// Token 报告
#[derive(Debug, Clone, serde::Serialize)]
pub struct TokenReport {
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub budget: Option<u64>,
    pub usage_percentage: Option<u8>,
    pub cost: f64,
    pub tokens_per_minute: f64,
    pub estimated_remaining_minutes: Option<f64>,
    pub model_name: String,
    pub duration_seconds: u64,
}

impl Default for TokenStatsComponent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_stats_creation() {
        let stats = TokenStatsComponent::new();
        assert_eq!(stats.total_tokens(), 0);
        assert_eq!(stats.input_tokens(), 0);
        assert_eq!(stats.output_tokens(), 0);
    }

    #[test]
    fn test_add_tokens() {
        let mut stats = TokenStatsComponent::new();
        stats.add_tokens(100, 50);
        assert_eq!(stats.input_tokens(), 100);
        assert_eq!(stats.output_tokens(), 50);
        assert_eq!(stats.total_tokens(), 150);
    }

    #[test]
    fn test_budget_warning() {
        let mut stats = TokenStatsComponent::new();
        stats.set_budget(1000);
        stats.add_tokens(600, 0); // 60%

        assert!(!stats.is_warning()); // 60% < 70%

        stats.add_tokens(200, 0); // 80%
        assert!(stats.is_warning()); // 80% >= 70%
    }

    #[test]
    fn test_budget_critical() {
        let mut stats = TokenStatsComponent::new();
        stats.set_budget(1000);
        stats.add_tokens(950, 0); // 95%

        assert!(stats.is_critical());
    }

    #[test]
    fn test_over_budget() {
        let mut stats = TokenStatsComponent::new();
        stats.set_budget(1000);
        stats.add_tokens(1200, 0);

        assert!(stats.is_over_budget());
    }

    #[test]
    fn test_cost_calculation() {
        let mut stats = TokenStatsComponent::new();
        stats.set_model("test", 3.0); // $3 per million tokens
        stats.add_tokens(500_000, 500_000);

        let cost = stats.calculate_cost();
        assert!((cost - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_reset() {
        let mut stats = TokenStatsComponent::new();
        stats.add_tokens(100, 50);
        stats.reset();
        assert_eq!(stats.total_tokens(), 0);
    }
}
