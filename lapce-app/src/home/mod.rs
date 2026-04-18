//! Workspace Home — per-workspace dashboard shown after opening a workspace.
//!
//! Phase 0 scope: minimal placeholder with the workspace name and two actions
//! ("Open editor" switches `WorkspaceMode::Editor`; "New assistant session" is
//! a stub until Phase 2). The real dashboard (active sessions, recent sessions,
//! background agents, recent diffs) lands in later phases.

use std::{rc::Rc, sync::Arc};

use floem::{
    View,
    reactive::{ReadSignal, SignalGet, SignalUpdate},
    style::CursorStyle,
    views::{Decorators, container, label, stack, text},
};

use lapce_agent::{StubProvider, ToolRegistry};

use crate::{
    agent::{
        engine,
        session::{AssistantSession, CoderSession},
    },
    config::{LapceConfig, color::LapceColor},
    mode::WorkspaceMode,
    window_tab::WindowTabData,
};

pub fn home(window_tab_data: Rc<WindowTabData>) -> impl View {
    let config = window_tab_data.common.config;
    let workspace = window_tab_data.workspace.clone();
    let workspace_mode = window_tab_data.workspace_mode;
    let agents = window_tab_data.agents.clone();
    let scope = window_tab_data.scope;

    let title_text = workspace
        .display()
        .unwrap_or_else(|| "Workspace".to_string());

    let header = stack((
        text(title_text).style(move |s| {
            s.font_size(24.0)
                .font_bold()
                .color(config.get().color(LapceColor::EDITOR_FOREGROUND))
        }),
        label(|| "Workspace Home".to_string())
            .style(move |s| s.color(config.get().color(LapceColor::EDITOR_DIM))),
    ))
    .style(|s| s.flex_col().padding(24.0).gap(4.0));

    let open_editor = action_card(
        "Open editor",
        "Inspect files, run terminals.",
        config,
        move || workspace_mode.set(WorkspaceMode::Editor),
    );

    let assistant_workspace = workspace.clone();
    let assistant_agents = agents.clone();
    let new_assistant = action_card(
        "New assistant session",
        "Plan, research, iterate — then hand off to a coder agent.",
        config,
        move || {
            let session = Rc::new(AssistantSession::new(
                scope,
                assistant_workspace.clone(),
                "Refactor parse_config".to_string(),
            ));
            let id = session.id;
            assistant_agents.insert_assistant(session);
            assistant_agents.active_assistant.set(Some(id));
            workspace_mode.set(WorkspaceMode::Assistant(id));
        },
    );

    let stub_workspace = workspace.clone();
    let stub_agents = agents.clone();
    let launch_stub = action_card(
        "Launch stub coder",
        "Spin up an in-process coder via lapce-agent (StubProvider + builtin tool registry).",
        config,
        move || {
            let session = Rc::new(CoderSession::new(
                scope,
                stub_workspace.clone(),
                "Stub: refactor parse_config".to_string(),
                "Plan:\n  1. Locate parse_config\n  2. Add AgentConfig struct\n  3. Run cargo check"
                    .to_string(),
            ));
            let id = session.id;
            stub_agents.insert_coder(session.clone());
            stub_agents.active_coder.set(Some(id));
            workspace_mode.set(WorkspaceMode::CoderAgent(id));
            engine::launch(
                Arc::new(StubProvider),
                Arc::new(ToolRegistry::with_builtins()),
                session,
            );
        },
    );

    stack((header, open_editor, new_assistant, launch_stub))
        .style(move |s| {
            s.size_full().flex_col().background(
                config.get().color(LapceColor::EDITOR_BACKGROUND),
            )
        })
        .debug_name("Workspace Home")
}

fn action_card(
    title_str: &'static str,
    subtitle_str: &'static str,
    config: ReadSignal<Arc<LapceConfig>>,
    on_click: impl Fn() + 'static,
) -> impl View {
    container(
        stack((
            text(title_str).style(move |s| {
                s.font_bold()
                    .color(config.get().color(LapceColor::EDITOR_FOREGROUND))
            }),
            label(move || subtitle_str.to_string()).style(move |s| {
                s.color(config.get().color(LapceColor::EDITOR_DIM))
                    .font_size(12.0)
            }),
        ))
        .style(|s| s.flex_col().gap(4.0)),
    )
    .on_click_stop(move |_| on_click())
    .style(move |s| {
        let cfg = config.get();
        s.padding(16.0)
            .margin_horiz(24.0)
            .margin_bottom(12.0)
            .border(1.0)
            .border_radius(8.0)
            .border_color(cfg.color(LapceColor::LAPCE_BORDER))
            .background(cfg.color(LapceColor::PANEL_BACKGROUND))
            .cursor(CursorStyle::Pointer)
            .hover(|s| {
                s.background(cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND))
            })
    })
}
