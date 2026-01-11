use crate::{
    app::error::AppErrorState,
    config::ProfileConfig,
    ui::components::form::{CursorState, FieldValue, FormField, FormState},
};

pub struct ProfileCreationState {
    form: FormState,
    profile_id: Option<String>,
    sync_mode: Option<String>,
    error: Option<AppErrorState>,
}

impl ProfileCreationState {
    pub fn new(profile: Option<ProfileConfig>) -> Self {
        match profile {
            Some(profile) => Self::from_profile(profile),
            None => Self::default(),
        }
    }

    pub fn set_profile_id(&mut self, id: String) {
        self.profile_id = Some(id);
    }

    pub fn profile_id(&self) -> Option<&str> {
        self.profile_id.as_deref()
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

    pub fn sync_mode(&self) -> Option<&String> {
        self.sync_mode.as_ref()
    }

    pub fn title(&self) -> &'static str {
        if self.profile_id.is_some() {
            "Edit Profile"
        } else {
            "Profile Creation"
        }
    }

    fn from_profile(profile: ProfileConfig) -> Self {
        let mut state = Self::default();
        state.profile_id = Some(profile.id);
        state.sync_mode = profile.sync_mode;
        if let Some(item) = state.form.fields_mut().get_mut(0) {
            item.value = FieldValue::Text(profile.name.clone());
            if let CursorState::Text { position } = &mut item.cursor {
                *position = profile.name.len();
            }
        }
        if let Some(item) = state.form.fields_mut().get_mut(1) {
            item.value = FieldValue::Text(profile.jira.base_url.clone());
            if let CursorState::Text { position } = &mut item.cursor {
                *position = profile.jira.base_url.len();
            }
        }
        if let Some(item) = state.form.fields_mut().get_mut(2) {
            item.value = FieldValue::Text(profile.jira.username.clone());
            if let CursorState::Text { position } = &mut item.cursor {
                *position = profile.jira.username.len();
            }
        }
        if let Some(item) = state.form.fields_mut().get_mut(3) {
            item.value = FieldValue::Text(profile.jira.api_token.clone());
            if let CursorState::Text { position } = &mut item.cursor {
                *position = profile.jira.api_token.len();
            }
        }
        state
    }
}

impl Default for ProfileCreationState {
    fn default() -> Self {
        let form = FormState::new(vec![
            FormField::text("Profile Name", false),
            FormField::text("Jira URL", false),
            FormField::text("Email", false),
            FormField::text("API Token", true),
        ]);
        Self {
            form,
            profile_id: None,
            sync_mode: None,
            error: None,
        }
    }
}
