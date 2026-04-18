//! Launchpad footer: left side shows mode + connection + version; right side
//! shows static links + a shell-mode indicator.

use std::sync::Arc;

use floem::{
    View,
    reactive::{RwSignal, SignalGet},
    views::{Decorators, empty, label, stack},
};

use crate::config::{LapceConfig, color::LapceColor};

pub fn footer(config: RwSignal<Arc<LapceConfig>>) -> impl View {
    let version = env!("CARGO_PKG_VERSION");

    let left = stack((
        label(|| "LACODER".to_string()).style(move |s| {
            s.font_size(10.0)
                .font_bold()
                .color(config.get().color(LapceColor::EDITOR_DIM))
                .margin_right(16.0)
        }),
        dot(config),
        label(|| "Local session".to_string()).style(move |s| {
            s.font_size(10.0)
                .color(config.get().color(LapceColor::SOURCE_CONTROL_ADDED))
                .margin_right(16.0)
        }),
        label(move || format!("v{version}")).style(move |s| {
            s.font_size(10.0)
                .color(config.get().color(LapceColor::EDITOR_DIM))
        }),
    ))
    .style(|s| s.items_center());

    let spacer = empty().style(|s| s.flex_grow(1.0).min_width(0.0));

    let right = stack((
        footer_link("PRIVACY", config),
        footer_link("TERMS", config),
        footer_link("DOCS", config),
        label(|| "\u{25EC} AI-enhanced shell".to_string()).style(move |s| {
            s.font_size(10.0)
                .color(config.get().color(LapceColor::LAPCE_TAB_ACTIVE_FOREGROUND))
                .margin_left(16.0)
        }),
    ))
    .style(|s| s.items_center().gap(14.0));

    stack((left, spacer, right))
        .style(move |s| {
            let cfg = config.get();
            s.width_full()
                .height(28.0)
                .padding_horiz(20.0)
                .items_center()
                .border_top(1.0)
                .border_color(cfg.color(LapceColor::LAPCE_BORDER))
                .background(cfg.color(LapceColor::STATUS_BACKGROUND))
        })
        .debug_name("Launchpad Footer")
}

fn dot(config: RwSignal<Arc<LapceConfig>>) -> impl View {
    label(|| "\u{25CF}".to_string()).style(move |s| {
        s.font_size(10.0)
            .color(config.get().color(LapceColor::SOURCE_CONTROL_ADDED))
            .margin_right(6.0)
    })
}

fn footer_link(
    text: &'static str,
    config: RwSignal<Arc<LapceConfig>>,
) -> impl View {
    label(move || text.to_string()).style(move |s| {
        s.font_size(10.0)
            .color(config.get().color(LapceColor::EDITOR_DIM))
    })
}
