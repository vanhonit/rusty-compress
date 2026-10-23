use open::that_detached;
use rand::Rng;
use serde_json;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use unrar::Archive;

use crate::data_type::{AppTempDir, ExtractionConfig, ExtractionProgress, FileInfo};
use crate::utils::{safe_join, INVALID_PASSWORD_PREFIX, PASSWORD_REQUIRED_PREFIX};

fn is_password_error(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("password") || lower.contains("encrypted")
}

fn wrap_password_error(e: &str, has_password: bool) -> String {
    if !has_password && is_password_error(e) {
        format!("{}{}", crate::utils::PASSWORD_REQUIRED_PREFIX, e)
    } else {
        e.to_string()
    }
}

/// Map an unrar error code to a password-related error string, if it is one.
fn password_code_error(code: unrar::error::Code, msg: &str) -> Option<String> {
    use unrar::error::Code;
    match code {
        Code::MissingPassword => Some(format!("{}{}", PASSWORD_REQUIRED_PREFIX, msg)),
        Code::BadPassword => Some(format!("{}{}", INVALID_PASSWORD_PREFIX, msg)),
        _ => None,
    }
}

/// Verify that `password` (if any) unlocks the RAR archive before extraction starts.
/// Picks the smallest encrypted file entry and test-decodes it, so the check is cheap.
pub fn check_rar_password(source: &str, password: Option<&str>) -> Result<(), String> {
    use unrar::error::Code;

    // Pass 1: list headers and pick the smallest encrypted, non-directory entry.
    let listing = match password {
        Some(pw) => Archive::with_password(source, pw),
        None => Archive::new(source),
    }
    .open_for_listing()
    .map_err(|e| {
        password_code_error(e.code, &format!("Failed to open RAR archive: {}", e))
            .unwrap_or_else(|| format!("Failed to open RAR archive: {}", e))
    })?;

    let mut target: Option<(u64, String)> = None;
    for entry in listing {
        let header = entry.map_err(|e| {
            password_code_error(e.code, &format!("Failed to read RAR entry: {}", e))
                .unwrap_or_else(|| format!("Failed to read RAR entry: {}", e))
        })?;
        if header.is_encrypted() && !header.is_directory() {
            let size = header.unpacked_size;
            let is_smaller = target.as_ref().is_none_or(|(best, _)| size < *best);
            if is_smaller {
                let name = header.filename.to_string_lossy().to_string();
                target = Some((size, name));
            }
        }
    }

    let target_name = match target {
        Some((_, name)) => name,
        None => return Ok(()),
    };

    let pw = match password {
        Some(pw) => pw,
        None => {
            return Err(format!(
                "{}Archive is password protected",
                PASSWORD_REQUIRED_PREFIX
            ))
        }
    };

    // Pass 2: decode the chosen entry to confirm the password.
    let mut archive = Archive::with_password(source, pw)
        .open_for_processing()
        .map_err(|e| {
            password_code_error(e.code, &format!("Failed to open RAR archive: {}", e))
                .unwrap_or_else(|| format!("Failed to open RAR archive: {}", e))
        })?;

    loop {
        let header = archive.read_header().map_err(|e| {
            password_code_error(e.code, &format!("Failed to read RAR header: {}", e))
                .unwrap_or_else(|| format!("Failed to read RAR header: {}", e))
        })?;

        match header {
            Some(at_file) => {
                let name = at_file.entry().filename.to_string_lossy().to_string();
                if name == target_name {
                    return match at_file.test() {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            // RAR4 wrong passwords surface as CRC/bad-data errors during test.
                            let code = match e.code {
                                Code::BadData => Code::BadPassword,
                                other => other,
                            };
                            Err(password_code_error(
                                code,
                                &format!("Failed to verify RAR password: {}", e),
                            )
                            .unwrap_or_else(|| format!("Failed to verify RAR password: {}", e)))
                        }
                    };
                }
                archive = at_file
                    .skip()
                    .map_err(|e| format!("Failed to skip to next entry: {}", e))?;
            }
            None => return Err("Encrypted RAR entry not found during verification".to_string()),
        }
    }
}

pub fn get_rar_details(
    source_path: PathBuf,
    file_path: Option<String>,
    password: Option<String>,
    _state: State<'_, Arc<AppTempDir>>,
) -> Result<Vec<FileInfo>, String> {
    let file_path = file_path.unwrap_or_default();

    let current_dir = if file_path.is_empty() {
        String::new()
    } else if file_path.ends_with('/') {
        file_path.clone()
    } else {
        format!("{}/", file_path)
    };

    let source_str = source_path
        .to_str()
        .ok_or("Invalid source path")?
        .to_string();
    let has_password = password.is_some();

    let archive = match &password {
        Some(pwd) => unrar::Archive::with_password(&source_str, pwd),
        None => Archive::new(&source_str),
    }
    .open_for_listing()
    .map_err(|e| {
        let msg = e.to_string();
        wrap_password_error(&format!("Failed to open RAR archive: {}", msg), has_password)
    })?;

    let mut file_details: Vec<FileInfo> = Vec::new();
    let mut seen_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

    for entry_result in archive {
        let entry_header = entry_result.map_err(|e| format!("Failed to read RAR entry: {}", e))?;
        let entry_path = entry_header
            .filename
            .to_string_lossy()
            .replace('\\', "/")
            .to_string();

        if entry_path.is_empty() {
            continue;
        }

        if !current_dir.is_empty()
            && entry_path.trim_end_matches('/') == current_dir.trim_end_matches('/')
        {
            continue;
        }

        if !current_dir.is_empty() && !entry_path.starts_with(&current_dir) {
            continue;
        }

        let relative = &entry_path[current_dir.len()..];

        if relative.is_empty() {
            continue;
        }

        match relative.find('/') {
            Some(pos) => {
                let dir_name = &relative[..pos];
                if dir_name.is_empty() {
                    continue;
                }
                let child_path = format!("{}{}/", current_dir, dir_name);

                if seen_paths.insert(child_path.clone()) {
                    file_details.push(FileInfo {
                        name: dir_name.to_string(),
                        is_dir: true,
                        last_modified: "N/A".to_string(),
                        size: 0,
                        mimetype: String::new(),
                        path: child_path,
                    });
                }
            }
            None => {
                let child_name = relative.trim_end_matches('/').to_string();
                if child_name.is_empty() {
                    continue;
                }

                let is_dir = entry_path.ends_with('/');
                let ext = child_name.rsplit('.').next().unwrap_or("");
                let mimetype = if is_dir {
                    String::new()
                } else {
                    mime_guess::from_ext(ext)
                        .first_raw()
                        .unwrap_or("")
                        .to_string()
                };

                let full_path = format!("{}{}", current_dir, relative.trim_end_matches('/'));

                if seen_paths.insert(full_path.clone()) {
                    file_details.push(FileInfo {
                        name: child_name,
                        is_dir,
                        last_modified: "N/A".to_string(),
                        size: entry_header.unpacked_size,
                        mimetype,
                        path: if is_dir {
                            format!("{}/", full_path)
                        } else {
                            full_path
                        },
                    });
                }
            }
        }
    }

    file_details.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(file_details)
}

pub fn unarchive_rar_file(app_handle: AppHandle, config: ExtractionConfig) -> Result<(), String> {
    let source_path = PathBuf::from(&config.source);
    let dest_path = PathBuf::from(&config.destination);

    std::fs::create_dir_all(&dest_path)
        .map_err(|e| format!("Failed to create extraction directory: {}", e))?;

    let selected_files = config.selected_files.unwrap_or_default();

    // Get list of files to calculate total
    let source_str = source_path
        .to_str()
        .ok_or("Invalid source path")?
        .to_string();

    let archive = match &config.password {
        Some(password) => unrar::Archive::with_password(&source_str, password)
            .open_for_listing()
            .map_err(|e| {
                let error_msg = e.to_string();
                if error_msg.contains("password") {
                    format!("Incorrect password for RAR archive: {}", error_msg)
                } else {
                    format!("Failed to open RAR archive: {}", error_msg)
                }
            })?,
        None => Archive::new(&source_str)
            .open_for_listing()
            .map_err(|e| format!("Failed to open RAR archive: {}", e))?,
    };

    let mut total_files = 0;

    for entry_result in archive {
        let entry_header = entry_result.map_err(|e| format!("Failed to read RAR entry: {}", e))?;
        let file_name = entry_header.filename.to_string_lossy().to_string();

        if !selected_files.is_empty() {
            if selected_files.iter().any(|sf| file_name.starts_with(sf)) {
                total_files += 1;
            }
        } else {
            total_files += 1;
        }
    }

    if total_files == 0 {
        total_files = 1;
    }

    let progress = ExtractionProgress::new(total_files);

    // Now extract files
    let mut archive = match &config.password {
        Some(password) => unrar::Archive::with_password(&source_str, password)
            .open_for_processing()
            .map_err(|e| {
                let error_msg = e.to_string();
                if error_msg.contains("password") {
                    format!("Incorrect password for RAR archive: {}", error_msg)
                } else {
                    format!("Failed to open RAR archive for processing: {}", error_msg)
                }
            })?,
        None => Archive::new(&source_str)
            .open_for_processing()
            .map_err(|e| format!("Failed to open RAR archive for processing: {}", e))?,
    };

    let mut count = 0;

    loop {
        if config.cancel.load(Ordering::Relaxed) {
            return Err("Extraction cancelled".to_string());
        }

        let header_result = archive
            .read_header()
            .map_err(|e| format!("Failed to read RAR header: {}", e))?;

        match header_result {
            Some(archive_at_file) => {
                let entry_header = archive_at_file.entry();
                let file_path_str = entry_header.filename.to_string_lossy().to_string();

                // Check if this file should be extracted
                let should_extract = if selected_files.is_empty() {
                    true
                } else {
                    selected_files
                        .iter()
                        .any(|sf| file_path_str.starts_with(sf))
                };

                if should_extract {
                    let file_path = match safe_join(&dest_path, &file_path_str) {
                        Some(path) => path,
                        None => {
                            eprintln!("Skipping unsafe RAR entry: {}", file_path_str);
                            archive = archive_at_file
                                .skip()
                                .map_err(|e| format!("Failed to skip to next entry: {}", e))?;
                            continue;
                        }
                    };

                    if entry_header.is_directory() {
                        std::fs::create_dir_all(&file_path)
                            .map_err(|e| format!("Failed to create directory: {}", e))?;
                        archive = archive_at_file
                            .skip()
                            .map_err(|e| format!("Failed to skip to next entry: {}", e))?;
                    } else {
                        if let Some(parent) = file_path.parent() {
                            std::fs::create_dir_all(parent)
                                .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                        }
                        archive = archive_at_file
                            .extract_to(file_path.to_string_lossy().as_ref())
                            .map_err(|e| format!("Failed to extract file: {}", e))?;
                    }

                    count += 1;
                    progress.update(count);
                    eprintln!("Extracted {}/{} files", count, total_files);
                    let progress_value = progress.get();
                    let progress_data = serde_json::json!({
                        "files": count,
                        "total": total_files,
                        "percentage": progress_value
                    });
                    let _ = app_handle.emit("extract-progress", progress_data);
                } else {
                    archive = archive_at_file
                        .skip()
                        .map_err(|e| format!("Failed to skip to next entry: {}", e))?;
                }
            }
            None => break,
        }
    }

    Ok(())
}

pub fn view_file_in_rar(
    source: String,
    file_name: String,
    password: Option<String>,
    state: State<'_, Arc<AppTempDir>>,
) -> Result<String, String> {
    if file_name.contains("..") {
        return Err(format!("Invalid file name (path traversal): {}", file_name));
    }

    let file_name_clean = file_name.trim();
    if file_name_clean.is_empty() {
        return Err("Empty file name provided".to_string());
    }

    let mut archive = match &password {
        Some(pwd) => unrar::Archive::with_password(&source, pwd),
        None => Archive::new(&source),
    }
    .open_for_processing()
    .map_err(|e| format!("Failed to open RAR archive: {}", e))?;

    let mut file_data: Option<Vec<u8>> = None;

    loop {
        let header_result = archive
            .read_header()
            .map_err(|e| format!("Failed to read RAR header: {}", e))?;

        match header_result {
            Some(archive_at_file) => {
                let entry_header = archive_at_file.entry();

                if entry_header.filename.to_string_lossy() == file_name_clean {
                    let (data, _) = archive_at_file
                        .read()
                        .map_err(|e| format!("Failed to read file from archive: {}", e))?;
                    file_data = Some(data);
                    break;
                }

                archive = archive_at_file
                    .skip()
                    .map_err(|e| format!("Failed to skip to next entry: {}", e))?;
            }
            None => break,
        }
    }

    let data =
        file_data.ok_or_else(|| format!("File not found in RAR archive: {}", file_name_clean))?;

    let temp_dir = &state.temp_dir;
    let mut rng = rand::thread_rng();

    let extension = Path::new(file_name_clean)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let random_name: String = (0..10)
        .map(|_| rng.gen_range(b'a'..=b'z') as char)
        .collect();

    let random_file_name: String = if extension.is_empty() {
        random_name
    } else {
        format!("{}.{}", random_name, extension)
    };

    let temp_file_path = temp_dir.path().join(&random_file_name);

    std::fs::write(&temp_file_path, &data)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    that_detached(&temp_file_path).map_err(|e| format!("Failed to open file: {}", e))?;

    let _ = std::fs::remove_file(&temp_file_path);

    Ok(temp_file_path.to_string_lossy().to_string())
}

pub fn get_image_preview_from_rar(
    archive_path: String,
    file_path: String,
    password: Option<String>,
) -> Result<String, String> {
    if file_path.contains("..") {
        return Err("Invalid file path (path traversal detected)".to_string());
    }

    let file_path_clean = file_path.trim();
    if file_path_clean.is_empty() {
        return Err("Empty file path provided".to_string());
    }

    if !is_image_file_rar(file_path_clean) {
        return Err("File is not a supported image format. Supported formats: jpg, jpeg, png, gif, webp, bmp, tiff, tif".to_string());
    }

    let mut archive = match &password {
        Some(pwd) => unrar::Archive::with_password(&archive_path, pwd),
        None => Archive::new(&archive_path),
    }
    .open_for_processing()
    .map_err(|e| format!("Failed to open RAR archive: {}", e))?;

    let mut file_data: Option<Vec<u8>> = None;

    loop {
        let header_result = archive
            .read_header()
            .map_err(|e| format!("Failed to read RAR header: {}", e))?;

        match header_result {
            Some(archive_at_file) => {
                let entry_header = archive_at_file.entry();

                if entry_header.filename.to_string_lossy() == file_path_clean {
                    let (data, _) = archive_at_file
                        .read()
                        .map_err(|e| format!("Failed to read file from archive: {}", e))?;
                    file_data = Some(data);
                    break;
                }

                archive = archive_at_file
                    .skip()
                    .map_err(|e| format!("Failed to skip to next entry: {}", e))?;
            }
            None => break,
        }
    }

    let data =
        file_data.ok_or_else(|| format!("File not found in archive: {}", file_path_clean))?;

    let base64_data = base64_encode(&data);

    let ext = file_path_clean
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();

    let mime_type = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "tiff" | "tif" => "image/tiff",
        _ => "image/unknown",
    };

    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}

fn is_image_file_rar(file_path: &str) -> bool {
    const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif"];
    let ext = file_path.rsplit('.').next().unwrap_or("").to_lowercase();
    IMAGE_EXTENSIONS.contains(&ext.as_str())
}

fn base64_encode(data: &[u8]) -> String {
    const BASE64_CHARS: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    let mut i = 0;
    while i < data.len() {
        let b1 = data[i];
        let b2 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b3 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        let n = ((b1 as u32) << 16) | ((b2 as u32) << 8) | (b3 as u32);

        result.push(BASE64_CHARS[((n >> 18) & 0x3f) as usize] as char);
        result.push(BASE64_CHARS[((n >> 12) & 0x3f) as usize] as char);

        if i + 1 < data.len() {
            result.push(BASE64_CHARS[((n >> 6) & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(BASE64_CHARS[(n & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{INVALID_PASSWORD_PREFIX, PASSWORD_REQUIRED_PREFIX};

    const CRYPTED: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/crypted.rar");

    #[test]
    fn check_rar_password_requires_password_for_encrypted_archive() {
        let err = check_rar_password(CRYPTED, None).unwrap_err();
        assert!(err.starts_with(PASSWORD_REQUIRED_PREFIX), "got: {err}");
    }

    #[test]
    fn check_rar_password_rejects_wrong_password() {
        let err = check_rar_password(CRYPTED, Some("wrong")).unwrap_err();
        assert!(err.starts_with(INVALID_PASSWORD_PREFIX), "got: {err}");
    }

    #[test]
    fn check_rar_password_accepts_correct_password() {
        assert_eq!(check_rar_password(CRYPTED, Some("unrar")), Ok(()));
    }
}
