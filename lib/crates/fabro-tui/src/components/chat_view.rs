use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::state::chat_state::{ChatMessage, ChatState};
use crate::state::run_state::ToolStatus;
use crate::theme;

/// Render the full-screen interactive chat view.
pub fn render_chat_view(f: &mut Frame, area: Rect, chat: &ChatState) {
    let title = format!(" {} [interactive] ", chat.stage_id);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Messages area
            Constraint::Length(5), // Input area
            Constraint::Length(1), // Keybinds bar
        ])
        .split(area);

    render_messages(f, chunks[0], chat, &title);
    render_input(f, chunks[1], chat);

    let keybinds = Line::from(vec![
        Span::styled(" Ctrl+D", theme::style_highlight()),
        Span::styled(":send  ", theme::style_dim()),
        Span::styled("Enter", theme::style_highlight()),
        Span::styled(":newline  ", theme::style_dim()),
        Span::styled("Ctrl+C", theme::style_highlight()),
        Span::styled(":skip  ", theme::style_dim()),
        Span::styled("Esc", theme::style_highlight()),
        Span::styled(":back", theme::style_dim()),
    ]);
    f.render_widget(Paragraph::new(keybinds), chunks[2]);
}

fn render_messages(f: &mut Frame, area: Rect, chat: &ChatState, title: &str) {
    let block = Block::default()
        .title(title.to_string())
        .borders(Borders::ALL)
        .border_style(theme::style_border());

    let inner = block.inner(area);

    let mut lines: Vec<Line<'_>> = Vec::new();

    for msg in &chat.messages {
        match msg {
            ChatMessage::System { text } => {
                lines.push(Line::from(Span::styled(
                    format!("  system: {text}"),
                    theme::style_dim(),
                )));
                lines.push(Line::from(""));
            }
            ChatMessage::Assistant {
                text,
                tool_calls,
                streaming,
            } => {
                lines.push(Line::from(Span::styled("  assistant:", theme::style_dim())));

                for text_line in text.lines() {
                    lines.push(Line::from(format!("  {text_line}")));
                }

                for tc in tool_calls {
                    let (glyph, style) = match tc.status {
                        ToolStatus::Succeeded => (theme::GLYPH_SUCCESS, theme::style_success()),
                        ToolStatus::Failed => (theme::GLYPH_FAIL, theme::style_fail()),
                        ToolStatus::Running => (theme::GLYPH_RUNNING, theme::style_running()),
                    };
                    let elapsed = theme::format_duration(tc.elapsed());
                    lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(format!("{glyph} "), style),
                        Span::styled(tc.display_name.clone(), theme::style_dim()),
                        Span::styled(format!("  {elapsed}"), theme::style_dim()),
                    ]));
                }

                if *streaming {
                    lines.push(Line::from(Span::styled(
                        "  \u{2588}",
                        theme::style_running(),
                    )));
                }

                lines.push(Line::from(""));
            }
            ChatMessage::User { text } => {
                lines.push(Line::from(Span::styled("  you:", theme::style_user_msg())));
                for text_line in text.lines() {
                    lines.push(Line::from(Span::styled(
                        format!("  {text_line}"),
                        theme::style_user_msg(),
                    )));
                }
                lines.push(Line::from(""));
            }
        }
    }

    let text = Text::from(lines);
    let total_lines = text.lines.len();
    let visible_height = inner.height as usize;

    // Auto-scroll to bottom
    let scroll = if total_lines > visible_height {
        (total_lines - visible_height) as u16
    } else {
        0
    };

    let para = Paragraph::new(text).block(block).scroll((scroll, 0));

    f.render_widget(para, area);
}

fn render_input(f: &mut Frame, area: Rect, chat: &ChatState) {
    let block = Block::default()
        .title(" > ")
        .borders(Borders::ALL)
        .border_style(theme::style_border());

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Render the input buffer content
    let input = &chat.input;
    let lines: Vec<Line<'_>> = input.lines.iter().map(|l| Line::from(l.as_str())).collect();
    let text = Text::from(lines);

    // Scroll input if it's taller than the available area
    let visible_height = inner.height as usize;
    let scroll = if input.cursor_row >= visible_height {
        (input.cursor_row - visible_height + 1) as u16
    } else {
        0
    };

    let para = Paragraph::new(text).scroll((scroll, 0));
    f.render_widget(para, inner);

    // Place the cursor
    let cursor_y = inner.y + (input.cursor_row as u16).saturating_sub(scroll);
    let cursor_x = inner.x + input.cursor_col as u16;
    if cursor_y < inner.y + inner.height && cursor_x < inner.x + inner.width {
        f.set_cursor_position((cursor_x, cursor_y));
    }
}
