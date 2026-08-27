use crate::{
    app::FormPurpose,
    contracts::error::AppErrorState,
    data::model::{IssueDraft, IssueMutation, IssuePatch},
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
    purpose: FormPurpose,
    project_key: Option<String>,
    initial_summary: String,
    initial_description: String,
    title: String,
    error: Option<AppErrorState>,
    active_surface: IssueFormSurface,
}

impl IssueFormState {
    #[cfg(test)]
    pub fn new() -> Self {
        Self::create(None, Vec::new())
    }

    #[cfg(test)]
    pub fn with_issue_types(issue_types: Vec<String>) -> Self {
        Self::create(None, issue_types)
    }

    pub fn create(project_key: Option<String>, issue_types: Vec<String>) -> Self {
        Self::with_values(
            FormPurpose::Create,
            project_key,
            issue_types,
            String::new(),
            String::new(),
        )
    }

    pub fn edit(key: String, summary: String, description: Option<String>) -> Self {
        Self::with_values(
            FormPurpose::Edit(key),
            None,
            Vec::new(),
            summary,
            description.unwrap_or_default(),
        )
    }

    fn with_values(
        purpose: FormPurpose,
        project_key: Option<String>,
        issue_types: Vec<String>,
        summary: String,
        description: String,
    ) -> Self {
        let include_issue_type = matches!(&purpose, FormPurpose::Create);
        let title = match &purpose {
            FormPurpose::Create => "Create Issue".to_string(),
            FormPurpose::Edit(key) => format!("Edit Issue {key}"),
        };
        let form = Self::create_form(issue_types, &summary, &description, include_issue_type);

        Self {
            form,
            purpose,
            project_key,
            initial_summary: summary,
            initial_description: description,
            title,
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

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn screen_name(&self) -> &'static str {
        match self.purpose {
            FormPurpose::Create => "Create Issue",
            FormPurpose::Edit(_) => "Edit Issue",
        }
    }

    pub fn submission(&self) -> Result<Option<IssueMutation>, String> {
        let summary = self.field_text(0, "Summary")?;
        let description = self.field_text(1, "Description")?;

        if summary.trim().is_empty() {
            return Err("Summary is required".to_string());
        }

        match &self.purpose {
            FormPurpose::Create => {
                let project_key = self
                    .project_key
                    .as_deref()
                    .filter(|key| !key.trim().is_empty())
                    .ok_or_else(|| "Project key is required".to_string())?;
                let issue_type = self
                    .form
                    .fields()
                    .get(2)
                    .and_then(|field| field.value.as_single())
                    .filter(|issue_type| !issue_type.trim().is_empty())
                    .ok_or_else(|| "Issue Type is required".to_string())?;

                Ok(Some(IssueMutation::Create(IssueDraft {
                    project_key: project_key.to_string(),
                    issue_type: issue_type.to_string(),
                    summary: summary.to_string(),
                    description: (!description.trim().is_empty()).then(|| description.to_string()),
                })))
            }
            FormPurpose::Edit(key) => {
                let patch = IssuePatch {
                    summary: (summary != self.initial_summary.as_str())
                        .then(|| summary.to_string()),
                    description: (description != self.initial_description.as_str())
                        .then(|| description.to_string()),
                    priority: None,
                };

                Ok((!patch.is_empty()).then(|| IssueMutation::Patch {
                    key: key.clone(),
                    patch,
                }))
            }
        }
    }

    fn field_text(&self, index: usize, label: &str) -> Result<&str, String> {
        self.form
            .fields()
            .get(index)
            .and_then(|field| field.value.as_text())
            .ok_or_else(|| format!("{label} field is unavailable"))
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
        match self.active_surface {
            IssueFormSurface::TextPopup { .. } => true,
            IssueFormSurface::Form | IssueFormSurface::Dropdown { .. } => false,
        }
    }

    pub fn is_dropdown_open(&self) -> bool {
        match self.active_surface {
            IssueFormSurface::Dropdown { .. } => true,
            IssueFormSurface::Form | IssueFormSurface::TextPopup { .. } => false,
        }
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

    fn create_form(
        issue_types: Vec<String>,
        summary: &str,
        description: &str,
        include_issue_type: bool,
    ) -> FormState {
        let mut fields = vec![
            FormField::text("Summary", false)
                .required()
                .with_value(summary),
            FormField::textarea("Description", 8).with_value(description),
        ];

        if include_issue_type {
            let issue_type_options = issue_types
                .into_iter()
                .map(|issue_type| SelectOption::new(issue_type.clone(), issue_type))
                .collect();
            fields.push(FormField::select("Issue Type", issue_type_options).required());
        }

        FormState::new(fields)
    }
}
