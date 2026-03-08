use crate::data::model::IssueSummary;

use super::{comments::extract_comments, custom_fields::extract_custom_fields};

pub(super) fn map_issue(issue: &gouqi::Issue, sprint_id: Option<i64>) -> IssueSummary {
    let key = issue.key.to_string();
    let project_key = key.split('-').next().map(str::to_string);

    let resolution = issue
        .resolution()
        .and_then(|_| issue.fields.get("resolution"))
        .and_then(|resolution| resolution.get("name"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);
    let (time_estimate, time_spent, time_remaining) = match issue.timetracking() {
        Some(tt) => (tt.original_estimate, tt.time_spent, tt.remaining_estimate),
        None => (None, None, None),
    };

    IssueSummary {
        key: key.clone(),
        summary: issue.summary().unwrap_or_default(),
        epic: None,
        status: issue
            .status()
            .map(|status| status.name.to_uppercase())
            .unwrap_or_else(|| "TODO".to_string()),
        issue_type: issue
            .issue_type()
            .map(|issue_type| issue_type.name)
            .unwrap_or_else(|| "Task".to_string()),
        assignee: issue
            .assignee()
            .map(|user| user.display_name)
            .unwrap_or_else(|| "Unassigned".to_string()),
        priority: issue
            .priority()
            .map(|priority| priority.name)
            .unwrap_or_else(|| "Medium".to_string()),
        story_points: None,
        project_key,
        sprint_id,
        updated_at: offset_datetime_to_system_time(issue.updated()),
        comments: extract_comments(issue, &key),
        dirty: false,
        conflict: false,
        remote_snapshot: None,
        description: issue.description(),
        reporter: issue.reporter().map(|user| user.display_name),
        creator: issue.creator().map(|user| user.display_name),
        created_at: offset_datetime_to_system_time(issue.created()),
        resolution_date: offset_datetime_to_system_time(issue.resolution_date()),
        resolution,
        labels: issue.labels(),
        fix_versions: issue
            .fix_versions()
            .into_iter()
            .map(|version| version.name)
            .collect(),
        parent_key: issue.parent().map(|parent| parent.key),
        environment: issue.environment(),
        time_estimate,
        time_spent,
        time_remaining,
        custom_fields: extract_custom_fields(issue),
    }
}

fn offset_datetime_to_system_time(
    value: Option<time::OffsetDateTime>,
) -> Option<std::time::SystemTime> {
    value.map(|ts| {
        std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts.unix_timestamp() as u64)
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::map_issue;

    #[test]
    fn map_issue_should_extract_domain_fields_from_jira_issue() {
        let issue: gouqi::Issue = serde_json::from_value(json!({
            "self": "https://jira.example/issues/1",
            "key": "TUIJI-42",
            "id": "42",
            "fields": {
                "summary": "Refactor repository layer",
                "status": {
                    "description": "",
                    "iconUrl": "https://jira.example/icon/status",
                    "id": "3",
                    "name": "In Progress",
                    "self": "https://jira.example/status/3"
                },
                "issuetype": {
                    "description": "",
                    "iconUrl": "https://jira.example/icon/type",
                    "id": "10001",
                    "name": "Story",
                    "self": "https://jira.example/type/10001",
                    "subtask": false
                },
                "assignee": {
                    "accountId": "acc-1",
                    "active": true,
                    "avatarUrls": null,
                    "displayName": "Bob",
                    "emailAddress": null,
                    "key": null,
                    "name": null,
                    "self": "https://jira.example/user/bob",
                    "timeZone": null
                },
                "priority": {
                    "iconUrl": "https://jira.example/icon/prio",
                    "id": "2",
                    "name": "High",
                    "self": "https://jira.example/priority/2"
                },
                "updated": "2026-01-01T12:00:00.000+0000",
                "description": "Detailed description",
                "reporter": {
                    "accountId": "acc-2",
                    "active": true,
                    "avatarUrls": null,
                    "displayName": "Alice",
                    "emailAddress": null,
                    "key": null,
                    "name": null,
                    "self": "https://jira.example/user/alice",
                    "timeZone": null
                },
                "creator": {
                    "accountId": "acc-3",
                    "active": true,
                    "avatarUrls": null,
                    "displayName": "Carol",
                    "emailAddress": null,
                    "key": null,
                    "name": null,
                    "self": "https://jira.example/user/carol",
                    "timeZone": null
                },
                "created": "2026-01-01T10:00:00.000+0000",
                "resolutiondate": "2026-01-01T13:00:00.000+0000",
                "resolution": { "name": "Done" },
                "labels": ["backend", "sync"],
                "fixVersions": [{
                    "archived": false,
                    "id": "200",
                    "name": "1.0.0",
                    "projectId": 1,
                    "released": false,
                    "self": "https://jira.example/version/200"
                }],
                "parent": {
                    "self": "https://jira.example/issues/10",
                    "key": "TUIJI-10",
                    "id": "10",
                    "fields": {}
                },
                "environment": "prod",
                "timetracking": {
                    "originalEstimate": "2h",
                    "originalEstimateSeconds": 7200,
                    "remainingEstimate": "1h",
                    "remainingEstimateSeconds": 3600,
                    "timeSpent": "1h",
                    "timeSpentSeconds": 3600
                },
                "comment": {
                    "comments": [{
                        "id": "c1",
                        "author": { "displayName": "Alice" },
                        "body": "Looks good",
                        "created": "2026-01-01T12:30:00.000+0000",
                        "updated": "2026-01-01T12:45:00.000+0000"
                    }]
                },
                "customfield_10002": 5
            }
        }))
        .unwrap();

        let mapped = map_issue(&issue, Some(99));

        assert_eq!(mapped.key, "TUIJI-42");
        assert_eq!(mapped.project_key.as_deref(), Some("TUIJI"));
        assert_eq!(mapped.status, "IN PROGRESS");
        assert_eq!(mapped.issue_type, "Story");
        assert_eq!(mapped.assignee, "Bob");
        assert_eq!(mapped.priority, "High");
        assert_eq!(mapped.reporter.as_deref(), Some("Alice"));
        assert_eq!(mapped.creator.as_deref(), Some("Carol"));
        assert_eq!(mapped.resolution.as_deref(), Some("Done"));
        assert_eq!(mapped.parent_key.as_deref(), Some("TUIJI-10"));
        assert_eq!(mapped.labels, vec!["backend", "sync"]);
        assert_eq!(mapped.fix_versions, vec!["1.0.0"]);
        assert_eq!(mapped.time_estimate.as_deref(), Some("2h"));
        assert_eq!(mapped.time_spent.as_deref(), Some("1h"));
        assert_eq!(mapped.time_remaining.as_deref(), Some("1h"));
        assert_eq!(mapped.sprint_id, Some(99));
        assert_eq!(mapped.comments.len(), 1);
        assert!(
            mapped
                .custom_fields
                .as_deref()
                .is_some_and(|json| json.contains("customfield_10002"))
        );
        assert!(mapped.updated_at.is_some());
        assert!(mapped.created_at.is_some());
        assert!(mapped.resolution_date.is_some());
    }
}
