use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    data::IssueSummary,
    ui::{
        components::layout::ModalFrame,
        context::RenderContext,
        interaction::{ActionHint, Mode},
    },
};

use super::state::IssueDetailState;

pub struct IssueDetailView;

impl IssueDetailView {
    /// Generate a consistent color for an author name
    fn author_color(author: &str) -> Color {
        // Use a simple hash to generate a consistent color per author
        let hash: u32 = author.bytes().map(|b| b as u32).sum();
        let colors = [
            Color::Yellow,
            Color::Cyan,
            Color::Magenta,
            Color::Blue,
            Color::Green,
            Color::LightYellow,
            Color::LightCyan,
            Color::LightMagenta,
            Color::LightBlue,
            Color::LightGreen,
        ];
        colors[(hash as usize) % colors.len()]
    }

    /// Build content lines from issue data
    fn build_content_lines(issue: &IssueSummary) -> Vec<Line<'static>> {
        let mut content_lines = vec![];

        // Status, Type, Priority
        content_lines.push(Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(issue.status.clone()),
            Span::raw("  "),
            Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(issue.issue_type.clone()),
            Span::raw("  "),
            Span::styled("Priority: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(issue.priority.clone()),
        ]));
        content_lines.push(Line::from(""));

        // Assignee
        if !issue.assignee.is_empty() {
            content_lines.push(Line::from(vec![
                Span::styled("Assignee: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(issue.assignee.clone()),
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
                Span::raw(epic.clone()),
            ]));
        }

        // Parent
        if let Some(ref parent) = issue.parent_key {
            content_lines.push(Line::from(vec![
                Span::styled("Parent: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(parent.clone()),
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

            // Split description by lines with basic formatting
            let mut in_code_block = false;
            for line in description.lines() {
                // Detect code blocks
                if line.trim().starts_with("```") {
                    in_code_block = !in_code_block;
                    content_lines.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(Color::DarkGray),
                    )));
                    continue;
                }

                // Format based on context
                if in_code_block {
                    // Code block line - use monospace-style color
                    content_lines.push(Line::from(Span::styled(
                        format!("  {}", line),
                        Style::default().fg(Color::Green),
                    )));
                } else if line.trim().starts_with("# ") {
                    // Heading level 1
                    content_lines.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    )));
                } else if line.trim().starts_with("## ") {
                    // Heading level 2
                    content_lines.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(Color::Cyan),
                    )));
                } else if line.trim().starts_with("* ") || line.trim().starts_with("- ") {
                    // List item
                    content_lines.push(Line::from(Span::styled(
                        line.to_string(),
                        Style::default().fg(Color::White),
                    )));
                } else {
                    // Normal line
                    content_lines.push(Line::from(line.to_string()));
                }
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
                let author_color = Self::author_color(&comment.author);
                content_lines.push(Line::from(vec![
                    Span::styled(comment.author.clone(), Style::default().fg(author_color)),
                    Span::raw(" - "),
                    Span::raw(
                        comment
                            .created_at
                            .as_deref()
                            .unwrap_or("Unknown")
                            .to_string(),
                    ),
                ]));
                // Add indentation and formatting for comment body
                let mut in_code_block = false;
                for line in comment.body.lines() {
                    // Detect code blocks
                    if line.trim().starts_with("```") {
                        in_code_block = !in_code_block;
                        content_lines.push(Line::from(Span::styled(
                            format!("  {}", line),
                            Style::default().fg(Color::DarkGray),
                        )));
                        continue;
                    }

                    // Format based on context
                    if in_code_block {
                        // Code block line
                        content_lines.push(Line::from(Span::styled(
                            format!("    {}", line),
                            Style::default().fg(Color::Green),
                        )));
                    } else {
                        // Normal comment line with indentation
                        content_lines.push(Line::from(format!("  {}", line)));
                    }
                }
                content_lines.push(Line::from(""));
            }
        }

        content_lines
    }

    pub fn draw(
        frame: &mut Frame,
        state: &mut IssueDetailState,
        _mode: Mode,
        _actions: &[ActionHint],
        context: &RenderContext,
    ) {
        let area = frame.area();
        let Some(issue) = state.issue() else {
            let message = state
                .unavailable_message()
                .unwrap_or("Issue details are unavailable");
            let block = Block::default()
                .borders(Borders::ALL)
                .title(" Issue Detail ");
            frame.render_widget(
                Paragraph::new(message)
                    .alignment(Alignment::Center)
                    .block(block),
                area,
            );
            return;
        };
        let issue_key = issue.key.clone();
        let issue_summary = issue.summary.clone();

        let layout = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).split(area);
        let title_text = vec![Line::from(vec![
            Span::styled(
                issue_key,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - "),
            Span::raw(issue_summary),
        ])];
        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());
        frame.render_widget(Paragraph::new(title_text).block(title_block), layout[0]);

        if state.cached_content().is_none() {
            let lines = state
                .issue()
                .map(Self::build_content_lines)
                .unwrap_or_default();
            state.cache_content(lines);
        }
        let content_lines = state.cached_content().cloned().unwrap_or_default();
        let temp_block = Block::default().borders(Borders::ALL);
        let inner_area = temp_block.inner(layout[1]).inner(Margin::new(1, 0));
        let measured = Paragraph::new(content_lines.clone()).wrap(Wrap { trim: false });
        let total_lines = measured.line_count(inner_area.width);
        state.update_bounds(total_lines, inner_area.height as usize);

        let title = if total_lines > inner_area.height as usize {
            format!(" Details [{}%] ", state.scroll_percentage())
        } else {
            " Details ".to_string()
        };
        let content_block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default());
        frame.render_widget(content_block, layout[1]);

        let vertical_offset = state.scroll_offset().min(u16::MAX as usize) as u16;
        let horizontal_offset = state.horizontal_offset().min(u16::MAX as usize) as u16;
        let content = Paragraph::new(content_lines)
            .wrap(Wrap { trim: false })
            .scroll((vertical_offset, horizontal_offset));
        frame.render_widget(content, inner_area);

        if state.transition_picker_open() {
            Self::draw_transition_picker(frame, state, context);
        } else if state.comment_input().is_some() {
            Self::draw_comment_input(frame, state, context);
        }
    }

    fn draw_transition_picker(
        frame: &mut Frame,
        state: &IssueDetailState,
        context: &RenderContext,
    ) {
        let area = frame.area();
        let width = (area.width.saturating_mul(6) / 10).max(32).min(area.width);
        let content_height = state.transitions().len().clamp(1, 12) as u16;
        let height = content_height.saturating_add(4).min(area.height);
        let modal = crate::ui::layout::modal_area(area, width, height);
        let title = if state.transitions_from_cache() {
            "Transition Issue (cached, Jira unreachable)"
        } else {
            "Transition Issue"
        };
        let inner = ModalFrame::new(
            title,
            modal,
            Style::default().fg(context.colors().accent),
            context,
        )
        .render(frame);

        let lines = if let Some(error) = state.transition_error() {
            vec![Line::from(error).style(Style::default().fg(context.colors().error))]
        } else if state.transitions().is_empty() {
            vec![Line::from("No transitions are available")]
        } else {
            state
                .transitions()
                .iter()
                .enumerate()
                .map(|(index, transition)| {
                    let marker = if index == state.transition_selected() {
                        "> "
                    } else {
                        "  "
                    };
                    let style = if index == state.transition_selected() {
                        Style::default()
                            .fg(context.colors().accent)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(context.colors().text)
                    };
                    Line::from(format!(
                        "{marker}{} → {}",
                        transition.name, transition.to_status
                    ))
                    .style(style)
                })
                .collect()
        };
        frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }

    fn draw_comment_input(frame: &mut Frame, state: &IssueDetailState, context: &RenderContext) {
        let Some(input) = state.comment_input() else {
            return;
        };
        let area = frame.area();
        let width = (area.width.saturating_mul(7) / 10).max(32).min(area.width);
        let height = 5.min(area.height);
        let modal = crate::ui::layout::modal_area(area, width, height);
        let inner = ModalFrame::new(
            "Add Comment",
            modal,
            Style::default().fg(context.colors().accent),
            context,
        )
        .render(frame);
        let scroll = input.visual_scroll(inner.width as usize);
        frame.render_widget(
            Paragraph::new(input.value()).scroll((0, scroll.min(u16::MAX as usize) as u16)),
            inner,
        );
        let cursor = input.visual_cursor().saturating_sub(scroll);
        let cursor_x = inner
            .x
            .saturating_add(cursor.min(inner.width.saturating_sub(1) as usize) as u16);
        frame.set_cursor_position((cursor_x, inner.y));
    }
}
