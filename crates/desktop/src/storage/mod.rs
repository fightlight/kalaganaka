mod sqlite;

use crate::settings::{Settings, ThemePreference};
use chrono::{DateTime, Utc};
use tracking_core::{Snapshot, Tracker, repository::Repository};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Data {
    pub snapshot: Snapshot,
    pub settings: Settings,
}

#[derive(Debug, Clone)]
pub enum Command {
    Load,
    SetTheme(ThemePreference),
    SetTaskLimit(u32),
    AddTask(String),
    RenameTask(Uuid, String),
    SetFrameMetadata(tracking_core::FrameSelection, tracking_core::FrameMetadata),
    Start(Uuid, DateTime<Utc>),
    Pause(DateTime<Utc>),
    AddFrame {
        task_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        now: DateTime<Utc>,
    },
    EditFrame {
        id: Option<Uuid>,
        expected_active: Option<(Uuid, DateTime<Utc>)>,
        start: DateTime<Utc>,
        end: Option<DateTime<Utc>>,
        now: DateTime<Utc>,
    },
}

/// Each operation runs on a blocking worker; the UI serializes mutations.
pub async fn execute(command: Command) -> Result<Data, String> {
    tokio::task::spawn_blocking(move || {
        let mut repository = sqlite::SqliteRepository::open(&crate::platform::paths::database()?)?;
        let snapshot = repository.load()?;
        let mut settings = repository.load_settings()?;
        match command {
            Command::Load => Ok(Data { snapshot, settings }),
            Command::SetTheme(theme) => {
                settings.theme = theme;
                repository.save_settings(settings)?;
                Ok(Data { snapshot, settings })
            }
            Command::SetTaskLimit(limit) => {
                if !(crate::settings::MIN_TASK_LIMIT..=crate::settings::MAX_TASK_LIMIT)
                    .contains(&limit)
                {
                    return Err("Task results limit must be between 1 and 30.".into());
                }
                settings.task_limit = limit;
                repository.save_settings(settings)?;
                Ok(Data { snapshot, settings })
            }
            command => {
                let mut tracker = Tracker::new(snapshot);
                match command {
                    Command::AddTask(name) => {
                        tracker.add_task(&name).map_err(|error| error.to_string())?;
                    }
                    Command::RenameTask(id, name) => {
                        tracker
                            .rename_task(id, &name)
                            .map_err(|error| error.to_string())?;
                    }
                    Command::SetFrameMetadata(selection, metadata) => tracker
                        .set_metadata(selection, metadata)
                        .map_err(|error| error.to_string())?,
                    Command::Start(id, now) => {
                        tracker.start(id, now).map_err(|error| error.to_string())?
                    }
                    Command::Pause(now) => tracker.pause(now).map_err(|error| error.to_string())?,
                    Command::AddFrame {
                        task_id,
                        start,
                        end,
                        now,
                    } => tracker
                        .add_frame(task_id, start, end, now)
                        .map_err(|error| error.to_string())?,
                    Command::EditFrame {
                        id,
                        expected_active,
                        start,
                        end,
                        now,
                    } => tracker
                        .edit_frame(id, expected_active, start, end, now)
                        .map_err(|error| error.to_string())?,
                    Command::Load | Command::SetTheme(_) | Command::SetTaskLimit(_) => {
                        unreachable!()
                    }
                }
                let snapshot = tracker.into_snapshot();
                repository.save(&snapshot)?;
                Ok(Data { snapshot, settings })
            }
        }
    })
    .await
    .map_err(|error| error.to_string())?
}
