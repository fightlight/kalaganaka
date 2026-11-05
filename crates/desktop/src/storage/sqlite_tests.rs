use super::*;
use chrono::Duration;
use std::path::PathBuf;
use tracking_core::Tracker;

struct TestDatabase(PathBuf);

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
