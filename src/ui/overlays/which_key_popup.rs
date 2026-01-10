use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::{app::key_handlers::ActionHint, ui::context::RenderContext};

pub struct WhichKeyPopup {
    pub prefix: String,
    pub hints: Vec<ActionHint>,
    context: RenderContext,
}

impl WhichKeyPopup {
    pub fn new(prefix: String, hints: Vec<ActionHint>, context: &RenderContext) -> Self {
        Self {
            prefix,
            hints,
            context: context.clone(),
        }
    }

    fn popup_rect(&self, area: Rect) -> Rect {
        let max_line = self
            .hints
            .iter()
            .map(|hint| hint.binding.chars().count() + 2 + hint.description.chars().count())
            .max()
            .unwrap_or(0) as u16;
        let width = max_line.saturating_add(4).min(area.width);
        let height = (self.hints.len() as u16).saturating_add(2).min(area.height);
        let x = area
            .x
            .saturating_add(area.width.saturating_sub(width).saturating_sub(1));
        let y = area
            .y
            .saturating_add(area.height.saturating_sub(height).saturating_sub(1));
        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

impl Widget for &WhichKeyPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.hints.is_empty() {
            return;
        }

        let rect = self.popup_rect(area);
        Clear.render(rect, buf);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.context.colors().border))
            .style(
                Style::default()
                    .fg(self.context.colors().text)
                    .bg(self.context.colors().background),
            )
            .title(self.prefix.clone());
        let inner = block.inner(rect);
        block.render(rect, buf);

        let lines = self
            .hints
            .iter()
            .map(|hint| {
                Line::from(vec![
                    Span::styled(
                        hint.binding.clone(),
                        Style::default().fg(self.context.colors().accent),
                    ),
                    Span::raw("  "),
                    Span::styled(
                        hint.description.clone(),
                        Style::default().fg(self.context.colors().text),
                    ),
                ])
            })
            .collect::<Vec<Line>>();
        let text = Text::from(lines);
        let paragraph = Paragraph::new(text);
        paragraph.render(inner, buf);
    }
}
