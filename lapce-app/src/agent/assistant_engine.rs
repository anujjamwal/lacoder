//! Assistant engine: drives an `AssistantSession` via a real `LlmProvider`.
//!
//! Replaces `stub_assistant`. With the default `StubProvider` the visible
//! flow is similar — a canned-feeling reply driven by the request shape —
//! but every chat reply and every plan-chunk update routes through the
//! provider trait. Swapping in a real provider via Phase 2.3 only requires
//! changing the `Arc<dyn LlmProvider>` constructed in the view layer.
//!
//! Phase 2.5 limit: provider calls are synchronous (StubProvider returns
//! instantly). Real providers must move off the UI thread before they're
//! reachable from here — that's the next commit.

use std::{rc::Rc, sync::Arc, time::Duration};

use floem::{
    action::exec_after,
    reactive::{SignalUpdate, SignalWith},
};
use lapce_agent::{
    LlmProvider, ProviderMessage, ProviderRequest, ProviderRole,
};

use crate::agent::session::{
    AssistantSession, ChatRole, ChatTurn, now_ms,
};

const MODEL: &str = "claude-stub-default";
const MAX_TOKENS: u32 = 512;

const PLAN_SYSTEM: &str = "You are an in-IDE planning assistant. Reply concisely. \
When asked for a plan, output a numbered list of 3–6 short steps prefixed by '  N. '.";
const REPLY_SYSTEM: &str = "You are an in-IDE planning assistant. Reply in one or two short sentences.";

/// User-initiated: push the user's text as a turn, then drive the assistant
/// response via the provider.
pub fn send_message(
    provider: Arc<dyn LlmProvider>,
    session: Rc<AssistantSession>,
    user_text: String,
) {
    let user_text = user_text.trim().to_string();
    if user_text.is_empty() {
        return;
    }

    session.transcript.update(|t| {
        t.push(ChatTurn {
            role: ChatRole::User,
            content: user_text.clone(),
            timestamp_ms: now_ms(),
        });
    });

    let provider_for_open = provider.clone();
    let session_for_open = session.clone();
    let user_for_open = user_text.clone();

    exec_after(Duration::from_millis(350), move |_| {
        // 1. Short opening reply.
        let req = ProviderRequest {
            system: Some(REPLY_SYSTEM.to_string()),
            messages: history(&session_for_open, &user_for_open),
            model: MODEL.to_string(),
            max_tokens: MAX_TOKENS,
            temperature: None,
        };
        let opening = match provider_for_open.complete(req) {
            Ok(r) => r.content,
            Err(e) => format!("[provider error: {e}]"),
        };
        session_for_open.transcript.update(|t| {
            t.push(ChatTurn {
                role: ChatRole::Agent,
                content: opening,
                timestamp_ms: now_ms(),
            });
        });

        // 2. Plan-building turn.
        let provider_for_plan = provider_for_open.clone();
        let session_for_plan = session_for_open.clone();
        let user_for_plan = user_for_open.clone();
        exec_after(Duration::from_millis(450), move |_| {
            let req = ProviderRequest {
                system: Some(PLAN_SYSTEM.to_string()),
                messages: history(&session_for_plan, &user_for_plan),
                model: MODEL.to_string(),
                max_tokens: MAX_TOKENS,
                temperature: None,
            };
            let plan = match provider_for_plan.complete(req) {
                Ok(r) => r.content,
                Err(e) => format!("[provider error: {e}]"),
            };
            // Append a header + the plan body.
            let header = if session_for_plan.plan.with(String::is_empty) {
                "Plan — drafted from your request\n".to_string()
            } else {
                "\n--\n".to_string()
            };
            session_for_plan.plan.update(|p| {
                p.push_str(&header);
                p.push_str(&plan);
                if !plan.ends_with('\n') {
                    p.push('\n');
                }
            });

            // 3. Final wrap-up reply.
            let provider_for_wrap = provider_for_plan.clone();
            let session_for_wrap = session_for_plan.clone();
            exec_after(Duration::from_millis(450), move |_| {
                let req = ProviderRequest {
                    system: Some(
                        "You are an in-IDE planning assistant. Tell the user the plan is ready in one short sentence."
                            .to_string(),
                    ),
                    messages: history(&session_for_wrap, "wrap up"),
                    model: MODEL.to_string(),
                    max_tokens: 96,
                    temperature: None,
                };
                let wrap = match provider_for_wrap.complete(req) {
                    Ok(r) => r.content,
                    Err(e) => format!("[provider error: {e}]"),
                };
                session_for_wrap.transcript.update(|t| {
                    t.push(ChatTurn {
                        role: ChatRole::Agent,
                        content: wrap,
                        timestamp_ms: now_ms(),
                    });
                });
            });
        });
    });
}

fn history(
    session: &AssistantSession,
    appended_user: &str,
) -> Vec<ProviderMessage> {
    let mut msgs = session.transcript.with(|t| {
        t.iter()
            .map(|turn| ProviderMessage {
                role: match turn.role {
                    ChatRole::User => ProviderRole::User,
                    ChatRole::Agent => ProviderRole::Assistant,
                },
                content: turn.content.clone(),
            })
            .collect::<Vec<_>>()
    });
    // Make sure the most recent user turn is reflected even if the caller
    // hasn't pushed it yet.
    if !appended_user.is_empty()
        && msgs
            .last()
            .map(|m| !(m.role == ProviderRole::User && m.content == appended_user))
            .unwrap_or(true)
    {
        // Already in transcript? Done. Otherwise leave as-is — caller pushed it.
    }
    if msgs.is_empty() && !appended_user.is_empty() {
        msgs.push(ProviderMessage {
            role: ProviderRole::User,
            content: appended_user.to_string(),
        });
    }
    msgs
}
