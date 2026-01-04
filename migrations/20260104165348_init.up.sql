PRAGMA foreign_keys = ON;

CREATE TABLE sync_state (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  last_full_sync INTEGER,
  last_pull INTEGER,
  last_push INTEGER,
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE boards (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  type_name TEXT,
  location_json TEXT,
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE board_config (
  board_id INTEGER NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
  estimation_type TEXT NOT NULL,
  estimation_field_id TEXT NOT NULL,
  raw_json TEXT,
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  PRIMARY KEY (board_id)
);

CREATE TABLE board_columns (
  board_id INTEGER NOT NULL,
  position INTEGER NOT NULL,
  name TEXT NOT NULL,
  status_ids_json TEXT NOT NULL,
  PRIMARY KEY (board_id, position),
  FOREIGN KEY (board_id) REFERENCES boards(id) ON DELETE CASCADE
);

CREATE TABLE sprints (
  id INTEGER PRIMARY KEY,
  board_id INTEGER NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  state TEXT,
  start_date INTEGER,
  end_date INTEGER,
  complete_date INTEGER,
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE issues (
  key TEXT PRIMARY KEY,
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
  FOREIGN KEY (sprint_id) REFERENCES sprints(id) ON DELETE SET NULL
);

CREATE TABLE issue_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  issue_key TEXT NOT NULL,
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
  FOREIGN KEY (issue_key) REFERENCES issues(key) ON DELETE CASCADE,
  FOREIGN KEY (sprint_id) REFERENCES sprints(id) ON DELETE SET NULL
);

CREATE TABLE outbox (
  id TEXT PRIMARY KEY,
  command_type TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  status TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE TABLE selected_boards (
  board_id INTEGER PRIMARY KEY REFERENCES boards(id) ON DELETE CASCADE,
  is_default INTEGER NOT NULL DEFAULT 0,
  selected_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_issues_sprint ON issues(sprint_id);
CREATE INDEX idx_issue_history_issue ON issue_history(issue_key, snapshot_at);
CREATE INDEX idx_outbox_status ON outbox(status, created_at);
CREATE INDEX idx_board_columns_board ON board_columns(board_id, position);
CREATE INDEX idx_sprints_board_state ON sprints(board_id, state);
CREATE INDEX idx_selected_boards_default ON selected_boards(is_default);
