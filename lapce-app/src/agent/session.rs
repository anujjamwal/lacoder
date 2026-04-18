//! Canonical agent session types. Kept pure — no UI, no RPC yet.
//!
//! Phase 1.0 scope: `CoderSession` with reactive signals for trace, chat,
//! modified files, status, and pending approval. An in-process stub runner
//! (see `stub_runner`) populates these directly; the real RPC/process model
//! lands in a follow-up.

use std::{path::PathBuf, sync::Arc};

use floem::reactive::{RwSignal, Scope};

use crate::{
    id::{AssistantSessionId, CoderSessionId},
    workspace::LapceWorkspace,
};

/// Lifecycle of a session (assistant or coder).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionState {
    Draft,
    Active,
    Locked,
    Archived,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepId(pub u64);

/// High-level state of the agent's execution loop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentStatus {
    Idle,
    Thinking { step_id: StepId },
    Acting { step_id: StepId, action: String },
    AwaitingApproval { step_id: StepId },
    Paused,
    Stopped { reason: StopReason },
    Failed(String),
}

impl Default for AgentStatus {
    fn default() -> Self {
        AgentStatus::Idle
    }
}

impl AgentStatus {
    pub fn label(&self) -> &'static str {
        match self {
            AgentStatus::Idle => "idle",
            AgentStatus::Thinking { .. } => "thinking",
            AgentStatus::Acting { .. } => "acting",
            AgentStatus::AwaitingApproval { .. } => "awaiting approval",
            AgentStatus::Paused => "paused",
            AgentStatus::Stopped { .. } => "stopped",
            AgentStatus::Failed(_) => "failed",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            AgentStatus::Stopped { .. } | AgentStatus::Failed(_)
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StopReason {
    UserRequested,
    Completed,
    Orphaned,
}

/// An entry in the append-only trace — what the agent did, step by step.
#[derive(Clone, Debug)]
pub struct TraceEntry {
    pub step_id: StepId,
    pub kind: TraceKind,
    pub summary: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug)]
pub enum TraceKind {
    Thought,
    ToolCall,
    FileWrite,
    Shell,
    Note,
}

impl TraceKind {
    pub fn badge(&self) -> &'static str {
        match self {
            TraceKind::Thought => "THINK",
            TraceKind::ToolCall => "TOOL",
            TraceKind::FileWrite => "WRITE",
            TraceKind::Shell => "SHELL",
            TraceKind::Note => "NOTE",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChatTurn {
    pub role: ChatRole,
    pub content: String,
    pub timestamp_ms: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Agent,
}

#[derive(Clone, Debug)]
pub struct FileChange {
    pub path: PathBuf,
    pub kind: FileChangeKind,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileChangeKind {
    Added,
    Modified,
    Deleted,
}

impl FileChangeKind {
    pub fn badge(&self) -> &'static str {
        match self {
            FileChangeKind::Added => "A",
            FileChangeKind::Modified => "M",
            FileChangeKind::Deleted => "D",
        }
    }
}

/// A running (or previously run) coder agent session.
#[derive(Clone)]
pub struct CoderSession {
    pub id: CoderSessionId,
    pub parent: Option<AssistantSessionId>,
    pub workspace: Arc<LapceWorkspace>,
    pub title: RwSignal<String>,
    pub state: RwSignal<SessionState>,
    pub status: RwSignal<AgentStatus>,
    pub plan: RwSignal<String>,
    pub trace: RwSignal<im::Vector<TraceEntry>>,
    pub chat: RwSignal<Vec<ChatTurn>>,
    pub modified_files: RwSignal<im::Vector<FileChange>>,
    pub created_at_ms: u64,
}

impl CoderSession {
    pub fn new(
        cx: Scope,
        workspace: Arc<LapceWorkspace>,
        title: String,
        plan: String,
    ) -> Self {
        Self {
            id: CoderSessionId::next(),
            parent: None,
            workspace,
            title: cx.create_rw_signal(title),
            state: cx.create_rw_signal(SessionState::Active),
            status: cx.create_rw_signal(AgentStatus::Idle),
            plan: cx.create_rw_signal(plan),
            trace: cx.create_rw_signal(im::Vector::new()),
            chat: cx.create_rw_signal(Vec::new()),
            modified_files: cx.create_rw_signal(im::Vector::new()),
            created_at_ms: now_ms(),
        }
    }
}

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
