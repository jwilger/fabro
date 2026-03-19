use std::time::Instant;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::Frame;
use tokio::sync::mpsc;

use fabro_agent::AgentEvent;
use fabro_workflows::event::WorkflowRunEvent;

use crate::state::chat_state::{ChatMessage, ChatState, PendingQuestion};
use crate::state::run_state::{
    ActiveRunState, AgentState, PhaseState, RunStatus, StageState, StageStatus, ToolCallState,
    ToolStatus,
};
use crate::theme;

use super::agent_panel::AgentPanel;
use super::chat_view;
use super::status_bar;
use super::workflow_panel;

/// Which panel has focus in Monitor mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Workflow,
    Agent,
}

/// Top-level app mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Monitor { focus: Panel },
    InteractiveChat,
}

/// Top-level TUI application state (compositor).
pub struct App {
    pub mode: AppMode,
    pub run: ActiveRunState,
    pub event_rx: mpsc::UnboundedReceiver<WorkflowRunEvent>,
    pub chat_rx: mpsc::UnboundedReceiver<PendingQuestion>,
    pub chat: Option<ChatState>,
    pub should_quit: bool,
    pub auto_follow: bool,
    pub engine_done: bool,
    pub agent_panel: AgentPanel,
}

impl App {
    pub fn new(
        run_id: String,
        event_rx: mpsc::UnboundedReceiver<WorkflowRunEvent>,
        chat_rx: mpsc::UnboundedReceiver<PendingQuestion>,
    ) -> Self {
        Self {
            mode: AppMode::Monitor {
                focus: Panel::Workflow,
            },
            run: ActiveRunState::new(run_id),
            event_rx,
            chat_rx,
            chat: None,
            should_quit: false,
            auto_follow: true,
            engine_done: false,
            agent_panel: AgentPanel::new(),
        }
    }

    /// Render the current state to the terminal frame.
    pub fn render(&mut self, f: &mut Frame) {
        let size = f.area();

        match self.mode {
            AppMode::Monitor { .. } => self.render_monitor(f, size),
            AppMode::InteractiveChat => {
                if let Some(ref chat) = self.chat {
                    chat_view::render_chat_view(f, size, chat);
                } else {
                    self.render_monitor(f, size);
                }
            }
        }
    }

    fn render_monitor(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(5),    // Main content
                Constraint::Length(1), // Status bar
            ])
            .split(area);

        if area.width < 80 {
            match self.mode {
                AppMode::Monitor {
                    focus: Panel::Workflow,
                } => {
                    workflow_panel::render_workflow_panel(f, chunks[0], &self.run);
                }
                _ => {
                    self.agent_panel.render(f, chunks[0], &self.run);
                }
            }
        } else {
            let panels = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(chunks[0]);

            workflow_panel::render_workflow_panel(f, panels[0], &self.run);
            self.agent_panel.render(f, panels[1], &self.run);
        }

        status_bar::render_status_bar(f, chunks[1], &self.run, &self.mode, self.engine_done);
    }

    /// Handle a crossterm terminal event.
    pub fn handle_terminal_event(&mut self, event: crossterm::event::Event) {
        use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

        match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => {
                match self.mode {
                    AppMode::Monitor { ref mut focus } => match code {
                        KeyCode::Char('q') => self.should_quit = true,
                        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                            self.should_quit = true;
                        }
                        KeyCode::Tab => {
                            *focus = match focus {
                                Panel::Workflow => Panel::Agent,
                                Panel::Agent => Panel::Workflow,
                            };
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            self.auto_follow = false;
                            self.agent_panel.auto_follow = false;
                            self.run.select_next_stage();
                            self.agent_panel.reset_scroll();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            self.auto_follow = false;
                            self.agent_panel.auto_follow = false;
                            self.run.select_prev_stage();
                            self.agent_panel.reset_scroll();
                        }
                        KeyCode::Char('g') => {
                            self.auto_follow = false;
                            self.agent_panel.auto_follow = false;
                            self.run.selected_stage_idx = 0;
                            self.agent_panel.reset_scroll();
                        }
                        KeyCode::Char('G') => {
                            self.auto_follow = false;
                            self.agent_panel.auto_follow = false;
                            if !self.run.stages.is_empty() {
                                self.run.selected_stage_idx = self.run.stages.len() - 1;
                            }
                            self.agent_panel.reset_scroll();
                        }
                        KeyCode::Char('f') => {
                            self.auto_follow = true;
                            self.agent_panel.auto_follow = true;
                            self.run.follow_running_stage();
                        }
                        KeyCode::PageUp => {
                            self.agent_panel.scroll_up_page();
                        }
                        KeyCode::PageDown => {
                            self.agent_panel.scroll_down_page();
                        }
                        _ => {}
                    },
                    AppMode::InteractiveChat => {
                        if let Some(ref mut chat) = self.chat {
                            match (code, modifiers) {
                                (KeyCode::Char('d'), m)
                                    if m.contains(KeyModifiers::CONTROL) && chat.submit() =>
                                {
                                    // Stay in chat mode; agent will continue
                                }
                                (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => {
                                    chat.skip();
                                    self.mode = AppMode::Monitor {
                                        focus: Panel::Workflow,
                                    };
                                }
                                (KeyCode::Esc, _) => {
                                    // Go back to monitor (don't skip — question stays pending)
                                    self.mode = AppMode::Monitor {
                                        focus: Panel::Workflow,
                                    };
                                }
                                (KeyCode::Enter, _) => {
                                    chat.input.insert_newline();
                                }
                                (KeyCode::Backspace, _) => {
                                    chat.input.backspace();
                                }
                                (KeyCode::Delete, _) => {
                                    chat.input.delete();
                                }
                                (KeyCode::Left, _) => {
                                    chat.input.move_left();
                                }
                                (KeyCode::Right, _) => {
                                    chat.input.move_right();
                                }
                                (KeyCode::Up, _) => {
                                    chat.input.move_up();
                                }
                                (KeyCode::Down, _) => {
                                    chat.input.move_down();
                                }
                                (KeyCode::Home, _) => {
                                    chat.input.home();
                                }
                                (KeyCode::End, _) => {
                                    chat.input.end();
                                }
                                (KeyCode::Char(c), m) if !m.contains(KeyModifiers::CONTROL) => {
                                    chat.input.insert_char(c);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Event::Resize(_, _) => {
                // ratatui handles resize automatically
            }
            _ => {}
        }
    }

    /// Apply a workflow run event to update state.
    pub fn apply_event(&mut self, event: WorkflowRunEvent) {
        match event {
            WorkflowRunEvent::WorkflowRunStarted {
                name, run_id, goal, ..
            } => {
                self.run.workflow_name = name;
                self.run.run_id = run_id;
                if let Some(g) = goal {
                    self.run.goal = g;
                }
            }
            WorkflowRunEvent::WorkflowRunCompleted { .. } => {
                self.run.status = RunStatus::Completed;
            }
            WorkflowRunEvent::WorkflowRunFailed { .. } => {
                self.run.status = RunStatus::Failed;
            }
            WorkflowRunEvent::StageStarted { node_id, name, .. } => {
                let stage = StageState {
                    node_id: node_id.clone(),
                    name,
                    status: StageStatus::Running,
                    duration_ms: None,
                    cost: None,
                    agent: AgentState::default(),
                    started_at: Some(Instant::now()),
                };
                self.run.stages.insert(node_id, stage);
                if self.auto_follow {
                    self.run.follow_running_stage();
                }
            }
            WorkflowRunEvent::StageCompleted {
                node_id,
                duration_ms,
                usage,
                ..
            } => {
                if let Some(stage) = self.run.stages.get_mut(&node_id) {
                    stage.status = StageStatus::Completed;
                    stage.duration_ms = Some(duration_ms);
                    if let Some(u) = &usage {
                        if let Some(cost) = fabro_workflows::cost::compute_stage_cost(u) {
                            stage.cost = Some(cost);
                            self.run.total_cost += cost;
                        }
                        self.run.total_input_tokens += u.input_tokens;
                        self.run.total_output_tokens += u.output_tokens;
                    }
                }
            }
            WorkflowRunEvent::StageFailed { node_id, .. } => {
                if let Some(stage) = self.run.stages.get_mut(&node_id) {
                    stage.status = StageStatus::Failed;
                }
            }
            WorkflowRunEvent::Agent { stage, event } => {
                self.apply_agent_event(&stage, &event);
            }
            WorkflowRunEvent::Sandbox { event } => {
                self.apply_sandbox_event(&event);
            }
            WorkflowRunEvent::SandboxInitialized { .. } => {
                if let Some(ref mut sb) = self.run.sandbox {
                    sb.status = StageStatus::Completed;
                }
            }
            WorkflowRunEvent::SetupStarted { command_count } => {
                self.run.setup = Some(PhaseState {
                    label: "Setup".to_string(),
                    status: StageStatus::Running,
                    duration_ms: None,
                    detail: Some(format!("{command_count} commands")),
                    started_at: Some(Instant::now()),
                });
            }
            WorkflowRunEvent::SetupCompleted { duration_ms } => {
                if let Some(ref mut setup) = self.run.setup {
                    setup.status = StageStatus::Completed;
                    setup.duration_ms = Some(duration_ms);
                }
            }
            WorkflowRunEvent::SetupFailed { .. } => {
                if let Some(ref mut setup) = self.run.setup {
                    setup.status = StageStatus::Failed;
                }
            }
            WorkflowRunEvent::ParallelStarted { branch_count, .. } => {
                self.run.parallel = Some(crate::state::run_state::ParallelState {
                    branch_count,
                    completed_count: 0,
                    failed_count: 0,
                });
            }
            WorkflowRunEvent::ParallelBranchCompleted { status, .. } => {
                if let Some(ref mut p) = self.run.parallel {
                    p.completed_count += 1;
                    if status != "success" {
                        p.failed_count += 1;
                    }
                }
            }
            WorkflowRunEvent::ParallelCompleted { .. } => {
                self.run.parallel = None;
            }
            WorkflowRunEvent::RetroStarted => {
                let stage = StageState {
                    node_id: "__retro__".to_string(),
                    name: "retro".to_string(),
                    status: StageStatus::Running,
                    duration_ms: None,
                    cost: None,
                    agent: AgentState::default(),
                    started_at: Some(Instant::now()),
                };
                self.run.stages.insert("__retro__".to_string(), stage);
                if self.auto_follow {
                    self.run.follow_running_stage();
                }
            }
            WorkflowRunEvent::RetroCompleted { duration_ms } => {
                if let Some(stage) = self.run.stages.get_mut("__retro__") {
                    stage.status = StageStatus::Completed;
                    stage.duration_ms = Some(duration_ms);
                }
            }
            WorkflowRunEvent::RetroFailed { .. } => {
                if let Some(stage) = self.run.stages.get_mut("__retro__") {
                    stage.status = StageStatus::Failed;
                }
            }
            _ => {}
        }
    }

    fn apply_agent_event(&mut self, stage_id: &str, event: &AgentEvent) {
        // Also forward to chat if we're in interactive mode for this stage
        if let Some(ref mut chat) = self.chat {
            if chat.stage_id == stage_id {
                Self::apply_agent_to_chat(chat, event);
            }
        }

        let stage = match self.run.stages.get_mut(stage_id) {
            Some(s) => s,
            None => return,
        };

        match event {
            AgentEvent::AssistantMessage { model, .. } => {
                stage.agent.model = Some(model.clone());
                stage.agent.turn_count += 1;
            }
            AgentEvent::AssistantTextStart => {}
            AgentEvent::TextDelta { delta } => {
                stage.agent.text_buffer.push_str(delta);
            }
            AgentEvent::AssistantOutputReplace { text, .. } => {
                stage.agent.text_buffer = text.clone();
            }
            AgentEvent::ToolCallStarted {
                tool_name,
                tool_call_id,
                arguments,
            } => {
                let display_name = tool_display_name(tool_name, arguments);
                stage.agent.tool_calls.push(ToolCallState {
                    tool_name: tool_name.clone(),
                    display_name,
                    tool_call_id: tool_call_id.clone(),
                    status: ToolStatus::Running,
                    started_at: Instant::now(),
                });
            }
            AgentEvent::ToolCallCompleted {
                tool_call_id,
                is_error,
                ..
            } => {
                if let Some(tc) = stage
                    .agent
                    .tool_calls
                    .iter_mut()
                    .find(|t| t.tool_call_id == *tool_call_id)
                {
                    tc.status = if *is_error {
                        ToolStatus::Failed
                    } else {
                        ToolStatus::Succeeded
                    };
                }
            }
            AgentEvent::CompactionStarted { .. } => {
                stage.agent.compacting = true;
            }
            AgentEvent::CompactionCompleted { .. } => {
                stage.agent.compacting = false;
            }
            AgentEvent::SubAgentEvent { event, .. } => {
                self.apply_subagent_event(stage_id, event);
            }
            _ => {}
        }
    }

    fn apply_subagent_event(&mut self, stage_id: &str, event: &AgentEvent) {
        let stage = match self.run.stages.get_mut(stage_id) {
            Some(s) => s,
            None => return,
        };

        match event {
            AgentEvent::ToolCallStarted {
                tool_name,
                tool_call_id,
                arguments,
            } => {
                let display_name = tool_display_name(tool_name, arguments);
                stage.agent.tool_calls.push(ToolCallState {
                    tool_name: tool_name.clone(),
                    display_name,
                    tool_call_id: tool_call_id.clone(),
                    status: ToolStatus::Running,
                    started_at: Instant::now(),
                });
            }
            AgentEvent::ToolCallCompleted {
                tool_call_id,
                is_error,
                ..
            } => {
                if let Some(tc) = stage
                    .agent
                    .tool_calls
                    .iter_mut()
                    .find(|t| t.tool_call_id == *tool_call_id)
                {
                    tc.status = if *is_error {
                        ToolStatus::Failed
                    } else {
                        ToolStatus::Succeeded
                    };
                }
            }
            AgentEvent::SubAgentEvent { event: inner, .. } => {
                self.apply_subagent_event(stage_id, inner);
            }
            _ => {}
        }
    }

    fn apply_agent_to_chat(chat: &mut ChatState, event: &AgentEvent) {
        match event {
            AgentEvent::AssistantTextStart => {
                chat.messages.push(ChatMessage::Assistant {
                    text: String::new(),
                    tool_calls: Vec::new(),
                    streaming: true,
                });
            }
            AgentEvent::TextDelta { delta } => {
                if let Some(ChatMessage::Assistant {
                    text, streaming, ..
                }) = chat.messages.last_mut()
                {
                    text.push_str(delta);
                    *streaming = true;
                }
            }
            AgentEvent::AssistantMessage { .. } => {
                if let Some(ChatMessage::Assistant { streaming, .. }) = chat.messages.last_mut() {
                    *streaming = false;
                }
            }
            AgentEvent::ToolCallStarted {
                tool_name,
                tool_call_id,
                arguments,
            } => {
                let display_name = tool_display_name(tool_name, arguments);
                let tc = ToolCallState {
                    tool_name: tool_name.clone(),
                    display_name,
                    tool_call_id: tool_call_id.clone(),
                    status: ToolStatus::Running,
                    started_at: Instant::now(),
                };
                if let Some(ChatMessage::Assistant { tool_calls, .. }) = chat.messages.last_mut() {
                    tool_calls.push(tc);
                }
            }
            AgentEvent::ToolCallCompleted {
                tool_call_id,
                is_error,
                ..
            } => {
                if let Some(ChatMessage::Assistant { tool_calls, .. }) = chat.messages.last_mut() {
                    if let Some(tc) = tool_calls
                        .iter_mut()
                        .find(|t| t.tool_call_id == *tool_call_id)
                    {
                        tc.status = if *is_error {
                            ToolStatus::Failed
                        } else {
                            ToolStatus::Succeeded
                        };
                    }
                }
            }
            _ => {}
        }
    }

    fn apply_sandbox_event(&mut self, event: &fabro_agent::SandboxEvent) {
        use fabro_agent::SandboxEvent;
        match event {
            SandboxEvent::Initializing { .. }
            | SandboxEvent::SnapshotPulling { .. }
            | SandboxEvent::SnapshotCreating { .. }
                if self.run.sandbox.is_none() =>
            {
                self.run.sandbox = Some(PhaseState {
                    label: "Sandbox".to_string(),
                    status: StageStatus::Running,
                    duration_ms: None,
                    detail: None,
                    started_at: Some(Instant::now()),
                });
            }
            SandboxEvent::Ready { provider, .. } => {
                if let Some(ref mut sb) = self.run.sandbox {
                    sb.detail = Some(provider.clone());
                }
            }
            SandboxEvent::InitializeFailed { error, .. } => {
                if let Some(ref mut sb) = self.run.sandbox {
                    sb.status = StageStatus::Failed;
                    sb.detail = Some(error.clone());
                } else {
                    self.run.sandbox = Some(PhaseState {
                        label: "Sandbox".to_string(),
                        status: StageStatus::Failed,
                        duration_ms: None,
                        detail: Some(error.clone()),
                        started_at: None,
                    });
                }
            }
            _ => {}
        }
    }

    /// Start interactive chat mode for a pending question.
    pub fn start_chat(&mut self, pending: PendingQuestion) {
        let mut chat = ChatState::new(pending.stage_id.clone());
        chat.messages.push(ChatMessage::System {
            text: pending.question_text,
        });
        chat.pending_answer_tx = Some(pending.answer_tx);
        self.chat = Some(chat);
        self.mode = AppMode::InteractiveChat;
    }
}

/// Build a display name for a tool call.
fn tool_display_name(tool_name: &str, arguments: &serde_json::Value) -> String {
    let arg = |key: &str| arguments.get(key).and_then(|v| v.as_str());

    let detail = match tool_name {
        "bash" | "shell" | "execute_command" => arg("command").map(|c| theme::truncate(c, 50)),
        "glob" => arg("pattern").map(String::from),
        "grep" | "ripgrep" => arg("pattern").map(|p| theme::truncate(p, 40)),
        "read_file" | "read" => arg("path")
            .or_else(|| arg("file_path"))
            .map(|p| theme::truncate(p, 50)),
        "write_file" | "write" | "create_file" => arg("path")
            .or_else(|| arg("file_path"))
            .map(|p| theme::truncate(p, 50)),
        "edit_file" | "edit" => arg("path")
            .or_else(|| arg("file_path"))
            .map(|p| theme::truncate(p, 50)),
        "web_search" => arg("query").map(|q| theme::truncate(q, 50)),
        "web_fetch" => arg("url").map(|u| theme::truncate(u, 50)),
        "spawn_agent" => arg("task").map(|t| theme::truncate(t, 50)),
        _ => None,
    };

    match detail {
        Some(d) => format!("{tool_name}({d})"),
        None => tool_name.to_string(),
    }
}
