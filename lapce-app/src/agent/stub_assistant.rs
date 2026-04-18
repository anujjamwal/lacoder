//! In-process stub assistant runner.
//!
//! When the user (stub-clicks) sends a message, this schedules a sequence of
//! fake AI reply turns and plan-doc updates via `exec_after`. Same pattern as
//! `stub_runner` for the coder — no threads, no RPC, no real LLM yet.

use std::{rc::Rc, time::Duration};

use floem::{action::exec_after, reactive::SignalUpdate};

use crate::agent::session::{
    AssistantSession, ChatRole, ChatTurn, now_ms,
};

/// Response scenarios the stub assistant can follow based on user input.
/// Routing is keyword-based; real routing moves to `LlmProvider` in Phase 2.3.
#[derive(Clone, Copy)]
enum Scenario {
    RefactorParseConfig,
    Generic,
}

fn route(user_text: &str) -> Scenario {
    let lc = user_text.to_lowercase();
    if lc.contains("parse_config") || lc.contains("agentconfig") {
        Scenario::RefactorParseConfig
    } else {
        Scenario::Generic
    }
}

impl Scenario {
    fn initial_reply(self) -> &'static str {
        match self {
            Scenario::RefactorParseConfig => {
                "Looking at the workspace. I'll scan for parse_config call sites \
                 and map out dependencies before drafting the plan."
            }
            Scenario::Generic => {
                "Drafting a plan from your request. Let me sketch the steps."
            }
        }
    }

    fn plan_chunks(self) -> &'static [&'static str] {
        match self {
            Scenario::RefactorParseConfig => &[
                "Plan — refactor parse_config\n",
                "  1. Locate parse_config and its call sites (src/config.rs, src/main.rs)\n",
                "  2. Extract AgentConfig struct with the existing fields\n",
                "  3. Update parse_config to delegate to AgentConfig::from_toml\n",
                "  4. Keep the old free function as a thin wrapper for backwards compat\n",
                "  5. Run cargo check across the workspace\n",
            ],
            Scenario::Generic => &[
                "Plan — scoped change\n",
                "  1. Identify the target files and their dependents\n",
                "  2. Outline the minimal edit set\n",
                "  3. Implement and run cargo check\n",
                "  4. Summarize the diff for review\n",
            ],
        }
    }

    fn final_reply(self) -> &'static str {
        match self {
            Scenario::RefactorParseConfig => {
                "Plan drafted. Five steps, no API break. Ready when you are — hit \
                 'Launch coder from this plan' to hand off."
            }
            Scenario::Generic => {
                "Plan drafted. Hit 'Launch coder from this plan' to hand off."
            }
        }
    }
}

/// User-initiated: push the user's text as a turn, then animate the canned
/// assistant reply and plan-building for whichever scenario matches.
pub fn send_message(session: Rc<AssistantSession>, user_text: String) {
    let user_text = user_text.trim().to_string();
    if user_text.is_empty() {
        return;
    }
    let scenario = route(&user_text);

    session.transcript.update(|t| {
        t.push(ChatTurn {
            role: ChatRole::User,
            content: user_text,
            timestamp_ms: now_ms(),
        });
    });

    let s1 = session.clone();
    exec_after(Duration::from_millis(400), move |_| {
        s1.transcript.update(|t| {
            t.push(ChatTurn {
                role: ChatRole::Agent,
                content: scenario.initial_reply().to_string(),
                timestamp_ms: now_ms(),
            });
        });
        append_plan_chunks(s1, scenario, 0);
    });
}

fn append_plan_chunks(
    session: Rc<AssistantSession>,
    scenario: Scenario,
    index: usize,
) {
    let chunks = scenario.plan_chunks();
    if index >= chunks.len() {
        exec_after(Duration::from_millis(500), move |_| {
            session.transcript.update(|t| {
                t.push(ChatTurn {
                    role: ChatRole::Agent,
                    content: scenario.final_reply().to_string(),
                    timestamp_ms: now_ms(),
                });
            });
        });
        return;
    }

    let chunk = chunks[index];
    let s = session.clone();
    exec_after(Duration::from_millis(450), move |_| {
        s.plan.update(|p| p.push_str(chunk));
        append_plan_chunks(s, scenario, index + 1);
    });
}
