//! Shared wire types for the SCM backend abstraction.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScmKind {
    GitHub,
    Sapling,
    Piper,
}

impl ScmKind {
    pub fn name(self) -> &'static str {
        match self {
            ScmKind::GitHub => "GitHub",
            ScmKind::Sapling => "Sapling",
            ScmKind::Piper => "Piper",
        }
    }
}

/// Per-file status returned by `ScmBackend::status`.
#[derive(Clone, Debug)]
pub struct FileStatus {
    pub path: PathBuf,
    pub kind: FileStatusKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileStatusKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Untracked,
    Conflicted,
}

#[derive(Clone, Debug)]
pub enum DiffScope {
    /// Working tree vs. HEAD.
    WorkingTree,
    /// All commits on the current branch since divergence from `against`.
    BranchSince { against: String },
    /// All edits attributed to a specific agent session (caller scopes by
    /// commit/branch namespace).
    AgentSession(String),
}

#[derive(Clone, Debug)]
pub struct UnifiedDiff {
    /// Raw unified-diff text. Hunk parsing happens in the consumer (the
    /// editor's diff view).
    pub text: String,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

/// The user-editable draft of a change description.
#[derive(Clone, Debug, Default)]
pub struct ChangeDraft {
    pub title: String,
    pub description: String,
    pub reviewers: Vec<String>,
    pub labels: Vec<String>,
}

/// A reference to a change after it has been created.
#[derive(Clone, Debug)]
pub struct ChangeRef {
    pub backend: ScmKind,
    /// Backend-specific identifier (PR number, CL number, etc.).
    pub id: String,
    /// Human-readable title cached from creation time.
    pub title: String,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SubmitOpts {
    pub squash: bool,
    pub allow_unreviewed: bool,
}

#[derive(Clone, Debug)]
pub struct SubmitOutcome {
    pub committed_sha: Option<String>,
    pub message: String,
}

/// CI / submit-queue status used to decorate workspace cards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipelineStatus {
    Pending,
    Running,
    Stable,
    NeedsReview,
    Failed,
    Unknown,
}

impl PipelineStatus {
    pub fn card_status(self) -> CardStatus {
        match self {
            PipelineStatus::Stable => CardStatus::Stable,
            PipelineStatus::NeedsReview => CardStatus::NeedsReview,
            PipelineStatus::Failed => CardStatus::PipelineFailed,
            PipelineStatus::Pending
            | PipelineStatus::Running
            | PipelineStatus::Unknown => CardStatus::Inactive,
        }
    }
}

/// Mirror of `lapce-app::launchpad::workspace_card::CardStatus` so the SCM
/// crate doesn't depend on the UI crate. Kept structurally identical.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardStatus {
    Stable,
    NeedsReview,
    PipelineFailed,
    Inactive,
}

#[derive(Debug, Error)]
pub enum ScmError {
    #[error("backend {backend:?} does not implement {operation}")]
    NotImplemented {
        backend: ScmKind,
        operation: &'static str,
    },

    #[error("auth required for {backend:?}: {hint}")]
    AuthRequired {
        backend: ScmKind,
        hint: &'static str,
    },

    #[error("git2 error: {0}")]
    Git2(#[from] git2::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("backend error: {0}")]
    Backend(String),
}
