use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeFrame {
    pub id: Uuid,
    pub task_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveSession {
    pub task_id: Uuid,
    pub started_at: DateTime<Utc>,
}

/// Identifies either a completed frame or the active session as observed by the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameSelection {
    Completed(Uuid),
    Active(Uuid, DateTime<Utc>),
}

pub enum FrameRef<'a> {
    Completed(&'a TimeFrame),
    Active(&'a ActiveSession),
}
