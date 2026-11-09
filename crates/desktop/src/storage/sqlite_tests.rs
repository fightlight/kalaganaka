use super::*;
use chrono::Duration;
use std::path::PathBuf;
use tracking_core::Tracker;

#[test]
fn metadata_and_tag_catalog_survive_reopening_and_pause() {
    use tracking_core::FrameSelection;
    let database = TestDatabase::new();
    let mut tracker = Tracker::new(Snapshot::default());
    let task = tracker.add_task("Work").unwrap();
    let start: DateTime<Utc> = "2026-11-02T09:00:00Z".parse().unwrap();
    tracker.start(task, start).unwrap();
    let metadata = FrameMetadata {
        tags: vec!["dev".into(), "build".into()],
        description: "First\nSecond".into(),
    };
    tracker
        .set_metadata(FrameSelection::Active(task, start), metadata.clone())
        .unwrap();
    let mut repository = SqliteRepository::open(&database.path()).unwrap();
    repository.save(&tracker.into_snapshot()).unwrap();
    drop(repository);
    let mut repository = SqliteRepository::open(&database.path()).unwrap();
    let snapshot = repository.load().unwrap();
    assert_eq!(snapshot.active.as_ref().unwrap().metadata, metadata);
    assert_eq!(snapshot.tags, vec!["dev", "build"]);
    let mut tracker = Tracker::new(snapshot);
    tracker.pause(start + Duration::minutes(10)).unwrap();
    repository.save(&tracker.into_snapshot()).unwrap();
    drop(repository);
    let reopened = SqliteRepository::open(&database.path())
        .unwrap()
        .load()
        .unwrap();
    assert!(reopened.active.is_none());
    assert_eq!(reopened.frames[0].metadata, metadata);
    assert_eq!(reopened.tags, vec!["dev", "build"]);
}

struct TestDatabase(PathBuf);

#[test]
fn metadata_migration_preserves_existing_frames_and_active_session() {
    let database = TestDatabase::new();
    let connection = Connection::open(database.path()).unwrap();
    for migration in [
        include_str!("../../migrations/001_initial.sql"),
        include_str!("../../migrations/002_settings.sql"),
        include_str!("../../migrations/003_task_limit.sql"),
        include_str!("../../migrations/004_remove_task_limit_constraint.sql"),
    ] {
        connection.execute_batch(migration).unwrap();
    }
    let task = Uuid::new_v4().to_string();
    let frame = Uuid::new_v4().to_string();
    connection
        .execute(
            "INSERT INTO tasks (id, name, colour) VALUES (?1, 'Work', 1)",
            [&task],
        )
        .unwrap();
    connection.execute("INSERT INTO time_frames (id, task_id, started_at, ended_at) VALUES (?1, ?2, '2026-11-02T09:00:00Z', '2026-11-02T10:00:00Z')", params![frame, task]).unwrap();
    connection.execute("INSERT INTO active_session (singleton, task_id, started_at) VALUES (1, ?1, '2026-11-02T11:00:00Z')", [&task]).unwrap();
    drop(connection);
    let snapshot = SqliteRepository::open(&database.path())
        .unwrap()
        .load()
        .unwrap();
    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.frames.len(), 1);
    assert_eq!(snapshot.frames[0].metadata, FrameMetadata::default());
    assert_eq!(snapshot.active.unwrap().metadata, FrameMetadata::default());
    assert!(snapshot.tags.is_empty());
}

impl TestDatabase {
    fn new() -> Self {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../build/tests")
            .join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(&directory).unwrap();
        Self(directory)
    }

    fn path(&self) -> PathBuf {
        self.0.join("tracking.sqlite3")
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("remove the test's database directory");
    }
}

#[test]
fn recorded_frames_and_active_sessions_survive_reopening_without_duplicate_saves() {
    let database = TestDatabase::new();
    let mut tracker = Tracker::new(Snapshot::default());
    let task_id = tracker.add_task("Development 🦀").unwrap();
    let start: DateTime<Utc> = "2026-11-02T09:00:00.123456789Z".parse().unwrap();
    tracker.start(task_id, start).unwrap();
    tracker.pause(start + Duration::minutes(25)).unwrap();
    tracker
        .start(task_id, start + Duration::minutes(40))
        .unwrap();
    let expected = tracker.into_snapshot();
    {
        let mut repository = SqliteRepository::open(&database.path()).unwrap();
        repository.save(&expected).unwrap();
        repository.save(&expected).unwrap();
    }
    let completed;
    {
        let mut repository = SqliteRepository::open(&database.path()).unwrap();
        let actual = repository.load().unwrap();
        assert_eq!(actual.tasks, expected.tasks);
        assert_eq!(actual.frames, expected.frames);
        assert_eq!(actual.active, expected.active);
        let mut tracker = Tracker::new(actual);
        tracker.pause(start + Duration::hours(1)).unwrap();
        completed = tracker.into_snapshot();
        repository.save(&completed).unwrap();
        repository.save(&completed).unwrap();
    }
    let repository = SqliteRepository::open(&database.path()).unwrap();
    let actual = repository.load().unwrap();
    assert_eq!(actual.tasks, completed.tasks);
    assert_eq!(actual.frames, completed.frames);
    assert_eq!(actual.frames.len(), 2);
    assert!(actual.active.is_none());
}

#[test]
fn a_failed_save_rolls_back_new_records_and_preserves_the_previous_active_session() {
    let mut repository = SqliteRepository::open(Path::new(":memory:")).unwrap();
    let mut tracker = Tracker::new(Snapshot::default());
    let task_id = tracker.add_task("Development").unwrap();
    let start: DateTime<Utc> = "2026-11-02T09:00:00Z".parse().unwrap();
    tracker.start(task_id, start).unwrap();
    let expected = tracker.into_snapshot();
    repository.save(&expected).unwrap();

    let mut tracker = Tracker::new(expected.clone());
    tracker.add_task("Review").unwrap();
    tracker.pause(start + Duration::minutes(25)).unwrap();
    let mut invalid = tracker.into_snapshot();
    // Fail after inserting tasks/frames and deleting the previous active session.
    invalid.active = Some(ActiveSession {
        task_id: Uuid::new_v4(),
        started_at: start,
        metadata: Default::default(),
    });
    assert!(repository.save(&invalid).is_err());

    let actual = repository.load().unwrap();
    assert_eq!(actual.tasks, expected.tasks);
    assert_eq!(actual.frames, expected.frames);
    assert_eq!(actual.active, expected.active);
}

#[test]
fn renamed_task_and_its_tracking_data_survive_reopening() {
    let database = TestDatabase::new();
    let mut tracker = Tracker::new(Snapshot::default());
    let id = tracker.add_task("Development").unwrap();
    let start: DateTime<Utc> = "2026-11-02T09:00:00Z".parse().unwrap();
    tracker.start(id, start).unwrap();
    tracker.pause(start + Duration::minutes(10)).unwrap();
    tracker.start(id, start + Duration::minutes(15)).unwrap();
    let mut repository = SqliteRepository::open(&database.path()).unwrap();
    repository.save(&tracker.into_snapshot()).unwrap();
    let mut tracker = Tracker::new(repository.load().unwrap());
    tracker.rename_task(id, "Coding").unwrap();
    let expected = tracker.into_snapshot();
    repository.save(&expected).unwrap();
    drop(repository);

    let actual = SqliteRepository::open(&database.path())
        .unwrap()
        .load()
        .unwrap();
    assert_eq!(actual.tasks, expected.tasks);
    assert_eq!(actual.frames, expected.frames);
    assert_eq!(actual.active, expected.active);
    assert_eq!(actual.tasks[0].name, "Coding");
}

#[test]
fn settings_migration_preserves_existing_data_and_removes_old_limit_constraint() {
    let database = TestDatabase::new();
    let connection = Connection::open(database.path()).unwrap();
    connection
        .execute_batch(include_str!("../../migrations/001_initial.sql"))
        .unwrap();
    connection
        .execute_batch(include_str!("../../migrations/002_settings.sql"))
        .unwrap();
    connection.execute_batch("ALTER TABLE settings ADD COLUMN task_limit INTEGER NOT NULL DEFAULT 10 CHECK (task_limit BETWEEN 3 AND 30); UPDATE settings SET theme = 'dark', task_limit = 20; PRAGMA user_version = 3;").unwrap();
    drop(connection);
    let repository = SqliteRepository::open(&database.path()).unwrap();
    let settings = repository.load_settings().unwrap();
    assert_eq!(settings.theme, crate::settings::ThemePreference::Dark);
    assert_eq!(settings.task_limit, 20);
    repository
        .save_settings(Settings {
            task_limit: 1,
            ..settings
        })
        .unwrap();
    drop(repository);
    let repository = SqliteRepository::open(&database.path()).unwrap();
    assert_eq!(repository.load_settings().unwrap().task_limit, 1);
}

#[test]
fn new_database_defaults_task_limit_to_five() {
    let database = TestDatabase::new();
    let repository = SqliteRepository::open(&database.path()).unwrap();
    assert_eq!(repository.load_settings().unwrap(), Settings::default());
    assert_eq!(repository.load_settings().unwrap().task_limit, 5);
}

#[test]
fn edited_frame_times_survive_reopening() {
    let database = TestDatabase::new();
    let mut tracker = Tracker::new(Snapshot::default());
    let task = tracker.add_task("Work").unwrap();
    let start: DateTime<Utc> = "2026-11-02T09:00:15Z".parse().unwrap();
    tracker.start(task, start).unwrap();
    tracker.pause(start + Duration::minutes(10)).unwrap();
    let snapshot = tracker.into_snapshot();
    let mut repository = SqliteRepository::open(&database.path()).unwrap();
    repository.save(&snapshot).unwrap();
    let mut tracker = Tracker::new(snapshot.clone());
    tracker
        .edit_frame(
            Some(snapshot.frames[0].id),
            None,
            start + Duration::minutes(1),
            Some(start + Duration::minutes(12)),
            start + Duration::hours(1),
        )
        .unwrap();
    let expected = tracker.into_snapshot();
    repository.save(&expected).unwrap();
    drop(repository);
    let reopened = SqliteRepository::open(&database.path())
        .unwrap()
        .load()
        .unwrap();
    assert_eq!(reopened.frames, expected.frames);
    assert_eq!(reopened.tasks, expected.tasks);
}
