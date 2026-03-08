PRAGMA foreign_keys = ON;

CREATE TABLE sync_state (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  last_full_sync INTEGER,
  last_pull INTEGER,
  last_push INTEGER,
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE sync_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  direction TEXT NOT NULL,
  status TEXT NOT NULL,
  error TEXT,
  profile_id TEXT,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE boards (
  id INTEGER NOT NULL,
  profile_id TEXT NOT NULL,
  name TEXT NOT NULL,
  type_name TEXT,
  location_json TEXT,
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  PRIMARY KEY (id, profile_id)
);

CREATE TABLE board_config (
  board_id INTEGER NOT NULL,
  profile_id TEXT NOT NULL,
  estimation_type TEXT NOT NULL,
  estimation_field_id TEXT NOT NULL,
  raw_json TEXT,
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  PRIMARY KEY (board_id, profile_id),
  FOREIGN KEY (board_id, profile_id) REFERENCES boards(id, profile_id) ON DELETE CASCADE
);

CREATE TABLE board_columns (
  board_id INTEGER NOT NULL,
  profile_id TEXT NOT NULL,
  position INTEGER NOT NULL,
  name TEXT NOT NULL,
  status_ids_json TEXT NOT NULL,
  PRIMARY KEY (board_id, profile_id, position),
  FOREIGN KEY (board_id, profile_id) REFERENCES boards(id, profile_id) ON DELETE CASCADE
);

CREATE TABLE sprints (
  id INTEGER NOT NULL,
  profile_id TEXT NOT NULL,
  board_id INTEGER NOT NULL,
  name TEXT NOT NULL,
  state TEXT,
  start_date INTEGER,
  end_date INTEGER,
  complete_date INTEGER,
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  PRIMARY KEY (id, profile_id),
  FOREIGN KEY (board_id, profile_id) REFERENCES boards(id, profile_id) ON DELETE CASCADE
);

CREATE TABLE issues (
  key TEXT NOT NULL,
  profile_id TEXT NOT NULL,
  summary TEXT NOT NULL,
  status TEXT NOT NULL,
  issue_type TEXT NOT NULL,
  priority TEXT NOT NULL,
  assignee TEXT NOT NULL,
  epic TEXT,
  story_points REAL,
  sprint_id INTEGER,
  project_key TEXT,
  updated_at INTEGER,
  synced_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  raw_json TEXT,
  dirty INTEGER NOT NULL DEFAULT 0,
  conflict INTEGER NOT NULL DEFAULT 0,
  remote_snapshot TEXT,
  PRIMARY KEY (key, profile_id),
  FOREIGN KEY (sprint_id, profile_id) REFERENCES sprints(id, profile_id) ON DELETE SET NULL
);

CREATE TABLE issue_comments (
  id TEXT NOT NULL,
  profile_id TEXT NOT NULL,
  issue_key TEXT NOT NULL,
  author TEXT,
  body TEXT,
  created_at TEXT,
  updated_at TEXT,
  dirty INTEGER NOT NULL DEFAULT 0,
  conflict INTEGER NOT NULL DEFAULT 0,
  remote_snapshot TEXT,
  PRIMARY KEY (id, profile_id),
  FOREIGN KEY (issue_key, profile_id) REFERENCES issues(key, profile_id) ON DELETE CASCADE
);

CREATE TABLE issue_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  issue_key TEXT NOT NULL,
  profile_id TEXT NOT NULL,
  snapshot_at INTEGER NOT NULL,
  summary TEXT,
  status TEXT,
  issue_type TEXT,
  priority TEXT,
  assignee TEXT,
  epic TEXT,
  story_points REAL,
  sprint_id INTEGER,
  raw_json TEXT,
  FOREIGN KEY (issue_key, profile_id) REFERENCES issues(key, profile_id) ON DELETE CASCADE
);

CREATE TABLE outbox (
  id TEXT PRIMARY KEY,
  profile_id TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  change_set TEXT NOT NULL,
  status TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE selected_boards (
  board_id INTEGER NOT NULL,
  profile_id TEXT NOT NULL,
  is_default INTEGER NOT NULL DEFAULT 0,
  selected_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  PRIMARY KEY (board_id, profile_id),
  FOREIGN KEY (board_id, profile_id) REFERENCES boards(id, profile_id) ON DELETE CASCADE
);

CREATE INDEX idx_boards_profile_id ON boards(profile_id);
CREATE INDEX idx_selected_boards_profile_id ON selected_boards(profile_id);
CREATE INDEX idx_board_config_profile_id ON board_config(profile_id);
CREATE INDEX idx_board_columns_profile_id ON board_columns(profile_id);
CREATE INDEX idx_sprints_profile_id ON sprints(profile_id);
CREATE INDEX idx_issues_profile_id ON issues(profile_id);
CREATE INDEX idx_issue_comments_profile_id ON issue_comments(profile_id);
CREATE INDEX idx_issue_comments_issue_key ON issue_comments(profile_id, issue_key);
CREATE INDEX idx_issue_history_profile_id ON issue_history(profile_id);

CREATE INDEX idx_issues_status ON issues(profile_id, status);
CREATE INDEX idx_issues_sprint ON issues(profile_id, sprint_id);
CREATE INDEX idx_issue_history_issue ON issue_history(profile_id, issue_key, snapshot_at);
CREATE INDEX idx_outbox_status ON outbox(profile_id, status, created_at);
CREATE INDEX idx_board_columns_board ON board_columns(profile_id, board_id, position);
CREATE INDEX idx_sprints_board_state ON sprints(profile_id, board_id, state);
CREATE INDEX idx_selected_boards_default ON selected_boards(profile_id, is_default);
CREATE INDEX idx_sync_log_created_at ON sync_log(created_at);
CREATE INDEX idx_issues_conflict ON issues(profile_id, conflict);
CREATE INDEX idx_issues_dirty ON issues(profile_id, dirty);
CREATE INDEX idx_issue_comments_conflict ON issue_comments(profile_id, conflict);
CREATE INDEX idx_issue_comments_dirty ON issue_comments(profile_id, dirty);
