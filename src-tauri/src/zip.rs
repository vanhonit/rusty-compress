use open::that_detached;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tauri::Emitter;
use tauri::State;
use zip::ZipArchive;

use crate::data_type::{AppTempDir, ExtractionConfig, ExtractionProgress, FileInfo};
use crate::utils::{INVALID_PASSWORD_PREFIX, PASSWORD_REQUIRED_PREFIX};
use rand::Rng;

/// Verify that `password` (if any) unlocks the archive before extraction starts.
/// Returns Ok when the archive has no encrypted entries or the password is correct.
pub fn check_zip_password(source: &str, password: Option<&str>) -> Result<(), String> {
    let file = File::open(source).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(BufReader::new(file)).map_err(|e| e.to_string())?;

    let mut encrypted_index = None;
    for i in 0..archive.len() {
        let entry = archive.by_index_raw(i).map_err(|e| e.to_string())?;
        if entry.encrypted() && !entry.is_dir() {
            encrypted_index = Some(i);
            break;
        }
    }

    let index = match encrypted_index {
        Some(i) => i,
        None => return Ok(()),
    };

    let password = match password {
        Some(pw) => pw,
        None => {
            return Err(format!(
                "{}Archive is password protected",
                PASSWORD_REQUIRED_PREFIX
            ))
        }
    };

    let result = match archive.by_index_decrypt(index, password.as_bytes()) {
        Ok(_) => Ok(()),
        Err(zip::result::ZipError::InvalidPassword) => {
            Err(format!("{}Incorrect password", INVALID_PASSWORD_PREFIX))
        }
        Err(e) => Err(e.to_string()),
    };
    result
}

fn is_password_error(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("password") || lower.contains("encrypted") || lower.contains("decrypt")
}

fn wrap_password_error(e: &str, has_password: bool) -> String {
    if !has_password && is_password_error(e) {
        format!("{}{}", crate::utils::PASSWORD_REQUIRED_PREFIX, e)
    } else {
        e.to_string()
    }
}

pub fn get_zip_details(
    source_path: PathBuf,
    file_path: Option<String>,
    password: Option<String>,
    _state: State<'_, Arc<AppTempDir>>,
) -> Result<Vec<FileInfo>, String> {
    let file_path = file_path.unwrap_or_default();
    let has_password = password.is_some();

    // Normalize current_dir to always end with '/' if non-empty
    let current_dir = if file_path.is_empty() {
        String::new()
    } else if file_path.ends_with('/') {
        file_path.clone()
    } else {
        format!("{}/", file_path)
    };

    // Open the archive
    let file = File::open(&source_path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader).map_err(|e| e.to_string())?;

    // First pass: collect all entry names and build name-to-index map
    let mut name_to_index: HashMap<String, usize> = HashMap::new();
    for i in 0..archive.len() {
        let password_bytes = password.as_ref().map(|p| p.as_bytes());
        let entry = archive.by_index_decrypt(i, password_bytes.as_deref().unwrap_or(&[]))
            .map_err(|e| {
                let msg = e.to_string();
                wrap_password_error(&msg, has_password)
            })?;
        let name = entry.name().to_string();
        name_to_index.insert(name.clone(), i);
    }

    // Build a deduplicated set of immediate children of current_dir
    let mut seen: HashSet<String> = HashSet::new();
    let mut child_paths: Vec<String> = Vec::new();

    for name in name_to_index.keys() {
        // Skip if this entry IS the current directory itself
        if name == &current_dir {
            continue;
        }
        // Skip entries outside the current directory
        if !current_dir.is_empty() && !name.starts_with(&current_dir) {
            continue;
        }

        // Strip the current_dir prefix to get the relative path
        let relative = &name[current_dir.len()..];

        let child_path = match relative.find('/') {
            Some(pos) => {
                // Entry lives inside a subdirectory — immediate child is the dir
                let dir_part = &relative[..pos];
                if dir_part.is_empty() {
                    continue;
                }
                format!("{}{}/", current_dir, dir_part)
            }
            None => {
                // Direct file child of current_dir
                name.clone()
            }
        };

        if seen.insert(child_path.clone()) {
            child_paths.push(child_path);
        }
    }

    // Second pass: fetch metadata for each immediate child
    let mut file_details: Vec<FileInfo> = Vec::new();

    for child_path in &child_paths {
        let is_dir = child_path.ends_with('/');
        let display_name = child_path
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or(child_path.as_str())
            .to_string();

        let (size, last_modified) = match name_to_index.get(child_path.as_str()) {
            Some(&idx) => {
                let password_bytes = password.as_ref().map(|p| p.as_bytes());
                match archive.by_index_decrypt(idx, password_bytes.as_deref().unwrap_or(&[])) {
                    Ok(entry) => {
                        let size = entry.size();
                        let last_modified = entry
                            .last_modified()
                            .map(|dt| {
                                chrono::NaiveDateTime::try_from(dt)
                                    .map(|ndt| ndt.format("%Y-%m-%d %H:%M:%S").to_string())
                                    .unwrap_or_else(|_| "N/A".to_string())
                            })
                            .unwrap_or_else(|| "N/A".to_string());
                        (size, last_modified)
                    }
                    Err(_) => (0, "N/A".to_string()),
                }
            }
            None => (0, "N/A".to_string()),
        };

        let mimetype = if is_dir {
            String::new()
        } else {
            let ext = child_path.rsplit('.').next().unwrap_or("");
            mime_guess::from_ext(ext)
                .first_raw()
                .unwrap_or("")
                .to_string()
        };

        file_details.push(FileInfo {
            name: display_name,
            is_dir,
            last_modified,
            size,
            mimetype,
            path: child_path.clone(),
        });
    }

    // Sort: directories first, then alphabetical by lowercase name
    file_details.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(file_details)
}

pub fn unarchive_zip_file(
    app_handle: tauri::AppHandle,
    config: ExtractionConfig,
) -> Result<(), String> {
    // Open the ZIP file
    let file = File::open(&config.source).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);
    let mut archive = ZipArchive::new(reader).map_err(|e| e.to_string())?;

    let selected_files = config.selected_files.unwrap_or_default();
    // Total number of files in the archive
    let total_files = if !selected_files.is_empty() {
        selected_files.len()
    } else {
        archive.len()
    };

    // Create progress tracker
    let progress = ExtractionProgress::new(total_files);

    // Create extraction directory if it doesn't exist
    std::fs::create_dir_all(&config.destination)
        .map_err(|e| format!("Failed to create extraction directory: {}", e))?;

    // Extract files with progress tracking and cancellation support
    let mut files_extracted_count = 0; // Track actual extracted files
    for i in 0..archive.len() {
        // Check for cancellation
        if config.cancel.load(Ordering::Relaxed) {
            return Err("Extraction cancelled".to_string());
        }

        let password_bytes = config.password.as_ref().map(|p| p.as_bytes());
        let mut file = match archive.by_index_decrypt(i, password_bytes.as_deref().unwrap_or(&[])) {
            Ok(file) => file,
            Err(e) => {
                // Check if this is a password-related error
                let error_msg = e.to_string();
                if error_msg.contains("password") || error_msg.contains("encrypted") {
                    return Err(format!("Password required or incorrect for this archive: {}", error_msg));
                }
                // Log and continue with next file for other errors
                eprintln!("Failed to extract file {}: {}", i, e);
                continue;
            }
        };

        // Check if the file path is in the selected files
        if !selected_files.is_empty() {
            let file_path = match file.enclosed_name() {
                Some(path) => path.to_string_lossy().to_string(),
                None => {
                    eprintln!("Skipping unsafe entry {}", i);
                    continue;
                }
            };
            if !selected_files
                .iter()
                .any(|selected| file_path.starts_with(selected))
            {
                println!("Skipping file: {}", file_path);
                continue;
            }
        }

        // Construct full file path
        let outpath = match file.enclosed_name() {
            Some(path) => Path::new(&config.destination).join(path),
            None => continue,
        };

        // Create parent directories if they don't exist
        if let Some(p) = outpath.parent() {
            std::fs::create_dir_all(p).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Skip if it's a directory
        if file.is_dir() {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
            continue;
        }

        // Write file
        let mut outfile = match File::create(&outpath) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Failed to create output file: {}", e);
                continue;
            }
        };

        // Copy file contents
        match io::copy(&mut file, &mut outfile) {
            Ok(_) => {
                // Increment the actual extracted file counter
                files_extracted_count += 1;

                // Update and emit progress
                progress.update(files_extracted_count);
                eprint!("Extracted {}/{} files\n", files_extracted_count, total_files);

                // Emit progress event with file count and percentage
                let progress_value = progress.get();
                let progress_data = serde_json::json!({
                    "files": files_extracted_count,
                    "total": total_files,
                    "percentage": progress_value
                });
                let _ = app_handle.emit("extract-progress", progress_data);
            }
            Err(e) => {
                eprintln!("Failed to write file: {}", e);
            }
        }
    }

    Ok(())
}

pub fn view_file_in_zip(
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

    // Open the archive
    let file = File::open(&source).map_err(|e| format!("Failed to open archive: {}", e))?;
    let reader = BufReader::new(file);
    let mut archive =
        ZipArchive::new(reader).map_err(|e| format!("Failed to read archive: {}", e))?;

    // Find entry index by name
    let password_bytes = password.as_ref().map(|p| p.as_bytes());
    let target_index = (0..archive.len())
        .find(|&i| {
            archive
                .by_index_decrypt(i, password_bytes.as_deref().unwrap_or(&[]))
                .map(|e| e.name() == file_name_clean)
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("File not found in archive: {}", file_name_clean))?;

    let mut file_in_zip = archive
        .by_index_decrypt(target_index, password_bytes.as_deref().unwrap_or(&[]))
        .map_err(|e| format!("Failed to read file from archive: {}", e))?;

    // Create a temporary file
    let temp_dir = &state.temp_dir;
    let mut rng = rand::thread_rng();

    // Extract file extension from the actual found path
    let actual_file_path = file_in_zip.name();
    let extension = Path::new(actual_file_path)
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

    // Write the file to the temporary directory
    let mut temp_file =
        File::create(&temp_file_path).map_err(|e| format!("Failed to create temp file: {}", e))?;
    let bytes_copied = std::io::copy(&mut file_in_zip, &mut temp_file)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;

    eprintln!("[DEBUG] Extracted {} bytes to temp file", bytes_copied);

    // Open the file with the default application
    that_detached(&temp_file_path).map_err(|e| format!("Failed to open file: {}", e))?;

    // Clean up the temp file after opening with that_detached
    // let _ = std::fs::remove_file(&temp_file_path);

    // Return the path to the temporary file
    Ok(temp_file_path.to_string_lossy().to_string())
}

pub fn get_image_preview_from_zip(
    archive_path: String,
    file_path: String,
    password: Option<String>,
) -> Result<String, String> {
    // Validate file path to prevent directory traversal attacks
    if file_path.contains("..") {
        return Err("Invalid file path (path traversal detected)".to_string());
    }

    let file_path_clean = file_path.trim();
    if file_path_clean.is_empty() {
        return Err("Empty file path provided".to_string());
    }

    // Check if it's an image file
    if !is_image_file_zip(file_path_clean) {
        return Err("File is not a supported image format. Supported formats: jpg, jpeg, png, gif, webp, bmp, tiff, tif".to_string());
    }

    // Open the archive
    let file = File::open(&archive_path).map_err(|e| format!("Failed to open archive: {}", e))?;
    let reader = BufReader::new(file);
    let mut archive =
        ZipArchive::new(reader).map_err(|e| format!("Failed to read archive: {}", e))?;

    // Find entry index by name
    let password_bytes = password.as_ref().map(|p| p.as_bytes());
    let target_index = (0..archive.len())
        .find(|&i| {
            archive
                .by_index_decrypt(i, password_bytes.as_deref().unwrap_or(&[]))
                .map(|e| e.name() == file_path_clean)
                .unwrap_or(false)
        })
        .ok_or_else(|| format!("File not found in archive: {}", file_path_clean))?;

    // Get the file from the archive
    let mut file_in_zip = archive
        .by_index_decrypt(target_index, password_bytes.as_deref().unwrap_or(&[]))
        .map_err(|e| format!("File not found in archive: {}", e))?;

    // Read the file contents into a vector
    let mut buffer = Vec::new();
    io::copy(&mut file_in_zip, &mut buffer)
        .map_err(|e| format!("Failed to read file from archive: {}", e))?;

    // Encode to base64
    let base64_data = base64_encode(&buffer);

    // Determine MIME type from file extension
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

    // Return as data URL
    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}

// Helper function to check if a file is a supported image
fn is_image_file_zip(file_path: &str) -> bool {
    const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif"];
    let ext = file_path.rsplit('.').next().unwrap_or("").to_lowercase();
    IMAGE_EXTENSIONS.contains(&ext.as_str())
}

// Helper function to encode bytes to base64
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

/// Create ZIP archive from files
pub fn create_zip_archive(
    app_handle: tauri::AppHandle,
    config: crate::data_type::CompressionConfig,
) -> Result<(), String> {
    use crate::data_type::CompressionProgress;
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    // Validate compression level (0-9)
    let compression_level = if config.compression_level > 9 {
        9
    } else {
        config.compression_level
    };

    // Get files to compress
    let files_to_compress = &config.files;
    if files_to_compress.is_empty() {
        return Err("No files specified for compression".to_string());
    }

    // Calculate total files (including directories)
    let total_files = files_to_compress.len();

    // Create progress tracker
    let progress = CompressionProgress::new(total_files);

    // Create output directory if it doesn't exist
    let output_path = Path::new(&config.output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // Create ZIP file
    let zip_file = fs::File::create(&config.output_path)
        .map_err(|e| format!("Failed to create ZIP file: {}", e))?;

    // Create ZIP writer with compression
    let mut zip_writer = zip::ZipWriter::new(zip_file);

    // Configure compression method based on level
    let compression_method = if compression_level == 0 {
        zip::CompressionMethod::Stored
    } else {
        zip::CompressionMethod::Deflated
    };

    // Use basic file options with compression method
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(compression_method)
        .unix_permissions(0o755); // Default permissions

    // Add each file to archive
    for (index, file_path) in files_to_compress.iter().enumerate() {
        // Check for cancellation
        if config.cancel.load(std::sync::atomic::Ordering::Relaxed) {
            return Err("Compression cancelled".to_string());
        }

        let path = Path::new(file_path);

        if !path.exists() {
            eprintln!("File not found: {}", file_path);
            continue;
        }

        // Set current file for progress tracking
        progress.set_current_file(
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
        );

        if path.is_dir() {
            // Add directory to archive with just the directory name
            let dir_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("directory")
                .to_string();
            let dir_entry = format!("{}/", dir_name);
            zip_writer
                .add_directory(&dir_entry, options.clone())
                .map_err(|e| format!("Failed to add directory: {}", e))?;
        } else {
            // Add file to archive with just the filename
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string();
            zip_writer
                .start_file(&file_name, options.clone())
                .map_err(|e| format!("Failed to start file: {}", e))?;

            // Read file contents and write to archive
            let file_content = fs::read(path)
                .map_err(|e| format!("Failed to read file {}: {}", file_path, e))?;

            zip_writer
                .write_all(&file_content)
                .map_err(|e| format!("Failed to write file to archive: {}", e))?;
        }

        // Update progress
        let progress_value = ((index + 1) as f64 / total_files as f64) * 100.0;
        progress.update(index + 1);

        // Emit progress event
        let progress_data = serde_json::json!({
            "percentage": progress_value,
            "files_processed": index + 1,
            "current_file": progress.current_file.lock().unwrap().clone().unwrap_or_default(),
            "compression_ratio": 0.0 // Calculate if needed
        });
        let _ = app_handle.emit("compress-progress", progress_data);
    }

    // Finish the archive
    zip_writer
        .finish()
        .map_err(|e| format!("Failed to finish archive: {}", e))?;

    // Emit completion event
    let _ = app_handle.emit("compress-complete", ());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{INVALID_PASSWORD_PREFIX, PASSWORD_REQUIRED_PREFIX};
    use std::io::Write;
    use tempfile::TempDir;
    use zip::write::SimpleFileOptions;

    fn write_zip(path: &Path, encrypted: bool) {
        let file = File::create(path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options = if encrypted {
            SimpleFileOptions::default().with_aes_encryption(zip::AesMode::Aes256, "secret")
        } else {
            SimpleFileOptions::default()
        };
        writer.start_file("hello.txt", options).unwrap();
        writer.write_all(b"hello world").unwrap();
        writer.finish().unwrap();
    }

    #[test]
    fn check_zip_password_ok_for_unencrypted_archive() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("plain.zip");
        write_zip(&path, false);
        assert_eq!(check_zip_password(path.to_str().unwrap(), None), Ok(()));
    }

    #[test]
    fn check_zip_password_requires_password_for_encrypted_archive() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("enc.zip");
        write_zip(&path, true);
        let err = check_zip_password(path.to_str().unwrap(), None).unwrap_err();
        assert!(err.starts_with(PASSWORD_REQUIRED_PREFIX), "got: {err}");
    }

    #[test]
    fn check_zip_password_rejects_wrong_password() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("enc.zip");
        write_zip(&path, true);
        let err = check_zip_password(path.to_str().unwrap(), Some("wrong")).unwrap_err();
        assert!(err.starts_with(INVALID_PASSWORD_PREFIX), "got: {err}");
    }

    #[test]
    fn check_zip_password_accepts_correct_password() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("enc.zip");
        write_zip(&path, true);
        assert_eq!(
            check_zip_password(path.to_str().unwrap(), Some("secret")),
            Ok(())
        );
    }
}
