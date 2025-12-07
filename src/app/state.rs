#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Normal,
    Visual,
    Insert,
    Command,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ScreenType {
    #[default]
    Home,
    CurrentSprint,
    MyIssues,
    SearchIssues,
    NewIssue,
    Profiles,
}
