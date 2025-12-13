use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Span, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Widget, Wrap},
};

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
    pub fn as_span(&'_ self) -> Span<'_> {
        match self {
            Priority::NoBusinessValue => {
                Span::styled("No Business Value", Style::default().fg(Color::Gray))
            }
            Priority::Low => Span::styled("Low", Style::default().fg(Color::Green)),
            Priority::Lowest => Span::styled("Lowest", Style::default().fg(Color::Cyan)),
            Priority::Medium => Span::styled("Medium", Style::default().fg(Color::Yellow)),
            Priority::High => Span::styled("High", Style::default().fg(Color::Red)),
            Priority::Critical => Span::styled("Critical", Style::default().fg(Color::Magenta)),
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
    pub fn as_span(&'_ self) -> Span<'_> {
        match self {
            IssueType::Bug => Span::styled("B", Style::default().fg(Color::Red)),
            IssueType::Task => Span::styled("T", Style::default().fg(Color::Blue)),
            IssueType::Story => Span::styled("S", Style::default().fg(Color::Green)),
            IssueType::Subtask => Span::styled("ST", Style::default().fg(Color::Cyan)),
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

    pub fn render_with_selection(&self, area: Rect, buf: &mut Buffer, selected: bool) {
        let border_style = if selected {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .style(border_style);
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
            .wrap(Wrap { trim: false })
            .render(chunks[0], buf);
        if let Some(epic) = &self.epic {
            Paragraph::new(Text::from(epic.clone())).render(chunks[1], buf);
        }
        let sp_prio_layout = Layout::horizontal([
            Constraint::Length(4),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(chunks[2]);
        if let Some(sp) = self.story_points {
            let sp_span = Span::styled(format!("{} SP", sp), Style::default().fg(Color::Yellow));
            Paragraph::new(Text::from(sp_span)).render(sp_prio_layout[0], buf);
        }
        Paragraph::new(Text::from(self.priority.as_span())).render(sp_prio_layout[2], buf);
        let type_key_layout = Layout::horizontal([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
        ])
        .split(chunks[3]);
        Paragraph::new(Text::from(self.issue_type.as_span())).render(type_key_layout[0], buf);
        Paragraph::new(Text::from(self.key.clone())).render(type_key_layout[2], buf);
        Paragraph::new(Text::from(self.assignee.clone())).render(chunks[4], buf);
    }
}

impl Widget for IssueCardComponent {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.render_with_selection(area, buf, false);
    }
}
