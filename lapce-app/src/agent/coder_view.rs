//! Coder Agent view — left rail (plan / trace / modified files), center
//! (chat + live status), top bar (title + Stop).
//!
//! Phase 1.0: chat is read-only, terminal is a placeholder banner. The real
//! chat composer, interactive terminal, and hunk-level diff arrive in later
//! phases.

use std::rc::Rc;

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
            AgentStatus, ChatRole, ChatTurn, CoderSession, FileChange, StopReason,
            TraceEntry,
        },
    },
    config::{LapceConfig, color::LapceColor},
    mode::WorkspaceMode,
    window_tab::WindowTabData,
};

type ConfigSig = ReadSignal<std::sync::Arc<LapceConfig>>;

fn resolve(
    agents: &AgentRegistry,
) -> Option<std::rc::Rc<CoderSession>> {
    let id = agents.active_coder.get()?;
    agents.coders.with(|m| m.get(&id).cloned())
}

fn resolve_untracked(
    agents: &AgentRegistry,
) -> Option<std::rc::Rc<CoderSession>> {
    let id = agents.active_coder.get_untracked()?;
    agents.coders.with_untracked(|m| m.get(&id).cloned())
}

pub fn coder(window_tab_data: Rc<WindowTabData>) -> impl View {
    let config = window_tab_data.common.config;
    let workspace_mode = window_tab_data.workspace_mode;
    let agents = window_tab_data.agents.clone();

    let agents_for_title = agents.clone();
    let title = label(move || {
        resolve(&agents_for_title)
            .map(|s| s.title.get())
            .unwrap_or_else(|| "No active coder".to_string())
    })
    .style(|s| s.font_size(16.0).font_bold());

    let agents_for_status = agents.clone();
    let status_badge = label(move || {
        resolve(&agents_for_status)
            .map(|s| s.status.get().label().to_string())
            .unwrap_or_else(|| "—".to_string())
    })
    .style(move |s| {
        let cfg = config.get();
        s.margin_left(12.0)
            .padding_horiz(8.0)
            .padding_vert(2.0)
            .border_radius(10.0)
            .font_size(11.0)
            .background(cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND))
            .color(cfg.color(LapceColor::EDITOR_DIM))
    });

    let agents_for_stop = agents.clone();
    let stop_btn = container(text("Stop Agent"))
        .on_click_stop(move |_| {
            if let Some(s) = resolve_untracked(&agents_for_stop) {
                s.status.set(AgentStatus::Stopped {
                    reason: StopReason::UserRequested,
                });
            }
        })
        .style(move |s| {
            let cfg = config.get();
            s.padding_horiz(12.0)
                .padding_vert(6.0)
                .margin_right(12.0)
                .border(1.0)
                .border_radius(6.0)
                .border_color(cfg.color(LapceColor::LAPCE_BORDER))
                .cursor(CursorStyle::Pointer)
                .hover(|s| {
                    s.background(cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND))
                })
        });

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
                .cursor(CursorStyle::Pointer)
                .hover(|s| {
                    s.background(cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND))
                })
        });

    let spacer = empty().style(|s| s.flex_grow(1.0).min_width(0.0));

    let header = stack((back_btn, title, status_badge, spacer, stop_btn))
        .style(move |s| {
            let cfg = config.get();
            s.width_full()
                .height(44.0)
                .items_center()
                .border_bottom(1.0)
                .border_color(cfg.color(LapceColor::LAPCE_BORDER))
        });

    let left_rail = left_rail(agents.clone(), config);
    let center = center(agents.clone(), config);

    let body = stack((left_rail, center)).style(|s| s.size_full().flex_grow(1.0));

    stack((header, body))
        .style(move |s| {
            s.size_full()
                .flex_col()
                .background(config.get().color(LapceColor::EDITOR_BACKGROUND))
        })
        .debug_name("Coder Agent")
}

fn section_header(
    title_str: &'static str,
    config: ConfigSig,
) -> impl View {
    label(move || title_str.to_string()).style(move |s| {
        s.color(config.get().color(LapceColor::EDITOR_DIM))
            .font_size(11.0)
            .padding_horiz(12.0)
            .padding_vert(8.0)
    })
}

fn left_rail(agents: AgentRegistry, config: ConfigSig) -> impl View {
    let agents_for_plan = agents.clone();
    let plan_text = label(move || {
        resolve(&agents_for_plan)
            .map(|s| s.plan.get())
            .unwrap_or_else(|| "—".to_string())
    })
    .style(move |s| {
        s.padding_horiz(12.0)
            .padding_bottom(8.0)
            .color(config.get().color(LapceColor::EDITOR_FOREGROUND))
            .font_size(12.0)
    });

    let agents_for_trace = agents.clone();
    let trace_list = dyn_stack(
        move || match resolve(&agents_for_trace) {
            Some(s) => s
                .trace
                .get()
                .into_iter()
                .enumerate()
                .collect::<Vec<_>>(),
            None => Vec::new(),
        },
        |(i, _)| *i,
        move |(_, entry)| trace_row(entry, config),
    )
    .style(|s| s.flex_col());

    let agents_for_files = agents.clone();
    let files_list = dyn_stack(
        move || match resolve(&agents_for_files) {
            Some(s) => s
                .modified_files
                .get()
                .into_iter()
                .enumerate()
                .collect::<Vec<_>>(),
            None => Vec::new(),
        },
        |(i, _)| *i,
        move |(_, change)| file_row(change, config),
    )
    .style(|s| s.flex_col());

    scroll(
        stack((
            section_header("PLAN", config),
            plan_text,
            section_header("TRACE", config),
            trace_list,
            section_header("MODIFIED FILES", config),
            files_list,
        ))
        .style(|s| s.flex_col().width_full()),
    )
    .style(move |s| {
        let cfg = config.get();
        s.width(280.0)
            .height_full()
            .border_right(1.0)
            .border_color(cfg.color(LapceColor::LAPCE_BORDER))
    })
}

fn trace_row(entry: TraceEntry, config: ConfigSig) -> impl View {
    let badge_str = entry.kind.badge().to_string();
    let summary_str = entry.summary.clone();
    let detail_str = entry.detail.clone().unwrap_or_default();
    let has_detail = entry.detail.is_some();

    container(
        stack((
            stack((
                label(move || badge_str.clone()).style(move |s| {
                    let cfg = config.get();
                    s.font_size(10.0)
                        .font_bold()
                        .padding_horiz(6.0)
                        .padding_vert(1.0)
                        .border_radius(3.0)
                        .margin_right(8.0)
                        .background(cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND))
                        .color(cfg.color(LapceColor::EDITOR_DIM))
                }),
                label(move || summary_str.clone())
                    .style(|s| s.font_size(12.0).flex_grow(1.0).min_width(0.0)),
            ))
            .style(|s| s.items_center()),
            label(move || detail_str.clone()).style(move |s| {
                s.apply_if(!has_detail, |s| s.hide())
                    .color(config.get().color(LapceColor::EDITOR_DIM))
                    .font_size(11.0)
                    .margin_top(2.0)
                    .padding_left(40.0)
            }),
        ))
        .style(|s| s.flex_col()),
    )
    .style(|s| s.padding_horiz(12.0).padding_vert(6.0))
}

fn file_row(change: FileChange, config: ConfigSig) -> impl View {
    let badge_str = change.kind.badge().to_string();
    let path_str = change.path.display().to_string();
    let summary_str = change.summary.clone();

    container(
        stack((
            label(move || badge_str.clone()).style(move |s| {
                s.font_size(10.0)
                    .font_bold()
                    .padding_horiz(6.0)
                    .padding_vert(1.0)
                    .border_radius(3.0)
                    .margin_right(8.0)
                    .background(
                        config.get().color(LapceColor::PANEL_HOVERED_BACKGROUND),
                    )
            }),
            label(move || path_str.clone())
                .style(|s| s.font_size(12.0).flex_grow(1.0).min_width(0.0)),
            label(move || summary_str.clone()).style(move |s| {
                s.font_size(11.0)
                    .color(config.get().color(LapceColor::EDITOR_DIM))
            }),
        ))
        .style(|s| s.items_center()),
    )
    .style(|s| s.padding_horiz(12.0).padding_vert(6.0))
}

fn center(agents: AgentRegistry, config: ConfigSig) -> impl View {
    let agents_for_chat = agents.clone();
    let chat_list = dyn_stack(
        move || match resolve(&agents_for_chat) {
            Some(s) => {
                s.chat.get().into_iter().enumerate().collect::<Vec<_>>()
            }
            None => Vec::new(),
        },
        |(i, _)| *i,
        move |(_, turn)| chat_row(turn, config),
    )
    .style(|s| s.flex_col().padding(16.0).gap(12.0));

    let chat_pane = scroll(chat_list).style(move |s| {
        s.width_full()
            .flex_grow(1.0)
            .min_height(0.0)
            .background(config.get().color(LapceColor::EDITOR_BACKGROUND))
    });

    let terminal_placeholder = container(
        label(|| {
            "Terminal (live): placeholder — Phase 1 will wire a real PTY.".to_string()
        })
        .style(move |s| {
            s.color(config.get().color(LapceColor::EDITOR_DIM)).font_size(12.0)
        }),
    )
    .style(move |s| {
        let cfg = config.get();
        s.width_full()
            .height(160.0)
            .padding(12.0)
            .border_top(1.0)
            .border_color(cfg.color(LapceColor::LAPCE_BORDER))
            .background(cfg.color(LapceColor::PANEL_BACKGROUND))
    });

    stack((chat_pane, terminal_placeholder))
        .style(|s| s.flex_col().flex_grow(1.0).height_full())
}

fn chat_row(turn: ChatTurn, config: ConfigSig) -> impl View {
    let content_str = turn.content.clone();
    let is_agent = turn.role == ChatRole::Agent;
    let role_label = if is_agent { "Agent" } else { "You" };

    stack((
        label(move || role_label.to_string()).style(move |s| {
            s.font_size(10.0)
                .font_bold()
                .color(config.get().color(LapceColor::EDITOR_DIM))
                .margin_bottom(2.0)
        }),
        label(move || content_str.clone()).style(|s| s.font_size(13.0)),
    ))
    .style(move |s| {
        let cfg = config.get();
        s.flex_col()
            .padding(10.0)
            .border_radius(6.0)
            .background(if is_agent {
                cfg.color(LapceColor::PANEL_BACKGROUND)
            } else {
                cfg.color(LapceColor::PANEL_HOVERED_BACKGROUND)
            })
    })
}
