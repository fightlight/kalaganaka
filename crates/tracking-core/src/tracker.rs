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
    #[error("The time frame no longer exists.")]
    UnknownFrame,
    #[error("Start must be before end and cannot be in the future.")]
    InvalidFrameRange,
    #[error("The time frame overlaps another frame.")]
    OverlappingFrame,
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

    pub fn rename_task(&mut self, task_id: Uuid, name: &str) -> Result<(), TrackingError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(TrackingError::EmptyName);
        }
        if self
            .snapshot
            .tasks
            .iter()
            .any(|task| task.id != task_id && task.name.to_lowercase() == name.to_lowercase())
        {
            return Err(TrackingError::DuplicateName);
        }
        let task = self
            .snapshot
            .tasks
            .iter_mut()
            .find(|task| task.id == task_id)
            .ok_or(TrackingError::UnknownTask)?;
        task.name = name.to_owned();
        Ok(())
    }

    pub fn edit_frame(
        &mut self,
        id: Option<Uuid>,
        expected_active: Option<(Uuid, DateTime<Utc>)>,
        start: DateTime<Utc>,
        end: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    ) -> Result<(), TrackingError> {
        let actual_end = end.unwrap_or(now);
        if start >= actual_end || start > now {
            return Err(TrackingError::InvalidFrameRange);
        }
        if let Some(frame_id) = id {
            if !self
                .snapshot
                .frames
                .iter()
                .any(|frame| frame.id == frame_id)
                || end.is_none()
            {
                return Err(TrackingError::UnknownFrame);
            }
        } else if end.is_some()
            || !self
                .snapshot
                .active
                .as_ref()
                .is_some_and(|active| Some((active.task_id, active.started_at)) == expected_active)
        {
            return Err(TrackingError::UnknownFrame);
        }
        if self.snapshot.frames.iter().any(|frame| {
            Some(frame.id) != id && start < frame.ended_at && frame.started_at < actual_end
        }) || (id.is_some()
            && self
                .snapshot
                .active
                .as_ref()
                .is_some_and(|active| start < now && active.started_at < actual_end))
        {
            return Err(TrackingError::OverlappingFrame);
        }
        if let Some(frame_id) = id {
            let frame = self
                .snapshot
                .frames
                .iter_mut()
                .find(|frame| frame.id == frame_id)
                .ok_or(TrackingError::UnknownFrame)?;
            frame.started_at = start;
            frame.ended_at = actual_end;
        } else if let Some(active) = self.snapshot.active.as_mut() {
            active.started_at = start;
        }
        Ok(())
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
