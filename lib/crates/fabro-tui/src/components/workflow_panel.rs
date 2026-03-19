use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::state::run_state::{ActiveRunState, StageStatus, ToolStatus};
use crate::theme;

/// Render the left panel: stage list with status/duration/cost.
pub fn render_workflow_panel(f: &mut Frame, area: Rect, run: &ActiveRunState) {
    let block = Block::default()
        .title(" WORKFLOW ")
        .borders(Borders::ALL)
        .border_style(theme::style_border());

    let mut items: Vec<ListItem> = Vec::new();

    // Sandbox phase
    if let Some(ref sandbox) = run.sandbox {
        items.push(phase_item("Sandbox", sandbox));
    }

    // Setup phase
    if let Some(ref setup) = run.setup {
        items.push(phase_item("Setup", setup));
    }

    // Stages
    for (_id, stage) in &run.stages {
        let (glyph, glyph_style) = match stage.status {
            StageStatus::Completed => (theme::GLYPH_SUCCESS, theme::style_success()),
            StageStatus::Failed => (theme::GLYPH_FAIL, theme::style_fail()),
            StageStatus::Running => (theme::GLYPH_RUNNING, theme::style_running()),
            StageStatus::Pending => (theme::GLYPH_PENDING, theme::style_pending()),
        };

        let model_suffix = stage
            .agent
            .model
            .as_deref()
            .map(|m| {
                let short = theme::shorten_model(m);
                format!(" [{short}]")
            })
            .unwrap_or_default();

        let cost_str = stage.cost.map(|c| format!("  ${c:.2}")).unwrap_or_default();

        let duration_str = if let Some(ms) = stage.duration_ms {
            format!("  {}", theme::format_duration_ms(ms))
        } else if stage.status == StageStatus::Running {
            if let Some(started) = stage.started_at {
                format!("  {}", theme::format_duration(started.elapsed()))
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let mut line_spans = vec![
            Span::styled(format!(" {glyph} "), glyph_style),
            Span::styled(
                stage.name.clone(),
                if stage.status == StageStatus::Running {
                    theme::style_running()
                } else {
                    ratatui::style::Style::default()
                },
            ),
        ];

        if !model_suffix.is_empty() {
            line_spans.push(Span::styled(model_suffix, theme::style_dim()));
        }
        if !cost_str.is_empty() {
            line_spans.push(Span::styled(cost_str, theme::style_cost()));
        }
        if !duration_str.is_empty() {
            line_spans.push(Span::styled(duration_str, theme::style_dim()));
        }

        let mut stage_lines = vec![Line::from(line_spans)];

        // Show running tool calls under the stage
        if stage.status == StageStatus::Running {
            let running_tools: Vec<_> = stage
                .agent
                .tool_calls
                .iter()
                .filter(|t| t.status == ToolStatus::Running)
                .collect();
            for tc in running_tools.iter().rev().take(3).rev() {
                let elapsed = theme::format_duration(tc.elapsed());
                stage_lines.push(Line::from(vec![
                    Span::styled("     ", theme::style_dim()),
                    Span::styled(theme::GLYPH_RUNNING, theme::style_running()),
                    Span::raw(" "),
                    Span::styled(theme::truncate(&tc.display_name, 30), theme::style_dim()),
                    Span::styled(format!("  {elapsed}"), theme::style_dim()),
                ]));
            }
            if stage.agent.compacting {
                stage_lines.push(Line::from(vec![
                    Span::styled("     ", theme::style_dim()),
                    Span::styled(theme::GLYPH_RUNNING, theme::style_running()),
                    Span::styled(" compacting context...", theme::style_dim()),
                ]));
            }
        }

        items.push(ListItem::new(stage_lines));
    }

    let mut state = ListState::default();
    let phase_count = run.sandbox.as_ref().map_or(0, |_| 1) + run.setup.as_ref().map_or(0, |_| 1);
    state.select(Some(run.selected_stage_idx + phase_count));

    let list = List::new(items)
        .block(block)
        .highlight_style(theme::style_selected());

    f.render_stateful_widget(list, area, &mut state);
}

fn phase_item(label: &str, phase: &crate::state::run_state::PhaseState) -> ListItem<'static> {
    let (glyph, style) = match phase.status {
        StageStatus::Completed => (theme::GLYPH_SUCCESS, theme::style_success()),
        StageStatus::Failed => (theme::GLYPH_FAIL, theme::style_fail()),
        StageStatus::Running => (theme::GLYPH_RUNNING, theme::style_running()),
        StageStatus::Pending => (theme::GLYPH_PENDING, theme::style_pending()),
    };

    let duration_str = if let Some(ms) = phase.duration_ms {
        format!("  {}", theme::format_duration_ms(ms))
    } else if phase.status == StageStatus::Running {
        if let Some(started) = phase.started_at {
            format!("  {}", theme::format_duration(started.elapsed()))
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let detail_str = phase
        .detail
        .as_deref()
        .map(|d| format!(": {d}"))
        .unwrap_or_default();

    ListItem::new(Line::from(vec![
        Span::styled(format!(" {glyph} "), style),
        Span::styled(format!("{label}{detail_str}"), theme::style_dim()),
        Span::styled(duration_str, theme::style_dim()),
    ]))
}
