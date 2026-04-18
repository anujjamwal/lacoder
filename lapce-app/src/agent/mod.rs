//! Agent surfaces — assistant (planning) and coder (running agent) views,
//! plus session data model and runner for coder agents.
//!
//! Phase 1.0 (this module set) runs an in-process stub coder that animates
//! trace, file changes, and chat. The real out-of-process agent, RPC layer,
//! and diff view land in follow-ups.

pub mod assistant_view;
pub mod coder_view;
pub mod registry;
pub mod session;
pub mod stub_runner;
