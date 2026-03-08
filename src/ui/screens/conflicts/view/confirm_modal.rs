use ratatui::{
    Frame,
    layout::Alignment,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::ui::{components::layout::ModalFrame, context::RenderContext};

use super::super::state::PendingResolve;

pub(super) fn render_confirm_modal(
    frame: &mut Frame,
    pending: PendingResolve,
    context: &RenderContext,
) {
    let area = frame.area();
    let width = (area.width.saturating_mul(6) / 10).max(28).min(area.width);
    let height = 7.min(area.height);
    let modal = crate::ui::layout::modal_area(area, width, height);
    let inner = ModalFrame::new(
        "Confirm",
        modal,
        Style::default().fg(context.colors().accent),
        context,
    )
    .render(frame);

    let action = match pending {
        PendingResolve::Local => "Use Local",
        PendingResolve::Remote => "Use Jira",
    };

    let text = vec![
        Line::from(Span::styled(
            format!("{action}?"),
            Style::default().fg(context.colors().text),
        ))
        .centered(),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(context.colors().accent)),
            Span::styled(" = Yes    ", Style::default().fg(context.colors().border)),
            Span::styled("q", Style::default().fg(context.colors().accent)),
            Span::styled(" = No", Style::default().fg(context.colors().border)),
        ])
        .centered(),
    ];
    frame.render_widget(Paragraph::new(text).alignment(Alignment::Center), inner);
}
