#[derive(Debug, Clone, PartialEq, Eq, Default, Copy)]
pub enum Mode {
    #[default]
    Normal,
    Visual,
    Insert,
    Command,
}

impl From<Mode> for &'static str {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Normal => "NORMAL",
            Mode::Visual => "VISUAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
        }
    }
}

impl Mode {
    pub fn label(self) -> &'static str {
        self.into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum ScreenType {
    #[default]
    Home,
    BoardSelection,
    CurrentSprint,
    MyIssues,
    SearchIssues,
    NewIssue,
    IssueDetail,
    Conflicts,
    SyncStatus,
    Settings,
    SettingsThemes,
    SettingsThemeForm,
    ProfileCreation,
    Profiles,
}
