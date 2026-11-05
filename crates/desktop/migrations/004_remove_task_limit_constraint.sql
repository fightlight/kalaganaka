CREATE TABLE settings_unconstrained (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    theme TEXT NOT NULL CHECK (theme IN ('light', 'dark', 'system')),
    task_limit INTEGER NOT NULL DEFAULT 5
);
INSERT INTO settings_unconstrained (singleton, theme, task_limit)
SELECT singleton, theme, task_limit FROM settings;
DROP TABLE settings;
ALTER TABLE settings_unconstrained RENAME TO settings;
PRAGMA user_version = 4;
