use crate::data::{
    CommentSnapshot, DiffEntry, IssueSnapshot, IssueSummary, diff_comment, diff_issue,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingResolve {
    Local,
    Remote,
}

pub struct CommentDiff {
    pub id: String,
    pub diffs: Vec<DiffEntry>,
}

pub struct ConflictsState {
    issues: Vec<IssueSummary>,
    selected_index: usize,
    issue_diffs: Vec<DiffEntry>,
    comment_diffs: Vec<CommentDiff>,
    pending_resolve: Option<PendingResolve>,
}

impl ConflictsState {
    pub fn new(issues: Vec<IssueSummary>) -> Self {
        let mut state = Self {
            issues,
            selected_index: 0,
            issue_diffs: Vec::new(),
            comment_diffs: Vec::new(),
            pending_resolve: None,
        };
        state.rebuild_diffs();
        state
    }

    pub fn issues(&self) -> &[IssueSummary] {
        &self.issues
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn issue_diffs(&self) -> &[DiffEntry] {
        &self.issue_diffs
    }

    pub fn comment_diffs(&self) -> &[CommentDiff] {
        &self.comment_diffs
    }

    pub fn selected_issue(&self) -> Option<&IssueSummary> {
        self.issues.get(self.selected_index)
    }

    pub fn selected_issue_key(&self) -> Option<&str> {
        self.selected_issue().map(|issue| issue.key.as_str())
    }

    pub fn pending_resolve(&self) -> Option<PendingResolve> {
        self.pending_resolve
    }

    pub fn request_resolve(&mut self, choice: PendingResolve) {
        self.pending_resolve = Some(choice);
    }

    pub fn clear_pending_resolve(&mut self) {
        self.pending_resolve = None;
    }

    pub fn move_up(&mut self, n: usize) {
        if self.issues.is_empty() {
            return;
        }
        let step = n.max(1);
        self.selected_index = self.selected_index.saturating_sub(step);
        self.rebuild_diffs();
    }

    pub fn move_down(&mut self, n: usize) {
        if self.issues.is_empty() {
            return;
        }
        let step = n.max(1);
        self.selected_index = (self.selected_index + step).min(self.issues.len() - 1);
        self.rebuild_diffs();
    }

    pub fn move_top(&mut self) {
        if !self.issues.is_empty() {
            self.selected_index = 0;
            self.rebuild_diffs();
        }
    }

    pub fn move_bottom(&mut self) {
        if !self.issues.is_empty() {
            self.selected_index = self.issues.len() - 1;
            self.rebuild_diffs();
        }
    }

    pub fn set_issues(&mut self, issues: Vec<IssueSummary>) {
        self.issues = issues;
        if self.issues.is_empty() {
            self.selected_index = 0;
        } else if self.selected_index >= self.issues.len() {
            self.selected_index = self.issues.len() - 1;
        }
        self.pending_resolve = None;
        self.rebuild_diffs();
    }

    fn rebuild_diffs(&mut self) {
        self.issue_diffs.clear();
        self.comment_diffs.clear();
        let Some(issue) = self.issues.get(self.selected_index) else {
            return;
        };
        if let Some(snapshot) = issue
            .remote_snapshot
            .as_ref()
            .and_then(|raw| serde_json::from_str::<IssueSnapshot>(raw).ok())
        {
            self.issue_diffs = diff_issue(issue, &snapshot);
        }

        for comment in &issue.comments {
            if !comment.conflict {
                continue;
            }
            let Some(raw) = comment.remote_snapshot.as_ref() else {
                continue;
            };
            let Ok(snapshot) = serde_json::from_str::<CommentSnapshot>(raw) else {
                continue;
            };
            let diffs = diff_comment(comment, &snapshot);
            self.comment_diffs.push(CommentDiff {
                id: comment.id.clone(),
                diffs,
            });
        }
    }
}
