//! In-process stub coder runner.
//!
//! Schedules fake trace entries, file changes, and chat replies via
//! `exec_after` callbacks on the UI thread. No cross-thread plumbing — lets
//! us validate the view layout and reactive wiring without a real LLM or a
//! separate agent process. The real runner (separate binary, RPC'd through
//! the proxy) lands in a follow-up.

use std::{path::PathBuf, rc::Rc, time::Duration};

use floem::{action::exec_after, reactive::SignalUpdate};

use crate::agent::session::{
    AgentStatus, ChatRole, ChatTurn, CoderSession, FileChange, FileChangeKind,
    SessionState, StepId, StopReason, TraceEntry, TraceKind, now_ms,
};

/// Drive a session through a hardcoded sequence of fake steps.
///
/// All callbacks run on the UI thread, so they can freely update signals.
pub fn launch(session: Rc<CoderSession>) {
    schedule(session, 0);
}

fn schedule(session: Rc<CoderSession>, step: usize) {
    let delay_ms = match step {
        0 => 300,
        _ => 700,
    };

    exec_after(Duration::from_millis(delay_ms), move |_| {
        let done = apply_step(&session, step);
        if !done {
            schedule(session, step + 1);
        }
    });
}

/// Apply step `n` to the session. Returns true when the sequence is over.
fn apply_step(session: &CoderSession, step: usize) -> bool {
    match step {
        0 => {
            push_chat(
                session,
                ChatRole::Agent,
                "Starting work on the handed-off plan.",
            );
            push_thought(session, 1, "Reading plan and locating target files");
        }
        1 => {
            push_tool(
                session,
                2,
                "search",
                "Searched for `parse_config` across the workspace",
                TraceKind::ToolCall,
                Some("grep -R 'parse_config' src/ -n"),
            );
        }
        2 => {
            push_tool(
                session,
                3,
                "read_file",
                "Read src/config.rs (142 lines)",
                TraceKind::ToolCall,
                None,
            );
        }
        3 => {
            push_tool(
                session,
                4,
                "write_file",
                "Edited src/config.rs — added `AgentConfig` struct",
                TraceKind::FileWrite,
                Some("+18 -2 lines"),
            );
            push_file(
                session,
                "src/config.rs",
                FileChangeKind::Modified,
                "+18 -2",
            );
        }
        4 => {
            push_tool(
                session,
                5,
                "write_file",
                "Created src/agent/mod.rs",
                TraceKind::FileWrite,
                Some("new file, 64 lines"),
            );
            push_file(
                session,
                "src/agent/mod.rs",
                FileChangeKind::Added,
                "new, 64 lines",
            );
        }
        5 => {
            push_tool(
                session,
                6,
                "shell",
                "Ran `cargo check`",
                TraceKind::Shell,
                Some("cargo check -p lapce-app\n    Finished dev [unoptimized + debuginfo]"),
            );
        }
        6 => {
            push_chat(
                session,
                ChatRole::Agent,
                "All changes applied. Build is clean. Ready for review.",
            );
            session.status.set(AgentStatus::Stopped {
                reason: StopReason::Completed,
            });
            session.state.set(SessionState::Archived);
            return true;
        }
        _ => return true,
    }
    false
}

fn push_thought(session: &CoderSession, step: u64, summary: &str) {
    session
        .status
        .set(AgentStatus::Thinking { step_id: StepId(step) });
    session.trace.update(|t| {
        t.push_back(TraceEntry {
            step_id: StepId(step),
            kind: TraceKind::Thought,
            summary: summary.to_string(),
            detail: None,
        });
    });
}

fn push_tool(
    session: &CoderSession,
    step: u64,
    action: &str,
    summary: &str,
    kind: TraceKind,
    detail: Option<&str>,
) {
    session.status.set(AgentStatus::Acting {
        step_id: StepId(step),
        action: action.to_string(),
    });
    session.trace.update(|t| {
        t.push_back(TraceEntry {
            step_id: StepId(step),
            kind,
            summary: summary.to_string(),
            detail: detail.map(|s| s.to_string()),
        });
    });
}

fn push_file(
    session: &CoderSession,
    path: &str,
    kind: FileChangeKind,
    summary: &str,
) {
    session.modified_files.update(|f| {
        f.push_back(FileChange {
            path: PathBuf::from(path),
            kind,
            summary: summary.to_string(),
        });
    });
}

fn push_chat(session: &CoderSession, role: ChatRole, content: &str) {
    session.chat.update(|c| {
        c.push(ChatTurn {
            role,
            content: content.to_string(),
            timestamp_ms: now_ms(),
        });
    });
}
