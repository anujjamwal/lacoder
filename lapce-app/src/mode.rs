//! Top-level mode model for the agentic-coding IDE shell.
//!
//! See `docs/designs/` and the Phase 0 roadmap. Two layers:
//! - `AppMode` lives at the window level and gates Launchpad vs. an open workspace.
//! - `WorkspaceMode` lives inside a `WindowTabData` and gates Home / Assistant /
//!   CoderAgent / Editor surfaces within a workspace.
//!
//! `Focus` in `window_tab.rs` stays orthogonal — it targets keyboard focus
//! within the active mode, not mode switching itself.

use serde::{Deserialize, Serialize};

use crate::id::{AssistantSessionId, CoderSessionId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppMode {
    /// No workspace is bound. The launchpad screen lists recent workspaces
    /// and running agents and lets the user open/create one.
    Launchpad,
    /// A workspace is bound; render the active window tab.
    Workspace,
}

impl Default for AppMode {
    fn default() -> Self {
        AppMode::Launchpad
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceMode {
    /// Dashboard for this workspace. Default surface after opening a workspace.
    Home,
    /// Chat-driven planning session. Produces a plan that can be handed off to
    /// coder agents.
    Assistant(AssistantSessionId),
    /// Running coder agent view (plan, trace, files, chat, terminal).
    CoderAgent(CoderSessionId),
    /// Classic editor surface (file tree + editor panes + terminal). View-only
    /// for now (terminal remains interactive).
    Editor,
}

impl Default for WorkspaceMode {
    fn default() -> Self {
        WorkspaceMode::Home
    }
}
