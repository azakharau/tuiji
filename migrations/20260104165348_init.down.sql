PRAGMA foreign_keys = ON;

DROP INDEX IF EXISTS idx_sprints_board_state;
DROP INDEX IF EXISTS idx_board_columns_board;
DROP INDEX IF EXISTS idx_outbox_status;
DROP INDEX IF EXISTS idx_issue_history_issue;
DROP INDEX IF EXISTS idx_issues_sprint;
DROP INDEX IF EXISTS idx_issues_status;
DROP INDEX IF EXISTS idx_selected_boards_default;

DROP TABLE IF EXISTS selected_boards;
DROP TABLE IF EXISTS outbox;
DROP TABLE IF EXISTS issue_history;
DROP TABLE IF EXISTS issues;
DROP TABLE IF EXISTS sprints;
DROP TABLE IF EXISTS board_columns;
DROP TABLE IF EXISTS board_config;
DROP TABLE IF EXISTS boards;
DROP TABLE IF EXISTS sync_state;
