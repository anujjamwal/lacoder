//! Top navigation bar: logo + page tabs + command-palette placeholder +
//! right-side icons (notifications, help, theme toggle).

use std::sync::Arc;

use floem::{
    View,
    reactive::{ReadSignal, RwSignal, SignalGet, SignalUpdate},
    style::CursorStyle,
    views::{Decorators, container, empty, label, stack, text},
};

use crate::config::{LapceConfig, color::LapceColor};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LaunchpadTab {
    Workspaces,
    Search,
    Agents,
    CLs,
}

impl LaunchpadTab {
    fn label(self) -> &'static str {
        match self {
            LaunchpadTab::Workspaces => "Workspaces",
            LaunchpadTab::Search => "Search",
            LaunchpadTab::Agents => "Agents",
            LaunchpadTab::CLs => "CLs",
        }
    }
}

pub fn top_nav(
    config: RwSignal<Arc<LapceConfig>>,
    active_tab: RwSignal<LaunchpadTab>,
) -> impl View {
    let logo = label(|| "Lacoder".to_string()).style(move |s| {
        s.font_size(14.0)
            .font_bold()
            .color(config.get().color(LapceColor::EDITOR_FOREGROUND))
            .padding_right(24.0)
    });

    let cfg_read = config.read_only();
    let tabs_row = stack((
        tab_btn(LaunchpadTab::Workspaces, active_tab, cfg_read),
        tab_btn(LaunchpadTab::Search, active_tab, cfg_read),
        tab_btn(LaunchpadTab::Agents, active_tab, cfg_read),
        tab_btn(LaunchpadTab::CLs, active_tab, cfg_read),
    ))
    .style(|s| s.items_center().gap(4.0));

    let spacer = empty().style(|s| s.flex_grow(1.0).min_width(0.0));

    let palette = container(
        stack((
            label(|| "\u{1F50D} ".to_string())
                .style(|s| s.font_size(11.0).margin_right(6.0)),
            label(|| "Ctrl + K".to_string()).style(|s| s.font_size(12.0)),
        ))
        .style(|s| s.items_center()),
    )
    .style(move |s| {
        let cfg = config.get();
        s.padding_horiz(10.0)
            .padding_vert(5.0)
            .border(1.0)
            .border_radius(6.0)
            .border_color(cfg.color(LapceColor::LAPCE_BORDER))
            .color(cfg.color(LapceColor::EDITOR_DIM))
            .margin_right(16.0)
            .cursor(CursorStyle::Pointer)
    });

    let icons = stack((
        icon_btn("\u{1F514}", config), // bell
        icon_btn("?", config),
        icon_btn("\u{1F4A1}", config), // bulb
    ))
    .style(|s| s.items_center().gap(12.0));

    stack((logo, tabs_row, spacer, palette, icons))
        .style(move |s| {
            let cfg = config.get();
            s.width_full()
                .height(48.0)
                .padding_horiz(20.0)
                .items_center()
                .border_bottom(1.0)
                .border_color(cfg.color(LapceColor::LAPCE_BORDER))
                .background(cfg.color(LapceColor::PANEL_BACKGROUND))
        })
        .debug_name("Launchpad Top Nav")
}

fn tab_btn(
    which: LaunchpadTab,
    active_tab: RwSignal<LaunchpadTab>,
    config: ReadSignal<Arc<LapceConfig>>,
) -> impl View {
    container(text(which.label()))
        .on_click_stop(move |_| active_tab.set(which))
        .style(move |s| {
            let cfg = config.get();
            let is_active = active_tab.get() == which;
            let fg = if is_active {
                cfg.color(LapceColor::LAPCE_TAB_ACTIVE_FOREGROUND)
            } else {
                cfg.color(LapceColor::EDITOR_DIM)
            };
            s.padding_horiz(10.0)
                .padding_vert(14.0)
                .color(fg)
                .font_size(13.0)
                .cursor(CursorStyle::Pointer)
                .apply_if(is_active, |s| {
                    s.border_bottom(2.0).border_color(
                        cfg.color(LapceColor::LAPCE_TAB_ACTIVE_UNDERLINE),
                    )
                })
        })
}

fn icon_btn(glyph: &'static str, config: RwSignal<Arc<LapceConfig>>) -> impl View {
    label(move || glyph.to_string()).style(move |s| {
        s.font_size(14.0)
            .color(config.get().color(LapceColor::EDITOR_DIM))
            .cursor(CursorStyle::Pointer)
    })
}
