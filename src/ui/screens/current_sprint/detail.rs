use ratatui::{
    Frame,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
};

use crate::{
    data::IssueSummary,
    ui::{components::layout::ModalFrame, context::RenderContext},
};

pub(super) fn render_issue_detail_modal(
    frame: &mut Frame,
    issue: &IssueSummary,
    context: &RenderContext,
) {
    let area = frame.area();
    let width = (area.width.saturating_mul(4) / 5).max(56).min(area.width);
    let height = (area.height.saturating_mul(4) / 5).max(14).min(area.height);
    let modal = crate::ui::layout::modal_area(area, width, height);
    let inner = ModalFrame::new(
        issue.key.as_str(),
        modal,
        Style::default().fg(context.colors().accent),
        context,
    )
    .render(frame);

    let labels = if issue.labels.is_empty() {
        "none".to_string()
    } else {
        issue.labels.join(", ")
    };
    let fix_versions = if issue.fix_versions.is_empty() {
        "none".to_string()
    } else {
        issue.fix_versions.join(", ")
    };
    let story_points = issue
        .story_points
        .map(format_story_points)
        .unwrap_or_else(|| "n/a".to_string());
    let comments_count = issue.comments.len().to_string();
    let description = issue
        .description
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("No description");

    let text = Text::from(vec![
        Line::from(Span::styled(
            issue.summary.as_str(),
            Style::default().fg(context.colors().text),
        )),
        Line::from(""),
        kv_line("Status", issue.status.as_str(), context),
        kv_line("Type", issue.issue_type.as_str(), context),
        kv_line("Priority", issue.priority.as_str(), context),
        kv_line("Assignee", issue.assignee.as_str(), context),
        kv_line("Epic", issue.epic.as_deref().unwrap_or("none"), context),
        kv_line("Story Points", story_points.as_str(), context),
        kv_line("Labels", labels.as_str(), context),
        kv_line("Fix Versions", fix_versions.as_str(), context),
        kv_line("Comments", comments_count.as_str(), context),
        Line::from(""),
        Line::from(Span::styled(
            "Description",
            Style::default().fg(context.colors().accent),
        )),
        Line::from(description),
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", Style::default().fg(context.colors().accent)),
            Span::styled(" / ", Style::default().fg(context.colors().border)),
            Span::styled("Esc", Style::default().fg(context.colors().accent)),
            Span::styled(" close", Style::default().fg(context.colors().border)),
        ]),
    ]);

    frame.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), inner);
}

fn kv_line<'a>(label: &'a str, value: &'a str, context: &RenderContext) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("{label}: "),
            Style::default().fg(context.colors().accent),
        ),
        Span::styled(
            value.to_string(),
            Style::default().fg(context.colors().text),
        ),
    ])
}

fn format_story_points(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.1}")
    }
}
