use std::path::PathBuf;

pub fn database() -> Result<PathBuf, String> {
    let directories = directories::ProjectDirs::from("", "", "kalaganaka")
        .ok_or("Could not locate the application-data directory.")?;
    let directory = directories.data_local_dir();
    std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    Ok(directory.join("kalaganaka.sqlite3"))
}
