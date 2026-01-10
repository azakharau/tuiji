#[derive(Clone, Debug)]
pub struct FormField {
    pub label: &'static str,
    pub value: String,
    pub cursor_position: usize,
    pub is_password: bool,
    pub validation_state: Option<FormError>,
}

#[derive(Clone, Debug)]
pub struct FormError {
    pub message: String,
}

impl FormField {
    pub fn new(label: &'static str, is_password: bool) -> Self {
        Self {
            label,
            value: String::new(),
            cursor_position: 0,
            is_password,
            validation_state: None,
        }
    }

    pub fn masked_value(&self) -> String {
        if self.is_password {
            "*".repeat(self.value.len())
        } else {
            self.value.clone()
        }
    }
}
