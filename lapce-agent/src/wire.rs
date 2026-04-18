//! Wire types for the eventual app↔agent RPC channel.
//!
//! Phase 2.3 / item-7 scaffold: types are defined here so both the agent
//! runner (when split out) and the app can serialize against them. Actual
//! transport (unix socket, framed json) lands with the proxy supervisor
//! commit.

use serde::{Deserialize, Serialize};

/// App → Agent: requests the app makes to a running agent.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgentRequest {
    Pause,
    Resume,
    Interrupt {
        reason: String,
    },
    /// Inject a new message into the conversation.
    /// `kind: Question` is non-blocking (the answerer task replies on a
    /// side channel without affecting the main loop). `kind: Directive`
    /// changes course — typically sent after Pause.
    InjectMessage {
        kind: InjectKind,
        text: String,
    },
    ApproveStep {
        step_id: u64,
    },
    RejectStep {
        step_id: u64,
        reason: String,
    },
    RequestDraftCL,
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InjectKind {
    Question,
    Directive,
}

/// Agent → App: notifications the agent streams back.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AgentEvent {
    StatusChanged(StatusWire),
    StepStarted {
        step_id: u64,
        kind: String,
    },
    StepCompleted {
        step_id: u64,
    },
    TraceAppended {
        step_id: u64,
        kind: String,
        summary: String,
        detail: Option<String>,
    },
    FileChanged {
        path: String,
        kind: FileChangeKindWire,
        summary: String,
    },
    AwaitingApproval {
        step_id: u64,
        gate: ApprovalGateWire,
    },
    TerminalOutput {
        bytes: Vec<u8>,
    },
    /// Reply to an InjectMessage { kind: Question }.
    ChatReply {
        text: String,
        anchor_step_id: Option<u64>,
    },
    /// Draft text the agent generated in response to RequestDraftCL.
    DraftCL {
        title: String,
        description: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusWire {
    Idle,
    Thinking { step_id: u64 },
    Acting { step_id: u64, action: String },
    AwaitingApproval { step_id: u64 },
    Paused,
    Stopped { reason: String },
    Failed(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeKindWire {
    Added,
    Modified,
    Deleted,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ApprovalGateWire {
    BeforeStep { action: String },
    BeforeShipping { files_changed: usize, summary: String },
    OnDestructive { action: String, reason: String },
}
