//! Launchpad — screen shown when no workspace is bound to the window.
//!
//! Assembles the top navigation, tab strip, responsive card grid, and footer
//! to match `docs/designs/launchpad.png` at the structural level. Status
//! badges, agent counts, and branch info are stubbed (Phase 0 has no real
//! sources for these); they will wire up in Phase 3 (SCM backend) and
//! Phase 6 (multi-agent UX).

pub mod footer;
pub mod tab_strip;
pub mod top_nav;
pub mod workspace_card;

use std::sync::Arc;

use floem::{
    IntoView, View,
    reactive::{RwSignal, SignalGet, create_rw_signal, use_context},
    style::FlexWrap,
    views::{Decorators, dyn_stack, empty, label, scroll, stack, text},
};

use crate::{
    config::{LapceConfig, color::LapceColor},
    db::LapceDb,
    window::WindowData,
    workspace::LapceWorkspace,
};

use self::{
    footer::footer,
    tab_strip::{WorkspaceFilter, tab_strip},
    top_nav::{LaunchpadTab, top_nav},
    workspace_card::{
        CardStatus, WorkspaceCard, launch_new_card, workspace_card,
    },
};

pub fn launchpad(window_data: WindowData) -> impl View {
    let config = window_data.config;
    let window_command = window_data.common.window_command;

    let active_tab = create_rw_signal(LaunchpadTab::Workspaces);
    let filter = create_rw_signal(WorkspaceFilter::Active);

    let db: Arc<LapceDb> = use_context().unwrap();
    let recent = db.recent_workspaces().unwrap_or_default();

    let page_header = page_header(config);
    let grid = card_grid(recent, window_command, config);

    let body = scroll(
        stack((page_header, tab_strip(config, filter), grid))
            .style(move |s| {
                s.flex_col().width_full().background(
                    config.get().color(LapceColor::EDITOR_BACKGROUND),
                )
            }),
    )
    .style(move |s| {
        s.width_full()
            .flex_grow(1.0)
            .min_height(0.0)
            .background(config.get().color(LapceColor::EDITOR_BACKGROUND))
    });

    stack((top_nav(config, active_tab), body, footer(config)))
        .style(move |s| {
            s.size_full()
                .flex_col()
                .color(config.get().color(LapceColor::EDITOR_FOREGROUND))
                .background(config.get().color(LapceColor::EDITOR_BACKGROUND))
        })
        .debug_name("Launchpad")
}

fn page_header(config: RwSignal<Arc<LapceConfig>>) -> impl View {
    stack((
        text("Workspaces").style(move |s| {
            s.font_size(24.0)
                .font_bold()
                .color(config.get().color(LapceColor::EDITOR_FOREGROUND))
        }),
        label(|| {
            "Manage your active development environments and AI agents."
                .to_string()
        })
        .style(move |s| {
            s.font_size(13.0)
                .color(config.get().color(LapceColor::EDITOR_DIM))
                .margin_top(6.0)
        }),
    ))
    .style(|s| s.flex_col().padding_horiz(24.0).padding_vert(24.0))
}

fn card_grid(
    recent: Vec<LapceWorkspace>,
    window_command: crate::listener::Listener<crate::command::WindowCommand>,
    config: RwSignal<Arc<LapceConfig>>,
) -> impl View {
    // Index 0 is the "Launch new instance" card; indices 1.. are workspaces.
    // We render all items via a single dyn_stack so flex-wrap just works.
    let cards: Vec<(usize, Option<WorkspaceCard>)> =
        std::iter::once((0usize, None::<WorkspaceCard>))
            .chain(recent.into_iter().enumerate().map(|(i, ws)| {
                (i + 1, Some(workspace_card_for(ws)))
            }))
            .collect();

    let cards = std::rc::Rc::new(cards);
    let cards_for_key = cards.clone();

    dyn_stack(
        move || cards.as_ref().clone(),
        move |(i, _)| *i,
        move |(_, card)| match card {
            Some(c) => workspace_card(c, window_command, config).into_any(),
            None => launch_new_card(window_command, config).into_any(),
        },
    )
    .style(move |s| {
        let has_any = cards_for_key.as_ref().iter().any(|(_, c)| c.is_some());
        s.width_full()
            .flex_wrap(FlexWrap::Wrap)
            .gap(16.0)
            .padding_horiz(24.0)
            .padding_bottom(24.0)
            .padding_top(16.0)
            .apply_if(!has_any, |s| s) // keep placeholder hint for future
    })
    .debug_name("Launchpad Card Grid")
}

fn workspace_card_for(ws: LapceWorkspace) -> WorkspaceCard {
    let accessed = accessed_label_for(ws.last_open);
    WorkspaceCard {
        workspace: ws,
        branch: None,
        agent_count: 0,
        pending_files: 0,
        accessed_label: accessed,
        status: CardStatus::Inactive,
    }
}

fn accessed_label_for(last_open_secs: u64) -> String {
    if last_open_secs == 0 {
        return "—".to_string();
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(last_open_secs);
    let delta = now.saturating_sub(last_open_secs);
    match delta {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{}m ago", delta / 60),
        3600..=86399 => format!("{}h ago", delta / 3600),
        _ => format!("{}d ago", delta / 86400),
    }
}

// Silence dead-code warning: `empty` is re-exported but kept available for
// future use (filter-empty state).
#[allow(dead_code)]
fn _keep_empty_import() -> impl View {
    empty()
}
