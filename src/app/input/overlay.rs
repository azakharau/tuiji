use ratatui::layout::{Constraint, Flex, Layout, Rect};

pub const MODAL_WIDTH: u16 = 72;
pub const MODAL_HEIGHT: u16 = 20;

pub fn command_line_area(area: Rect) -> Rect {
    let height = 3.min(area.height);
    let max_width = area.width.saturating_sub(4);
    let width = if max_width == 0 {
        area.width
    } else {
        max_width.clamp(10, 60)
    };
    let mut rect = popup_area(area, width, height);
    rect.y = rect.y.saturating_sub(2);
    rect
}

pub fn modal_area(area: Rect, width: u16, height: u16) -> Rect {
    popup_area(area, width, height)
}

pub fn modal_dialog_area(area: Rect) -> Rect {
    let width = MODAL_WIDTH.min(area.width);
    let height = MODAL_HEIGHT.min(area.height);
    modal_area(area, width, height)
}

fn popup_area(area: Rect, width: u16, height: u16) -> Rect {
    let [area] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::horizontal([Constraint::Length(width)])
        .flex(Flex::Center)
        .areas(area);
    area
}
