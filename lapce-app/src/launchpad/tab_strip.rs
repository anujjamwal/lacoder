//! Segmented tab strip (Active / All / Shared / Archived) + filter-input
//! placeholder. Filter wiring is left as a follow-up.

use std::sync::Arc;

use floem::{
    View,
    reactive::{ReadSignal, RwSignal, SignalGet, SignalUpdate},
    style::CursorStyle,
    views::{Decorators, container, empty, label, stack, text},
};

use crate::config::{LapceConfig, color::LapceColor};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceFilter {
    Active,
    All,
    Shared,
    Archived,
}

impl WorkspaceFilter {
    fn label(self) -> &'static str {
        match self {
            WorkspaceFilter::Active => "Active",
            WorkspaceFilter::All => "All",
            WorkspaceFilter::Shared => "Shared",
            WorkspaceFilter::Archived => "Archived",
        }
    }
}

pub fn tab_strip(
    config: RwSignal<Arc<LapceConfig>>,
    filter: RwSignal<WorkspaceFilter>,
) -> impl View {
    let cfg_read = config.read_only();

    let tabs = stack((
        filter_tab(WorkspaceFilter::Active, filter, cfg_read),
        filter_tab(WorkspaceFilter::All, filter, cfg_read),
        filter_tab(WorkspaceFilter::Shared, filter, cfg_read),
        filter_tab(WorkspaceFilter::Archived, filter, cfg_read),
    ))
    .style(move |s| {
        s.items_center()
            .gap(0.0)
            .border_bottom(1.0)
            .border_color(config.get().color(LapceColor::LAPCE_BORDER))
            .flex_grow(1.0)
    });

    let filter_placeholder = container(
        stack((
            label(|| "\u{1F50E}".to_string())
                .style(|s| s.font_size(11.0).margin_right(6.0)),
            label(|| "Filter workspaces...".to_string())
                .style(|s| s.font_size(12.0)),
        ))
        .style(|s| s.items_center()),
    )
    .style(move |s| {
        let cfg = config.get();
        s.padding_horiz(12.0)
            .padding_vert(6.0)
            .border(1.0)
            .border_radius(6.0)
            .border_color(cfg.color(LapceColor::LAPCE_BORDER))
            .color(cfg.color(LapceColor::EDITOR_DIM))
            .width(280.0)
            .cursor(CursorStyle::Text)
    });

    stack((tabs, empty().style(|s| s.width(16.0)), filter_placeholder))
        .style(move |s| {
            s.width_full()
                .items_end()
                .padding_horiz(24.0)
                .padding_top(8.0)
                .border_bottom(1.0)
                .border_color(config.get().color(LapceColor::LAPCE_BORDER))
        })
}

fn filter_tab(
    which: WorkspaceFilter,
    active: RwSignal<WorkspaceFilter>,
    config: ReadSignal<Arc<LapceConfig>>,
) -> impl View {
    container(text(which.label()))
        .on_click_stop(move |_| active.set(which))
        .style(move |s| {
            let cfg = config.get();
            let is_active = active.get() == which;
            s.padding_horiz(14.0)
                .padding_vert(10.0)
                .font_size(13.0)
                .cursor(CursorStyle::Pointer)
                .color(if is_active {
                    cfg.color(LapceColor::LAPCE_TAB_ACTIVE_FOREGROUND)
                } else {
                    cfg.color(LapceColor::EDITOR_DIM)
                })
                .margin_bottom(-1.0)
                .apply_if(is_active, |s| {
                    s.border_bottom(2.0).border_color(
                        cfg.color(LapceColor::LAPCE_TAB_ACTIVE_UNDERLINE),
                    )
                })
        })
}
