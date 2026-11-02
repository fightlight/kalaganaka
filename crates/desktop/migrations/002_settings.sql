CREATE TABLE settings (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    theme TEXT NOT NULL CHECK (theme IN ('light', 'dark', 'system'))
);
INSERT INTO settings (singleton, theme) VALUES (1, 'system');
PRAGMA user_version = 2;
