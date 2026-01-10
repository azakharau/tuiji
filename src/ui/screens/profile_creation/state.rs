use crate::{
    app::error::AppErrorState,
    config::ProfileConfig,
    ui::components::form::{FormField, FormState},
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
            item.value = profile.name;
            item.cursor_position = item.value.len();
        }
        if let Some(item) = state.form.fields_mut().get_mut(1) {
            item.value = profile.jira.base_url;
            item.cursor_position = item.value.len();
        }
        if let Some(item) = state.form.fields_mut().get_mut(2) {
            item.value = profile.jira.username;
            item.cursor_position = item.value.len();
        }
        if let Some(item) = state.form.fields_mut().get_mut(3) {
            item.value = profile.jira.api_token;
            item.cursor_position = item.value.len();
        }
        state
    }
}

impl Default for ProfileCreationState {
    fn default() -> Self {
        let form = FormState::new(vec![
            FormField::new("Profile Name", false),
            FormField::new("Jira URL", false),
            FormField::new("Email", false),
            FormField::new("API Token", true),
        ]);
        Self {
            form,
            profile_id: None,
            sync_mode: None,
            error: None,
        }
    }
}
