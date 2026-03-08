use crate::data::model::IssueSummary;

use super::{
    helpers::{push_if_diff, push_if_diff_opt, push_if_diff_opt_f64, push_if_diff_opt_i64},
    snapshot::{DiffEntry, IssueSnapshot},
};

pub fn diff_issue(local: &IssueSummary, remote: &IssueSnapshot) -> Vec<DiffEntry> {
    let mut diffs = Vec::new();
    push_if_diff("summary", &local.summary, &remote.summary, &mut diffs);
    push_if_diff("status", &local.status, &remote.status, &mut diffs);
    push_if_diff(
        "issue_type",
        &local.issue_type,
        &remote.issue_type,
        &mut diffs,
    );
    push_if_diff("assignee", &local.assignee, &remote.assignee, &mut diffs);
    push_if_diff("priority", &local.priority, &remote.priority, &mut diffs);
    push_if_diff_opt(
        "epic",
        local.epic.as_deref(),
        remote.epic.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt_f64(
        "story_points",
        local.story_points,
        remote.story_points,
        &mut diffs,
    );
    push_if_diff_opt(
        "project_key",
        local.project_key.as_deref(),
        remote.project_key.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt_i64("sprint_id", local.sprint_id, remote.sprint_id, &mut diffs);
    push_if_diff_opt(
        "description",
        local.description.as_deref(),
        remote.description.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt(
        "reporter",
        local.reporter.as_deref(),
        remote.reporter.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt(
        "creator",
        local.creator.as_deref(),
        remote.creator.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt_i64(
        "created_at",
        local.created_at.map(system_time_to_ts),
        remote.created_at,
        &mut diffs,
    );
    push_if_diff_opt_i64(
        "resolution_date",
        local.resolution_date.map(system_time_to_ts),
        remote.resolution_date,
        &mut diffs,
    );
    push_if_diff_opt(
        "resolution",
        local.resolution.as_deref(),
        remote.resolution.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt(
        "labels",
        serialize_json_array(&local.labels).as_deref(),
        remote.labels.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt(
        "fix_versions",
        serialize_json_array(&local.fix_versions).as_deref(),
        remote.fix_versions.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt(
        "parent_key",
        local.parent_key.as_deref(),
        remote.parent_key.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt(
        "environment",
        local.environment.as_deref(),
        remote.environment.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt(
        "time_estimate",
        local.time_estimate.as_deref(),
        remote.time_estimate.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt(
        "time_spent",
        local.time_spent.as_deref(),
        remote.time_spent.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt(
        "time_remaining",
        local.time_remaining.as_deref(),
        remote.time_remaining.as_deref(),
        &mut diffs,
    );
    push_if_diff_opt(
        "custom_fields",
        local.custom_fields.as_deref(),
        remote.custom_fields.as_deref(),
        &mut diffs,
    );
    diffs
}

fn system_time_to_ts(value: std::time::SystemTime) -> i64 {
    value
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn serialize_json_array(value: &[String]) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        serde_json::to_string(value).ok()
    }
}

#[cfg(test)]
mod tests {
    use crate::data::model::{IssueComment, IssueSummary};

    use super::{IssueSnapshot, diff_issue};

    #[test]
    fn diff_issue_should_include_extended_fields() {
        let local = IssueSummary {
            key: "TUIJI-1".to_string(),
            summary: "Local".to_string(),
            epic: Some("Epic A".to_string()),
            status: "TODO".to_string(),
            issue_type: "Task".to_string(),
            assignee: "Alice".to_string(),
            priority: "High".to_string(),
            story_points: Some(3.0),
            project_key: Some("TUIJI".to_string()),
            sprint_id: Some(7),
            updated_at: None,
            comments: Vec::<IssueComment>::new(),
            dirty: false,
            conflict: false,
            remote_snapshot: None,
            description: Some("Local description".to_string()),
            reporter: Some("Reporter A".to_string()),
            creator: Some("Creator A".to_string()),
            created_at: Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(10)),
            resolution_date: Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(20)),
            resolution: Some("Done".to_string()),
            labels: vec!["backend".to_string()],
            fix_versions: vec!["1.0.0".to_string()],
            parent_key: Some("TUIJI-0".to_string()),
            environment: Some("prod".to_string()),
            time_estimate: Some("2h".to_string()),
            time_spent: Some("1h".to_string()),
            time_remaining: Some("1h".to_string()),
            custom_fields: Some(r#"{"customfield_10002":5}"#.to_string()),
        };
        let mut remote = IssueSnapshot::from(&local);
        remote.description = Some("Remote description".to_string());
        remote.reporter = Some("Reporter B".to_string());
        remote.creator = Some("Creator B".to_string());
        remote.created_at = Some(11);
        remote.resolution_date = Some(21);
        remote.resolution = Some("Won't Do".to_string());
        remote.labels = Some(r#"["frontend"]"#.to_string());
        remote.fix_versions = Some(r#"["2.0.0"]"#.to_string());
        remote.parent_key = Some("TUIJI-9".to_string());
        remote.environment = Some("stage".to_string());
        remote.time_estimate = Some("4h".to_string());
        remote.time_spent = Some("3h".to_string());
        remote.time_remaining = Some("1m".to_string());
        remote.custom_fields = Some(r#"{"customfield_10002":8}"#.to_string());

        let diffs = diff_issue(&local, &remote);
        let fields = diffs.iter().map(|entry| entry.field).collect::<Vec<_>>();

        assert!(fields.contains(&"description"));
        assert!(fields.contains(&"reporter"));
        assert!(fields.contains(&"creator"));
        assert!(fields.contains(&"created_at"));
        assert!(fields.contains(&"resolution_date"));
        assert!(fields.contains(&"resolution"));
        assert!(fields.contains(&"labels"));
        assert!(fields.contains(&"fix_versions"));
        assert!(fields.contains(&"parent_key"));
        assert!(fields.contains(&"environment"));
        assert!(fields.contains(&"time_estimate"));
        assert!(fields.contains(&"time_spent"));
        assert!(fields.contains(&"time_remaining"));
        assert!(fields.contains(&"custom_fields"));
    }
}
