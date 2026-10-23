use crate::tar::get_tar_details;
use crate::zip::get_zip_details;
use crate::data_type::{FileInfo, AppTempDir};
use std::fs;
use std::path::PathBuf;
use tauri::State;
use std::sync::Arc;



#[tauri::command]
pub fn read_directory(path: String) -> Result<Vec<FileInfo>, String> {
    let path = PathBuf::from(path);
    if !path.is_dir() {
        return Err("Provided path is not a directory.".to_string());
    }

    let mut entries: Vec<FileInfo> = fs::read_dir(&path)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok())
        .map(|entry| {
            let name = entry.file_name().into_string().unwrap_or_default();
            let is_dir = entry.path().is_dir();
            let size: u64 = if is_dir {
                0
            } else {
                entry.metadata().ok().map(|m| m.len()).unwrap_or(0)
            };
            let last_modified = entry.metadata().ok().and_then(|m| m.modified().ok())
                .map(|modified| {
                    modified.duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .ok().map(|d| d.as_secs()).unwrap_or(0)
                })
                .map(|secs| {
                    let dt = chrono::DateTime::from_timestamp(secs as i64, 0).unwrap();
                    dt.format("%Y-%m-%d %H:%M:%S").to_string()
                })
                .unwrap_or("N/A".to_string());
            let mimetype = mime_guess::from_path(&entry.path()).first_raw().unwrap_or("").to_string();
            FileInfo { name, is_dir, size, last_modified, mimetype, path: entry.path().to_string_lossy().to_string() }
        })
        .collect();

    // Sort: folders first, then files (lexical order within each group)
    entries.sort_by(|a, b| {
        b.is_dir.cmp(&a.is_dir) // Folders (true) come before files (false)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase())) // Alphabetical order
    });

    Ok(entries)
}
#[tauri::command]
pub fn open_file(path: String, file_path: Option<String>, state: State<'_, Arc<AppTempDir>>) -> Result<Option<Vec<FileInfo>>, String> {
    let path = PathBuf::from(path);
    if !path.is_file() {
        return Err("Provided path is not a file.".to_string());
    }

    match path.extension().and_then(|ext| ext.to_str()) {
        Some("zip") => {
            get_zip_details(path, file_path, None, state).map(|details| Ok(Some(details))).unwrap_or_else(Err)
        },
        Some("tar") => {
            get_tar_details(path, file_path, state).map(|details| Ok(Some(details))).unwrap_or_else(Err)
        },
        _ => {
            let result = open::that(&path);
            if result.is_ok() {
                Ok(None)
            } else {
                Err(result.err().unwrap().to_string())
            }
        }
    }
}