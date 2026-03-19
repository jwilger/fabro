use std::time::Duration;

use ratatui::style::{Color, Modifier, Style};

// Status glyphs
pub const GLYPH_SUCCESS: &str = "\u{2713}"; // ✓
pub const GLYPH_FAIL: &str = "\u{2717}"; // ✗
pub const GLYPH_RUNNING: &str = "\u{25C9}"; // ◉
pub const GLYPH_PENDING: &str = "\u{25CB}"; // ○

// Colors
pub const COLOR_SUCCESS: Color = Color::Green;
pub const COLOR_FAIL: Color = Color::Red;
pub const COLOR_RUNNING: Color = Color::Cyan;
pub const COLOR_PENDING: Color = Color::DarkGray;
pub const COLOR_DIM: Color = Color::DarkGray;
pub const COLOR_HIGHLIGHT: Color = Color::Yellow;
pub const COLOR_BORDER: Color = Color::DarkGray;
pub const COLOR_HEADER: Color = Color::White;
pub const COLOR_USER_MSG: Color = Color::Blue;
pub const COLOR_ASSISTANT_MSG: Color = Color::White;
pub const COLOR_COST: Color = Color::Yellow;

// Styles
pub fn style_success() -> Style {
    Style::default().fg(COLOR_SUCCESS)
}

pub fn style_fail() -> Style {
    Style::default().fg(COLOR_FAIL)
}

pub fn style_running() -> Style {
    Style::default().fg(COLOR_RUNNING)
}

pub fn style_pending() -> Style {
    Style::default().fg(COLOR_PENDING)
}

pub fn style_dim() -> Style {
    Style::default().fg(COLOR_DIM)
}

pub fn style_highlight() -> Style {
    Style::default().fg(COLOR_HIGHLIGHT)
}

pub fn style_selected() -> Style {
    Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

pub fn style_header() -> Style {
    Style::default()
        .fg(COLOR_HEADER)
        .add_modifier(Modifier::BOLD)
}

pub fn style_border() -> Style {
    Style::default().fg(COLOR_BORDER)
}

pub fn style_cost() -> Style {
    Style::default().fg(COLOR_COST)
}

pub fn style_user_msg() -> Style {
    Style::default().fg(COLOR_USER_MSG)
}

pub fn style_assistant_msg() -> Style {
    Style::default().fg(COLOR_ASSISTANT_MSG)
}

/// UTF-8 safe truncation by character count. Collapses whitespace to single spaces
/// and appends "..." if truncated.
pub fn truncate(s: &str, max_chars: usize) -> String {
    let single_line: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    let char_count = single_line.chars().count();
    if char_count > max_chars {
        let end = max_chars.saturating_sub(3);
        let truncated: String = single_line.chars().take(end).collect();
        format!("{truncated}...")
    } else {
        single_line
    }
}

/// Format a Duration as a human-readable string.
pub fn format_duration(d: Duration) -> String {
    format_duration_ms(d.as_millis() as u64)
}

/// Format milliseconds as a human-readable string.
pub fn format_duration_ms(ms: u64) -> String {
    let secs = ms / 1000;
    if secs >= 60 {
        format!("{}m{:02}s", secs / 60, secs % 60)
    } else if ms >= 1000 {
        format!("{secs}s")
    } else {
        format!("{ms}ms")
    }
}

/// Shorten a model name for display.
pub fn shorten_model(model: &str) -> String {
    let s = model
        .replace("claude-", "")
        .replace("gpt-", "gpt")
        .replace("gemini-", "gem");
    let char_count = s.chars().count();
    if char_count > 20 {
        let truncated: String = s.chars().take(17).collect();
        format!("{truncated}...")
    } else {
        s
    }
}
