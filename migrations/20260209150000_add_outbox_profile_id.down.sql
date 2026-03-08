PRAGMA foreign_keys = OFF;

ALTER TABLE outbox RENAME TO outbox_new;

CREATE TABLE outbox (
  id TEXT PRIMARY KEY,
  entity_type TEXT NOT NULL,
  entity_id TEXT NOT NULL,
  change_set TEXT NOT NULL,
  status TEXT NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
  updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);

INSERT INTO outbox (
  id,
  entity_type,
  entity_id,
  change_set,
  status,
  attempts,
  last_error,
  created_at,
  updated_at
)
SELECT
  id,
  entity_type,
  entity_id,
  change_set,
  status,
  attempts,
  last_error,
  created_at,
  updated_at
FROM outbox_new;

DROP TABLE outbox_new;

DROP INDEX IF EXISTS idx_outbox_status;
CREATE INDEX idx_outbox_status ON outbox(status, created_at);

PRAGMA foreign_keys = ON;
