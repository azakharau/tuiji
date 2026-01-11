use crate::{
    app::error::AppErrorState,
    ui::components::form::{FormField, FormState, SelectOption},
};

pub struct IssueFormState {
    form: FormState,
    error: Option<AppErrorState>,
    text_popup_open: bool,
}

impl IssueFormState {
    pub fn new() -> Self {
        Self {
            form: Self::create_form(),
            error: None,
            text_popup_open: false,
        }
    }

    pub fn set_error(&mut self, error: AppErrorState) {
        self.error = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn error(&self) -> Option<&AppErrorState> {
        self.error.as_ref()
    }

    pub fn form(&self) -> &FormState {
        &self.form
    }

    pub fn form_mut(&mut self) -> &mut FormState {
        &mut self.form
    }

    pub fn title(&self) -> &'static str {
        "Create Issue"
    }

    pub fn is_text_popup_open(&self) -> bool {
        self.text_popup_open
    }

    pub fn open_text_popup(&mut self) {
        self.text_popup_open = true;
    }

    pub fn close_text_popup(&mut self) {
        self.text_popup_open = false;
    }

    fn create_form() -> FormState {
        FormState::new(vec![
            // Summary (required)
            FormField::text("Summary", false).required(),
            // Description (multi-line)
            FormField::textarea("Description", 8),
            // Issue Type
            FormField::select(
                "Issue Type",
                vec![
                    SelectOption::new("Story", "story").selected(),
                    SelectOption::new("Task", "task"),
                    SelectOption::new("Bug", "bug"),
                    SelectOption::new("Epic", "epic"),
                    SelectOption::new("Sub-task", "subtask"),
                ],
            ),
            // Status
            FormField::select(
                "Status",
                vec![
                    SelectOption::new("To Do", "todo").selected(),
                    SelectOption::new("In Progress", "in_progress"),
                    SelectOption::new("In Review", "in_review"),
                    SelectOption::new("Done", "done"),
                    SelectOption::new("Blocked", "blocked"),
                ],
            ),
            // Priority
            FormField::select(
                "Priority",
                vec![
                    SelectOption::new("Highest", "highest"),
                    SelectOption::new("High", "high"),
                    SelectOption::new("Medium", "medium").selected(),
                    SelectOption::new("Low", "low"),
                    SelectOption::new("Lowest", "lowest"),
                ],
            ),
            // Assignee
            FormField::select(
                "Assignee",
                vec![
                    SelectOption::new("Unassigned", "unassigned").selected(),
                    SelectOption::new("John Doe", "john.doe"),
                    SelectOption::new("Jane Smith", "jane.smith"),
                    SelectOption::new("Bob Johnson", "bob.johnson"),
                    SelectOption::new("Alice Williams", "alice.williams"),
                ],
            ),
            // Reporter
            FormField::select(
                "Reporter",
                vec![
                    SelectOption::new("John Doe", "john.doe").selected(),
                    SelectOption::new("Jane Smith", "jane.smith"),
                    SelectOption::new("Bob Johnson", "bob.johnson"),
                    SelectOption::new("Alice Williams", "alice.williams"),
                ],
            ),
            // Labels (multi-select)
            FormField::multiselect(
                "Labels",
                vec![
                    SelectOption::new("backend", "backend"),
                    SelectOption::new("frontend", "frontend"),
                    SelectOption::new("bug", "bug"),
                    SelectOption::new("feature", "feature"),
                    SelectOption::new("urgent", "urgent"),
                    SelectOption::new("tech-debt", "tech-debt"),
                ],
            ),
            // Components (multi-select)
            FormField::multiselect(
                "Components",
                vec![
                    SelectOption::new("API", "api"),
                    SelectOption::new("UI", "ui"),
                    SelectOption::new("Database", "database"),
                    SelectOption::new("Authentication", "auth"),
                    SelectOption::new("Documentation", "docs"),
                ],
            ),
            // Story Points (text field)
            FormField::text("Story Points", false),
            // Sprint
            FormField::select(
                "Sprint",
                vec![
                    SelectOption::new("None", "none").selected(),
                    SelectOption::new("Sprint 23", "sprint-23"),
                    SelectOption::new("Sprint 24", "sprint-24"),
                    SelectOption::new("Sprint 25", "sprint-25"),
                ],
            ),
            // Epic Link
            FormField::select(
                "Epic Link",
                vec![
                    SelectOption::new("None", "none").selected(),
                    SelectOption::new("EPIC-1: User Authentication", "epic-1"),
                    SelectOption::new("EPIC-2: Performance Improvements", "epic-2"),
                    SelectOption::new("EPIC-3: Mobile App", "epic-3"),
                ],
            ),
            // Environment (textarea)
            FormField::textarea("Environment", 3),
            // Due Date (text field - would be date picker in real app)
            FormField::text("Due Date (YYYY-MM-DD)", false),
        ])
    }
}
