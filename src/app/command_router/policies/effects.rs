use crate::{data::model::IssueMutation, ui::screens::ScreenState};

pub enum SyncControl {
    Now,
    Pause,
    Retry,
    Resume,
}

pub enum ScreenEffect<'a> {
    ResolveConflict { key: &'a str, use_remote: bool },
    Mutate(&'a IssueMutation),
    OpenInBrowser(&'a str),
    RunSearch(&'a str),
    Sync(SyncControl),
    None,
}

pub struct ScreenEffectPolicy;

impl ScreenEffectPolicy {
    pub fn classify(state: &ScreenState) -> ScreenEffect<'_> {
        match state {
            ScreenState::ResolveConflictLocal(key) => ScreenEffect::ResolveConflict {
                key,
                use_remote: false,
            },
            ScreenState::ResolveConflictRemote(key) => ScreenEffect::ResolveConflict {
                key,
                use_remote: true,
            },
            ScreenState::Mutate(mutation) => ScreenEffect::Mutate(mutation),
            ScreenState::OpenInBrowser(url) => ScreenEffect::OpenInBrowser(url),
            ScreenState::RunSearch(jql) => ScreenEffect::RunSearch(jql),
            ScreenState::SyncNow => ScreenEffect::Sync(SyncControl::Now),
            ScreenState::SyncPause => ScreenEffect::Sync(SyncControl::Pause),
            ScreenState::SyncRetry => ScreenEffect::Sync(SyncControl::Retry),
            ScreenState::SyncResume => ScreenEffect::Sync(SyncControl::Resume),
            _ => ScreenEffect::None,
        }
    }
}
