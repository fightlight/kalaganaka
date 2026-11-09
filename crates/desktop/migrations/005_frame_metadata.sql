ALTER TABLE time_frames ADD COLUMN description TEXT NOT NULL DEFAULT '';
ALTER TABLE active_session ADD COLUMN description TEXT NOT NULL DEFAULT '';
CREATE TABLE tags (name TEXT PRIMARY KEY COLLATE NOCASE);
CREATE TABLE frame_tags (
    frame_id TEXT NOT NULL REFERENCES time_frames(id),
    tag TEXT NOT NULL REFERENCES tags(name),
    PRIMARY KEY (frame_id, tag)
);
CREATE TABLE active_tags (tag TEXT PRIMARY KEY REFERENCES tags(name));
PRAGMA user_version = 5;
