PRAGMA foreign_keys = OFF;

DROP TABLE IF EXISTS selected_boards;
DROP TABLE IF EXISTS outbox;
DROP TABLE IF EXISTS issue_history;
DROP TABLE IF EXISTS issue_comments;
DROP TABLE IF EXISTS issues;
DROP TABLE IF EXISTS sprints;
DROP TABLE IF EXISTS board_columns;
DROP TABLE IF EXISTS board_config;
DROP TABLE IF EXISTS boards;
DROP TABLE IF EXISTS sync_log;
DROP TABLE IF EXISTS sync_state;

PRAGMA foreign_keys = ON;
