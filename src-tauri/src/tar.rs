use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use open::that_detached;
use rand::Rng;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tar::{Archive, Builder};
use tauri::{AppHandle, Emitter, State};

use crate::data_type::{
    AppTempDir, CompressionConfig, CompressionProgress, ExtractionConfig, ExtractionProgress,
    FileInfo,
};
use crate::utils::detect_archive_type;

/// Helper function to open a tar file with appropriate decompression
fn open_tar_archive(source_path: &PathBuf) -> Result<Box<dyn std::io::Read>, String> {
    let file = File::open(source_path).map_err(|e| e.to_string())?;
    let file_name = source_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    match detect_archive_type(file_name) {
        Some(crate::utils::ArchiveType::TarGz) | Some(crate::utils::ArchiveType::Tgz) => {
            let decoder = GzDecoder::new(file);
            Ok(Box::new(decoder))
        }
        Some(crate::utils::ArchiveType::TarBz2) | Some(crate::utils::ArchiveType::Tbz2) => {
            let decoder = BzDecoder::new(file);
            Ok(Box::new(decoder))
        }
        _ => {
            // Assume it's an uncompressed tar file
            Ok(Box::new(file))
        }
    }
}

pub fn get_tar_details(
    source_path: PathBuf,
    file_path: Option<String>,
    _state: State<'_, Arc<AppTempDir>>,
) -> Result<Vec<FileInfo>, String> {
    let file_path = file_path.unwrap_or_default();

    // Normalize current_dir to always end with '/' if non-empty
    let current_dir = if file_path.is_empty() {
        String::new()
    } else if file_path.ends_with('/') {
        file_path.clone()
    } else {
        format!("{}/", file_path)
    };

    let reader = open_tar_archive(&source_path)?;
    let mut archive = Archive::new(BufReader::new(reader));

    let mut children: HashMap<String, FileInfo> = HashMap::new();

    for entry in archive.entries().map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?;
        let path_str = path.to_string_lossy().replace('\\', "/");

        // Skip if this entry IS the current directory
        if !current_dir.is_empty()
            && path_str.trim_end_matches('/') == current_dir.trim_end_matches('/')
        {
            continue;
        }
        // Skip entries that are outside the current directory
        if !current_dir.is_empty() && !path_str.starts_with(&current_dir) {
            continue;
        }

        let relative = &path_str[current_dir.len()..];

        // Skip empty relative paths
        if relative.is_empty() {
            continue;
        }

        match relative.find('/') {
            Some(pos) => {
                let dir_name = &relative[..pos];
                if dir_name.is_empty() {
                    continue;
                }
                let child_key = dir_name.to_string();
                let child_path = format!("{}{}/", current_dir, dir_name);

                if pos + 1 == relative.len() {
                    // This IS the actual directory entry (e.g. "folder/")
                    let last_modified = match entry.header().mtime() {
                        Ok(mtime) => chrono::DateTime::from_timestamp(mtime as i64, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_else(|| "N/A".to_string()),
                        Err(_) => "N/A".to_string(),
                    };
                    children.insert(
                        child_key,
                        FileInfo {
                            name: dir_name.to_string(),
                            is_dir: true,
                            last_modified,
                            size: 0,
                            mimetype: String::new(),
                            path: child_path,
                        },
                    );
                } else {
                    // A file lives inside this sub-directory — infer the directory entry
                    children.entry(child_key).or_insert(FileInfo {
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
                // Direct child of current_dir
                let child_name = relative.trim_end_matches('/').to_string();
                if child_name.is_empty() {
                    continue;
                }
                let is_dir = entry.header().entry_type().is_dir();
                let size = entry.header().size().unwrap_or(0);
                let last_modified = match entry.header().mtime() {
                    Ok(mtime) => chrono::DateTime::from_timestamp(mtime as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "N/A".to_string()),
                    Err(_) => "N/A".to_string(),
                };
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
                children.entry(child_name.clone()).or_insert(FileInfo {
                    name: child_name,
                    is_dir,
                    last_modified,
                    size,
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

    let mut file_details: Vec<FileInfo> = children.into_values().collect();

    // Sort: directories first, then alphabetical by lowercase name
    file_details.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(file_details)
}

pub fn unarchive_tar_file(app_handle: AppHandle, config: ExtractionConfig) -> Result<(), String> {
    extract_tar(&config, |files, total, percentage| {
        let progress_data = serde_json::json!({
            "files": files,
            "total": total,
            "percentage": percentage
        });
        let _ = app_handle.emit("extract-progress", progress_data);
    })
}

/// Extract a tar / tar.gz / tar.bz2 archive, calling `on_progress(files, total,
/// percentage)` after each unpacked entry. Kept free of Tauri types so it can
/// be unit-tested.
fn extract_tar<F>(config: &ExtractionConfig, mut on_progress: F) -> Result<(), String>
where
    F: FnMut(usize, usize, f64),
{
    let source_path = PathBuf::from(&config.source);
    let selected_files = config.selected_files.clone().unwrap_or_default();
    let is_selected = |path_str: &str| {
        selected_files.is_empty()
            || selected_files
                .iter()
                .any(|selected| path_str.starts_with(selected.as_str()))
    };

    // Pass 1: count entries for progress. Entries share one sequential stream,
    // so they must never be collected and unpacked later (their data would
    // already have been consumed); counting needs its own reader.
    let total_files = {
        let mut archive = Archive::new(BufReader::new(open_tar_archive(&source_path)?));
        let mut count = 0;
        for entry in archive.entries().map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path().map_err(|e| e.to_string())?;
            if is_selected(&path.to_string_lossy()) {
                count += 1;
            }
        }
        count
    };

    let progress = ExtractionProgress::new(total_files);

    // Create extraction directory if it doesn't exist
    std::fs::create_dir_all(&config.destination)
        .map_err(|e| format!("Failed to create extraction directory: {}", e))?;

    // Pass 2: stream and unpack each entry while the reader is positioned on it.
    let mut archive = Archive::new(BufReader::new(open_tar_archive(&source_path)?));
    let mut file_count = 0;
    for entry_result in archive.entries().map_err(|e| e.to_string())? {
        // Check for cancellation
        if config.cancel.load(Ordering::Relaxed) {
            break;
        }

        let mut entry = entry_result.map_err(|e| e.to_string())?;

        let path_str = entry
            .path()
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string();

        // Check if the file path is in the selected files
        if !is_selected(&path_str) {
            println!("Skipping file: {}", path_str);
            continue;
        }

        // Write file contents; unpack_in sanitises the path and creates parent dirs
        let unpacked = entry
            .unpack_in(&config.destination)
            .map_err(|e| format!("Failed to unpack file {}: {}", path_str, e))?;
        if !unpacked {
            eprintln!("Skipping unsafe TAR entry: {}", path_str);
            continue;
        }

        // Update and report progress
        file_count += 1;
        progress.update(file_count);
        on_progress(file_count, total_files, progress.get());
    }

    Ok(())
}

pub fn view_file_in_tar(
    source: String,
    file_name: String,
    state: State<'_, Arc<AppTempDir>>,
) -> Result<String, String> {
    // Open the TAR file with appropriate decompression
    let source_path = PathBuf::from(&source);
    let reader =
        open_tar_archive(&source_path).map_err(|e| format!("Failed to open TAR file: {}", e))?;
    let mut archive = Archive::new(BufReader::new(reader));

    // Locate the target file
    let mut file_in_tar = None;
    for entry in archive
        .entries()
        .map_err(|e| format!("Failed to read TAR archive: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        if entry
            .path()
            .map_err(|e| format!("Failed to get entry path: {}", e))?
            .to_string_lossy()
            == file_name
        {
            file_in_tar = Some(entry);
            break;
        }
    }

    let mut file_in_tar =
        file_in_tar.ok_or_else(|| format!("File not found in TAR archive: {}", file_name))?;

    // Create a temporary file
    let temp_dir = &state.temp_dir;
    let mut rng = rand::thread_rng();
    let random_name: String = (0..10)
        .map(|_| (0x61u8 + (rng.gen::<f32>() * 26.0) as u8) as char)
        .collect();
    let extension = Path::new(&file_name)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    let random_file_name: String = if extension.is_empty() {
        random_name
    } else {
        format!("{}.{}", random_name, extension)
    };

    let temp_file_path = temp_dir.path().join(random_file_name);
    let mut temp_file = File::create(&temp_file_path)
        .map_err(|e| format!("Failed to create temporary file: {}", e))?;

    // Write the file contents to the temporary file
    io::copy(&mut file_in_tar, &mut temp_file)
        .map_err(|e| format!("Failed to write to temporary file: {}", e))?;

    that_detached(&temp_file_path).map_err(|e| format!("Failed to open file: {}", e))?;

    // Return the path to the temporary file
    Ok(temp_file_path.to_string_lossy().to_string())
}

pub fn get_image_preview_from_tar(
    archive_path: String,
    file_path: String,
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
    if !is_image_file_tar(file_path_clean) {
        return Err("File is not a supported image format. Supported formats: jpg, jpeg, png, gif, webp, bmp, tiff, tif".to_string());
    }

    // Open the TAR file with appropriate decompression
    let source_path = PathBuf::from(&archive_path);
    let reader =
        open_tar_archive(&source_path).map_err(|e| format!("Failed to open archive: {}", e))?;
    let mut archive = Archive::new(BufReader::new(reader));

    // Find the target file
    let mut file_in_tar = None;
    for entry in archive
        .entries()
        .map_err(|e| format!("Failed to read TAR archive: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        if entry
            .path()
            .map_err(|e| format!("Failed to get entry path: {}", e))?
            .to_string_lossy()
            == file_path_clean
        {
            file_in_tar = Some(entry);
            break;
        }
    }

    let mut tar_file =
        file_in_tar.ok_or_else(|| format!("File not found in archive: {}", file_path_clean))?;

    // Read the file contents into a vector
    let mut buffer = Vec::new();
    io::copy(&mut tar_file, &mut buffer)
        .map_err(|e| format!("Failed to read file from archive: {}", e))?;

    // Encode to base64
    let base64_data = base64_encode(&buffer);

    // Determine MIME type from file extension
    let ext = Path::new(file_path_clean)
        .extension()
        .and_then(|ext| ext.to_str())
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
fn is_image_file_tar(file_path: &str) -> bool {
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

/// Create uncompressed TAR archive from files
pub fn create_tar_archive(app_handle: AppHandle, config: CompressionConfig) -> Result<(), String> {
    use std::fs;

    let files_to_compress = &config.files;
    if files_to_compress.is_empty() {
        return Err("No files specified for compression".to_string());
    }

    let total_files = files_to_compress.len();
    let progress = CompressionProgress::new(total_files);

    // Create output directory if it doesn't exist
    let output_path = Path::new(&config.output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // Create TAR file
    let tar_file = File::create(&config.output_path)
        .map_err(|e| format!("Failed to create TAR file: {}", e))?;

    let mut tar_builder = Builder::new(tar_file);

    // Add each file to the archive
    for (index, file_path) in files_to_compress.iter().enumerate() {
        // Check for cancellation
        if config.cancel.load(Ordering::Relaxed) {
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
            let dir_path = PathBuf::from(&dir_name);
            tar_builder
                .append_dir(&dir_path, path)
                .map_err(|e| format!("Failed to add directory {}: {}", file_path, e))?;
        } else {
            // Add file to archive with just the filename
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string();
            let file_path_in_archive = PathBuf::from(&file_name);
            let mut file = File::open(&path)
                .map_err(|e| format!("Failed to open file {}: {}", file_path, e))?;
            tar_builder
                .append_file(&file_path_in_archive, &mut file)
                .map_err(|e| format!("Failed to add file {}: {}", file_path, e))?;
        }

        // Update progress
        let progress_value = ((index + 1) as f64 / total_files as f64) * 100.0;
        progress.update(index + 1);

        // Emit progress event
        let progress_data = serde_json::json!({
            "percentage": progress_value,
            "files_processed": index + 1,
            "current_file": progress.current_file.lock().unwrap().clone().unwrap_or_default(),
            "compression_ratio": 0.0
        });
        let _ = app_handle.emit("compress-progress", progress_data);
    }

    // Emit completion event
    let _ = app_handle.emit("compress-complete", ());

    Ok(())
}

/// Create TAR.GZ (gzipped TAR) archive from files
#[allow(dead_code)]
pub fn create_tar_gz_archive(
    app_handle: AppHandle,
    config: CompressionConfig,
) -> Result<(), String> {
    use std::fs;

    let files_to_compress = &config.files;
    if files_to_compress.is_empty() {
        return Err("No files specified for compression".to_string());
    }

    let total_files = files_to_compress.len();
    let progress = CompressionProgress::new(total_files);

    // Create output directory if it doesn't exist
    let output_path = Path::new(&config.output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // Create TAR.GZ file with compression
    let tar_gz_file = File::create(&config.output_path)
        .map_err(|e| format!("Failed to create TAR.GZ file: {}", e))?;

    // Compression level from config (1-9, default 6)
    let compression_level = if config.compression_level > 0 && config.compression_level <= 9 {
        config.compression_level
    } else {
        6
    };

    let gz_encoder = GzEncoder::new(
        tar_gz_file,
        flate2::Compression::new(compression_level as u32),
    );
    let mut tar_builder = Builder::new(gz_encoder);

    // Add each file to the archive
    for (index, file_path) in files_to_compress.iter().enumerate() {
        // Check for cancellation
        if config.cancel.load(Ordering::Relaxed) {
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
            let dir_path = PathBuf::from(&dir_name);
            tar_builder
                .append_dir(&dir_path, path)
                .map_err(|e| format!("Failed to add directory {}: {}", file_path, e))?;
        } else {
            // Add file to archive with just the filename
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string();
            let file_path_in_archive = PathBuf::from(&file_name);
            let mut file = File::open(&path)
                .map_err(|e| format!("Failed to open file {}: {}", file_path, e))?;
            tar_builder
                .append_file(&file_path_in_archive, &mut file)
                .map_err(|e| format!("Failed to add file {}: {}", file_path, e))?;
        }

        // Update progress
        let progress_value = ((index + 1) as f64 / total_files as f64) * 100.0;
        progress.update(index + 1);

        // Emit progress event
        let progress_data = serde_json::json!({
            "percentage": progress_value,
            "files_processed": index + 1,
            "current_file": progress.current_file.lock().unwrap().clone().unwrap_or_default(),
            "compression_ratio": 0.0
        });
        let _ = app_handle.emit("compress-progress", progress_data);
    }

    // Emit completion event
    let _ = app_handle.emit("compress-complete", ());

    Ok(())
}

/// Create TAR.BZ2 (bzip2-compressed TAR) archive from files
#[allow(dead_code)]
pub fn create_tar_bz2_archive(
    app_handle: AppHandle,
    config: CompressionConfig,
) -> Result<(), String> {
    use std::fs;

    let files_to_compress = &config.files;
    if files_to_compress.is_empty() {
        return Err("No files specified for compression".to_string());
    }

    let total_files = files_to_compress.len();
    let progress = CompressionProgress::new(total_files);

    // Create output directory if it doesn't exist
    let output_path = Path::new(&config.output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // Create TAR.BZ2 file with compression
    let tar_bz2_file = File::create(&config.output_path)
        .map_err(|e| format!("Failed to create TAR.BZ2 file: {}", e))?;

    // Compression level from config (1-9, default 6)
    let compression_level = if config.compression_level > 0 && config.compression_level <= 9 {
        config.compression_level
    } else {
        6
    };

    let bz_encoder = BzEncoder::new(
        tar_bz2_file,
        bzip2::Compression::new(compression_level as u32),
    );
    let mut tar_builder = Builder::new(bz_encoder);

    // Add each file to the archive
    for (index, file_path) in files_to_compress.iter().enumerate() {
        // Check for cancellation
        if config.cancel.load(Ordering::Relaxed) {
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
            let dir_path = PathBuf::from(&dir_name);
            tar_builder
                .append_dir(&dir_path, path)
                .map_err(|e| format!("Failed to add directory {}: {}", file_path, e))?;
        } else {
            // Add file to archive with just the filename
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string();
            let file_path_in_archive = PathBuf::from(&file_name);
            let mut file = File::open(&path)
                .map_err(|e| format!("Failed to open file {}: {}", file_path, e))?;
            tar_builder
                .append_file(&file_path_in_archive, &mut file)
                .map_err(|e| format!("Failed to add file {}: {}", file_path, e))?;
        }

        // Update progress
        let progress_value = ((index + 1) as f64 / total_files as f64) * 100.0;
        progress.update(index + 1);

        // Emit progress event
        let progress_data = serde_json::json!({
            "percentage": progress_value,
            "files_processed": index + 1,
            "current_file": progress.current_file.lock().unwrap().clone().unwrap_or_default(),
            "compression_ratio": 0.0
        });
        let _ = app_handle.emit("compress-progress", progress_data);
    }

    // Emit completion event
    let _ = app_handle.emit("compress-complete", ());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use tempfile::TempDir;

    /// Build a tar with files larger than the 8 KiB read buffer, the case that
    /// broke when entries were collected before unpacking.
    fn write_tar(path: &Path, files: &[(&str, Vec<u8>)]) {
        let mut builder = Builder::new(File::create(path).unwrap());
        for (name, data) in files {
            let mut header = tar::Header::new_gnu();
            header.set_size(data.len() as u64);
            header.set_mode(0o600);
            header.set_cksum();
            builder.append_data(&mut header, name, data.as_slice()).unwrap();
        }
        builder.finish().unwrap();
    }

    fn config(source: &Path, dest: &Path, selected: Option<Vec<String>>) -> ExtractionConfig {
        ExtractionConfig {
            source: source.to_string_lossy().to_string(),
            destination: dest.to_string_lossy().to_string(),
            selected_files: selected,
            cancel: Arc::new(AtomicBool::new(false)),
            password: None,
        }
    }

    fn sample_files() -> Vec<(&'static str, Vec<u8>)> {
        vec![
            ("big.bin", (0..10_138u32).map(|i| (i % 251) as u8).collect()),
            ("dir/second.txt", vec![b'x'; 20_000]),
            ("small.txt", b"hello".to_vec()),
        ]
    }

    #[test]
    fn extract_tar_writes_full_contents_of_large_entries() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("in.tar");
        let files = sample_files();
        write_tar(&src, &files);
        let out = tmp.path().join("out");

        let mut last = (0, 0, 0.0);
        extract_tar(&config(&src, &out, None), |f, t, p| last = (f, t, p)).unwrap();

        for (name, data) in &files {
            assert_eq!(&std::fs::read(out.join(name)).unwrap(), data, "{name}");
        }
        assert_eq!(last, (3, 3, 100.0));
    }

    #[test]
    fn extract_tar_gz_and_selected_files() {
        let tmp = TempDir::new().unwrap();
        let raw = tmp.path().join("in.tar");
        let files = sample_files();
        write_tar(&raw, &files);
        let gz = tmp.path().join("in.tar.gz");
        let mut enc = GzEncoder::new(File::create(&gz).unwrap(), flate2::Compression::default());
        io::copy(&mut File::open(&raw).unwrap(), &mut enc).unwrap();
        enc.finish().unwrap();
        let out = tmp.path().join("out");

        let mut count = 0;
        extract_tar(
            &config(&gz, &out, Some(vec!["dir/".to_string()])),
            |f, t, _| count = f.max(t),
        )
        .unwrap();

        assert_eq!(std::fs::read(out.join("dir/second.txt")).unwrap(), files[1].1);
        assert!(!out.join("big.bin").exists());
        assert!(!out.join("small.txt").exists());
        assert_eq!(count, 1);
    }
}
