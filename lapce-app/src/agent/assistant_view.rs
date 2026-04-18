//! Assistant view placeholder. Real implementation in Phase 2.

use std::rc::Rc;

use floem::{
    View,
    reactive::{SignalGet, SignalUpdate},
    style::CursorStyle,
    views::{Decorators, container, label, stack, text},
};

use crate::{
    config::color::LapceColor, mode::WorkspaceMode, window_tab::WindowTabData,
};

pub fn assistant(window_tab_data: Rc<WindowTabData>) -> impl View {
    let config = window_tab_data.common.config;
    let workspace_mode = window_tab_data.workspace_mode;

    let back = container(text("← Back to Home"))
        .on_click_stop(move |_| workspace_mode.set(WorkspaceMode::Home))
        .style(move |s| {
            s.padding(8.0)
                .margin(12.0)
                .border(1.0)
                .border_radius(6.0)
                .border_color(config.get().color(LapceColor::LAPCE_BORDER))
                .cursor(CursorStyle::Pointer)
        });

    let body = stack((
        text("Workspace Assistant").style(|s| s.font_size(24.0).font_bold()),
        label(|| {
            "Chat-driven planning and research. Real chat & plan doc arrive in Phase 2."
                .to_string()
        })
        .style(move |s| s.color(config.get().color(LapceColor::EDITOR_DIM))),
    ))
    .style(|s| s.flex_col().padding(24.0).gap(8.0));

    stack((back, body))
        .style(move |s| {
            s.size_full().flex_col().background(
                config.get().color(LapceColor::EDITOR_BACKGROUND),
            )
        })
        .debug_name("Assistant (placeholder)")
}
