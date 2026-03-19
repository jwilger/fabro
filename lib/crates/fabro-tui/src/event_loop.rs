use std::time::Duration;

use anyhow::Result;
use futures_util::StreamExt;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc::error::TryRecvError;

use crate::components::app::App;

/// Run the main TUI event loop with tick-based rendering.
///
/// Uses `crossterm::event::EventStream` (async, no `spawn_blocking` needed)
/// and a 100ms tick interval for timer updates + event batching.
pub async fn run_event_loop(app: &mut App, terminal: &mut DefaultTerminal) -> Result<()> {
    let mut tick = tokio::time::interval(Duration::from_millis(100));
    let mut event_stream = crossterm::event::EventStream::new();

    loop {
        // Render current state
        terminal.draw(|frame| app.render(frame))?;

        tokio::select! {
            _ = tick.tick() => {
                // Drain all pending workflow events (batch processing)
                loop {
                    match app.event_rx.try_recv() {
                        Ok(event) => app.apply_event(event),
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => {
                            app.engine_done = true;
                            break;
                        }
                    }
                }
                // Drain pending chat questions
                while let Ok(pending) = app.chat_rx.try_recv() {
                    app.start_chat(pending);
                }
            }
            maybe_event = event_stream.next() => {
                match maybe_event {
                    Some(Ok(event)) => app.handle_terminal_event(event),
                    Some(Err(_)) => break,
                    None => break,
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
