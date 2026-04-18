//! Source-control backend abstraction for Lacoder.
//!
//! Defines the `ScmBackend` trait and ships two implementations today:
//! - `github::GitHubBackend` — git2 for local ops + (later) octocrab for PR
//!   creation/submit. Phase 4.0 ships local-only ops; PR creation is wired
//!   in a follow-up once the auth surface is decided.
//! - `piper::PiperBackend` — trait-complete stub that returns
//!   `ScmError::NotImplemented` from every operation. Lets the IDE wire the
//!   handoff/CL flow against an internal-monorepo workspace without
//!   pretending it works.
//!
//! `Sapling/hg` slot reserved for Phase 7.

pub mod github;
pub mod piper;
pub mod types;

pub use crate::types::{
    CardStatus, ChangeDraft, ChangeRef, DiffScope, FileStatus, PipelineStatus,
    ScmError, ScmKind, SubmitOpts, SubmitOutcome, UnifiedDiff,
};

use std::path::Path;

use url::Url;

/// One Source-control backend per workspace. All ops are synchronous —
/// callers (typically the proxy or a Phase-1 agent runner) should run them
/// off the UI thread.
pub trait ScmBackend: Send + Sync {
    fn id(&self) -> ScmKind;

    /// Working-tree status. May involve disk I/O.
    fn status(&self) -> Result<Vec<FileStatus>, ScmError>;

    /// Unified diff for a given scope.
    fn diff(&self, scope: DiffScope) -> Result<UnifiedDiff, ScmError>;

    /// The active change being worked on (current branch / CL number).
    fn current_change(&self) -> Result<Option<ChangeRef>, ScmError>;

    /// Create a new change (PR / CL / phab diff) from the current branch.
    fn create_change(&self, draft: ChangeDraft) -> Result<ChangeRef, ScmError>;

    /// Update title/description/reviewers on an existing change.
    fn update_change(
        &self,
        change: &ChangeRef,
        draft: ChangeDraft,
    ) -> Result<(), ScmError>;

    /// Submit / merge / land a change.
    fn submit(
        &self,
        change: &ChangeRef,
        opts: SubmitOpts,
    ) -> Result<SubmitOutcome, ScmError>;

    /// Web URL for a change, if the backend has one.
    fn open_url(&self, change: &ChangeRef) -> Option<Url>;

    /// Pipeline / CI status for a change.
    fn fetch_pipeline(
        &self,
        change: &ChangeRef,
    ) -> Result<PipelineStatus, ScmError>;
}

/// Detect which backend (if any) applies to the given workspace path.
///
/// Detection rules (in order):
/// 1. `<root>/.piper/.config` → Piper stub (treated as a hint; real internal
///    monorepos have their own marker).
/// 2. `<root>/.git` directory → GitHub backend (git2 over the local repo).
/// 3. Otherwise → `None`.
pub fn detect(workspace: &Path) -> Option<Box<dyn ScmBackend>> {
    if workspace.join(".piper").join(".config").is_file() {
        return Some(Box::new(piper::PiperBackend::new(workspace.to_owned())));
    }
    if workspace.join(".git").exists() {
        return github::GitHubBackend::open(workspace).ok().map(|b| {
            let b: Box<dyn ScmBackend> = Box::new(b);
            b
        });
    }
    None
}
