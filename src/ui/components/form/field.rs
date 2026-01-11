use super::field_type::{CursorState, FieldType, FieldValue, SelectOption};

#[derive(Clone, Debug)]
pub struct FormField {
    pub label: String,
    pub field_type: FieldType,
    pub value: FieldValue,
    pub cursor: CursorState,
    pub validation_state: Option<FormError>,
    pub required: bool,
}

#[derive(Clone, Debug)]
pub struct FormError {
    pub message: String,
}

impl FormField {
    /// Create a new form field with the given type
    pub fn new(label: impl Into<String>, field_type: FieldType) -> Self {
        let value = field_type.initial_value();
        let cursor = field_type.initial_cursor();
        Self {
            label: label.into(),
            field_type,
            value,
            cursor,
            validation_state: None,
            required: false,
        }
    }

    /// Create a simple text field (backwards compatibility)
    pub fn text(label: impl Into<String>, is_password: bool) -> Self {
        Self::new(label, FieldType::Text { is_password })
    }

    /// Create a multi-line textarea field
    pub fn textarea(label: impl Into<String>, rows: usize) -> Self {
        Self::new(
            label,
            FieldType::TextArea {
                rows,
                max_rows: None,
            },
        )
    }

    /// Create a select dropdown field
    pub fn select(label: impl Into<String>, options: Vec<SelectOption>) -> Self {
        Self::new(
            label,
            FieldType::Select {
                options,
                expanded: false,
            },
        )
    }

    /// Create a multi-select field
    pub fn multiselect(label: impl Into<String>, options: Vec<SelectOption>) -> Self {
        Self::new(
            label,
            FieldType::MultiSelect {
                options,
                expanded: false,
            },
        )
    }

    /// Mark field as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set initial value for text fields
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        match &mut self.value {
            FieldValue::Text(s) => {
                *s = value.into();
                // Update cursor position to end of value
                if let CursorState::Text { position } = &mut self.cursor {
                    *position = s.len();
                }
            }
            _ => {}
        }
        self
    }

    /// Set initial selected value for select fields
    pub fn with_selected(mut self, value: impl Into<String>) -> Self {
        let value_str = value.into();
        match &mut self.value {
            FieldValue::Single(opt) => *opt = Some(value_str),
            _ => {}
        }
        self
    }

    /// Get display value (masked for passwords)
    pub fn display_value(&self) -> String {
        match (&self.field_type, &self.value) {
            (FieldType::Text { is_password: true }, FieldValue::Text(s)) => "*".repeat(s.len()),
            (_, FieldValue::Text(s)) => s.clone(),
            (_, FieldValue::Single(Some(s))) => s.clone(),
            (_, FieldValue::Single(None)) => String::new(),
            (_, FieldValue::Multiple(v)) => v.join(", "),
        }
    }

    /// For backwards compatibility - returns masked value if password
    pub fn masked_value(&self) -> String {
        self.display_value()
    }

    /// Get cursor position for text fields (backwards compatibility)
    pub fn cursor_position(&self) -> usize {
        match self.cursor {
            CursorState::Text { position } => position,
            CursorState::TextArea { col, .. } => col,
            _ => 0,
        }
    }

    /// Check if value is valid
    pub fn validate(&self) -> Option<FormError> {
        if self.required && self.value.is_empty() {
            return Some(FormError {
                message: format!("{} is required", self.label),
            });
        }
        None
    }
}
