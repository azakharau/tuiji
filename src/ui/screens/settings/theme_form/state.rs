use crate::{
    contracts::error::AppErrorState,
    ui::components::form::{CursorState, FieldValue, FormField, FormState},
    ui::theme::{ThemePalette, color_to_hex},
};

pub struct SettingsThemeFormState {
    form: FormState,
    error: Option<AppErrorState>,
    existing_ids: Vec<String>,
}

impl SettingsThemeFormState {
    pub fn new(palette: ThemePalette, existing_ids: Vec<String>) -> Self {
        let mut fields = vec![
            FormField::text("Theme name", false),
            FormField::text("Background", false),
            FormField::text("Text", false),
            FormField::text("Accent", false),
            FormField::text("Selection", false),
            FormField::text("Border", false),
            FormField::text("Error", false),
            FormField::text("Warning", false),
            FormField::text("Info", false),
            FormField::text("Success", false),
        ];
        let defaults = vec![
            String::new(),
            color_to_hex(palette.background),
            color_to_hex(palette.text),
            color_to_hex(palette.accent),
            color_to_hex(palette.selection),
            color_to_hex(palette.border),
            color_to_hex(palette.error),
            color_to_hex(palette.warning),
            color_to_hex(palette.info),
            color_to_hex(palette.success),
        ];
        for (field, value_str) in fields.iter_mut().zip(defaults.into_iter()) {
            field.value = FieldValue::Text(value_str.clone());
            if let CursorState::Text { position } = &mut field.cursor {
                *position = value_str.len();
            }
        }
        Self {
            form: FormState::new(fields),
            error: None,
            existing_ids: existing_ids
                .into_iter()
                .map(|id| id.to_lowercase())
                .collect(),
        }
    }

    pub fn form(&self) -> &FormState {
        &self.form
    }

    pub fn form_mut(&mut self) -> &mut FormState {
        &mut self.form
    }

    pub fn error(&self) -> Option<&AppErrorState> {
        self.error.as_ref()
    }

    pub fn set_error(&mut self, err: AppErrorState) {
        self.error = Some(err);
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }

    pub fn unique_theme_id(&self, name: &str) -> String {
        let mut base = slugify(name);
        if base.is_empty() {
            base = "custom".to_string();
        }
        if !self
            .existing_ids
            .iter()
            .any(|id| id.eq_ignore_ascii_case(&base))
        {
            return base;
        }
        let mut idx = 2;
        loop {
            let candidate = format!("{base}-{idx}");
            if !self
                .existing_ids
                .iter()
                .any(|id| id.eq_ignore_ascii_case(&candidate))
            {
                return candidate;
            }
            idx += 1;
        }
    }
}

fn slugify(value: &str) -> String {
    let mut out = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            out.push(lower);
            last_dash = false;
        } else if !last_dash {
            out.push('-');
            last_dash = true;
        }
    }
    while out.starts_with('-') {
        out.remove(0);
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}
