use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::ui::context::RenderContext;

pub(super) fn render_header(frame: &mut Frame, area: Rect, context: &RenderContext) {
    let header = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).split(area);

    let title = Paragraph::new(Line::from("Conflicts").centered())
        .style(Style::default().fg(context.colors().accent))
        .alignment(Alignment::Center);
    frame.render_widget(title, header[0]);

    let hint_line = Line::from(vec![
        Span::styled("L", Style::default().fg(context.colors().accent)),
        Span::styled(
            " = Use Local  ",
            Style::default().fg(context.colors().border),
        ),
        Span::styled("J", Style::default().fg(context.colors().accent)),
        Span::styled(" = Use Jira", Style::default().fg(context.colors().border)),
    ])
    .centered();
    frame.render_widget(
        Paragraph::new(hint_line).alignment(Alignment::Center),
        header[1],
    );
}
