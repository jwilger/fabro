use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::components::app::{AppMode, Panel};
use crate::state::run_state::{ActiveRunState, RunStatus};
use crate::theme;

/// Render the bottom status bar with run info and keybinds.
pub fn render_status_bar(
    f: &mut Frame,
    area: Rect,
    run: &ActiveRunState,
    mode: &AppMode,
    engine_done: bool,
) {
    let elapsed = theme::format_duration(run.elapsed());
    let cost = if run.total_cost > 0.0 {
        format!("  ${:.2}", run.total_cost)
    } else {
        String::new()
    };

    let status_str = match run.status {
        RunStatus::Running if engine_done => "Done",
        RunStatus::Running => "Running",
        RunStatus::Completed => "Completed",
        RunStatus::Failed => "Failed",
    };

    let status_style = match run.status {
        RunStatus::Running if engine_done => theme::style_success(),
        RunStatus::Running => theme::style_running(),
        RunStatus::Completed => theme::style_success(),
        RunStatus::Failed => theme::style_fail(),
    };

    let keybinds = match mode {
        AppMode::Monitor { focus } => {
            let focus_hint = match focus {
                Panel::Workflow => "workflow",
                Panel::Agent => "agent",
            };
            if engine_done {
                format!(" Tab:panel({focus_hint})  j/k:scroll  PgUp/PgDn:page  q:quit")
            } else {
                format!(" Tab:panel({focus_hint})  j/k:scroll  f:follow  PgUp/PgDn:page  q:quit")
            }
        }
        AppMode::InteractiveChat => " Ctrl+D:send  Esc:back  Ctrl+C:skip".to_string(),
    };

    let run_info = if run.workflow_name.is_empty() {
        format!(" {}  {}  {}{}", run.run_id, status_str, elapsed, cost)
    } else {
        format!(
            " {}  {}  {}  {}{}",
            run.workflow_name, run.run_id, status_str, elapsed, cost
        )
    };

    // UTF-8 safe truncation
    let available = area.width as usize;
    let keybinds_chars = keybinds.chars().count();
    let info_max = available.saturating_sub(keybinds_chars);
    let info_chars = run_info.chars().count();
    let info_display = if info_chars > info_max {
        let end = info_max.saturating_sub(3);
        let truncated: String = run_info.chars().take(end).collect();
        format!("{truncated}...")
    } else {
        run_info
    };

    let info_display_chars = info_display.chars().count();
    let padding = available
        .saturating_sub(info_display_chars)
        .saturating_sub(keybinds_chars);

    let line = Line::from(vec![
        Span::styled(info_display, status_style),
        Span::raw(" ".repeat(padding)),
        Span::styled(keybinds, theme::style_dim()),
    ]);

    let bar = Paragraph::new(line);
    f.render_widget(bar, area);
}
