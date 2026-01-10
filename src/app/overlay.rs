use std::collections::VecDeque;

use crate::app::{error::AppErrorState, notification::AppNotification, render::RenderState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhichKeyMode {
    Screen,
    Prefix(char),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardRequiredBindings<'a> {
    pub open: &'a str,
    pub profiles: Option<&'a str>,
    pub quit: &'a str,
}

pub enum OverlayItem<'a> {
    Error(&'a AppErrorState),
    Notification(&'a VecDeque<AppNotification>),
    CommandLine(&'a str),
    WhichKey(WhichKeyMode),
    BoardRequired(&'a BoardRequiredBindings<'a>),
}

pub struct OverlayBus;

impl OverlayBus {
    pub fn top_overlay<'a>(state: &'a RenderState<'a>) -> Option<OverlayItem<'a>> {
        if let Some(error) = state.error {
            return Some(OverlayItem::Error(error));
        }
        if !state.notifications.is_empty() {
            return Some(OverlayItem::Notification(state.notifications));
        }
        if let Some(buffer) = state.command_buffer {
            return Some(OverlayItem::CommandLine(buffer));
        }
        if state.show_hints {
            return Some(OverlayItem::WhichKey(WhichKeyMode::Screen));
        }
        if let Some(prefix) = state.pending_prefix {
            return Some(OverlayItem::WhichKey(WhichKeyMode::Prefix(prefix)));
        }
        if let Some(bindings) = state.board_required.as_ref() {
            return Some(OverlayItem::BoardRequired(bindings));
        }
        None
    }
}
