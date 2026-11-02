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
    AddTask(String),
    Start(Uuid, DateTime<Utc>),
    Pause(DateTime<Utc>),
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
            command => {
                let mut tracker = Tracker::new(snapshot);
                match command {
                    Command::AddTask(name) => {
                        tracker.add_task(&name).map_err(|error| error.to_string())?;
                    }
                    Command::Start(id, now) => {
                        tracker.start(id, now).map_err(|error| error.to_string())?
                    }
                    Command::Pause(now) => tracker.pause(now).map_err(|error| error.to_string())?,
                    Command::Load | Command::SetTheme(_) => unreachable!(),
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
