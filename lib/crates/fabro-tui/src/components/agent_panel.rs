use ratatui::layout::Rect;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::state::run_state::{ActiveRunState, ToolStatus};
use crate::theme;

/// Agent detail panel with viewport-based scrolling.
pub struct AgentPanel {
    /// First visible line (scroll offset).
    pub offset: usize,
    /// Total content lines from last render.
    pub total_lines: usize,
    /// Visible height from last render.
    pub viewport_height: usize,
    /// When true, auto-scroll to bottom on new content.
    pub auto_follow: bool,
}

impl Default for AgentPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentPanel {
    pub fn new() -> Self {
        Self {
            offset: 0,
            total_lines: 0,
            viewport_height: 0,
            auto_follow: true,
        }
    }

    /// Reset scroll offset (e.g. when switching selected stage).
    pub fn reset_scroll(&mut self) {
        self.offset = 0;
    }

    /// Scroll up by one page.
    pub fn scroll_up_page(&mut self) {
        self.auto_follow = false;
        let page = self.viewport_height.max(1);
        self.offset = self.offset.saturating_sub(page);
    }

    /// Scroll down by one page.
    pub fn scroll_down_page(&mut self) {
        self.auto_follow = false;
        let page = self.viewport_height.max(1);
        self.offset = (self.offset + page).min(self.max_offset());
    }

    fn max_offset(&self) -> usize {
        self.total_lines.saturating_sub(self.viewport_height)
    }

    /// Render the agent detail panel for the selected stage.
    pub fn render(&mut self, f: &mut Frame, area: Rect, run: &ActiveRunState) {
        let stage = match run.selected_stage() {
            Some(s) => s,
            None => {
                let block = Block::default()
                    .title(" AGENT ")
                    .borders(Borders::ALL)
                    .border_style(theme::style_border());
                let para = Paragraph::new("No stage selected").block(block);
                f.render_widget(para, area);
                return;
            }
        };

        let model_info = stage
            .agent
            .model
            .as_deref()
            .map(|m| format!(" [{m}]"))
            .unwrap_or_default();

        let title = format!(" AGENT: {}{} ", stage.name, model_info);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(theme::style_border());

        let inner = block.inner(area);
        self.viewport_height = inner.height as usize;
        let max_width = inner.width as usize;

        // Build content lines — each line is truncated to panel width (no wrapping)
        let mut lines: Vec<Line<'_>> = Vec::new();

        if !stage.agent.text_buffer.is_empty() {
            for text_line in stage.agent.text_buffer.lines() {
                lines.push(Line::from(Span::styled(
                    truncate_line(text_line, max_width),
                    theme::style_assistant_msg(),
                )));
            }
            lines.push(Line::from(""));
        }

        // Tool calls
        for tc in &stage.agent.tool_calls {
            let (glyph, style) = match tc.status {
                ToolStatus::Succeeded => (theme::GLYPH_SUCCESS, theme::style_success()),
                ToolStatus::Failed => (theme::GLYPH_FAIL, theme::style_fail()),
                ToolStatus::Running => (theme::GLYPH_RUNNING, theme::style_running()),
            };
            let elapsed = theme::format_duration(tc.elapsed());
            // " ◉ " = 3 chars, "  {elapsed}" = 2 + elapsed_len
            let elapsed_str = format!("  {elapsed}");
            let overhead = 3 + elapsed_str.chars().count();
            let name_max = max_width.saturating_sub(overhead);
            lines.push(Line::from(vec![
                Span::styled(format!(" {glyph} "), style),
                Span::styled(
                    theme::truncate(&tc.display_name, name_max),
                    theme::style_dim(),
                ),
                Span::styled(elapsed_str, theme::style_dim()),
            ]));
        }

        if stage.agent.compacting {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "  Compacting context...",
                theme::style_running(),
            )));
        }

        self.total_lines = lines.len();

        // Auto-scroll to bottom when following
        if self.auto_follow {
            self.offset = self.max_offset();
        } else {
            self.offset = self.offset.min(self.max_offset());
        }

        let text = Text::from(lines);
        let para = Paragraph::new(text)
            .block(block)
            .scroll((self.offset as u16, 0));

        f.render_widget(para, area);
    }
}

/// Truncate a string to fit within `max_chars` columns. No wrapping — just clip.
fn truncate_line(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        s.to_string()
    } else {
        s.chars().take(max_chars).collect()
    }
}
