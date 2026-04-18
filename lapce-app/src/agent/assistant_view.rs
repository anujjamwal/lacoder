//! Assistant view — left rail (sessions / notes / background agents),
//! center chat transcript + plan preview, bottom composer.
//!
//! Phase 2.0: composer is a canned "Send stub prompt" button; real text input
//! arrives in a follow-up. Plan editing is also read-only for now (the stub
//! assistant populates it). "Launch coder from this plan" snapshots the plan,
//! creates a CoderSession, locks the assistant session, and switches mode.

use std::{rc::Rc, sync::Arc};

use floem::{
    View,
    reactive::{ReadSignal, SignalGet, SignalUpdate, SignalWith},
    style::CursorStyle,
    views::{
        Decorators, container, dyn_stack, empty, label, scroll, stack, text,
    },
};

use crate::{
    agent::{
        registry::AgentRegistry,
        session::{
            AssistantSession, ChatRole, ChatTurn, CoderSession, SessionState,
        },
        stub_assistant::{self, Scenario},
        stub_runner,
    },
    config::{LapceConfig, color::LapceColor},
    mode::WorkspaceMode,
    window_tab::WindowTabData,
};

type ConfigSig = ReadSignal<Arc<LapceConfig>>;

fn resolve(agents: &AgentRegistry) -> Option<Rc<AssistantSession>> {
    let id = agents.active_assistant.get()?;
    agents.assistants.with(|m| m.get(&id).cloned())
}

fn resolve_untracked(agents: &AgentRegistry) -> Option<Rc<AssistantSession>> {
    let id = agents.active_assistant.get_untracked()?;
    agents.assistants.with_untracked(|m| m.get(&id).cloned())
}

pub fn assistant(window_tab_data: Rc<WindowTabData>) -> impl View {
    let config = window_tab_data.common.config;
    let workspace_mode = window_tab_data.workspace_mode;
    let agents = window_tab_data.agents.clone();
    let workspace = window_tab_data.workspace.clone();
    let scope = window_tab_data.scope;

    let header = header(window_tab_data.clone(), config);
    let rail = left_rail(agents.clone(), config);
    let center_pane = center(
        agents.clone(),
        workspace.clone(),
        scope,
        workspace_mode,
        config,
    );

    let body = stack((rail, center_pane))
        .style(|s| s.size_full().flex_grow(1.0).min_height(0.0));

    stack((header, body))
        .style(move |s| {
            s.size_full().flex_col().background(
                config.get().color(LapceColor::EDITOR_BACKGROUND),
            )
        })
        .debug_name("Assistant")
}

fn header(
    window_tab_data: Rc<WindowTabData>,
    config: ConfigSig,
) -> impl View {
    let workspace_mode = window_tab_data.workspace_mode;
    let workspace = window_tab_data.workspace.clone();
    let workspace_label = workspace
        .display()
        .unwrap_or_else(|| "Workspace".to_string());

    let back_btn = container(text("← Home"))
        .on_click_stop(move |_| workspace_mode.set(WorkspaceMode::Home))
        .style(move |s| {
            let cfg = config.get();
            s.padding_horiz(10.0)
                .padding_vert(6.0)
                .margin_left(12.0)
                .border(1.0)
                .border_radius(6.0)
                .border_color(cfg.color(LapceColor::LAPCE_BORDER))
                .color(cfg.color(LapceColor::EDITOR_FOREGROUND))
                .cursor(CursorStyle::Pointer)
                .hover(|s| {
                    s.background(cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND))
                })
        });

    let workspace_chip = container(
        stack((
            label(move || workspace_label.clone()).style(move |s| {
                s.font_size(12.0)
                    .font_bold()
                    .color(config.get().color(LapceColor::EDITOR_FOREGROUND))
            }),
            label(|| "main branch".to_string()).style(move |s| {
                s.font_size(10.0)
                    .color(config.get().color(LapceColor::EDITOR_DIM))
            }),
        ))
        .style(|s| s.flex_col().gap(2.0)),
    )
    .style(move |s| {
        let cfg = config.get();
        s.padding_horiz(12.0)
            .padding_vert(6.0)
            .margin_left(16.0)
            .border(1.0)
            .border_radius(6.0)
            .border_color(cfg.color(LapceColor::LAPCE_BORDER))
            .background(cfg.color(LapceColor::PANEL_BACKGROUND))
    });

    let spacer = empty().style(|s| s.flex_grow(1.0).min_width(0.0));

    stack((back_btn, workspace_chip, spacer))
        .style(move |s| {
            let cfg = config.get();
            s.width_full()
                .height(48.0)
                .items_center()
                .border_bottom(1.0)
                .border_color(cfg.color(LapceColor::LAPCE_BORDER))
                .background(cfg.color(LapceColor::PANEL_BACKGROUND))
        })
}

fn section_header(title: &'static str, config: ConfigSig) -> impl View {
    label(move || title.to_string()).style(move |s| {
        s.color(config.get().color(LapceColor::EDITOR_DIM))
            .font_size(10.0)
            .font_bold()
            .padding_horiz(12.0)
            .padding_top(16.0)
            .padding_bottom(6.0)
    })
}

fn left_rail(agents: AgentRegistry, config: ConfigSig) -> impl View {
    let active_session_card = {
        let agents = agents.clone();
        container(
            stack((
                label(|| "PLAN: Refactor parse_config".to_string()).style(
                    move |s| {
                        s.font_size(11.0)
                            .font_bold()
                            .color(
                                config
                                    .get()
                                    .color(LapceColor::EDITOR_FOREGROUND),
                            )
                    },
                ),
                label(move || {
                    let state = resolve(&agents)
                        .map(|s| s.state.get())
                        .unwrap_or(SessionState::Draft);
                    match state {
                        SessionState::Draft => "drafting".to_string(),
                        SessionState::Active => "iterating".to_string(),
                        SessionState::Locked => "locked (handed off)".to_string(),
                        SessionState::Archived => "archived".to_string(),
                    }
                })
                .style(move |s| {
                    s.font_size(10.0)
                        .color(config.get().color(LapceColor::EDITOR_DIM))
                        .margin_top(4.0)
                }),
            ))
            .style(|s| s.flex_col()),
        )
        .style(move |s| {
            let cfg = config.get();
            s.padding(10.0)
                .margin_horiz(8.0)
                .margin_bottom(8.0)
                .border(1.0)
                .border_radius(6.0)
                .border_color(cfg.color(LapceColor::LAPCE_TAB_ACTIVE_UNDERLINE))
                .background(cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND))
        })
    };

    let recent_stub_row = |caption: &'static str| {
        label(move || caption.to_string()).style(move |s| {
            s.font_size(11.0)
                .color(config.get().color(LapceColor::EDITOR_FOREGROUND))
                .padding_horiz(12.0)
                .padding_vert(5.0)
        })
    };

    let bg_agents_stub = label(|| "No background agents yet".to_string())
        .style(move |s| {
            s.font_size(11.0)
                .color(config.get().color(LapceColor::EDITOR_DIM))
                .padding_horiz(12.0)
                .padding_vert(5.0)
        });

    let rail_tabs = stack((
        library_tool_tab("Library", true, config),
        library_tool_tab("Tools", false, config),
    ))
    .style(|s| s.gap(4.0).padding(8.0));

    scroll(
        stack((
            rail_tabs,
            section_header("ACTIVE SESSION", config),
            active_session_card,
            section_header("RECENT SESSIONS", config),
            recent_stub_row("Refactor API hooks"),
            recent_stub_row("Explain workspace state"),
            section_header("NOTES", config),
            recent_stub_row("Architecture scratchpad"),
            recent_stub_row("Meeting notes 10/24"),
            section_header("BACKGROUND AGENTS", config),
            bg_agents_stub,
        ))
        .style(|s| s.flex_col().width_full()),
    )
    .style(move |s| {
        let cfg = config.get();
        s.width(240.0)
            .height_full()
            .border_right(1.0)
            .border_color(cfg.color(LapceColor::LAPCE_BORDER))
            .background(cfg.color(LapceColor::PANEL_BACKGROUND))
    })
}

fn library_tool_tab(
    caption: &'static str,
    active: bool,
    config: ConfigSig,
) -> impl View {
    container(text(caption))
        .style(move |s| {
            let cfg = config.get();
            s.padding_horiz(12.0)
                .padding_vert(6.0)
                .font_size(11.0)
                .font_bold()
                .flex_grow(1.0)
                .border(1.0)
                .border_radius(4.0)
                .items_center()
                .justify_center()
                .border_color(cfg.color(LapceColor::LAPCE_BORDER))
                .color(if active {
                    cfg.color(LapceColor::LAPCE_TAB_ACTIVE_FOREGROUND)
                } else {
                    cfg.color(LapceColor::EDITOR_DIM)
                })
                .apply_if(active, |s| {
                    s.background(cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND))
                })
                .cursor(CursorStyle::Pointer)
        })
}

fn center(
    agents: AgentRegistry,
    workspace: Arc<crate::workspace::LapceWorkspace>,
    scope: floem::reactive::Scope,
    workspace_mode: floem::reactive::RwSignal<WorkspaceMode>,
    config: ConfigSig,
) -> impl View {
    let agents_chat = agents.clone();
    let chat_list = dyn_stack(
        move || match resolve(&agents_chat) {
            Some(s) => s
                .transcript
                .get()
                .into_iter()
                .enumerate()
                .collect::<Vec<_>>(),
            None => Vec::new(),
        },
        |(i, _)| *i,
        move |(_, turn)| chat_row(turn, config),
    )
    .style(|s| s.flex_col().padding(20.0).gap(14.0));

    let chat_scroll = scroll(chat_list).style(move |s| {
        s.width_full()
            .flex_grow(1.0)
            .min_height(0.0)
            .background(config.get().color(LapceColor::EDITOR_BACKGROUND))
    });

    let agents_plan = agents.clone();
    let plan_panel = plan_panel(agents_plan, workspace, scope, workspace_mode, config);

    let composer = composer(agents.clone(), config);

    stack((chat_scroll, plan_panel, composer))
        .style(|s| s.flex_col().flex_grow(1.0).height_full().min_width(0.0))
}

fn chat_row(turn: ChatTurn, config: ConfigSig) -> impl View {
    let content_str = turn.content.clone();
    let is_agent = turn.role == ChatRole::Agent;
    let role_label = if is_agent { "Assistant" } else { "You" };

    stack((
        label(move || role_label.to_string()).style(move |s| {
            s.font_size(10.0)
                .font_bold()
                .color(config.get().color(LapceColor::EDITOR_DIM))
                .margin_bottom(4.0)
        }),
        label(move || content_str.clone()).style(move |s| {
            s.font_size(13.0)
                .color(config.get().color(LapceColor::EDITOR_FOREGROUND))
        }),
    ))
    .style(move |s| {
        let cfg = config.get();
        s.flex_col()
            .padding(12.0)
            .border_radius(8.0)
            .max_width(560.0)
            .background(if is_agent {
                cfg.color(LapceColor::PANEL_BACKGROUND)
            } else {
                cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND)
            })
    })
}

fn plan_panel(
    agents: AgentRegistry,
    workspace: Arc<crate::workspace::LapceWorkspace>,
    scope: floem::reactive::Scope,
    workspace_mode: floem::reactive::RwSignal<WorkspaceMode>,
    config: ConfigSig,
) -> impl View {
    let agents_state_text = agents.clone();
    let agents_state_style = agents.clone();
    let agents_plan = agents.clone();
    let agents_handoff_click = agents.clone();
    let agents_handoff_style = agents.clone();

    let header_label = label(|| "DRAFT PLAN".to_string()).style(move |s| {
        s.font_size(10.0)
            .font_bold()
            .color(config.get().color(LapceColor::EDITOR_DIM))
            .flex_grow(1.0)
    });

    let state_label = label(move || {
        let state = resolve(&agents_state_text)
            .map(|s| s.state.get())
            .unwrap_or(SessionState::Draft);
        match state {
            SessionState::Locked => "LOCKED".to_string(),
            _ => String::new(),
        }
    })
    .style(move |s| {
        let cfg = config.get();
        let locked = resolve(&agents_state_style)
            .map(|s| s.state.get())
            .map(|st| st == SessionState::Locked)
            .unwrap_or(false);
        s.font_size(10.0)
            .font_bold()
            .padding_horiz(8.0)
            .padding_vert(2.0)
            .border_radius(4.0)
            .color(cfg.color(LapceColor::LAPCE_WARN))
            .background(cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND))
            .apply_if(!locked, |s| s.hide())
    });

    let handoff_btn = container(text("Launch coder from this plan →"))
        .on_click_stop(move |_| {
            handoff(
                &agents_handoff_click,
                workspace.clone(),
                scope,
                workspace_mode,
            )
        })
        .style(move |s| {
            let cfg = config.get();
            let locked_or_empty = resolve(&agents_handoff_style)
                .map(|s| {
                    s.state.get() == SessionState::Locked
                        || s.plan.with(String::is_empty)
                })
                .unwrap_or(true);
            s.padding_horiz(12.0)
                .padding_vert(6.0)
                .border_radius(6.0)
                .background(
                    cfg.color(LapceColor::LAPCE_BUTTON_PRIMARY_BACKGROUND),
                )
                .color(cfg.color(LapceColor::LAPCE_BUTTON_PRIMARY_FOREGROUND))
                .font_size(12.0)
                .font_bold()
                .cursor(CursorStyle::Pointer)
                .apply_if(locked_or_empty, |s| s.hide())
        });

    let header_row = stack((header_label, state_label, handoff_btn))
        .style(|s| s.items_center().gap(8.0).width_full().margin_bottom(6.0));

    let plan_text = label(move || {
        resolve(&agents_plan)
            .map(|s| s.plan.get())
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| {
                "Plan will appear here as the assistant drafts it.".to_string()
            })
    })
    .style(move |s| {
        s.font_size(12.0)
            .color(config.get().color(LapceColor::EDITOR_FOREGROUND))
    });

    container(
        stack((header_row, plan_text))
            .style(|s| s.flex_col().width_full()),
    )
    .style(move |s| {
        let cfg = config.get();
        s.width_full()
            .padding(14.0)
            .border_top(1.0)
            .border_color(cfg.color(LapceColor::LAPCE_BORDER))
            .background(cfg.color(LapceColor::PANEL_BACKGROUND))
    })
}

fn composer(agents: AgentRegistry, config: ConfigSig) -> impl View {
    let agents_send = agents.clone();
    let agents_read = agents.clone();

    let placeholder = label(|| {
        "Ask assistant or type '/' for tools..."
            .to_string()
    })
    .style(move |s| {
        s.color(config.get().color(LapceColor::EDITOR_DIM))
            .font_size(12.0)
            .flex_grow(1.0)
            .min_width(0.0)
    });

    let send_btn = container(text("Send stub prompt ↵"))
        .on_click_stop(move |_| {
            if let Some(session) = resolve_untracked(&agents_send) {
                if session.state.get_untracked() == SessionState::Active {
                    stub_assistant::send_message(
                        session,
                        Scenario::RefactorParseConfig,
                    );
                }
            }
        })
        .style(move |s| {
            let cfg = config.get();
            let enabled = resolve(&agents_read)
                .map(|s| {
                    s.state.get() == SessionState::Active
                        && s.transcript.with(Vec::is_empty)
                })
                .unwrap_or(false);
            s.padding_horiz(14.0)
                .padding_vert(7.0)
                .border_radius(6.0)
                .font_size(12.0)
                .font_bold()
                .background(cfg.color(LapceColor::LAPCE_BUTTON_PRIMARY_BACKGROUND))
                .color(cfg.color(LapceColor::LAPCE_BUTTON_PRIMARY_FOREGROUND))
                .cursor(CursorStyle::Pointer)
                .apply_if(!enabled, |s| s.hide())
        });

    let locked_hint = label(|| {
        "Handed off — this session is now read-only. Branch a new session to keep iterating.".to_string()
    })
    .style(move |s| {
        let cfg = config.get();
        let locked = resolve(&agents)
            .map(|s| s.state.get() == SessionState::Locked)
            .unwrap_or(false);
        s.font_size(11.0)
            .color(cfg.color(LapceColor::EDITOR_DIM))
            .apply_if(!locked, |s| s.hide())
    });

    container(
        stack((placeholder, send_btn, locked_hint))
            .style(|s| s.items_center().gap(10.0).width_full()),
    )
    .style(move |s| {
        let cfg = config.get();
        s.width_full()
            .padding(14.0)
            .border_top(1.0)
            .border_color(cfg.color(LapceColor::LAPCE_BORDER))
            .background(cfg.color(LapceColor::EDITOR_BACKGROUND))
    })
}

fn handoff(
    agents: &AgentRegistry,
    workspace: Arc<crate::workspace::LapceWorkspace>,
    scope: floem::reactive::Scope,
    workspace_mode: floem::reactive::RwSignal<WorkspaceMode>,
) {
    let Some(assistant) = resolve_untracked(agents) else {
        return;
    };
    let plan = assistant.plan.with_untracked(|p| p.clone());
    if plan.is_empty() {
        return;
    }
    let title = assistant.title.with_untracked(|t| t.clone());

    let coder = Rc::new(CoderSession::new_with_parent(
        scope,
        assistant.id,
        workspace,
        title,
        plan,
    ));
    let coder_id = coder.id;
    agents.insert_coder(coder.clone());
    agents.active_coder.set(Some(coder_id));

    assistant.state.set(SessionState::Locked);
    assistant.spawned_coders.update(|v| v.push(coder_id));

    workspace_mode.set(WorkspaceMode::CoderAgent(coder_id));
    stub_runner::launch(coder);
}
