pub mod components;
pub mod event_loop;
pub mod interviewer;
pub mod source;
pub mod state;
pub mod theme;

use anyhow::Result;
use tokio::sync::mpsc;

use fabro_workflows::event::WorkflowRunEvent;

use components::app::App;
use state::chat_state::PendingQuestion;

pub use interviewer::TuiInterviewer;
pub use source::live::register_tui_listener;

/// Configuration for launching the TUI.
pub struct TuiConfig {
    pub run_id: String,
    pub event_rx: mpsc::UnboundedReceiver<WorkflowRunEvent>,
    pub chat_rx: mpsc::UnboundedReceiver<PendingQuestion>,
}

/// Run the TUI in alternate screen mode. Blocks until the user quits.
pub fn run_tui(config: TuiConfig) -> Result<()> {
    // Enter alternate screen with raw mode
    let mut terminal = ratatui::init();

    // Ensure terminal cleanup on panic
    let _guard = scopeguard::guard((), |()| {
        ratatui::restore();
    });

    let result = run_tui_inner(&mut terminal, config);

    // Restore terminal
    ratatui::restore();

    // Defuse guard since we already restored
    std::mem::forget(_guard);

    result
}

fn run_tui_inner(terminal: &mut ratatui::DefaultTerminal, config: TuiConfig) -> Result<()> {
    let TuiConfig {
        run_id,
        event_rx,
        chat_rx,
    } = config;

    let mut app = App::new(run_id, event_rx, chat_rx);

    // We need a tokio runtime for the async event loop.
    // Since we're called from an async context, use the current runtime.
    let rt = tokio::runtime::Handle::current();
    rt.block_on(event_loop::run_event_loop(&mut app, terminal))?;

    Ok(())
}
