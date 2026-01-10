use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap, Widget},
};

use crate::ui::context::RenderContext;

#[derive(Debug, Clone)]
pub enum Status {
    ToDo,
    InProgress,
    CodeReview,
    Done,
}

impl From<&str> for Status {
    fn from(s: &str) -> Self {
        match s {
            "To Do" => Status::ToDo,
            "In Progress" => Status::InProgress,
            "Code Review" => Status::CodeReview,
            "Done" => Status::Done,
            _ => Status::ToDo,
        }
    }
}

impl From<Status> for &str {
    fn from(status: Status) -> Self {
        match status {
            Status::ToDo => "To Do",
            Status::InProgress => "In Progress",
            Status::CodeReview => "Code Review",
            Status::Done => "Done",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum Priority {
    NoBusinessValue,
    Low,
    Lowest,
    #[default]
    Medium,
    High,
    Critical,
}

impl From<&str> for Priority {
    fn from(s: &str) -> Self {
        match s {
            "No Business Value" => Priority::NoBusinessValue,
            "Low" => Priority::Low,
            "Lowest" => Priority::Lowest,
            "Medium" => Priority::Medium,
            "High" => Priority::High,
            "Critical" => Priority::Critical,
            _ => Priority::Medium,
        }
    }
}

impl Priority {
    pub fn as_span<'a>(&'a self, context: &'a RenderContext) -> Span<'a> {
        let colors = context.colors();
        let color = match self {
            Priority::NoBusinessValue => colors.border,
            Priority::Low => colors.success,
            Priority::Lowest => colors.info,
            Priority::Medium => colors.accent,
            Priority::High => colors.warning,
            Priority::Critical => colors.error,
        };
        Span::styled(self.label(), Style::default().fg(color))
    }

    fn label(&self) -> &'static str {
        match self {
            Priority::NoBusinessValue => "No Business Value",
            Priority::Low => "Low",
            Priority::Lowest => "Lowest",
            Priority::Medium => "Medium",
            Priority::High => "High",
            Priority::Critical => "Critical",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum IssueType {
    Bug,
    #[default]
    Task,
    Story,
    Subtask,
}

impl From<&str> for IssueType {
    fn from(s: &str) -> Self {
        match s {
            "Bug" => IssueType::Bug,
            "Task" => IssueType::Task,
            "Story" => IssueType::Story,
            "Subtask" => IssueType::Subtask,
            _ => IssueType::Task,
        }
    }
}

impl IssueType {
    pub fn as_span<'a>(&'a self, context: &'a RenderContext) -> Span<'a> {
        let colors = context.colors();
        let color = match self {
            IssueType::Bug => colors.error,
            IssueType::Task => colors.info,
            IssueType::Story => colors.success,
            IssueType::Subtask => colors.accent,
        };
        Span::styled(self.label(), Style::default().fg(color))
    }

    fn label(&self) -> &'static str {
        match self {
            IssueType::Bug => "B",
            IssueType::Task => "T",
            IssueType::Story => "S",
            IssueType::Subtask => "ST",
        }
    }
}

#[derive(Debug, Clone)]
pub struct IssueCardComponent {
    pub key: String,
    pub summary: String,
    pub epic: Option<String>,
    pub status: String,
    pub issue_type: IssueType,
    pub assignee: String,
    pub priority: Priority,
    pub story_points: Option<f64>,
}

impl IssueCardComponent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: String,
        summary: String,
        epic: Option<String>,
        status: String,
        issue_type: IssueType,
        assignee: String,
        priority: Priority,
        story_points: Option<f64>,
    ) -> Self {
        Self {
            key,
            summary,
            epic,
            status,
            issue_type,
            assignee,
            priority,
            story_points,
        }
    }

    pub fn height(&self) -> u16 {
        8
    }

    pub fn render_with_selection(
        &self,
        area: Rect,
        buf: &mut Buffer,
        selected: bool,
        context: &RenderContext,
    ) {
        let colors = context.colors();
        let border_style = if selected {
            Style::default().fg(colors.accent)
        } else {
            Style::default().fg(colors.border)
        };
        let base_style = Style::default()
            .fg(colors.text)
            .bg(colors.background);

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .style(base_style)
            .border_style(border_style);
        let inner_area = block.inner(area);
        block.render(area, buf);
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner_area);
        Paragraph::new(Text::from(self.summary.clone()))
            .style(base_style)
            .wrap(Wrap { trim: false })
            .render(chunks[0], buf);
        if let Some(epic) = &self.epic {
            Paragraph::new(Text::from(epic.clone()))
                .style(base_style)
                .render(chunks[1], buf);
        }
        let sp_prio_layout = Layout::horizontal([
            Constraint::Length(4),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(chunks[2]);
        if let Some(sp) = self.story_points {
            let sp_span =
                Span::styled(format!("{} SP", sp), Style::default().fg(colors.warning));
            Paragraph::new(Text::from(sp_span))
                .style(base_style)
                .render(sp_prio_layout[0], buf);
        }
        Paragraph::new(Text::from(self.priority.as_span(context)))
            .style(base_style)
            .render(sp_prio_layout[2], buf);
        let type_key_layout = Layout::horizontal([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(chunks[3]);
        Paragraph::new(Text::from(self.issue_type.as_span(context)))
            .style(base_style)
            .render(type_key_layout[0], buf);
        Paragraph::new(Text::from(self.key.clone()))
            .style(base_style)
            .render(type_key_layout[2], buf);
        Paragraph::new(Text::from(self.assignee.clone()))
            .style(base_style)
            .render(chunks[4], buf);
    }
}
