use serde_json::json;

use super::*;
type SqliteTx<'a> = sqlx::Transaction<'a, sqlx::Sqlite>;

struct SeedBoard<'a> {
    id: i64,
    name: &'a str,
    type_name: &'a str,
    estimation_type: &'a str,
    estimation_field_id: &'a str,
    columns: &'a [(&'a str, &'a [&'a str])],
}

struct SeedSprint<'a> {
    id: i64,
    board_id: i64,
    name: &'a str,
    state: &'a str,
    start_date: i64,
    end_date: i64,
    complete_date: Option<i64>,
}

struct SeedIssue<'a> {
    key: &'a str,
    summary: &'a str,
    status: &'a str,
    issue_type: &'a str,
    priority: &'a str,
    assignee: &'a str,
    epic: Option<&'a str>,
    story_points: Option<f64>,
    sprint_id: Option<i64>,
    project_key: Option<&'a str>,
    updated_at: i64,
    dirty: bool,
    conflict: bool,
    remote_snapshot: Option<String>,
    description: Option<&'a str>,
    reporter: Option<&'a str>,
    creator: Option<&'a str>,
    created_at: Option<i64>,
    resolution_date: Option<i64>,
    resolution: Option<&'a str>,
    labels: &'a [&'a str],
    fix_versions: &'a [&'a str],
    parent_key: Option<&'a str>,
    environment: Option<&'a str>,
    time_estimate: Option<&'a str>,
    time_spent: Option<&'a str>,
    time_remaining: Option<&'a str>,
    custom_fields: Option<String>,
}

struct SeedComment<'a> {
    id: &'a str,
    issue_key: &'a str,
    author: &'a str,
    body: &'a str,
    created_at: Option<&'a str>,
    updated_at: Option<&'a str>,
    dirty: bool,
    conflict: bool,
    remote_snapshot: Option<String>,
}

struct SeedOutbox<'a> {
    id: &'a str,
    entity_type: &'a str,
    entity_id: &'a str,
    change_set: String,
    status: &'a str,
    attempts: i64,
    last_error: Option<&'a str>,
    created_at: i64,
    updated_at: i64,
}

struct SeedSyncLog<'a> {
    direction: &'a str,
    status: &'a str,
    error: Option<&'a str>,
    created_at: i64,
}

impl SqliteRepository {
    pub(super) async fn seed_mock_data_if_empty_impl(&self) -> Result<Option<u64>> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM boards WHERE profile_id = ?")
            .bind(self.profile_id())
            .fetch_one(&self.pool)
            .await?;
        let count: i64 = row.get("count");
        if count > 0 {
            return Ok(None);
        }

        let now = current_ts();
        let default_board_id = 175_u64;
        let mut tx = self.pool.begin().await?;

        for board in demo_boards() {
            insert_board(&mut tx, self.profile_id(), board, now).await?;
        }
        for sprint in demo_sprints(now) {
            insert_sprint(&mut tx, self.profile_id(), &sprint, now).await?;
        }
        for issue in demo_issues(now) {
            insert_issue(&mut tx, self.profile_id(), &issue, now).await?;
            insert_issue_history(&mut tx, self.profile_id(), &issue, issue.updated_at - 600)
                .await?;
            insert_issue_history(&mut tx, self.profile_id(), &issue, issue.updated_at).await?;
        }
        for comment in demo_comments() {
            insert_comment(&mut tx, self.profile_id(), &comment).await?;
        }

        insert_selected_boards(&mut tx, self.profile_id(), now).await?;
        insert_sync_state(&mut tx, now).await?;
        for entry in demo_sync_log(now) {
            insert_sync_log_entry(&mut tx, self.profile_id(), &entry).await?;
        }
        for entry in demo_outbox(now) {
            insert_outbox_entry(&mut tx, self.profile_id(), &entry).await?;
        }

        tx.commit().await?;
        Ok(Some(default_board_id))
    }
}

fn demo_boards() -> &'static [SeedBoard<'static>] {
    &[
        SeedBoard {
            id: 175,
            name: "Demo Board",
            type_name: "scrum",
            estimation_type: "StoryPoints",
            estimation_field_id: "customfield_10002",
            columns: &[
                ("TODO", &["1", "2"]),
                ("IN PROGRESS", &["3"]),
                ("CODE REVIEW", &["5"]),
                ("DONE", &["4"]),
            ],
        },
        SeedBoard {
            id: 176,
            name: "Operations Board",
            type_name: "kanban",
            estimation_type: "DateBased",
            estimation_field_id: "customfield_10016",
            columns: &[("BACKLOG", &["10"]), ("DOING", &["11"]), ("DONE", &["12"])],
        },
        SeedBoard {
            id: 177,
            name: "Support Board",
            type_name: "scrum",
            estimation_type: "StoryPoints",
            estimation_field_id: "customfield_10101",
            columns: &[("NEW", &["20"]), ("TRIAGE", &["21"]), ("SOLVED", &["22"])],
        },
    ]
}

fn demo_sprints(now: i64) -> Vec<SeedSprint<'static>> {
    vec![
        SeedSprint {
            id: 1001,
            board_id: 175,
            name: "Demo Sprint 1",
            state: "active",
            start_date: now - 7 * 24 * 3600,
            end_date: now + 7 * 24 * 3600,
            complete_date: None,
        },
        SeedSprint {
            id: 1000,
            board_id: 175,
            name: "Demo Sprint 0",
            state: "closed",
            start_date: now - 21 * 24 * 3600,
            end_date: now - 14 * 24 * 3600,
            complete_date: Some(now - 13 * 24 * 3600),
        },
        SeedSprint {
            id: 2001,
            board_id: 176,
            name: "Ops Planning",
            state: "future",
            start_date: now + 24 * 3600,
            end_date: now + 14 * 24 * 3600,
            complete_date: None,
        },
        SeedSprint {
            id: 3001,
            board_id: 177,
            name: "Support Sprint",
            state: "active",
            start_date: now - 2 * 24 * 3600,
            end_date: now + 5 * 24 * 3600,
            complete_date: None,
        },
    ]
}

fn demo_issues(now: i64) -> Vec<SeedIssue<'static>> {
    vec![
        SeedIssue {
            key: "DEMO-1",
            summary: "Setup local cache",
            status: "TODO",
            issue_type: "Task",
            priority: "Medium",
            assignee: "Alex",
            epic: Some("Platform"),
            story_points: Some(3.0),
            sprint_id: Some(1001),
            project_key: Some("DEMO"),
            updated_at: now - 3600,
            dirty: false,
            conflict: false,
            remote_snapshot: None,
            description: Some("Create sqlite cache and bootstrap config flow."),
            reporter: Some("Product"),
            creator: Some("Product"),
            created_at: Some(now - 8 * 24 * 3600),
            resolution_date: None,
            resolution: None,
            labels: &["offline", "cache"],
            fix_versions: &["0.1.0"],
            parent_key: None,
            environment: Some("local"),
            time_estimate: Some("2h"),
            time_spent: Some("30m"),
            time_remaining: Some("90m"),
            custom_fields: Some(json!({"customfield_10002": 3}).to_string()),
        },
        SeedIssue {
            key: "DEMO-2",
            summary: "Fix login error",
            status: "IN PROGRESS",
            issue_type: "Bug",
            priority: "High",
            assignee: "Maria",
            epic: Some("Auth"),
            story_points: Some(5.0),
            sprint_id: Some(1001),
            project_key: Some("DEMO"),
            updated_at: now - 2400,
            dirty: false,
            conflict: false,
            remote_snapshot: None,
            description: Some("Investigate token refresh edge cases."),
            reporter: Some("QA"),
            creator: Some("QA"),
            created_at: Some(now - 6 * 24 * 3600),
            resolution_date: None,
            resolution: None,
            labels: &["bug", "auth"],
            fix_versions: &["0.1.0"],
            parent_key: None,
            environment: Some("staging"),
            time_estimate: Some("4h"),
            time_spent: Some("2h"),
            time_remaining: Some("2h"),
            custom_fields: Some(
                json!({"customfield_10002": 5, "customfield_10010": "Auth"}).to_string(),
            ),
        },
        SeedIssue {
            key: "DEMO-3",
            summary: "Review sync conflict strategy",
            status: "CODE REVIEW",
            issue_type: "Story",
            priority: "Medium",
            assignee: "Unassigned",
            epic: Some("Sync"),
            story_points: None,
            sprint_id: Some(1001),
            project_key: Some("DEMO"),
            updated_at: now - 1800,
            dirty: false,
            conflict: false,
            remote_snapshot: None,
            description: Some("Validate local-first merge behavior before release."),
            reporter: Some("Tech Lead"),
            creator: Some("Tech Lead"),
            created_at: Some(now - 5 * 24 * 3600),
            resolution_date: None,
            resolution: None,
            labels: &["sync"],
            fix_versions: &[],
            parent_key: None,
            environment: None,
            time_estimate: None,
            time_spent: None,
            time_remaining: None,
            custom_fields: None,
        },
        SeedIssue {
            key: "DEMO-4",
            summary: "Ship MVP",
            status: "DONE",
            issue_type: "Story",
            priority: "Low",
            assignee: "Kate",
            epic: Some("Launch"),
            story_points: Some(8.0),
            sprint_id: Some(1001),
            project_key: Some("DEMO"),
            updated_at: now - 7200,
            dirty: false,
            conflict: false,
            remote_snapshot: None,
            description: Some("Release candidate shipped to users."),
            reporter: Some("CEO"),
            creator: Some("CEO"),
            created_at: Some(now - 12 * 24 * 3600),
            resolution_date: Some(now - 2 * 24 * 3600),
            resolution: Some("Done"),
            labels: &["release"],
            fix_versions: &["0.1.0"],
            parent_key: None,
            environment: Some("prod"),
            time_estimate: Some("1d"),
            time_spent: Some("1d"),
            time_remaining: Some("0m"),
            custom_fields: Some(json!({"customfield_10031": true}).to_string()),
        },
        SeedIssue {
            key: "DEMO-5",
            summary: "Implement issue editor subtask",
            status: "TODO",
            issue_type: "Subtask",
            priority: "Lowest",
            assignee: "Sam",
            epic: Some("Editor"),
            story_points: Some(1.0),
            sprint_id: Some(1001),
            project_key: Some("DEMO"),
            updated_at: now - 900,
            dirty: true,
            conflict: false,
            remote_snapshot: None,
            description: Some("Subtask with local dirty state and pending outbox."),
            reporter: Some("PM"),
            creator: Some("PM"),
            created_at: Some(now - 2 * 24 * 3600),
            resolution_date: None,
            resolution: None,
            labels: &["editor", "subtask"],
            fix_versions: &[],
            parent_key: Some("DEMO-3"),
            environment: Some("local"),
            time_estimate: Some("45m"),
            time_spent: Some("10m"),
            time_remaining: Some("35m"),
            custom_fields: Some(json!({"customfield_10002": 1}).to_string()),
        },
        SeedIssue {
            key: "DEMO-6",
            summary: "Resolve remote/local summary conflict",
            status: "IN PROGRESS",
            issue_type: "Task",
            priority: "Critical",
            assignee: "Lee",
            epic: Some("Sync"),
            story_points: Some(2.0),
            sprint_id: Some(1001),
            project_key: Some("DEMO"),
            updated_at: now - 600,
            dirty: false,
            conflict: true,
            remote_snapshot: Some(issue_conflict_snapshot(now)),
            description: Some("Local copy differs from remote snapshot."),
            reporter: Some("Ops"),
            creator: Some("Ops"),
            created_at: Some(now - 4 * 24 * 3600),
            resolution_date: None,
            resolution: None,
            labels: &["sync", "conflict"],
            fix_versions: &["0.2.0"],
            parent_key: None,
            environment: Some("staging"),
            time_estimate: Some("1h"),
            time_spent: Some("15m"),
            time_remaining: Some("45m"),
            custom_fields: Some(json!({"customfield_10002": 2}).to_string()),
        },
        SeedIssue {
            key: "DEMO-7",
            summary: "Investigate comment conflict",
            status: "TODO",
            issue_type: "Task",
            priority: "Medium",
            assignee: "Nina",
            epic: Some("Sync"),
            story_points: None,
            sprint_id: Some(1001),
            project_key: Some("DEMO"),
            updated_at: now - 1200,
            dirty: false,
            conflict: false,
            remote_snapshot: None,
            description: Some("Issue itself is clean, comment is conflicting."),
            reporter: Some("Support"),
            creator: Some("Support"),
            created_at: Some(now - 24 * 3600),
            resolution_date: None,
            resolution: None,
            labels: &["comments"],
            fix_versions: &[],
            parent_key: None,
            environment: None,
            time_estimate: None,
            time_spent: None,
            time_remaining: None,
            custom_fields: None,
        },
        SeedIssue {
            key: "DEMO-8",
            summary: "Keep unscheduled backlog issue",
            status: "TODO",
            issue_type: "Task",
            priority: "Low",
            assignee: "Backlog Bot",
            epic: None,
            story_points: None,
            sprint_id: None,
            project_key: Some("DEMO"),
            updated_at: now - 300,
            dirty: false,
            conflict: false,
            remote_snapshot: None,
            description: Some("Backlog-only item to verify sprint filtering."),
            reporter: Some("PM"),
            creator: Some("PM"),
            created_at: Some(now - 3 * 24 * 3600),
            resolution_date: None,
            resolution: None,
            labels: &["backlog"],
            fix_versions: &[],
            parent_key: None,
            environment: None,
            time_estimate: None,
            time_spent: None,
            time_remaining: None,
            custom_fields: None,
        },
        SeedIssue {
            key: "DEMO-9",
            summary: "Closed sprint carry-over",
            status: "DONE",
            issue_type: "Task",
            priority: "Medium",
            assignee: "Alex",
            epic: Some("Platform"),
            story_points: Some(2.0),
            sprint_id: Some(1000),
            project_key: Some("DEMO"),
            updated_at: now - 15 * 24 * 3600,
            dirty: false,
            conflict: false,
            remote_snapshot: None,
            description: Some("Closed sprint history fixture."),
            reporter: Some("Product"),
            creator: Some("Product"),
            created_at: Some(now - 20 * 24 * 3600),
            resolution_date: Some(now - 14 * 24 * 3600),
            resolution: Some("Done"),
            labels: &["history"],
            fix_versions: &["0.0.9"],
            parent_key: None,
            environment: Some("local"),
            time_estimate: Some("1h"),
            time_spent: Some("1h"),
            time_remaining: Some("0m"),
            custom_fields: None,
        },
        SeedIssue {
            key: "OPS-1",
            summary: "Prepare ops board backlog",
            status: "BACKLOG",
            issue_type: "Task",
            priority: "Medium",
            assignee: "Ops Bot",
            epic: None,
            story_points: None,
            sprint_id: Some(2001),
            project_key: Some("OPS"),
            updated_at: now - 4000,
            dirty: false,
            conflict: false,
            remote_snapshot: None,
            description: Some("Secondary board entry for board selection testing."),
            reporter: Some("Ops"),
            creator: Some("Ops"),
            created_at: Some(now - 9 * 24 * 3600),
            resolution_date: None,
            resolution: None,
            labels: &["ops"],
            fix_versions: &[],
            parent_key: None,
            environment: Some("infra"),
            time_estimate: Some("3h"),
            time_spent: None,
            time_remaining: Some("3h"),
            custom_fields: None,
        },
        SeedIssue {
            key: "SUP-1",
            summary: "Answer top support tickets",
            status: "TRIAGE",
            issue_type: "Task",
            priority: "High",
            assignee: "Dana",
            epic: None,
            story_points: Some(2.0),
            sprint_id: Some(3001),
            project_key: None,
            updated_at: now - 180,
            dirty: false,
            conflict: false,
            remote_snapshot: None,
            description: Some("Project key intentionally omitted to exercise fallback logic."),
            reporter: Some("Support Lead"),
            creator: Some("Support Lead"),
            created_at: Some(now - 36 * 3600),
            resolution_date: None,
            resolution: None,
            labels: &["support", "fallback"],
            fix_versions: &[],
            parent_key: None,
            environment: Some("customer"),
            time_estimate: Some("2h"),
            time_spent: Some("30m"),
            time_remaining: Some("90m"),
            custom_fields: Some(json!({"customfield_10101": 2}).to_string()),
        },
    ]
}

fn demo_comments() -> Vec<SeedComment<'static>> {
    vec![
        SeedComment {
            id: "c-demo-2-1",
            issue_key: "DEMO-2",
            author: "Alice",
            body: "Need better logs around refresh token flow.",
            created_at: Some("2026-03-01T10:00:00.000+0000"),
            updated_at: Some("2026-03-01T11:00:00.000+0000"),
            dirty: false,
            conflict: false,
            remote_snapshot: None,
        },
        SeedComment {
            id: "c-demo-2-2",
            issue_key: "DEMO-2",
            author: "Maria",
            body: "Added trace logs in staging.",
            created_at: Some("2026-03-01T12:00:00.000+0000"),
            updated_at: Some("2026-03-01T12:10:00.000+0000"),
            dirty: false,
            conflict: false,
            remote_snapshot: None,
        },
        SeedComment {
            id: "c-demo-5-1",
            issue_key: "DEMO-5",
            author: "Sam",
            body: "Local draft comment waiting for push.",
            created_at: Some("2026-03-02T09:00:00.000+0000"),
            updated_at: Some("2026-03-02T09:10:00.000+0000"),
            dirty: true,
            conflict: false,
            remote_snapshot: None,
        },
        SeedComment {
            id: "c-demo-7-1",
            issue_key: "DEMO-7",
            author: "Nina",
            body: "Local comment body",
            created_at: Some("2026-03-03T12:00:00.000+0000"),
            updated_at: Some("2026-03-03T12:30:00.000+0000"),
            dirty: false,
            conflict: true,
            remote_snapshot: Some(comment_conflict_snapshot()),
        },
        SeedComment {
            id: "c-sup-1-1",
            issue_key: "SUP-1",
            author: "Dana",
            body: "Escalated customer incident to backend.",
            created_at: Some("2026-03-04T08:00:00.000+0000"),
            updated_at: Some("2026-03-04T08:05:00.000+0000"),
            dirty: false,
            conflict: false,
            remote_snapshot: None,
        },
    ]
}

fn demo_outbox(now: i64) -> Vec<SeedOutbox<'static>> {
    vec![
        SeedOutbox {
            id: "outbox-demo-5-issue",
            entity_type: "issue",
            entity_id: "DEMO-5",
            change_set: json!({"summary": "Implement issue editor subtask"}).to_string(),
            status: "pending",
            attempts: 0,
            last_error: None,
            created_at: now - 500,
            updated_at: now - 500,
        },
        SeedOutbox {
            id: "outbox-demo-5-comment",
            entity_type: "comment",
            entity_id: "c-demo-5-1",
            change_set: json!({"body": "Local draft comment waiting for push."}).to_string(),
            status: "pending",
            attempts: 0,
            last_error: None,
            created_at: now - 450,
            updated_at: now - 450,
        },
        SeedOutbox {
            id: "outbox-demo-6-conflict",
            entity_type: "issue",
            entity_id: "DEMO-6",
            change_set: json!({"summary": "Resolve remote/local summary conflict"}).to_string(),
            status: "conflict",
            attempts: 1,
            last_error: Some("remote changed while local edit was pending"),
            created_at: now - 420,
            updated_at: now - 400,
        },
        SeedOutbox {
            id: "outbox-demo-7-comment",
            entity_type: "comment",
            entity_id: "c-demo-7-1",
            change_set: json!({"body": "Local comment body"}).to_string(),
            status: "failed",
            attempts: 2,
            last_error: Some("jira returned 409 conflict"),
            created_at: now - 390,
            updated_at: now - 350,
        },
        SeedOutbox {
            id: "outbox-demo-4-done",
            entity_type: "issue",
            entity_id: "DEMO-4",
            change_set: json!({"resolution": "Done"}).to_string(),
            status: "done",
            attempts: 1,
            last_error: None,
            created_at: now - 800,
            updated_at: now - 760,
        },
        SeedOutbox {
            id: "outbox-ops-1-processing",
            entity_type: "issue",
            entity_id: "OPS-1",
            change_set: json!({"status": "DOING"}).to_string(),
            status: "processing",
            attempts: 1,
            last_error: None,
            created_at: now - 200,
            updated_at: now - 180,
        },
    ]
}

fn demo_sync_log(now: i64) -> Vec<SeedSyncLog<'static>> {
    vec![
        SeedSyncLog {
            direction: "pull",
            status: "success",
            error: None,
            created_at: now - 900,
        },
        SeedSyncLog {
            direction: "push",
            status: "failed",
            error: Some("jira returned 409 conflict"),
            created_at: now - 600,
        },
        SeedSyncLog {
            direction: "push",
            status: "success",
            error: None,
            created_at: now - 300,
        },
        SeedSyncLog {
            direction: "pull",
            status: "success",
            error: None,
            created_at: now - 60,
        },
    ]
}

fn issue_conflict_snapshot(now: i64) -> String {
    json!({
        "key": "DEMO-6",
        "summary": "Remote changed summary",
        "status": "DONE",
        "issue_type": "Task",
        "assignee": "Lee",
        "priority": "Critical",
        "epic": "Sync",
        "story_points": 2.0,
        "project_key": "DEMO",
        "sprint_id": 1001,
        "comments": [{
            "id": "c-demo-6-remote",
            "author": "Ops",
            "body": "Remote comment added during sync.",
            "created_at": "2026-03-03T07:00:00.000+0000",
            "updated_at": "2026-03-03T07:00:00.000+0000"
        }],
        "description": "Remote description",
        "reporter": "Ops",
        "creator": "Ops",
        "created_at": now - 3 * 24 * 3600,
        "resolution_date": now - 120,
        "resolution": "Done",
        "labels": "[\"sync\",\"conflict\",\"remote\"]",
        "fix_versions": "[\"0.2.0\",\"0.2.1\"]",
        "parent_key": null,
        "environment": "staging",
        "time_estimate": "1h",
        "time_spent": "45m",
        "time_remaining": "15m",
        "custom_fields": "{\"customfield_10002\":2,\"customfield_10031\":false}"
    })
    .to_string()
}

fn comment_conflict_snapshot() -> String {
    json!({
        "id": "c-demo-7-1",
        "author": "Nina",
        "body": "Remote comment body",
        "created_at": "2026-03-03T12:00:00.000+0000",
        "updated_at": "2026-03-03T13:00:00.000+0000"
    })
    .to_string()
}

fn serialize_seed_array(values: &[&str]) -> Option<String> {
    if values.is_empty() {
        None
    } else {
        serde_json::to_string(values).ok()
    }
}

async fn insert_board(
    tx: &mut SqliteTx<'_>,
    profile_id: &str,
    board: &SeedBoard<'_>,
    now: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO boards (id, name, type_name, location_json, updated_at, profile_id)
        VALUES (?, ?, ?, NULL, ?, ?)
        "#,
    )
    .bind(board.id)
    .bind(board.name)
    .bind(board.type_name)
    .bind(now)
    .bind(profile_id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO board_config (board_id, estimation_type, estimation_field_id, raw_json, updated_at, profile_id)
        VALUES (?, ?, ?, NULL, ?, ?)
        "#,
    )
    .bind(board.id)
    .bind(board.estimation_type)
    .bind(board.estimation_field_id)
    .bind(now)
    .bind(profile_id)
    .execute(&mut **tx)
    .await?;

    for (position, (name, status_ids)) in board.columns.iter().enumerate() {
        sqlx::query(
            r#"
            INSERT INTO board_columns (board_id, profile_id, position, name, status_ids_json)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(board.id)
        .bind(profile_id)
        .bind(position as i64)
        .bind(*name)
        .bind(serde_json::to_string(status_ids).unwrap_or_else(|_| "[]".to_string()))
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn insert_sprint(
    tx: &mut SqliteTx<'_>,
    profile_id: &str,
    sprint: &SeedSprint<'_>,
    now: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO sprints (
            id,
            profile_id,
            board_id,
            name,
            state,
            start_date,
            end_date,
            complete_date,
            updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(sprint.id)
    .bind(profile_id)
    .bind(sprint.board_id)
    .bind(sprint.name)
    .bind(sprint.state)
    .bind(sprint.start_date)
    .bind(sprint.end_date)
    .bind(sprint.complete_date)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn insert_issue(
    tx: &mut SqliteTx<'_>,
    profile_id: &str,
    issue: &SeedIssue<'_>,
    now: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO issues (
            key,
            profile_id,
            summary,
            status,
            issue_type,
            priority,
            assignee,
            epic,
            story_points,
            sprint_id,
            project_key,
            updated_at,
            synced_at,
            raw_json,
            dirty,
            conflict,
            remote_snapshot,
            description,
            reporter,
            creator,
            created_at,
            resolution_date,
            resolution,
            labels,
            fix_versions,
            parent_key,
            environment,
            time_estimate,
            time_spent,
            time_remaining,
            custom_fields
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(issue.key)
    .bind(profile_id)
    .bind(issue.summary)
    .bind(issue.status)
    .bind(issue.issue_type)
    .bind(issue.priority)
    .bind(issue.assignee)
    .bind(issue.epic)
    .bind(issue.story_points)
    .bind(issue.sprint_id)
    .bind(issue.project_key)
    .bind(issue.updated_at)
    .bind(now)
    .bind(if issue.dirty { 1 } else { 0 })
    .bind(if issue.conflict { 1 } else { 0 })
    .bind(&issue.remote_snapshot)
    .bind(issue.description)
    .bind(issue.reporter)
    .bind(issue.creator)
    .bind(issue.created_at)
    .bind(issue.resolution_date)
    .bind(issue.resolution)
    .bind(serialize_seed_array(issue.labels))
    .bind(serialize_seed_array(issue.fix_versions))
    .bind(issue.parent_key)
    .bind(issue.environment)
    .bind(issue.time_estimate)
    .bind(issue.time_spent)
    .bind(issue.time_remaining)
    .bind(&issue.custom_fields)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn insert_issue_history(
    tx: &mut SqliteTx<'_>,
    profile_id: &str,
    issue: &SeedIssue<'_>,
    snapshot_at: i64,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO issue_history (
            issue_key,
            profile_id,
            snapshot_at,
            summary,
            status,
            issue_type,
            priority,
            assignee,
            epic,
            story_points,
            sprint_id,
            raw_json,
            description,
            reporter,
            creator,
            created_at,
            resolution_date,
            resolution,
            labels,
            fix_versions,
            parent_key,
            environment,
            time_estimate,
            time_spent,
            time_remaining,
            custom_fields
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(issue.key)
    .bind(profile_id)
    .bind(snapshot_at)
    .bind(issue.summary)
    .bind(issue.status)
    .bind(issue.issue_type)
    .bind(issue.priority)
    .bind(issue.assignee)
    .bind(issue.epic)
    .bind(issue.story_points)
    .bind(issue.sprint_id)
    .bind(issue.description)
    .bind(issue.reporter)
    .bind(issue.creator)
    .bind(issue.created_at)
    .bind(issue.resolution_date)
    .bind(issue.resolution)
    .bind(serialize_seed_array(issue.labels))
    .bind(serialize_seed_array(issue.fix_versions))
    .bind(issue.parent_key)
    .bind(issue.environment)
    .bind(issue.time_estimate)
    .bind(issue.time_spent)
    .bind(issue.time_remaining)
    .bind(&issue.custom_fields)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn insert_comment(
    tx: &mut SqliteTx<'_>,
    profile_id: &str,
    comment: &SeedComment<'_>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO issue_comments (
            id,
            profile_id,
            issue_key,
            author,
            body,
            created_at,
            updated_at,
            dirty,
            conflict,
            remote_snapshot
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(comment.id)
    .bind(profile_id)
    .bind(comment.issue_key)
    .bind(comment.author)
    .bind(comment.body)
    .bind(comment.created_at)
    .bind(comment.updated_at)
    .bind(if comment.dirty { 1 } else { 0 })
    .bind(if comment.conflict { 1 } else { 0 })
    .bind(&comment.remote_snapshot)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn insert_selected_boards(tx: &mut SqliteTx<'_>, profile_id: &str, now: i64) -> Result<()> {
    for (board_id, is_default, selected_at) in
        [(175_i64, 1_i64, now - 30), (176_i64, 0_i64, now - 10)]
    {
        sqlx::query(
            r#"
            INSERT INTO selected_boards (board_id, profile_id, is_default, selected_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(board_id)
        .bind(profile_id)
        .bind(is_default)
        .bind(selected_at)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn insert_sync_state(tx: &mut SqliteTx<'_>, now: i64) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO sync_state (id, last_full_sync, last_pull, last_push, updated_at)
        VALUES (1, ?, ?, ?, ?)
        "#,
    )
    .bind(now - 3600)
    .bind(now - 300)
    .bind(now - 120)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn insert_sync_log_entry(
    tx: &mut SqliteTx<'_>,
    profile_id: &str,
    entry: &SeedSyncLog<'_>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO sync_log (direction, status, error, profile_id, created_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(entry.direction)
    .bind(entry.status)
    .bind(entry.error)
    .bind(profile_id)
    .bind(entry.created_at)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

async fn insert_outbox_entry(
    tx: &mut SqliteTx<'_>,
    profile_id: &str,
    entry: &SeedOutbox<'_>,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO outbox (
            id,
            profile_id,
            entity_type,
            entity_id,
            change_set,
            status,
            attempts,
            last_error,
            created_at,
            updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(entry.id)
    .bind(profile_id)
    .bind(entry.entity_type)
    .bind(entry.entity_id)
    .bind(&entry.change_set)
    .bind(entry.status)
    .bind(entry.attempts)
    .bind(entry.last_error)
    .bind(entry.created_at)
    .bind(entry.updated_at)
    .execute(&mut **tx)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::QueryRepository;

    fn temp_db_path() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("tuiji-seed-{}.db", uuid::Uuid::now_v7()));
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        path
    }

    #[tokio::test]
    async fn seed_mock_data_should_populate_functional_scenarios() {
        let db_path = temp_db_path();
        let repo = SqliteRepository::connect(
            SqliteRepositoryConfig {
                db_path: db_path.clone(),
            },
            "test-profile".to_string(),
        )
        .await
        .unwrap();

        let seeded_board = repo.seed_mock_data_if_empty().await.unwrap();
        assert_eq!(seeded_board, Some(175));

        let board_ids = repo.selected_board_ids().await.unwrap();
        assert_eq!(repo.default_board_id().await.unwrap(), Some(175));
        assert_eq!(board_ids.len(), 2);
        assert!(board_ids.contains(&175));
        assert!(board_ids.contains(&176));

        let board_cfg = repo.board_config(175).await.unwrap();
        assert_eq!(board_cfg.columns.len(), 4);

        let sprint_issues = repo.current_sprint_issues(175).await.unwrap();
        assert_eq!(sprint_issues.len(), 7);
        assert!(!sprint_issues.iter().any(|issue| issue.key == "DEMO-8"));
        assert!(repo.current_sprint_issues(176).await.unwrap().is_empty());

        let support_issue = repo.fetch_issue("SUP-1").await.unwrap().unwrap();
        assert_eq!(support_issue.project_key.as_deref(), Some("SUP"));

        let conflict_keys = repo
            .list_conflict_issues()
            .await
            .unwrap()
            .into_iter()
            .map(|issue| issue.key)
            .collect::<Vec<_>>();
        assert_eq!(
            conflict_keys,
            vec!["DEMO-6".to_string(), "DEMO-7".to_string()]
        );

        let pending = repo.fetch_pending_outbox(10).await.unwrap();
        assert_eq!(pending.len(), 2);
        assert_eq!(pending[0].entity_id, "DEMO-5");
        assert_eq!(pending[1].entity_id, "c-demo-5-1");

        let sync_log = repo.sync_log(10, SyncLogFilter::All).await.unwrap();
        assert_eq!(sync_log.len(), 4);
        assert_eq!(sync_log[0].direction, "pull");

        let sync_state = repo.sync_state().await.unwrap();
        assert!(sync_state.last_full_sync.is_some());
        assert!(sync_state.last_pull.is_some());
        assert!(sync_state.last_push.is_some());

        let history_row =
            sqlx::query("SELECT COUNT(*) as count FROM issue_history WHERE profile_id = ?")
                .bind("test-profile")
                .fetch_one(repo.pool())
                .await
                .unwrap();
        let history_count: i64 = history_row.get("count");
        assert_eq!(history_count, 22);

        let reseed = repo.seed_mock_data_if_empty().await.unwrap();
        assert_eq!(reseed, None);

        let _ = std::fs::remove_file(db_path);
    }
}
