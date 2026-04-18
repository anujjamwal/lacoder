//! Agent surfaces — assistant (planning) and coder (running agent) views.
//!
//! Phase 0 ships placeholder views so `WorkspaceMode::Assistant` and
//! `WorkspaceMode::CoderAgent` have something to render. Real implementations
//! (session model, RPC, chat, trace, diff, terminal) arrive in Phase 1 and
//! Phase 2.

pub mod assistant_view;
pub mod coder_view;
