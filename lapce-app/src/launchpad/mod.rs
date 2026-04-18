//! Launchpad — the screen shown when no workspace is bound to the window.
//!
//! Phase 0 scope: list recent workspaces as clickable rows and an "Open folder…"
//! action that delegates to the existing folder-picker flow. The richer card UI
//! (agent counts, pipeline badges, top nav) lands in later phases.

use std::{rc::Rc, sync::Arc};

use floem::{
    View,
    action::open_file,
    file::FileDialogOptions,
    reactive::{RwSignal, SignalGet, use_context},
    style::CursorStyle,
    views::{
        Decorators, container, dyn_stack, empty, label, scroll, stack, text,
    },
};

use crate::{
    command::WindowCommand,
    config::{LapceConfig, color::LapceColor},
    db::LapceDb,
    listener::Listener,
    window::WindowData,
    workspace::{LapceWorkspace, LapceWorkspaceType},
};

pub fn launchpad(window_data: WindowData) -> impl View {
    let config = window_data.config;
    let window_command = window_data.common.window_command;

    let db: Arc<LapceDb> = use_context().unwrap();
    let recent = Rc::new(db.recent_workspaces().unwrap_or_default());

    let header = stack((
        text("Lacoder").style(|s| s.font_size(28.0).font_bold()),
        label(|| "Agentic coding workspace".to_string())
            .style(move |s| s.color(config.get().color(LapceColor::EDITOR_DIM))),
    ))
    .style(|s| s.flex_col().padding(24.0).gap(4.0));

    let launch_new_cmd = window_command;
    let launch_new = container(
        stack((
            text("+ Launch new instance").style(|s| s.font_bold()),
            label(|| "Open a folder as a new workspace".to_string())
                .style(move |s| {
                    s.color(config.get().color(LapceColor::EDITOR_DIM))
                        .font_size(12.0)
                }),
        ))
        .style(|s| s.flex_col().gap(4.0)),
    )
    .on_click_stop(move |_| {
        let options = FileDialogOptions::new()
            .title("Choose a folder")
            .select_directories();
        open_file(options, move |file| {
            let Some(mut file) = file else {
                return;
            };
            let Some(path) = file.path.pop() else {
                return;
            };
            let workspace = LapceWorkspace {
                kind: LapceWorkspaceType::Local,
                path: Some(path),
                last_open: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
            };
            launch_new_cmd.send(WindowCommand::SetWorkspace { workspace });
        });
    })
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
    });

    let recent_header = label(|| "Recent workspaces".to_string()).style(move |s| {
        s.margin_horiz(24.0)
            .margin_bottom(8.0)
            .color(config.get().color(LapceColor::EDITOR_DIM))
            .font_size(12.0)
    });

    let recent_for_list = recent.clone();
    let recent_list = dyn_stack(
        move || {
            recent_for_list
                .iter()
                .cloned()
                .enumerate()
                .collect::<Vec<_>>()
        },
        |(i, _)| *i,
        move |(_, workspace)| workspace_row(workspace, window_command, config),
    )
    .style(|s| s.flex_col());

    let recent_empty = empty().style(move |s| {
        s.apply_if(!recent.is_empty(), |s| s.hide()).padding(24.0)
    });

    scroll(
        stack((header, launch_new, recent_header, recent_list, recent_empty))
            .style(|s| s.flex_col().width_full()),
    )
    .style(move |s| {
        s.size_full()
            .background(config.get().color(LapceColor::EDITOR_BACKGROUND))
    })
    .debug_name("Launchpad")
}

fn workspace_row(
    workspace: LapceWorkspace,
    window_command: Listener<WindowCommand>,
    config: RwSignal<Arc<LapceConfig>>,
) -> impl View {
    let title = workspace
        .display()
        .unwrap_or_else(|| "Untitled workspace".to_string());
    let subtitle = workspace
        .path
        .as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string();
    let kind_label = match &workspace.kind {
        LapceWorkspaceType::Local => "Local".to_string(),
        LapceWorkspaceType::RemoteSSH(host) => format!("SSH • {host}"),
        #[cfg(windows)]
        LapceWorkspaceType::RemoteWSL(host) => format!("WSL • {host}"),
    };

    let click_ws = workspace.clone();
    container(
        stack((
            text(title).style(|s| s.font_bold()),
            label(move || subtitle.clone()).style(move |s| {
                s.color(config.get().color(LapceColor::EDITOR_DIM))
                    .font_size(12.0)
            }),
            label(move || kind_label.clone()).style(move |s| {
                s.color(config.get().color(LapceColor::EDITOR_DIM))
                    .font_size(11.0)
            }),
        ))
        .style(|s| s.flex_col().gap(2.0)),
    )
    .on_click_stop(move |_| {
        window_command.send(WindowCommand::SetWorkspace {
            workspace: click_ws.clone(),
        });
    })
    .style(move |s| {
        let cfg = config.get();
        s.padding(14.0)
            .margin_horiz(24.0)
            .margin_bottom(8.0)
            .border(1.0)
            .border_radius(6.0)
            .border_color(cfg.color(LapceColor::LAPCE_BORDER))
            .cursor(CursorStyle::Pointer)
            .hover(|s| {
                s.background(cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND))
            })
    })
}
