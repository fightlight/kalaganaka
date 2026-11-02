CREATE TABLE tasks (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL CHECK (length(trim(name)) > 0),
    colour INTEGER NOT NULL
);
CREATE TABLE time_frames (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    started_at TEXT NOT NULL,
    ended_at TEXT NOT NULL,
    CHECK (ended_at > started_at)
);
CREATE INDEX time_frames_started_at ON time_frames(started_at);
CREATE TABLE active_session (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    task_id TEXT NOT NULL REFERENCES tasks(id),
    started_at TEXT NOT NULL
);
PRAGMA user_version = 1;
