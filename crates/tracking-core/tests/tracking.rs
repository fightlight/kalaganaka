use chrono::{DateTime, Duration, Utc};
use tracking_core::{ActiveSession, Snapshot, Task, TimeFrame, Tracker, TrackingError, statistics};
use uuid::Uuid;

fn time(value: &str) -> DateTime<Utc> {
    value.parse().expect("valid test timestamp")
}

#[test]
fn rename_validates_names_and_preserves_task_identity_and_tracking() {
    let mut tracker = Tracker::new(Snapshot::default());
    let id = tracker.add_task("Development").unwrap();
    tracker.add_task("Review").unwrap();
    let start = time("2026-11-02T09:00:00Z");
    tracker.start(id, start).unwrap();
    tracker.pause(start + Duration::minutes(10)).unwrap();
    tracker.start(id, start + Duration::minutes(15)).unwrap();
    let before = tracker.into_snapshot();
    let mut tracker = Tracker::new(before.clone());

    assert!(matches!(
        tracker.rename_task(id, "  "),
        Err(TrackingError::EmptyName)
    ));
    assert!(matches!(
        tracker.rename_task(id, " REVIEW "),
        Err(TrackingError::DuplicateName)
    ));
    assert!(matches!(
        tracker.rename_task(Uuid::new_v4(), "Other"),
        Err(TrackingError::UnknownTask)
    ));
    tracker.rename_task(id, " development ").unwrap();
    tracker.rename_task(id, "  Coding  ").unwrap();

    let after = tracker.into_snapshot();
    assert_eq!(after.tasks[0].id, id);
    assert_eq!(after.tasks[0].name, "Coding");
    assert_eq!(after.tasks[0].colour, before.tasks[0].colour);
    assert_eq!(after.frames, before.frames);
    assert_eq!(after.active, before.active);
}

#[test]
fn pause_records_only_running_intervals_and_restart_creates_a_new_frame() {
    let mut tracker = Tracker::new(Snapshot::default());
    let task_id = tracker.add_task("Development").unwrap();
    let start = time("2026-11-02T09:00:00Z");
    tracker.start(task_id, start).unwrap();
    let snapshot = tracker.into_snapshot();
    assert!(snapshot.frames.is_empty());
    assert_eq!(
        snapshot.active,
        Some(ActiveSession {
            task_id,
            started_at: start
        })
    );

    let mut tracker = Tracker::new(snapshot);
    tracker.pause(start + Duration::minutes(25)).unwrap();
    tracker.pause(start + Duration::minutes(30)).unwrap();
    tracker
        .start(task_id, start + Duration::minutes(40))
        .unwrap();
    tracker.pause(start + Duration::minutes(50)).unwrap();
    let snapshot = tracker.into_snapshot();

    assert!(snapshot.active.is_none());
    assert_eq!(snapshot.frames.len(), 2);
    let first = &snapshot.frames[0];
    let second = &snapshot.frames[1];
    assert_eq!(
        (first.task_id, first.started_at, first.ended_at),
        (task_id, start, start + Duration::minutes(25))
    );
    assert_eq!(
        (second.task_id, second.started_at, second.ended_at),
        (
            task_id,
            start + Duration::minutes(40),
            start + Duration::minutes(50)
        )
    );
    assert_ne!(first.id, second.id);
}

#[test]
fn switching_tasks_closes_the_previous_frame_without_gaps_or_overlap() {
    let mut tracker = Tracker::new(Snapshot::default());
    let first = tracker.add_task("Development").unwrap();
    let second = tracker.add_task("Review").unwrap();
    let start = time("2026-11-02T09:00:00Z");
    let switch = start + Duration::minutes(30);
    let end = start + Duration::hours(1);

    tracker.start(first, start).unwrap();
    tracker.start(second, switch).unwrap();
    tracker.pause(end).unwrap();
    let snapshot = tracker.into_snapshot();

    assert_eq!(snapshot.frames.len(), 2);
    assert_eq!(snapshot.frames[0].task_id, first);
    assert_eq!(snapshot.frames[1].task_id, second);
    assert_eq!(snapshot.frames[0].started_at, start);
    assert_eq!(snapshot.frames[0].ended_at, switch);
    assert_eq!(snapshot.frames[1].started_at, switch);
    assert_eq!(snapshot.frames[1].ended_at, end);
    assert!(snapshot.active.is_none());
}

#[test]
fn selecting_the_running_task_preserves_its_original_start() {
    let mut tracker = Tracker::new(Snapshot::default());
    let task_id = tracker.add_task("Development").unwrap();
    let start = time("2026-11-02T09:00:00Z");
    tracker.start(task_id, start).unwrap();
    tracker
        .start(task_id, start + Duration::minutes(10))
        .unwrap();
    let snapshot = tracker.into_snapshot();
    assert!(snapshot.frames.is_empty());
    assert_eq!(
        snapshot.active,
        Some(ActiveSession {
            task_id,
            started_at: start
        })
    );
}

#[test]
fn invalid_task_input_does_not_change_the_running_session() {
    let mut tracker = Tracker::new(Snapshot::default());
    let task_id = tracker.add_task("  Development  ").unwrap();
    let start = time("2026-11-02T09:00:00Z");
    tracker.start(task_id, start).unwrap();

    assert!(matches!(
        tracker.add_task("  "),
        Err(TrackingError::EmptyName)
    ));
    assert!(matches!(
        tracker.add_task("development"),
        Err(TrackingError::DuplicateName)
    ));
    assert!(matches!(
        tracker.start(Uuid::new_v4(), start),
        Err(TrackingError::UnknownTask)
    ));
    let snapshot = tracker.into_snapshot();
    assert_eq!(snapshot.tasks.len(), 1);
    assert_eq!(snapshot.tasks[0].name, "Development");
    assert!(snapshot.frames.is_empty());
    assert_eq!(
        snapshot.active,
        Some(ActiveSession {
            task_id,
            started_at: start
        })
    );
}

#[test]
fn a_backwards_clock_does_not_close_or_switch_the_running_session() {
    let mut tracker = Tracker::new(Snapshot::default());
    let task_id = tracker.add_task("Development").unwrap();
    let other_id = tracker.add_task("Review").unwrap();
    let start = time("2026-11-02T09:00:00Z");
    tracker.start(task_id, start).unwrap();
    let earlier = start - Duration::seconds(1);
    assert!(matches!(
        tracker.pause(earlier),
        Err(TrackingError::ClockMovedBackwards)
    ));
    assert!(matches!(
        tracker.start(other_id, earlier),
        Err(TrackingError::ClockMovedBackwards)
    ));
    let snapshot = tracker.into_snapshot();
    assert!(snapshot.frames.is_empty());
    assert_eq!(
        snapshot.active,
        Some(ActiveSession {
            task_id,
            started_at: start
        })
    );
}

#[test]
fn statistics_clip_midnight_crossings_and_exclude_outside_or_running_time() {
    let mut tracker = Tracker::new(Snapshot::default());
    let task_id = tracker.add_task("Development").unwrap();
    // Fixed offset represents a user's local calendar day without depending on the host timezone.
    let start = time("2026-11-02T00:00:00+03:00");
    let end = time("2026-11-03T00:00:00+03:00");
    for (from, to) in [
        (start - Duration::hours(2), start),
        (start - Duration::minutes(30), start + Duration::minutes(30)),
        (start + Duration::hours(12), start + Duration::hours(13)),
        (end - Duration::minutes(15), end + Duration::minutes(45)),
        (end + Duration::hours(1), end + Duration::hours(2)),
    ] {
        tracker.start(task_id, from).unwrap();
        tracker.pause(to).unwrap();
    }
    let mut snapshot = tracker.into_snapshot();
    snapshot.active = Some(ActiveSession {
        task_id,
        started_at: start,
    });
    let totals = statistics::totals(&snapshot, start, end);
    assert_eq!(totals.len(), 1);
    assert_eq!(totals[0].task_id, task_id);
    assert_eq!(totals[0].duration, Duration::minutes(105));
    assert!(statistics::totals(&snapshot, end, end).is_empty());
}

#[test]
fn statistics_omit_tasks_without_completed_time_in_the_reporting_interval() {
    let mut tracker = Tracker::new(Snapshot::default());
    let recorded = tracker.add_task("Recorded").unwrap();
    tracker.add_task("Never started").unwrap();
    let before = tracker.add_task("Before period").unwrap();
    let after = tracker.add_task("After period").unwrap();
    let running = tracker.add_task("Still running").unwrap();
    let start = time("2026-11-02T00:00:00Z");
    let end = start + Duration::days(1);

    tracker.start(before, start - Duration::hours(1)).unwrap();
    tracker.pause(start).unwrap();
    tracker.start(recorded, start).unwrap();
    tracker.pause(start + Duration::minutes(10)).unwrap();
    tracker.start(after, end).unwrap();
    tracker.pause(end + Duration::hours(1)).unwrap();
    tracker.start(running, end + Duration::hours(2)).unwrap();
    let snapshot = tracker.into_snapshot();

    let totals = statistics::totals(&snapshot, start, end);
    assert_eq!(totals.len(), 1);
    assert_eq!(totals[0].task_id, recorded);
    assert_eq!(totals[0].duration, Duration::minutes(10));
    assert!(
        statistics::totals(&snapshot, end + Duration::days(1), end + Duration::days(2)).is_empty()
    );
}

#[test]
fn statistics_group_by_task_identity_and_sum_separate_frames() {
    let first = Task {
        id: Uuid::new_v4(),
        name: "Development".into(),
        colour: 0x4979c9,
    };
    let second = Task {
        id: Uuid::new_v4(),
        name: first.name.clone(),
        colour: 0x2c917c,
    };
    let start = time("2026-11-02T09:00:00Z");
    let frame = |task_id, from, to| TimeFrame {
        id: Uuid::new_v4(),
        task_id,
        started_at: start + Duration::minutes(from),
        ended_at: start + Duration::minutes(to),
    };
    let snapshot = Snapshot {
        tasks: vec![first.clone(), second.clone()],
        frames: vec![
            frame(first.id, 0, 10),
            frame(second.id, 10, 25),
            frame(first.id, 25, 45),
        ],
        active: None,
    };
    let totals = statistics::totals(&snapshot, start, start + Duration::days(1));
    assert_eq!(totals.len(), 2);
    assert_eq!(
        (totals[0].task_id, totals[0].duration),
        (first.id, Duration::minutes(30))
    );
    assert_eq!(
        (totals[1].task_id, totals[1].duration),
        (second.id, Duration::minutes(15))
    );
    assert!(statistics::totals(&Snapshot::default(), start, start + Duration::days(1)).is_empty());
}

#[test]
fn frame_edits_validate_ranges_overlaps_and_stale_sessions() {
    let mut tracker = Tracker::new(Snapshot::default());
    let task = tracker.add_task("Work").unwrap();
    let start = time("2026-11-02T09:00:00Z");
    tracker.start(task, start).unwrap();
    tracker.pause(start + Duration::minutes(15)).unwrap();
    tracker.start(task, start + Duration::minutes(30)).unwrap();
    let snapshot = tracker.into_snapshot();
    let id = snapshot.frames[0].id;
    let active_start = snapshot.active.as_ref().unwrap().started_at;
    let now = start + Duration::minutes(45);
    let mut tracker = Tracker::new(snapshot.clone());
    assert!(matches!(
        tracker.edit_frame(Some(id), None, start, Some(start), now),
        Err(TrackingError::InvalidFrameRange)
    ));
    assert!(matches!(
        tracker.edit_frame(
            Some(id),
            None,
            start,
            Some(active_start + Duration::minutes(1)),
            now
        ),
        Err(TrackingError::OverlappingFrame)
    ));
    assert!(matches!(
        tracker.edit_frame(
            None,
            Some((task, active_start)),
            start + Duration::minutes(10),
            None,
            now
        ),
        Err(TrackingError::OverlappingFrame)
    ));
    assert!(matches!(
        tracker.edit_frame(None, Some((task, start)), active_start, None, now),
        Err(TrackingError::UnknownFrame)
    ));
    assert!(matches!(
        tracker.edit_frame(
            None,
            Some((task, active_start)),
            now + Duration::minutes(1),
            None,
            now
        ),
        Err(TrackingError::InvalidFrameRange)
    ));
    let unchanged = tracker.into_snapshot();
    assert_eq!(unchanged.frames, snapshot.frames);
    assert_eq!(unchanged.active, snapshot.active);
    let mut tracker = Tracker::new(unchanged);
    tracker
        .edit_frame(Some(id), None, start, Some(active_start), now)
        .unwrap();
    tracker
        .edit_frame(None, Some((task, active_start)), active_start, None, now)
        .unwrap();
    assert_eq!(tracker.into_snapshot().frames[0].ended_at, active_start);
}
