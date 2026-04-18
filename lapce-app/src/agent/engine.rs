//! In-process coder engine that drives a `CoderSession` through the real
//! `lapce-agent` infrastructure: `LlmProvider` for "thought" turns and
//! `ToolRegistry` for actions.
//!
//! This replaces `stub_runner` as the canonical runner. The default provider
//! is `StubProvider` (always available, no network); real Anthropic / OpenAI
//! providers slot in via `LapceConfig.plugins["agent"]` once the config
//! schema lands. Tool stubs return `ToolError::NotImplemented`; those errors
//! are recorded as trace entries so users can see the wiring without
//! pretending the tools work yet.
//!
//! Phase 2.5 limit: the step sequence is still scripted (matches the old
//! stub_runner demo) so the visible flow is predictable. The fully-driven
//! "ask the LLM, parse tool_use, loop" arrives with the out-of-process
//! runner — at which point real provider calls won't be blocking the UI
//! thread.

use std::{path::PathBuf, rc::Rc, sync::Arc, time::Duration};

use floem::{
    action::exec_after,
    reactive::{SignalUpdate, SignalWith},
};
use lapce_agent::{
    LlmProvider, ProviderMessage, ProviderRequest, ProviderRole, ToolInvocation,
    ToolRegistry, ToolResult,
};

use crate::agent::session::{
    AgentStatus, ChatRole, ChatTurn, CoderSession, FileChange, FileChangeKind,
    SessionState, StepId, StopReason, TraceEntry, TraceKind, now_ms,
};

const MODEL: &str = "claude-stub-default";

#[derive(Clone, Copy)]
struct ToolStep {
    step: u64,
    tool: &'static str,
    summary: &'static str,
    detail: &'static str,
    args_json: &'static str,
    file_change: Option<(&'static str, FileChangeKind, &'static str)>,
}

const TOOL_SCRIPT: &[ToolStep] = &[
    ToolStep {
        step: 2,
        tool: "search",
        summary: "Searched for `parse_config` across the workspace",
        detail: "grep -R 'parse_config' src/ -n",
        args_json: r#"{"pattern":"parse_config"}"#,
        file_change: None,
    },
    ToolStep {
        step: 3,
        tool: "read_file",
        summary: "Read src/config.rs (142 lines)",
        detail: "src/config.rs",
        args_json: r#"{"path":"src/config.rs"}"#,
        file_change: None,
    },
    ToolStep {
        step: 4,
        tool: "write_file",
        summary: "Edited src/config.rs — added `AgentConfig` struct",
        detail: "+18 -2 lines",
        args_json: r#"{"path":"src/config.rs","content":"…"}"#,
        file_change: Some(("src/config.rs", FileChangeKind::Modified, "+18 -2")),
    },
    ToolStep {
        step: 5,
        tool: "write_file",
        summary: "Created src/agent/mod.rs",
        detail: "new file, 64 lines",
        args_json: r#"{"path":"src/agent/mod.rs","content":"…"}"#,
        file_change: Some((
            "src/agent/mod.rs",
            FileChangeKind::Added,
            "new, 64 lines",
        )),
    },
    ToolStep {
        step: 6,
        tool: "shell",
        summary: "Ran `cargo check`",
        detail: "cargo check -p lapce-app",
        args_json: r#"{"cmd":"cargo check -p lapce-app"}"#,
        file_change: None,
    },
];

pub fn launch(
    provider: Arc<dyn LlmProvider>,
    tools: Arc<ToolRegistry>,
    session: Rc<CoderSession>,
) {
    let plan = session.plan.with(String::clone);
    let provider_for_open = provider.clone();
    let session_for_open = session.clone();

    exec_after(Duration::from_millis(300), move |_| {
        // Opening: greet via provider so users see provider responses driving
        // the chat pane (even with the stub).
        let greeting = run_provider(
            provider_for_open.as_ref(),
            "You are an in-process coder agent. Acknowledge the handed-off plan in one sentence.",
            &plan,
        );
        push_chat(&session_for_open, ChatRole::Agent, &greeting);
        push_thought(&session_for_open, 1, "Reading plan and locating target files");
        run_tools(provider_for_open, tools, session_for_open, 0);
    });
}

fn run_tools(
    provider: Arc<dyn LlmProvider>,
    tools: Arc<ToolRegistry>,
    session: Rc<CoderSession>,
    index: usize,
) {
    if index >= TOOL_SCRIPT.len() {
        // Wrap-up reply.
        exec_after(Duration::from_millis(700), move |_| {
            let summary = run_provider(
                provider.as_ref(),
                "Summarize the run in one short sentence.",
                "tool sequence complete",
            );
            push_chat(&session, ChatRole::Agent, &summary);
            session.status.set(AgentStatus::Stopped {
                reason: StopReason::Completed,
            });
            session.state.set(SessionState::Archived);
        });
        return;
    }

    let step = TOOL_SCRIPT[index];
    let provider_for_next = provider.clone();
    let tools_for_next = tools.clone();
    let session_for_next = session.clone();

    exec_after(Duration::from_millis(700), move |_| {
        session.status.set(AgentStatus::Acting {
            step_id: StepId(step.step),
            action: step.tool.to_string(),
        });

        let args: serde_json::Value =
            serde_json::from_str(step.args_json).unwrap_or(serde_json::Value::Null);
        let outcome = tools.invoke(ToolInvocation {
            name: step.tool.to_string(),
            arguments: args,
        });

        let detail = match &outcome {
            Ok(ToolResult { output, is_error }) => {
                if *is_error {
                    format!("{} (tool reported error)\n{}", step.detail, output)
                } else if output.is_empty() {
                    step.detail.to_string()
                } else {
                    format!("{}\n{}", step.detail, output)
                }
            }
            Err(e) => format!("{}\n[stub] {}", step.detail, e),
        };

        let kind = match step.tool {
            "write_file" => TraceKind::FileWrite,
            "shell" => TraceKind::Shell,
            _ => TraceKind::ToolCall,
        };

        push_trace(&session, step.step, kind, step.summary, Some(&detail));

        if let Some((path, kind, summary)) = step.file_change {
            push_file(&session, path, kind, summary);
        }

        run_tools(provider_for_next, tools_for_next, session_for_next, index + 1);
    });
}

fn run_provider(provider: &dyn LlmProvider, system: &str, user: &str) -> String {
    let req = ProviderRequest {
        system: Some(system.to_string()),
        messages: vec![ProviderMessage {
            role: ProviderRole::User,
            content: user.to_string(),
        }],
        model: MODEL.to_string(),
        max_tokens: 256,
        temperature: None,
    };
    match provider.complete(req) {
        Ok(r) => r.content,
        Err(e) => format!("[provider error: {e}]"),
    }
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

fn push_trace(
    session: &CoderSession,
    step: u64,
    kind: TraceKind,
    summary: &str,
    detail: Option<&str>,
) {
    session.trace.update(|t| {
        t.push_back(TraceEntry {
            step_id: StepId(step),
            kind,
            summary: summary.to_string(),
            detail: detail.map(str::to_string),
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
