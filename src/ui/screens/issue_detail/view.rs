use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    app::{key_handlers::ActionHint, state::Mode},
    ui::context::RenderContext,
};

use super::state::IssueDetailState;

pub struct IssueDetailView;

impl IssueDetailView {
    pub fn draw(
        frame: &mut Frame,
        state: &mut IssueDetailState,
        _mode: Mode,
        _actions: &[ActionHint],
        _context: &RenderContext,
    ) {
        let area = frame.area();

        // Clone issue data to avoid borrow conflicts
        let issue = state.issue().clone();

        // Layout: title + content
        let layout = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(area);

        // Title block
        let title_text = vec![Line::from(vec![
            Span::styled(
                &issue.key,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - "),
            Span::raw(&issue.summary),
        ])];
        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());
        let title = Paragraph::new(title_text).block(title_block);
        frame.render_widget(title, layout[0]);

        // Content area
        let mut content_lines = vec![];

        // Status, Type, Priority
        content_lines.push(Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&issue.status),
            Span::raw("  "),
            Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&issue.issue_type),
            Span::raw("  "),
            Span::styled("Priority: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(&issue.priority),
        ]));
        content_lines.push(Line::from(""));

        // Assignee
        if !issue.assignee.is_empty() {
            content_lines.push(Line::from(vec![
                Span::styled("Assignee: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&issue.assignee),
            ]));
        }

        // Story points
        if let Some(points) = issue.story_points {
            content_lines.push(Line::from(vec![
                Span::styled(
                    "Story Points: ",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{}", points)),
            ]));
        }

        // Epic
        if let Some(ref epic) = issue.epic {
            content_lines.push(Line::from(vec![
                Span::styled("Epic: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(epic),
            ]));
        }

        // Parent
        if let Some(ref parent) = issue.parent_key {
            content_lines.push(Line::from(vec![
                Span::styled("Parent: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(parent),
            ]));
        }

        content_lines.push(Line::from(""));

        // Description
        if let Some(ref description) = issue.description {
            content_lines.push(Line::from(Span::styled(
                "Description:",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            content_lines.push(Line::from(""));

            // Split description by lines
            for line in description.lines() {
                content_lines.push(Line::from(line.to_string()));
            }
            content_lines.push(Line::from(""));
        }

        // Labels
        if !issue.labels.is_empty() {
            content_lines.push(Line::from(vec![
                Span::styled("Labels: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(issue.labels.join(", ")),
            ]));
        }

        // Comments section
        if !issue.comments.is_empty() {
            content_lines.push(Line::from(""));
            content_lines.push(Line::from(Span::styled(
                "Comments:",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            content_lines.push(Line::from(""));

            for comment in &issue.comments {
                content_lines.push(Line::from(vec![
                    Span::styled(&comment.author, Style::default().fg(Color::Yellow)),
                    Span::raw(" - "),
                    Span::raw(comment.created_at.as_deref().unwrap_or("Unknown")),
                ]));
                content_lines.push(Line::from(comment.body.as_str()));
                content_lines.push(Line::from(""));
            }
        }

        // Create initial block to calculate inner area
        let temp_block = Block::default().borders(Borders::ALL);
        let inner_area = temp_block.inner(layout[1]);

        // Update scroll bounds based on content size
        let total_lines = content_lines.len();
        let viewport_height = inner_area.inner(Margin::new(1, 0)).height as usize;
        state.update_bounds(total_lines, viewport_height);

        // Get scroll position for display
        let scroll_pct = state.scroll_percentage();
        let title = if total_lines > viewport_height {
            format!(" Details [{}%] ", scroll_pct)
        } else {
            " Details ".to_string()
        };

        // Create final content block with scroll info
        let content_block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default());

        frame.render_widget(content_block, layout[1]);

        // Apply scroll offset (get it after update_bounds to get clamped value)
        let scroll_offset = state.scroll_offset();
        let visible_lines: Vec<_> = content_lines.into_iter().skip(scroll_offset).collect();

        let content = Paragraph::new(visible_lines).wrap(Wrap { trim: false });

        frame.render_widget(content, inner_area.inner(Margin::new(1, 0)));
    }
}
