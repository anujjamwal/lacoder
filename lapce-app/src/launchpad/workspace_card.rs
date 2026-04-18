//! Workspace card + "Launch new instance" card used by the Launchpad grid.

use std::sync::Arc;

use floem::{
    View,
    action::open_file,
    file::FileDialogOptions,
    peniko::Color,
    reactive::{RwSignal, SignalGet},
    style::CursorStyle,
    views::{Decorators, container, empty, label, stack},
};

use crate::{
    command::WindowCommand,
    config::{LapceConfig, color::LapceColor},
    listener::Listener,
    workspace::{LapceWorkspace, LapceWorkspaceType},
};

/// Display metadata a workspace card needs. For Phase 0 these are
/// best-effort — fields like `status` and `agent_count` are not yet populated
/// by real data sources; defaults render the empty "inactive" state.
#[derive(Clone, Debug)]
pub struct WorkspaceCard {
    pub workspace: LapceWorkspace,
    pub branch: Option<String>,
    pub agent_count: u32,
    pub pending_files: u32,
    pub accessed_label: String,
    pub status: CardStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardStatus {
    Stable,
    NeedsReview,
    PipelineFailed,
    Inactive,
}

impl CardStatus {
    fn label(self) -> &'static str {
        match self {
            CardStatus::Stable => "STABLE",
            CardStatus::NeedsReview => "NEEDS REVIEW",
            CardStatus::PipelineFailed => "PIPELINE FAILED",
            CardStatus::Inactive => "INACTIVE",
        }
    }

    fn fg_bg(self, cfg: &LapceConfig) -> (Color, Color) {
        match self {
            CardStatus::Stable => (
                cfg.color(LapceColor::SOURCE_CONTROL_ADDED),
                cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND),
            ),
            CardStatus::NeedsReview => (
                cfg.color(LapceColor::LAPCE_WARN),
                cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND),
            ),
            CardStatus::PipelineFailed => (
                cfg.color(LapceColor::LAPCE_ERROR),
                cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND),
            ),
            CardStatus::Inactive => (
                cfg.color(LapceColor::EDITOR_DIM),
                cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND),
            ),
        }
    }
}

pub fn workspace_card(
    card: WorkspaceCard,
    window_command: Listener<WindowCommand>,
    config: RwSignal<Arc<LapceConfig>>,
) -> impl View {
    let path_title = card
        .workspace
        .path
        .as_ref()
        .and_then(|p| p.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unnamed workspace".to_string());

    let branch = card.branch.clone().unwrap_or_default();
    let has_branch = !branch.is_empty();
    let status = card.status;
    let is_inactive = matches!(status, CardStatus::Inactive);
    let agent_count = card.agent_count;
    let pending_files = card.pending_files;
    let accessed = card.accessed_label.clone();
    let click_ws = card.workspace.clone();

    let kind_icon = match &card.workspace.kind {
        LapceWorkspaceType::Local => "\u{1F4C1}",
        LapceWorkspaceType::RemoteSSH(_) => "\u{1F517}",
        #[cfg(windows)]
        LapceWorkspaceType::RemoteWSL(_) => "\u{1F410}",
    };

    let header_row = stack((
        label(move || kind_icon.to_string()).style(move |s| {
            s.font_size(16.0)
                .color(config.get().color(LapceColor::EDITOR_DIM))
                .margin_right(8.0)
        }),
        label(move || path_title.clone()).style(move |s| {
            s.font_size(13.0)
                .font_bold()
                .color(config.get().color(LapceColor::EDITOR_FOREGROUND))
                .flex_grow(1.0)
                .min_width(0.0)
                .text_ellipsis()
        }),
        status_badge(status, config),
    ))
    .style(|s| s.items_center().width_full());

    let branch_row = stack((
        label(move || "\u{2387} ".to_string()).style(move |s| {
            s.font_size(11.0)
                .color(config.get().color(LapceColor::EDITOR_DIM))
                .margin_right(4.0)
        }),
        label(move || branch.clone()).style(move |s| {
            s.font_size(11.0)
                .color(config.get().color(LapceColor::EDITOR_DIM))
                .apply_if(!has_branch, |s| s.hide())
        }),
    ))
    .style(move |s| {
        s.items_center()
            .margin_top(6.0)
            .apply_if(!has_branch, |s| s.hide())
    });

    let agents_row = stack((
        label(move || {
            if is_inactive {
                "Inactive".to_string()
            } else {
                format!("\u{25C9} {agent_count} Agent{} Active",
                    if agent_count == 1 { "" } else { "s" })
            }
        })
        .style(move |s| {
            let cfg = config.get();
            s.font_size(11.0)
                .color(if is_inactive {
                    cfg.color(LapceColor::EDITOR_DIM)
                } else {
                    cfg.color(LapceColor::LAPCE_TAB_ACTIVE_FOREGROUND)
                })
                .flex_grow(1.0)
        }),
        label(move || {
            if is_inactive {
                "No agents running".to_string()
            } else {
                String::new()
            }
        })
        .style(move |s| {
            s.font_size(11.0)
                .color(config.get().color(LapceColor::EDITOR_DIM))
                .apply_if(!is_inactive, |s| s.hide())
        }),
    ))
    .style(|s| s.items_center().margin_top(14.0).width_full());

    let stats_row = stack((
        stat_cell("PENDING", format!("{pending_files} files"), config),
        empty().style(|s| s.width(8.0)),
        stat_cell("ACCESSED", accessed, config),
    ))
    .style(|s| s.width_full().margin_top(14.0));

    container(
        stack((header_row, branch_row, agents_row, stats_row))
            .style(|s| s.flex_col().width_full()),
    )
    .on_click_stop(move |_| {
        window_command.send(WindowCommand::SetWorkspace {
            workspace: click_ws.clone(),
        });
    })
    .style(move |s| card_frame(s, config.get().as_ref()))
}

pub fn launch_new_card(
    window_command: Listener<WindowCommand>,
    config: RwSignal<Arc<LapceConfig>>,
) -> impl View {
    let action = move || {
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
            window_command.send(WindowCommand::SetWorkspace { workspace });
        });
    };

    container(
        stack((
            label(|| "+".to_string()).style(move |s| {
                s.font_size(40.0)
                    .color(config.get().color(LapceColor::EDITOR_FOREGROUND))
                    .margin_bottom(10.0)
            }),
            label(|| "Launch new instance".to_string()).style(move |s| {
                s.font_size(13.0)
                    .font_bold()
                    .color(config.get().color(LapceColor::EDITOR_FOREGROUND))
                    .margin_bottom(6.0)
            }),
            label(|| "Spin up a clean environment from a repository or template.".to_string())
                .style(move |s| {
                    s.font_size(11.0)
                        .color(config.get().color(LapceColor::EDITOR_DIM))
                })
                .style(|s| s.items_center()),
        ))
        .style(|s| s.flex_col().items_center().width_full()),
    )
    .on_click_stop(move |_| action())
    .style(move |s| {
        let cfg = config.get();
        card_frame(s, cfg.as_ref())
            .items_center()
            .justify_center()
    })
}

fn card_frame(
    s: floem::style::Style,
    cfg: &LapceConfig,
) -> floem::style::Style {
    s.padding(16.0)
        .min_width(280.0)
        .flex_basis(320.0)
        .flex_grow(1.0)
        .min_height(170.0)
        .border(1.0)
        .border_radius(10.0)
        .border_color(cfg.color(LapceColor::LAPCE_BORDER))
        .background(cfg.color(LapceColor::PANEL_BACKGROUND))
        .cursor(CursorStyle::Pointer)
        .hover(|s| s.background(cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND)))
}

fn status_badge(
    status: CardStatus,
    config: RwSignal<Arc<LapceConfig>>,
) -> impl View {
    label(move || status.label().to_string()).style(move |s| {
        let cfg = config.get();
        let (fg, bg) = status.fg_bg(cfg.as_ref());
        s.font_size(10.0)
            .font_bold()
            .padding_horiz(8.0)
            .padding_vert(3.0)
            .border_radius(4.0)
            .color(fg)
            .background(bg)
    })
}

fn stat_cell(
    caption: &'static str,
    value: String,
    config: RwSignal<Arc<LapceConfig>>,
) -> impl View {
    container(
        stack((
            label(move || caption.to_string()).style(move |s| {
                s.font_size(9.0)
                    .font_bold()
                    .color(config.get().color(LapceColor::EDITOR_DIM))
                    .margin_bottom(4.0)
            }),
            label(move || value.clone()).style(move |s| {
                s.font_size(12.0)
                    .color(config.get().color(LapceColor::EDITOR_FOREGROUND))
            }),
        ))
        .style(|s| s.flex_col()),
    )
    .style(move |s| {
        let cfg = config.get();
        s.padding_horiz(10.0)
            .padding_vert(8.0)
            .border(1.0)
            .border_radius(6.0)
            .border_color(cfg.color(LapceColor::LAPCE_BORDER))
            .flex_grow(1.0)
            .flex_basis(0.0)
    })
}
