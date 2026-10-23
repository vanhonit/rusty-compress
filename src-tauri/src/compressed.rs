//! Single-file compressed streams: `name.gz` and `name.bz2` that wrap exactly one
//! file (not a tar). Listing exposes one entry named after the stream minus its
//! final `.gz` / `.bz2`.
//!
//! Only the Tauri-facing wrappers (`unarchive_compressed_file`,
//! `view_file_in_compressed`) touch Tauri types; everything else is plain Rust
//! so it can be unit-tested.

use bzip2::read::MultiBzDecoder;
use flate2::read::MultiGzDecoder;
use open::that_detached;
use rand::Rng;
use std::fs::{self, File};
use std::io::{self, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tauri::{AppHandle, Emitter, State};

use crate::data_type::{AppTempDir, ExtractionConfig, FileInfo};
use crate::utils::{detect_archive_type, is_single_compressed, ArchiveType};

const COPY_BUFFER_SIZE: usize = 64 * 1024;
/// Largest decompressed file the preview command will load into memory.
const MAX_PREVIEW_BYTES: u64 = 50 * 1024 * 1024;

/// Strip `suffix` from the end of `s` if it matches case-insensitively.
/// Works on bytes so non-ASCII names with odd lowercasing cannot misalign.
fn strip_suffix_ci<'a>(s: &'a str, suffix: &str) -> Option<&'a str> {
    if s.len() < suffix.len() {
        return None;
    }
    let split = s.len() - suffix.len();
    if !s.is_char_boundary(split) {
        return None;
    }
    if s[split..].eq_ignore_ascii_case(suffix) {
        Some(&s[..split])
    } else {
        None
    }
}

/// Name of the single file stored inside `archive_path`: the file name with its
/// final `.gz` / `.bz2` removed. Falls back to `file` when nothing is left
/// (for example `.gz`).
pub fn inner_name(archive_path: &str) -> String {
    let file_name = Path::new(archive_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let stem = strip_suffix_ci(file_name, ".gz")
        .or_else(|| strip_suffix_ci(file_name, ".bz2"))
        .unwrap_or(file_name);
    if stem.is_empty() {
        "file".to_string()
    } else {
        stem.to_string()
    }
}

/// Reject inner names that are not a bare file name.
fn check_inner_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name == ".." || name == "." || name.contains('/') || name.contains('\\') {
        return Err(format!("Unsafe file name in compressed archive: {}", name));
    }
    Ok(())
}

/// Map the archive path to its single-file compression type (gz or bz2 only).
fn compressed_type(path: &Path) -> Result<ArchiveType, String> {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    match detect_archive_type(name) {
        Some(t) if is_single_compressed(t) => Ok(t),
        _ => Err("Unsupported archive type".to_string()),
    }
}

fn label_for(archive_type: ArchiveType) -> &'static str {
    match archive_type {
        ArchiveType::Gz => "gzip",
        ArchiveType::Bz2 => "bzip2",
        _ => "compressed",
    }
}

/// Open the file and wrap it in a multi-member decoder, so concatenated
/// gzip / bzip2 streams are read to the end.
fn open_stream(path: &Path, archive_type: ArchiveType) -> Result<Box<dyn Read>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open archive: {}", e))?;
    match archive_type {
        ArchiveType::Gz => Ok(Box::new(MultiGzDecoder::new(file))),
        ArchiveType::Bz2 => Ok(Box::new(MultiBzDecoder::new(file))),
        _ => Err("Unsupported archive type".to_string()),
    }
}

/// Confirm the file really is gzip / bzip2 by decoding its first block.
/// A plain-text file named `x.gz` fails here with a clear message.
fn validate_stream(path: &Path, archive_type: ArchiveType) -> Result<(), String> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let label = label_for(archive_type);
    let len = fs::metadata(path)
        .map_err(|e| format!("Failed to read archive metadata: {}", e))?
        .len();
    if len == 0 {
        return Err(format!("Not a valid {} file (empty): {}", label, name));
    }
    let mut reader = open_stream(path, archive_type)?;
    let mut probe = [0u8; 1];
    reader
        .read(&mut probe)
        .map_err(|e| format!("Not a valid {} file: {} ({})", label, name, e))?;
    Ok(())
}

/// Uncompressed size from the gzip ISIZE trailer (last 4 bytes, little-endian,
/// modulo 2^32). This is the size of the *last* member only, so it is wrong for
/// multi-member files and for files over 4 GiB. bzip2 has no such trailer.
fn gzip_isize(path: &Path) -> Result<u64, String> {
    let mut file = File::open(path).map_err(|e| format!("Failed to open archive: {}", e))?;
    let len = file
        .metadata()
        .map_err(|e| format!("Failed to read archive metadata: {}", e))?
        .len();
    if len < 4 {
        return Err("Not a valid gzip file: too short".to_string());
    }
    file.seek(SeekFrom::End(-4))
        .map_err(|e| format!("Failed to read gzip trailer: {}", e))?;
    let mut trailer = [0u8; 4];
    file.read_exact(&mut trailer)
        .map_err(|e| format!("Failed to read gzip trailer: {}", e))?;
    Ok(u32::from_le_bytes(trailer) as u64)
}

/// Format the archive file's mtime like tar.rs does (`%Y-%m-%d %H:%M:%S`, or `N/A`).
fn modified_string(path: &Path) -> String {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .and_then(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "N/A".to_string())
}

/// List the single entry of a `.gz` / `.bz2` stream. Only the root level exists;
/// any non-empty `file_path` yields an empty listing.
pub fn get_compressed_details(
    source: PathBuf,
    file_path: Option<String>,
) -> Result<Vec<FileInfo>, String> {
    if file_path.as_deref().is_some_and(|p| !p.is_empty()) {
        return Ok(Vec::new());
    }

    let archive_type = compressed_type(&source)?;
    validate_stream(&source, archive_type)?;

    let size = match archive_type {
        ArchiveType::Gz => gzip_isize(&source)?,
        _ => 0,
    };

    let name = inner_name(&source.to_string_lossy());
    let ext = name.rsplit('.').next().unwrap_or("");
    let mimetype = mime_guess::from_ext(ext)
        .first_raw()
        .unwrap_or("")
        .to_string();

    Ok(vec![FileInfo {
        name: name.clone(),
        is_dir: false,
        last_modified: modified_string(&source),
        size,
        mimetype,
        path: name,
    }])
}

/// Copy `reader` to `writer` in 64 KiB chunks, checking `cancel` between chunks.
fn copy_cancellable(
    reader: &mut dyn Read,
    writer: &mut dyn Write,
    cancel: &AtomicBool,
) -> Result<u64, String> {
    let mut buf = vec![0u8; COPY_BUFFER_SIZE];
    let mut total = 0u64;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("Extraction cancelled".to_string());
        }
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(format!("Failed to read compressed data: {}", e)),
        };
        writer
            .write_all(&buf[..n])
            .map_err(|e| format!("Failed to write output file: {}", e))?;
        total += n as u64;
    }
    Ok(total)
}

/// Decompress the single file into `config.destination`, calling
/// `on_progress(1, 1, 100.0)` once it is written. Kept free of Tauri types so it
/// can be unit-tested. On error or cancel the partial output is removed.
pub fn extract_compressed<F>(config: &ExtractionConfig, mut on_progress: F) -> Result<(), String>
where
    F: FnMut(usize, usize, f64),
{
    let source = PathBuf::from(&config.source);
    let archive_type = compressed_type(&source)?;
    let inner = inner_name(&config.source);
    check_inner_name(&inner)?;

    if let Some(selected) = config.selected_files.as_ref() {
        if !selected.is_empty() && !selected.iter().any(|s| s == &inner) {
            return Ok(());
        }
    }

    validate_stream(&source, archive_type)?;

    fs::create_dir_all(&config.destination)
        .map_err(|e| format!("Failed to create extraction directory: {}", e))?;
    let output = Path::new(&config.destination).join(&inner);

    let mut reader = open_stream(&source, archive_type)?;
    let mut out =
        File::create(&output).map_err(|e| format!("Failed to create output file: {}", e))?;

    if let Err(e) = copy_cancellable(&mut reader, &mut out, &config.cancel) {
        drop(out);
        let _ = fs::remove_file(&output);
        return Err(e);
    }
    out.flush()
        .map_err(|e| format!("Failed to write output file: {}", e))?;
    drop(out);

    on_progress(1, 1, 100.0);
    Ok(())
}

pub fn unarchive_compressed_file(
    app_handle: AppHandle,
    config: ExtractionConfig,
) -> Result<(), String> {
    extract_compressed(&config, |files, total, percentage| {
        let progress_data = serde_json::json!({
            "files": files,
            "total": total,
            "percentage": percentage
        });
        let _ = app_handle.emit("extract-progress", progress_data);
    })
}

/// Decompress the single file to a temp file with a random name and open it with
/// the system default app. `file_name` must equal the inner name.
pub fn view_file_in_compressed(
    source: String,
    file_name: String,
    state: State<'_, Arc<AppTempDir>>,
) -> Result<String, String> {
    let source_path = PathBuf::from(&source);
    let archive_type = compressed_type(&source_path)?;
    let inner = inner_name(&source);
    if file_name != inner {
        return Err(format!("File not found in archive: {}", file_name));
    }
    validate_stream(&source_path, archive_type)?;
    let mut reader = open_stream(&source_path, archive_type)?;

    let temp_dir = &state.temp_dir;
    let mut rng = rand::thread_rng();
    let random_name: String = (0..10)
        .map(|_| rng.gen_range(b'a'..=b'z') as char)
        .collect();
    let extension = Path::new(&inner)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    let random_file_name = if extension.is_empty() {
        random_name
    } else {
        format!("{}.{}", random_name, extension)
    };

    let temp_file_path = temp_dir.path().join(random_file_name);
    let mut temp_file = File::create(&temp_file_path)
        .map_err(|e| format!("Failed to create temporary file: {}", e))?;
    if let Err(e) = io::copy(&mut reader, &mut temp_file) {
        drop(temp_file);
        let _ = fs::remove_file(&temp_file_path);
        return Err(format!("Failed to write to temporary file: {}", e));
    }
    drop(temp_file);

    that_detached(&temp_file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    Ok(temp_file_path.to_string_lossy().to_string())
}

fn is_image_file(file_path: &str) -> bool {
    const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "bmp", "tiff", "tif"];
    let ext = file_path.rsplit('.').next().unwrap_or("").to_lowercase();
    IMAGE_EXTENSIONS.contains(&ext.as_str())
}

fn image_mime(file_path: &str) -> &'static str {
    let ext = Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "tiff" | "tif" => "image/tiff",
        _ => "image/unknown",
    }
}

/// Return the decompressed image as a `data:` URL. Refuses files over 50 MiB.
pub fn get_image_preview_from_compressed(
    archive_path: String,
    file_path: String,
) -> Result<String, String> {
    if file_path.contains("..") {
        return Err("Invalid file path (path traversal detected)".to_string());
    }
    let file_path_clean = file_path.trim();
    if file_path_clean.is_empty() {
        return Err("Empty file path provided".to_string());
    }
    if !is_image_file(file_path_clean) {
        return Err("File is not a supported image format. Supported formats: jpg, jpeg, png, gif, webp, bmp, tiff, tif".to_string());
    }

    let source_path = PathBuf::from(&archive_path);
    let archive_type = compressed_type(&source_path)?;
    if file_path_clean != inner_name(&archive_path) {
        return Err(format!("File not found in archive: {}", file_path_clean));
    }
    validate_stream(&source_path, archive_type)?;
    let reader = open_stream(&source_path, archive_type)?;

    let mut buffer = Vec::new();
    reader
        .take(MAX_PREVIEW_BYTES + 1)
        .read_to_end(&mut buffer)
        .map_err(|e| format!("Failed to read file from archive: {}", e))?;
    if buffer.len() as u64 > MAX_PREVIEW_BYTES {
        return Err(format!(
            "Image is larger than the {} MiB preview limit",
            MAX_PREVIEW_BYTES / (1024 * 1024)
        ));
    }

    use base64::Engine as _;
    let base64_data = base64::engine::general_purpose::STANDARD.encode(&buffer);
    Ok(format!(
        "data:{};base64,{}",
        image_mime(file_path_clean),
        base64_data
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use bzip2::write::BzEncoder;
    use flate2::write::GzEncoder;
    use std::sync::atomic::AtomicBool;
    use tempfile::TempDir;

    fn gz_bytes(data: &[u8]) -> Vec<u8> {
        let mut enc = GzEncoder::new(Vec::new(), flate2::Compression::default());
        enc.write_all(data).unwrap();
        enc.finish().unwrap()
    }

    fn bz2_bytes(data: &[u8]) -> Vec<u8> {
        let mut enc = BzEncoder::new(Vec::new(), bzip2::Compression::default());
        enc.write_all(data).unwrap();
        enc.finish().unwrap()
    }

    /// Deterministic, incompressible-ish bytes so the test data is not trivially small.
    fn pseudo_random(len: usize) -> Vec<u8> {
        let mut x: u32 = 0x1234_5678;
        (0..len)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                (x & 0xff) as u8
            })
            .collect()
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

    fn write(path: &Path, bytes: &[u8]) {
        fs::write(path, bytes).unwrap();
    }

    #[test]
    fn inner_name_strips_final_gz_or_bz2_case_insensitively() {
        assert_eq!(inner_name("/x/kubeconfig.gz"), "kubeconfig");
        assert_eq!(inner_name("a.B.GZ"), "a.B");
        assert_eq!(inner_name("notes.txt.bz2"), "notes.txt");
        assert_eq!(inner_name("DATA.Bz2"), "DATA");
        assert_eq!(inner_name(".gz"), "file");
        assert_eq!(inner_name("/x/.bz2"), "file");
    }

    #[test]
    fn check_inner_name_rejects_paths_and_dotdot() {
        assert!(check_inner_name("kubeconfig").is_ok());
        assert!(check_inner_name("..").is_err());
        assert!(check_inner_name("a/b").is_err());
        assert!(check_inner_name("a\\b").is_err());
        assert!(check_inner_name("").is_err());
    }

    #[test]
    fn gz_roundtrip_large_file_is_byte_equal() {
        let tmp = TempDir::new().unwrap();
        let data = pseudo_random(300_000);
        let src = tmp.path().join("kubeconfig.gz");
        write(&src, &gz_bytes(&data));
        let out = tmp.path().join("out");

        let mut last = (0, 0, 0.0);
        extract_compressed(&config(&src, &out, None), |f, t, p| last = (f, t, p)).unwrap();

        assert_eq!(fs::read(out.join("kubeconfig")).unwrap(), data);
        assert_eq!(last, (1, 1, 100.0));
    }

    #[test]
    fn bz2_roundtrip_is_byte_equal() {
        let tmp = TempDir::new().unwrap();
        let data = pseudo_random(150_000);
        let src = tmp.path().join("notes.txt.bz2");
        write(&src, &bz2_bytes(&data));
        let out = tmp.path().join("out");

        extract_compressed(&config(&src, &out, None), |_, _, _| {}).unwrap();

        assert_eq!(fs::read(out.join("notes.txt")).unwrap(), data);
    }

    #[test]
    fn multi_member_gz_yields_all_parts() {
        let tmp = TempDir::new().unwrap();
        let part1 = pseudo_random(70_000);
        let part2 = b"second member\n".repeat(1000);
        let mut bytes = gz_bytes(&part1);
        bytes.extend(gz_bytes(&part2));
        let src = tmp.path().join("multi.gz");
        write(&src, &bytes);
        let out = tmp.path().join("out");

        extract_compressed(&config(&src, &out, None), |_, _, _| {}).unwrap();

        let mut expected = part1.clone();
        expected.extend(&part2);
        assert_eq!(fs::read(out.join("multi")).unwrap(), expected);
    }

    #[test]
    fn multi_member_bz2_yields_all_parts() {
        let tmp = TempDir::new().unwrap();
        let part1 = b"alpha ".repeat(5000);
        let part2 = pseudo_random(20_000);
        let mut bytes = bz2_bytes(&part1);
        bytes.extend(bz2_bytes(&part2));
        let src = tmp.path().join("multi.bz2");
        write(&src, &bytes);
        let out = tmp.path().join("out");

        extract_compressed(&config(&src, &out, None), |_, _, _| {}).unwrap();

        let mut expected = part1.clone();
        expected.extend(&part2);
        assert_eq!(fs::read(out.join("multi")).unwrap(), expected);
    }

    #[test]
    fn fake_gz_fails_listing_and_extract_without_partial_output() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("fake.gz");
        write(&src, b"this is plain text, not gzip\n");
        let out = tmp.path().join("out");

        let err = get_compressed_details(src.clone(), None).err().expect("expected Err");
        assert!(err.starts_with("Not a valid gzip file"), "{err}");

        let err = extract_compressed(&config(&src, &out, None), |_, _, _| {}).unwrap_err();
        assert!(err.starts_with("Not a valid gzip file"), "{err}");
        assert!(!out.join("fake").exists());
    }

    #[test]
    fn fake_bz2_is_rejected() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("fake.bz2");
        write(&src, b"plain text\n");

        let err = get_compressed_details(src.clone(), None).err().expect("expected Err");
        assert!(err.starts_with("Not a valid bzip2 file"), "{err}");
    }

    #[test]
    fn corrupt_body_removes_partial_output() {
        let tmp = TempDir::new().unwrap();
        let mut bytes = gz_bytes(&pseudo_random(200_000));
        // Keep the header and first block valid, corrupt the tail so the
        // failure happens mid-copy, after the output file was created.
        let len = bytes.len();
        for b in &mut bytes[len - 64..] {
            *b ^= 0xA5;
        }
        let src = tmp.path().join("bad.gz");
        write(&src, &bytes);
        let out = tmp.path().join("out");

        let result = extract_compressed(&config(&src, &out, None), |_, _, _| {});
        assert!(result.is_err());
        assert!(!out.join("bad").exists());
    }

    #[test]
    fn gz_details_report_isize_and_one_entry() {
        let tmp = TempDir::new().unwrap();
        let data = pseudo_random(300_000);
        let src = tmp.path().join("kubeconfig.gz");
        write(&src, &gz_bytes(&data));

        let entries = get_compressed_details(src.clone(), None).unwrap();
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.name, "kubeconfig");
        assert_eq!(e.path, "kubeconfig");
        assert!(!e.is_dir);
        assert_eq!(e.size, 300_000);
        assert_ne!(e.last_modified, "");
    }

    #[test]
    fn bz2_details_report_size_zero() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("notes.txt.bz2");
        write(&src, &bz2_bytes(b"hello world"));

        let entries = get_compressed_details(src, None).unwrap();
        assert_eq!(entries[0].name, "notes.txt");
        assert_eq!(entries[0].mimetype, "text/plain");
        assert_eq!(entries[0].size, 0);
    }

    #[test]
    fn details_for_subpath_are_empty() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("kubeconfig.gz");
        write(&src, &gz_bytes(b"apiVersion: v1\n"));

        assert!(get_compressed_details(src.clone(), Some("sub".into()))
            .unwrap()
            .is_empty());
        assert_eq!(
            get_compressed_details(src, Some(String::new()))
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn selected_files_filter_skips_or_writes() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("kubeconfig.gz");
        write(&src, &gz_bytes(b"data"));
        let out = tmp.path().join("out");

        let mut calls = 0;
        extract_compressed(
            &config(&src, &out, Some(vec!["other".to_string()])),
            |_, _, _| calls += 1,
        )
        .unwrap();
        assert_eq!(calls, 0);
        assert!(!out.join("kubeconfig").exists());

        extract_compressed(
            &config(&src, &out, Some(vec!["kubeconfig".to_string()])),
            |_, _, _| calls += 1,
        )
        .unwrap();
        assert_eq!(calls, 1);
        assert_eq!(fs::read(out.join("kubeconfig")).unwrap(), b"data");
    }

    #[test]
    fn preview_returns_data_url_for_image_and_rejects_others() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("pic.png.gz");
        write(&src, &gz_bytes(b"\x89PNG fake"));
        let path = src.to_string_lossy().to_string();

        let url = get_image_preview_from_compressed(path.clone(), "pic.png".into()).unwrap();
        assert!(url.starts_with("data:image/png;base64,"), "{url}");

        assert!(get_image_preview_from_compressed(path.clone(), "../pic.png".into()).is_err());
        assert!(get_image_preview_from_compressed(path, "notes.txt".into()).is_err());
    }

    #[test]
    fn preview_rejects_decompressed_size_over_limit() {
        let tmp = TempDir::new().unwrap();
        let big = vec![0u8; (MAX_PREVIEW_BYTES + 1) as usize];
        let src = tmp.path().join("huge.png.gz");
        write(&src, &gz_bytes(&big));

        let err =
            get_image_preview_from_compressed(src.to_string_lossy().to_string(), "huge.png".into())
                .unwrap_err();
        assert!(err.contains("preview limit"), "{err}");
    }
}
