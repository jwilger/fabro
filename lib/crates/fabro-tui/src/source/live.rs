use tokio::sync::mpsc;

use fabro_workflows::event::{EventEmitter, WorkflowRunEvent};

/// Register a listener on the `EventEmitter` that forwards events to an mpsc channel
/// for consumption by the TUI event loop.
pub fn register_tui_listener(
    emitter: &mut EventEmitter,
) -> mpsc::UnboundedReceiver<WorkflowRunEvent> {
    let (tx, rx) = mpsc::unbounded_channel();
    emitter.on_event(move |event| {
        let _ = tx.send(event.clone());
    });
    rx
}
