PRAGMA foreign_keys = OFF;

ALTER TABLE outbox RENAME TO outbox_old;

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
)
SELECT
  id,
  COALESCE(
    (SELECT profile_id FROM issues WHERE key = outbox_old.entity_id LIMIT 1),
    (SELECT profile_id FROM issue_comments WHERE id = outbox_old.entity_id LIMIT 1),
    'default'
  ) AS profile_id,
  entity_type,
  entity_id,
  change_set,
  status,
  attempts,
  last_error,
  created_at,
  updated_at
FROM outbox_old;

DROP TABLE outbox_old;

DROP INDEX IF EXISTS idx_outbox_status;
CREATE INDEX idx_outbox_status ON outbox(profile_id, status, created_at);

PRAGMA foreign_keys = ON;
