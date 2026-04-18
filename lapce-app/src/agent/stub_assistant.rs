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

/// A pre-baked conversation scenario the assistant will walk through.
#[derive(Clone, Copy)]
pub enum Scenario {
    RefactorParseConfig,
}

impl Scenario {
    fn user_prompt(self) -> &'static str {
        match self {
            Scenario::RefactorParseConfig => {
                "Plan a refactor of parse_config that introduces an AgentConfig struct and keeps the public API stable."
            }
        }
    }

    fn initial_reply(self) -> &'static str {
        match self {
            Scenario::RefactorParseConfig => {
                "Looking at the workspace. I'll scan for parse_config call sites \
                 and map out dependencies before drafting the plan."
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
        }
    }

    fn final_reply(self) -> &'static str {
        match self {
            Scenario::RefactorParseConfig => {
                "Plan drafted. Five steps, no API break. Ready when you are — hit \
                 'Launch coder from this plan' to hand off."
            }
        }
    }
}

/// User-initiated: push a user turn and animate the assistant's reply.
pub fn send_message(session: Rc<AssistantSession>, scenario: Scenario) {
    // User turn appears immediately.
    session.transcript.update(|t| {
        t.push(ChatTurn {
            role: ChatRole::User,
            content: scenario.user_prompt().to_string(),
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
