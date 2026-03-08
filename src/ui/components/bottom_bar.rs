use std::sync::Arc;

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Modifier, Style},
    widgets::{Block, Paragraph, Widget},
};

use crate::{
    ui::context::RenderContext,
    ui::interaction::{ActionHint, Mode},
};

pub struct BottomBar<'a> {
    pub mode: Mode,
    pub actions: Arc<Vec<ActionHint>>,
    pub context: &'a RenderContext,
}

impl<'a> BottomBar<'a> {
    pub fn new(mode: Mode, actions: Arc<Vec<ActionHint>>, context: &'a RenderContext) -> Self {
        BottomBar {
            mode,
            actions,
            context,
        }
    }
}

impl Widget for BottomBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let actions_str = self
            .actions
            .iter()
            .map(|action| action.render())
            .collect::<Vec<String>>()
            .join("  ");
        let actions_paragraph = Paragraph::new(actions_str)
            .style(Style::default().fg(self.context.colors().text))
            .alignment(Alignment::Left);

        let chunks = Layout::horizontal([Constraint::Length(12), Constraint::Min(0)]).split(area);

        let mode_style = Style::default()
            .fg(self.context.colors().mode_text)
            .bg(mode_color(self.mode, self.context))
            .add_modifier(Modifier::BOLD);
        let mode_area = chunks[0];
        Block::default().style(mode_style).render(mode_area, buf);
        let padded_mode_area = Rect {
            x: mode_area.x.saturating_add(1),
            y: mode_area.y,
            width: mode_area.width.saturating_sub(2),
            height: mode_area.height,
        };
        Paragraph::new(self.mode.label())
            .style(mode_style)
            .alignment(Alignment::Center)
            .render(padded_mode_area, buf);
        actions_paragraph.render(chunks[1], buf);
    }
}

fn mode_color(mode: Mode, context: &RenderContext) -> ratatui::style::Color {
    let colors = context.colors();
    match mode {
        Mode::Normal => colors.mode_normal_bg,
        Mode::Insert => colors.mode_insert_bg,
        Mode::Visual => colors.mode_visual_bg,
        Mode::Command => colors.mode_command_bg,
    }
}
