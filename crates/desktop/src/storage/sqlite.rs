use crate::settings::Settings;
use chrono::{DateTime, SecondsFormat, Utc};
use rusqlite::{Connection, params};
use std::path::Path;
use tracking_core::{ActiveSession, Snapshot, Task, TimeFrame, repository::Repository};
use uuid::Uuid;

#[cfg(test)]
#[path = "sqlite_tests.rs"]
mod tests;

pub struct SqliteRepository(Connection);

impl SqliteRepository {
    pub fn open(path: &Path) -> Result<Self, String> {
        let mut connection = Connection::open(path).map_err(|error| error.to_string())?;
        connection
            .busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|error| error.to_string())?;
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|error| error.to_string())?;
        let version: u32 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .map_err(|error| error.to_string())?;
        if version > 4 {
            return Err("This database requires a newer version of Kalaganaka.".into());
        }
        if version < 4 {
            let transaction = connection
                .transaction()
                .map_err(|error| error.to_string())?;
            if version == 0 {
                transaction
                    .execute_batch(include_str!("../../migrations/001_initial.sql"))
                    .map_err(|error| error.to_string())?;
            }
            if version < 2 {
                transaction
                    .execute_batch(include_str!("../../migrations/002_settings.sql"))
                    .map_err(|error| error.to_string())?;
            }
            if version < 3 {
                transaction
                    .execute_batch(include_str!("../../migrations/003_task_limit.sql"))
                    .map_err(|error| error.to_string())?;
            }
            transaction
                .execute_batch(include_str!(
                    "../../migrations/004_remove_task_limit_constraint.sql"
                ))
                .map_err(|error| error.to_string())?;
            transaction.commit().map_err(|error| error.to_string())?;
        }
        Ok(Self(connection))
    }

    pub fn load_settings(&self) -> Result<Settings, String> {
        let (value, task_limit): (String, u32) = self
            .0
            .query_row(
                "SELECT theme, task_limit FROM settings WHERE singleton = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|error| error.to_string())?;
        Ok(Settings {
            theme: value.parse()?,
            task_limit,
        })
    }

    pub fn save_settings(&self, settings: Settings) -> Result<(), String> {
        self.0
            .execute(
                "INSERT INTO settings (singleton, theme, task_limit) VALUES (1, ?1, ?2)
             ON CONFLICT(singleton) DO UPDATE SET theme = excluded.theme, task_limit = excluded.task_limit",
                params![settings.theme.as_str(), settings.task_limit],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}

fn parse_id(value: String) -> Result<Uuid, String> {
    Uuid::parse_str(&value).map_err(|error| error.to_string())
}
fn parse_time(value: String) -> Result<DateTime<Utc>, String> {
    DateTime::parse_from_rfc3339(&value)
        .map(|time| time.with_timezone(&Utc))
        .map_err(|error| error.to_string())
}
fn timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Nanos, true)
}

impl Repository for SqliteRepository {
    type Error = String;

    fn load(&self) -> Result<Snapshot, String> {
        let mut snapshot = Snapshot::default();
        let mut statement = self
            .0
            .prepare("SELECT id, name, colour FROM tasks ORDER BY rowid")
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, u32>(2)?,
                ))
            })
            .map_err(|error| error.to_string())?;
        for row in rows {
            let (id, name, colour) = row.map_err(|error| error.to_string())?;
            snapshot.tasks.push(Task {
                id: parse_id(id)?,
                name,
                colour,
            });
        }
        let mut statement = self
            .0
            .prepare(
                "SELECT id, task_id, started_at, ended_at FROM time_frames ORDER BY started_at",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            })
            .map_err(|error| error.to_string())?;
        for row in rows {
            let (id, task_id, start, end) = row.map_err(|error| error.to_string())?;
            snapshot.frames.push(TimeFrame {
                id: parse_id(id)?,
                task_id: parse_id(task_id)?,
                started_at: parse_time(start)?,
                ended_at: parse_time(end)?,
            });
        }
        let mut statement = self
            .0
            .prepare("SELECT task_id, started_at FROM active_session")
            .map_err(|error| error.to_string())?;
        let mut rows = statement.query([]).map_err(|error| error.to_string())?;
        if let Some(row) = rows.next().map_err(|error| error.to_string())? {
            snapshot.active = Some(ActiveSession {
                task_id: parse_id(row.get(0).map_err(|error| error.to_string())?)?,
                started_at: parse_time(row.get(1).map_err(|error| error.to_string())?)?,
            });
        }
        Ok(snapshot)
    }

    fn save(&mut self, snapshot: &Snapshot) -> Result<(), String> {
        let transaction = self.0.transaction().map_err(|error| error.to_string())?;
        for task in &snapshot.tasks {
            transaction.execute("INSERT INTO tasks (id, name, colour) VALUES (?1, ?2, ?3) ON CONFLICT(id) DO UPDATE SET name = excluded.name",
                params![task.id.to_string(), task.name, task.colour]).map_err(|error| error.to_string())?;
        }
        for frame in &snapshot.frames {
            transaction.execute("INSERT INTO time_frames (id, task_id, started_at, ended_at) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(id) DO UPDATE SET started_at = excluded.started_at, ended_at = excluded.ended_at",
                params![frame.id.to_string(), frame.task_id.to_string(), timestamp(frame.started_at), timestamp(frame.ended_at)])
                .map_err(|error| error.to_string())?;
        }
        transaction
            .execute("DELETE FROM active_session", [])
            .map_err(|error| error.to_string())?;
        if let Some(active) = &snapshot.active {
            transaction.execute("INSERT INTO active_session (singleton, task_id, started_at) VALUES (1, ?1, ?2)",
                params![active.task_id.to_string(), timestamp(active.started_at)]).map_err(|error| error.to_string())?;
        }
        transaction.commit().map_err(|error| error.to_string())
    }
}
