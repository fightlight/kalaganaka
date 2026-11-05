ALTER TABLE settings ADD COLUMN task_limit INTEGER NOT NULL DEFAULT 5;
PRAGMA user_version = 3;
