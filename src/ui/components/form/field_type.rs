/// Represents different types of form fields
#[derive(Clone, Debug)]
pub enum FieldType {
    /// Single-line text input
    Text { is_password: bool },
    /// Multi-line text input
    TextArea {
        rows: usize,             // Initial/minimum height in lines
        max_rows: Option<usize>, // Maximum height before scrolling
    },
    /// Single-choice dropdown
    Select {
        options: Vec<SelectOption>,
        expanded: bool, // Whether dropdown is currently expanded
    },
    /// Multiple-choice with checkboxes
    MultiSelect {
        options: Vec<SelectOption>,
        expanded: bool,
    },
}

/// Represents an option in Select/MultiSelect fields
#[derive(Clone, Debug)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
    pub selected: bool, // Used by MultiSelect to track selection
}

impl SelectOption {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            selected: false,
        }
    }

    pub fn selected(mut self) -> Self {
        self.selected = true;
        self
    }
}

/// Represents the value stored in a form field
#[derive(Clone, Debug)]
pub enum FieldValue {
    /// For Text and TextArea fields
    Text(String),
    /// For Select fields (single selected value)
    Single(Option<String>),
    /// For MultiSelect fields (multiple selected values)
    Multiple(Vec<String>),
}

impl FieldValue {
    /// Get value as a single string (for Text/TextArea/Select)
    pub fn as_text(&self) -> Option<&str> {
        match self {
            FieldValue::Text(s) => Some(s.as_str()),
            FieldValue::Single(Some(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Get value as a mutable string (for Text/TextArea)
    pub fn as_text_mut(&mut self) -> Option<&mut String> {
        match self {
            FieldValue::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Get selected option value (for Select)
    pub fn as_single(&self) -> Option<&str> {
        match self {
            FieldValue::Single(Some(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Get selected option values (for MultiSelect)
    pub fn as_multiple(&self) -> Option<&[String]> {
        match self {
            FieldValue::Multiple(v) => Some(v.as_slice()),
            _ => None,
        }
    }

    /// Check if value is empty
    pub fn is_empty(&self) -> bool {
        match self {
            FieldValue::Text(s) => s.is_empty(),
            FieldValue::Single(opt) => opt.is_none() || opt.as_ref().unwrap().is_empty(),
            FieldValue::Multiple(v) => v.is_empty(),
        }
    }
}

/// Represents cursor position for different field types
#[derive(Clone, Debug)]
pub enum CursorState {
    /// Single-line text cursor
    Text { position: usize },
    /// Multi-line text cursor
    TextArea { row: usize, col: usize },
    /// Dropdown selection cursor
    Select { index: usize },
    /// Multi-select list cursor
    MultiSelect { index: usize },
}

impl CursorState {
    pub fn text() -> Self {
        CursorState::Text { position: 0 }
    }

    pub fn textarea() -> Self {
        CursorState::TextArea { row: 0, col: 0 }
    }

    pub fn select() -> Self {
        CursorState::Select { index: 0 }
    }

    pub fn multiselect() -> Self {
        CursorState::MultiSelect { index: 0 }
    }
}

impl FieldType {
    /// Create initial cursor state for this field type
    pub fn initial_cursor(&self) -> CursorState {
        match self {
            FieldType::Text { .. } => CursorState::text(),
            FieldType::TextArea { .. } => CursorState::textarea(),
            FieldType::Select { .. } => CursorState::select(),
            FieldType::MultiSelect { .. } => CursorState::multiselect(),
        }
    }

    /// Create initial value for this field type
    pub fn initial_value(&self) -> FieldValue {
        match self {
            FieldType::Text { .. } => FieldValue::Text(String::new()),
            FieldType::TextArea { .. } => FieldValue::Text(String::new()),
            FieldType::Select { .. } => FieldValue::Single(None),
            FieldType::MultiSelect { .. } => FieldValue::Multiple(Vec::new()),
        }
    }

    /// Get mutable access to options (for Select/MultiSelect)
    pub fn options_mut(&mut self) -> Option<&mut Vec<SelectOption>> {
        match self {
            FieldType::Select { options, .. } => Some(options),
            FieldType::MultiSelect { options, .. } => Some(options),
            _ => None,
        }
    }

    /// Get current selected index (for Select/MultiSelect)
    pub fn selected_index(&self, cursor: &CursorState) -> Option<usize> {
        match (self, cursor) {
            (FieldType::Select { .. }, CursorState::Select { index }) => Some(*index),
            (FieldType::MultiSelect { .. }, CursorState::MultiSelect { index }) => Some(*index),
            _ => None,
        }
    }

    /// Check if field is currently expanded (for Select/MultiSelect)
    pub fn is_expanded(&self) -> bool {
        match self {
            FieldType::Select { expanded, .. } => *expanded,
            FieldType::MultiSelect { expanded, .. } => *expanded,
            _ => false,
        }
    }

    /// Set expanded state (for Select/MultiSelect)
    pub fn set_expanded(&mut self, value: bool) {
        match self {
            FieldType::Select { expanded, .. } => *expanded = value,
            FieldType::MultiSelect { expanded, .. } => *expanded = value,
            _ => {}
        }
    }
}
