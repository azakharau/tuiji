use crate::{
    contracts::error::AppErrorState,
    ui::components::form::{FieldType, FormField, FormState, SelectOption},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueFormSurface {
    Form,
    TextPopup { field_index: usize },
    Dropdown { field_index: usize },
}

pub struct IssueFormState {
    form: FormState,
    error: Option<AppErrorState>,
    active_surface: IssueFormSurface,
}

impl IssueFormState {
    pub fn new() -> Self {
        Self {
            form: Self::create_form(),
            error: None,
            active_surface: IssueFormSurface::Form,
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

    pub fn active_surface(&self) -> IssueFormSurface {
        self.active_surface
    }

    pub fn active_overlay_field_index(&self) -> Option<usize> {
        match self.active_surface {
            IssueFormSurface::Form => None,
            IssueFormSurface::TextPopup { field_index }
            | IssueFormSurface::Dropdown { field_index } => Some(field_index),
        }
    }

    pub fn is_text_popup_open(&self) -> bool {
        matches!(self.active_surface, IssueFormSurface::TextPopup { .. })
    }

    pub fn is_dropdown_open(&self) -> bool {
        matches!(self.active_surface, IssueFormSurface::Dropdown { .. })
    }

    pub fn open_text_popup(&mut self) {
        self.active_surface = IssueFormSurface::TextPopup {
            field_index: self.form.selected_index(),
        };
    }

    pub fn close_text_popup(&mut self) {
        if self.is_text_popup_open() {
            self.active_surface = IssueFormSurface::Form;
        }
    }

    pub fn open_dropdown(&mut self) {
        let selected_index = self.form.selected_index();
        if let Some(field) = self.form.selected_field_mut()
            && matches!(
                field.field_type,
                FieldType::Select { .. } | FieldType::MultiSelect { .. }
            )
        {
            field.field_type.set_expanded(true);
            self.active_surface = IssueFormSurface::Dropdown {
                field_index: selected_index,
            };
        }
    }

    pub fn close_dropdown(&mut self) {
        if let Some(field_index) = self.active_overlay_field_index()
            && let Some(field) = self.form.fields_mut().get_mut(field_index)
        {
            field.field_type.set_expanded(false);
        }

        if self.is_dropdown_open() {
            self.active_surface = IssueFormSurface::Form;
        }
    }

    pub fn close_active_overlay(&mut self) {
        match self.active_surface {
            IssueFormSurface::Form => {}
            IssueFormSurface::TextPopup { .. } => self.close_text_popup(),
            IssueFormSurface::Dropdown { .. } => self.close_dropdown(),
        }
    }

    pub fn hide_form_content_for(&self) -> Option<usize> {
        match self.active_surface {
            IssueFormSurface::TextPopup { field_index } => Some(field_index),
            IssueFormSurface::Form | IssueFormSurface::Dropdown { .. } => None,
        }
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
