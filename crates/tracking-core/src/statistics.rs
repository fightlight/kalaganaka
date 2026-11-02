use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::Snapshot;

#[derive(Debug, Clone)]
pub struct TaskTotal {
    pub task_id: Uuid,
    pub name: String,
    pub duration: Duration,
}

/// Completed time only, clipped to the half-open reporting interval.
pub fn totals(snapshot: &Snapshot, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<TaskTotal> {
    let mut totals: Vec<_> = snapshot
        .tasks
        .iter()
        .map(|task| {
            let duration = snapshot
                .frames
                .iter()
                .filter(|frame| frame.task_id == task.id)
                .map(|frame| {
                    (frame.ended_at.min(end) - frame.started_at.max(start)).max(Duration::zero())
                })
                .fold(Duration::zero(), |total, duration| total + duration);
            TaskTotal {
                task_id: task.id,
                name: task.name.clone(),
                duration,
            }
        })
        .collect();
    totals.sort_by(|a, b| {
        b.duration
            .cmp(&a.duration)
            .then_with(|| a.name.cmp(&b.name))
    });
    totals
}
