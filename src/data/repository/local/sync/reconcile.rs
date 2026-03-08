use std::collections::HashMap;

use color_eyre::Result;

use super::*;
use crate::data::{
    CommentSnapshot, IssueComment, IssueSnapshot, diff_comment, diff_issue,
    repository::CommandRepository,
};

#[derive(Debug, PartialEq, Eq)]
enum CommentSyncAction {
    ClearDirty {
        comment_id: String,
    },
    MarkConflict {
        comment_id: String,
        snapshot: String,
    },
}

impl RepositoryHub {
    pub(super) async fn reconcile_pull_issue(&self, issue: &IssueSummary) -> Result<()> {
        let local = self.cache.fetch_issue(&issue.key).await?;
        let remote_snapshot = IssueSnapshot::from(issue);
        let mut has_conflict = false;

        if let Some(local_issue) = &local {
            if local_issue.dirty && !diff_issue(local_issue, &remote_snapshot).is_empty() {
                has_conflict = true;
            }

            let comment_conflict = self
                .reconcile_comment_conflicts(local_issue, &remote_snapshot)
                .await?;
            if has_conflict || comment_conflict {
                let snapshot = serde_json::to_string(&remote_snapshot).unwrap_or_default();
                if !snapshot.is_empty() {
                    self.cache
                        .mark_issue_conflict(&issue.key, &snapshot)
                        .await?;
                }
                return Ok(());
            }

            if local_issue.dirty {
                self.cache.clear_issue_dirty(&issue.key).await?;
            }
        }

        self.cache.upsert_issues(std::slice::from_ref(issue)).await
    }

    async fn reconcile_comment_conflicts(
        &self,
        local_issue: &IssueSummary,
        remote_snapshot: &IssueSnapshot,
    ) -> Result<bool> {
        let actions = comment_sync_actions(&local_issue.comments, &remote_snapshot.comments);
        let comment_conflict = actions
            .iter()
            .any(|action| matches!(action, CommentSyncAction::MarkConflict { .. }));

        for action in actions {
            match action {
                CommentSyncAction::ClearDirty { comment_id } => {
                    self.cache.clear_comment_dirty(&comment_id).await?;
                }
                CommentSyncAction::MarkConflict {
                    comment_id,
                    snapshot,
                } => {
                    if !snapshot.is_empty() {
                        self.cache
                            .mark_comment_conflict(&comment_id, &snapshot)
                            .await?;
                    }
                }
            }
        }

        Ok(comment_conflict)
    }
}

fn comment_sync_actions(
    local_comments: &[IssueComment],
    remote_comments: &[CommentSnapshot],
) -> Vec<CommentSyncAction> {
    let remote_by_id = remote_comments
        .iter()
        .map(|comment| (comment.id.as_str(), comment))
        .collect::<HashMap<_, _>>();
    let mut actions = Vec::new();

    for local_comment in local_comments {
        if !local_comment.dirty {
            continue;
        }

        match remote_by_id.get(local_comment.id.as_str()) {
            Some(remote_comment) => {
                if diff_comment(local_comment, remote_comment).is_empty() {
                    actions.push(CommentSyncAction::ClearDirty {
                        comment_id: local_comment.id.clone(),
                    });
                } else {
                    actions.push(CommentSyncAction::MarkConflict {
                        comment_id: local_comment.id.clone(),
                        snapshot: serde_json::to_string(remote_comment).unwrap_or_default(),
                    });
                }
            }
            None => actions.push(CommentSyncAction::MarkConflict {
                comment_id: local_comment.id.clone(),
                snapshot: serde_json::to_string(&missing_comment_snapshot(local_comment))
                    .unwrap_or_default(),
            }),
        }
    }

    actions
}

fn missing_comment_snapshot(local_comment: &IssueComment) -> CommentSnapshot {
    CommentSnapshot {
        id: local_comment.id.clone(),
        author: String::new(),
        body: String::new(),
        created_at: None,
        updated_at: None,
    }
}

#[cfg(test)]
mod tests {
    use super::{CommentSyncAction, comment_sync_actions};
    use crate::data::{CommentSnapshot, IssueComment};

    #[test]
    fn comment_sync_actions_should_clear_when_remote_matches() {
        let local = vec![IssueComment {
            id: "c1".to_string(),
            issue_key: "TUIJI-1".to_string(),
            author: "Alice".to_string(),
            body: "same".to_string(),
            created_at: Some("2026-01-01".to_string()),
            updated_at: Some("2026-01-02".to_string()),
            dirty: true,
            conflict: false,
            remote_snapshot: None,
        }];
        let remote = vec![CommentSnapshot {
            id: "c1".to_string(),
            author: "Alice".to_string(),
            body: "same".to_string(),
            created_at: Some("2026-01-01".to_string()),
            updated_at: Some("2026-01-02".to_string()),
        }];

        let actions = comment_sync_actions(&local, &remote);

        assert_eq!(
            actions,
            vec![CommentSyncAction::ClearDirty {
                comment_id: "c1".to_string()
            }]
        );
    }

    #[test]
    fn comment_sync_actions_should_mark_conflict_when_remote_differs_or_is_missing() {
        let local = vec![
            IssueComment {
                id: "c1".to_string(),
                issue_key: "TUIJI-1".to_string(),
                author: "Alice".to_string(),
                body: "local".to_string(),
                created_at: None,
                updated_at: None,
                dirty: true,
                conflict: false,
                remote_snapshot: None,
            },
            IssueComment {
                id: "c2".to_string(),
                issue_key: "TUIJI-1".to_string(),
                author: "Bob".to_string(),
                body: "missing".to_string(),
                created_at: None,
                updated_at: None,
                dirty: true,
                conflict: false,
                remote_snapshot: None,
            },
        ];
        let remote = vec![CommentSnapshot {
            id: "c1".to_string(),
            author: "Alice".to_string(),
            body: "remote".to_string(),
            created_at: None,
            updated_at: None,
        }];

        let actions = comment_sync_actions(&local, &remote);

        assert_eq!(actions.len(), 2);
        assert!(matches!(
            &actions[0],
            CommentSyncAction::MarkConflict { comment_id, snapshot }
                if comment_id == "c1" && snapshot.contains("\"body\":\"remote\"")
        ));
        assert!(matches!(
            &actions[1],
            CommentSyncAction::MarkConflict { comment_id, snapshot }
                if comment_id == "c2" && snapshot.contains("\"id\":\"c2\"")
        ));
    }
}
