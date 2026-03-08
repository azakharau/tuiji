#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppErrorLevel {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct AppErrorState {
    pub title: String,
    pub message: String,
    pub level: AppErrorLevel,
}

impl AppErrorState {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            level: AppErrorLevel::Error,
        }
    }

    pub fn with_level(
        title: impl Into<String>,
        message: impl Into<String>,
        level: AppErrorLevel,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            level,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::with_level("Error", message, AppErrorLevel::Error)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::with_level("Warning", message, AppErrorLevel::Warning)
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::with_level("Info", message, AppErrorLevel::Info)
    }
}
