use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{ActiveSession, Task, TimeFrame};

#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub tasks: Vec<Task>,
    pub frames: Vec<TimeFrame>,
    pub active: Option<ActiveSession>,
}

#[derive(Debug, thiserror::Error)]
pub enum TrackingError {
    #[error("Enter a task name.")]
    EmptyName,
    #[error("A task with this name already exists.")]
    DuplicateName,
    #[error("Select an existing task first.")]
    UnknownTask,
    #[error(
        "The system clock is earlier than the session start. Correct the clock before pausing."
    )]
    ClockMovedBackwards,
}

pub struct Tracker {
    snapshot: Snapshot,
}

impl Tracker {
    pub fn new(snapshot: Snapshot) -> Self {
        Self { snapshot }
    }

    pub fn into_snapshot(self) -> Snapshot {
        self.snapshot
    }

    pub fn add_task(&mut self, name: &str) -> Result<Uuid, TrackingError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(TrackingError::EmptyName);
        }
        if self
            .snapshot
            .tasks
            .iter()
            .any(|task| task.name.to_lowercase() == name.to_lowercase())
        {
            return Err(TrackingError::DuplicateName);
        }
        const PALETTE: [u32; 6] = [0x4979c9, 0x2c917c, 0xb17730, 0x9360b8, 0xc05e74, 0x398aab];
        let task = Task {
            id: Uuid::new_v4(),
            name: name.to_owned(),
            colour: PALETTE[self.snapshot.tasks.len() % PALETTE.len()],
        };
        let id = task.id;
        self.snapshot.tasks.push(task);
        Ok(id)
    }

    pub fn start(&mut self, task_id: Uuid, now: DateTime<Utc>) -> Result<(), TrackingError> {
        if !self.snapshot.tasks.iter().any(|task| task.id == task_id) {
            return Err(TrackingError::UnknownTask);
        }
        if self
            .snapshot
            .active
            .as_ref()
            .is_some_and(|active| active.task_id == task_id)
        {
            return Ok(());
        }
        self.pause(now)?;
        self.snapshot.active = Some(ActiveSession {
            task_id,
            started_at: now,
        });
        Ok(())
    }

    pub fn pause(&mut self, now: DateTime<Utc>) -> Result<(), TrackingError> {
        let Some(active) = self.snapshot.active.as_ref() else {
            return Ok(());
        };
        if now < active.started_at {
            return Err(TrackingError::ClockMovedBackwards);
        }
        if now > active.started_at {
            self.snapshot.frames.push(TimeFrame {
                id: Uuid::new_v4(),
                task_id: active.task_id,
                started_at: active.started_at,
                ended_at: now,
            });
        }
        self.snapshot.active = None;
        Ok(())
    }
}
