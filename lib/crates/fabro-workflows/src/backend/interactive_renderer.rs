use std::io::Write;

use fabro_agent::{AgentEvent, Session};

/// Spawn a background task that subscribes to session events and renders
/// interactive conversation output to stderr.
///
/// Handles:
/// - `AssistantTextStart` — prints a blank line + dim role indicator
/// - `TextDelta` — streams raw text directly to stderr
/// - `AssistantMessage` — prints a trailing newline
/// - `ToolCallStarted` — prints a dim status line
/// - `ToolCallCompleted` — prints ✓/✗ result line
/// - `SubAgentEvent` — unwraps and renders the inner event
///
/// All other events are ignored.
pub fn spawn_interactive_renderer(session: &Session) -> tokio::task::JoinHandle<()> {
    let mut rx = session.subscribe();
    tokio::spawn(async move {
        while let Ok(session_event) = rx.recv().await {
            render_event(&session_event.event);
        }
    })
}

fn render_event(event: &AgentEvent) {
    let stderr = std::io::stderr();
    match event {
        AgentEvent::AssistantTextStart => {
            let mut out = stderr.lock();
            let _ = writeln!(out);
            let _ = write!(out, "\x1b[2m  assistant:\x1b[0m ");
            let _ = out.flush();
        }
        AgentEvent::TextDelta { delta } => {
            let mut out = stderr.lock();
            let _ = write!(out, "{delta}");
            let _ = out.flush();
        }
        AgentEvent::AssistantMessage { .. } => {
            let mut out = stderr.lock();
            let _ = writeln!(out);
            let _ = out.flush();
        }
        AgentEvent::ToolCallStarted {
            tool_name,
            arguments,
            ..
        } => {
            let detail = tool_call_summary(tool_name, arguments);
            let mut out = stderr.lock();
            let _ = writeln!(out, "\x1b[2m  \u{25e6} {detail}\x1b[0m");
            let _ = out.flush();
        }
        AgentEvent::ToolCallCompleted {
            tool_name,
            is_error,
            ..
        } => {
            let glyph = if *is_error {
                "\x1b[31m\u{2717}\x1b[0m"
            } else {
                "\x1b[32m\u{2713}\x1b[0m"
            };
            let mut out = stderr.lock();
            let _ = writeln!(out, "\x1b[2m  {glyph} {tool_name}\x1b[0m");
            let _ = out.flush();
        }
        AgentEvent::SubAgentEvent { event: inner, .. } => {
            render_event(inner);
        }
        _ => {}
    }
}

fn tool_call_summary(tool_name: &str, arguments: &serde_json::Value) -> String {
    let arg = |key: &str| arguments.get(key).and_then(|v| v.as_str());
    let detail = match tool_name {
        "bash" | "shell" | "execute_command" => arg("command").map(|c| truncate(c, 60)),
        "read_file" | "read" => arg("file_path")
            .or_else(|| arg("path"))
            .map(|p| truncate(p, 60)),
        "write_file" | "write" | "create_file" => arg("file_path")
            .or_else(|| arg("path"))
            .map(|p| truncate(p, 60)),
        "edit_file" | "edit" => arg("file_path")
            .or_else(|| arg("path"))
            .map(|p| truncate(p, 60)),
        "glob" => arg("pattern").map(String::from),
        "grep" | "ripgrep" => arg("pattern").map(|p| truncate(p, 40)),
        "web_search" => arg("query").map(|q| truncate(q, 60)),
        "web_fetch" => arg("url").map(|u| truncate(u, 60)),
        "spawn_agent" => arg("task").map(|t| truncate(t, 60)),
        "ask_human" => arg("question").map(|q| truncate(q, 60)),
        _ => None,
    };
    match detail {
        Some(d) => format!("{tool_name}({d})"),
        None => tool_name.to_string(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    let single_line: String = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if single_line.len() > max {
        let mut t: String = single_line.chars().take(max - 3).collect();
        t.push_str("...");
        t
    } else {
        single_line
    }
}
