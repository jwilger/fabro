use std::time::{Duration, Instant};

use indexmap::IndexMap;

/// Overall run status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Running,
    Completed,
    Failed,
}

/// Per-stage status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Per-tool-call status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolStatus {
    Running,
    Succeeded,
    Failed,
}

/// State of a single tool call.
#[derive(Debug, Clone)]
pub struct ToolCallState {
    pub tool_name: String,
    pub display_name: String,
    pub tool_call_id: String,
    pub status: ToolStatus,
    pub started_at: Instant,
}

impl ToolCallState {
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }
}

/// State of an agent within a stage.
#[derive(Debug, Clone, Default)]
pub struct AgentState {
    pub model: Option<String>,
    pub turn_count: u32,
    pub tool_calls: Vec<ToolCallState>,
    pub text_buffer: String,
    pub compacting: bool,
}

/// State of a single workflow stage.
#[derive(Debug, Clone)]
pub struct StageState {
    pub node_id: String,
    pub name: String,
    pub status: StageStatus,
    pub duration_ms: Option<u64>,
    pub cost: Option<f64>,
    pub agent: AgentState,
    pub started_at: Option<Instant>,
}

/// Phase state for setup/sandbox.
#[derive(Debug, Clone)]
pub struct PhaseState {
    pub label: String,
    pub status: StageStatus,
    pub duration_ms: Option<u64>,
    pub detail: Option<String>,
    pub started_at: Option<Instant>,
}

/// Parallel execution tracking.
#[derive(Debug, Clone)]
pub struct ParallelState {
    pub branch_count: usize,
    pub completed_count: usize,
    pub failed_count: usize,
}

/// Top-level state for the active workflow run.
pub struct ActiveRunState {
    pub run_id: String,
    pub workflow_name: String,
    pub goal: String,
    pub status: RunStatus,

    /// Stages in insertion order.
    pub stages: IndexMap<String, StageState>,
    /// Which stage is focused in the agent detail panel.
    pub selected_stage_idx: usize,

    pub parallel: Option<ParallelState>,
    pub setup: Option<PhaseState>,
    pub sandbox: Option<PhaseState>,

    pub total_cost: f64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,

    pub workflow_scroll: usize,
    pub agent_scroll: usize,

    pub started_at: Instant,
}

impl ActiveRunState {
    pub fn new(run_id: String) -> Self {
        Self {
            run_id,
            workflow_name: String::new(),
            goal: String::new(),
            status: RunStatus::Running,
            stages: IndexMap::new(),
            selected_stage_idx: 0,
            parallel: None,
            setup: None,
            sandbox: None,
            total_cost: 0.0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            workflow_scroll: 0,
            agent_scroll: 0,
            started_at: Instant::now(),
        }
    }

    /// Get the currently selected stage's node_id.
    pub fn selected_stage_id(&self) -> Option<&str> {
        self.stages
            .get_index(self.selected_stage_idx)
            .map(|(id, _)| id.as_str())
    }

    /// Get the currently selected stage.
    pub fn selected_stage(&self) -> Option<&StageState> {
        self.stages
            .get_index(self.selected_stage_idx)
            .map(|(_, s)| s)
    }

    /// Move selection to the last running stage (auto-follow).
    pub fn follow_running_stage(&mut self) {
        if let Some(idx) = self
            .stages
            .values()
            .rposition(|s| s.status == StageStatus::Running)
        {
            self.selected_stage_idx = idx;
        }
    }

    /// Total elapsed duration.
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Number of stages.
    pub fn stage_count(&self) -> usize {
        self.stages.len()
    }

    /// Select next stage.
    pub fn select_next_stage(&mut self) {
        if !self.stages.is_empty() {
            self.selected_stage_idx =
                (self.selected_stage_idx + 1).min(self.stages.len().saturating_sub(1));
        }
    }

    /// Select previous stage.
    pub fn select_prev_stage(&mut self) {
        self.selected_stage_idx = self.selected_stage_idx.saturating_sub(1);
    }
}
