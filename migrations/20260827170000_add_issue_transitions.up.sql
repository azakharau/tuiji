-- Cache of the transitions Jira offered for an issue, so the picker still works
-- without a connection. `status` records the issue status the list was fetched
-- under: Jira only returns transitions available from the current status, so a
-- cached list is stale the moment the status changes.
CREATE TABLE issue_transitions (
  issue_key TEXT NOT NULL,
  profile_id TEXT NOT NULL,
  status TEXT NOT NULL,
  choices_json TEXT NOT NULL,
  fetched_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  PRIMARY KEY (issue_key, profile_id)
);

CREATE INDEX idx_issue_transitions_profile_id ON issue_transitions(profile_id);
