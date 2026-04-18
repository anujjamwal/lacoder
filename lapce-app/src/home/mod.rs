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

use crate::{
    config::{LapceConfig, color::LapceColor},
    mode::WorkspaceMode,
    window_tab::WindowTabData,
};

pub fn home(window_tab_data: Rc<WindowTabData>) -> impl View {
    let config = window_tab_data.common.config;
    let workspace = window_tab_data.workspace.clone();
    let workspace_mode = window_tab_data.workspace_mode;

    let title_text = workspace
        .display()
        .unwrap_or_else(|| "Workspace".to_string());

    let header = stack((
        text(title_text).style(|s| s.font_size(24.0).font_bold()),
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

    let new_assistant = action_card(
        "New assistant session",
        "Plan, research, and hand off to coder agents. (Stub — Phase 2.)",
        config,
        || {},
    );

    stack((header, open_editor, new_assistant))
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
            text(title_str).style(|s| s.font_bold()),
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
            .cursor(CursorStyle::Pointer)
            .hover(|s| {
                s.background(cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND))
            })
    })
}
