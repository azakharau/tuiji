use crate::data::model::IssueComment;

use super::{
    helpers::{push_if_diff, push_if_diff_opt},
    snapshot::{CommentSnapshot, DiffEntry},
};

pub fn diff_comment(local: &IssueComment, remote: &CommentSnapshot) -> Vec<DiffEntry> {
    let mut diffs = Vec::new();
    push_if_diff("author", &local.author, &remote.author, &mut diffs);
    push_if_diff("body", &local.body, &remote.body, &mut diffs);
    push_if_diff_opt(
        "created_at",
        local.created_at.as_deref(),
        remote.created_at.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt(
        "updated_at",
        local.updated_at.as_deref(),
        remote.updated_at.as_deref(),
        &mut diffs,
    );
    diffs
}
