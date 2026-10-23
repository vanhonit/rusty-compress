// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod compressed;
mod data_type;
mod file_manager;
mod finder;
mod rar;
mod sevenz;
mod tar;
mod utils;
mod zip;
use std::sync::Mutex;

use std::fs;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
// use zip::ZipArchive;
use data_type::{
    AppTempDir, ArchiveFormat, CompressionConfig, ExtractionConfig, FileInfo, SingleChecksum,
};
use file_manager::{open_file, read_directory};
use rar::get_image_preview_from_rar;
use rar::get_rar_details;
use rar::unarchive_rar_file;
use rar::view_file_in_rar;
use sevenz::create_sevenz_archive;
use std::thread;
use tar::create_tar_archive;
use tar::get_image_preview_from_tar;
use tar::view_file_in_tar;
use tauri::{AppHandle, Emitter, Listener, Manager, RunEvent, WebviewWindowBuilder};
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};
use tempfile::TempDir;
use compressed::{
    get_compressed_details, get_image_preview_from_compressed, unarchive_compressed_file,
    view_file_in_compressed,
};
use utils::{
    detect_archive_type, is_rar_based, is_single_compressed, is_tar_based, is_zip_based,
};
use zip::create_zip_archive;
use zip::get_image_preview_from_zip;
use zip::view_file_in_zip;

#[tauri::command]
fn path_exists(path: String) -> bool {
    PathBuf::from(path).exists()
}

#[tauri::command]
async fn create_archive(
    app_handle: AppHandle,
    files: Vec<String>,
    output_path: String,
    archive_format: String,
    compression_level: u8,
) -> Result<(), String> {
    // Convert string format to ArchiveFormat enum
    let format = match archive_format.as_str() {
        "zip" => ArchiveFormat::Zip,
        "tar" => ArchiveFormat::Tar,
        "7z" => ArchiveFormat::Sevenz,
        _ => return Err(format!("Unsupported archive format: {}", archive_format)),
    };

    // Create compression configuration
    let config = CompressionConfig {
        files,
        output_path,
        archive_format: format,
        compression_level,
        cancel: Arc::new(AtomicBool::new(false)),
    };

    // Route to appropriate archive creation function
    match format {
        ArchiveFormat::Zip => {
            tauri::async_runtime::spawn_blocking(move || create_zip_archive(app_handle, config))
                .await
                .map_err(|e| format!("Archive creation failed: {}", e))?
        }
        ArchiveFormat::Tar => {
            tauri::async_runtime::spawn_blocking(move || create_tar_archive(app_handle, config))
                .await
                .map_err(|e| format!("Archive creation failed: {}", e))?
        }
        ArchiveFormat::Sevenz => {
            tauri::async_runtime::spawn_blocking(move || create_sevenz_archive(app_handle, config))
                .await
                .map_err(|e| format!("Archive creation failed: {}", e))?
        }
    }
}

#[tauri::command]
fn archive_file_details(
    source_path: String,
    file_path: Option<String>,
    password: Option<String>,
    state: tauri::State<'_, Arc<AppTempDir>>,
) -> Result<Vec<FileInfo>, String> {
    let source = PathBuf::from(source_path.clone());

    // Use the archive type detector to identify the archive format
    let archive_type =
        detect_archive_type(&source_path).ok_or_else(|| "Unsupported archive type".to_string())?;

    if is_zip_based(archive_type) {
        zip::get_zip_details(source, file_path, password, state).map_err(|e| e.to_string())
    } else if is_tar_based(archive_type) {
        tar::get_tar_details(source, file_path, state).map_err(|e| e.to_string())
    } else if is_single_compressed(archive_type) {
        get_compressed_details(source, file_path)
    } else if is_rar_based(archive_type) {
        get_rar_details(source, file_path, password, state).map_err(|e| e.to_string())
    } else {
        Err("Unsupported archive type".to_string())
    }
}

/// Check the password against the archive before any extraction UI is shown.
/// Returns Err with a PASSWORD_REQUIRED / INVALID_PASSWORD prefix on failure.
fn verify_extraction_password(archive_path: &str, password: Option<&str>) -> Result<(), String> {
    let archive_type = detect_archive_type(archive_path)
        .ok_or_else(|| "Unsupported archive type".to_string())?;

    if is_zip_based(archive_type) {
        zip::check_zip_password(archive_path, password)
    } else if is_rar_based(archive_type) {
        rar::check_rar_password(archive_path, password)
    } else if is_tar_based(archive_type) || is_single_compressed(archive_type) {
        Ok(())
    } else {
        Err("Unsupported archive type".to_string())
    }
}

#[tauri::command]
async fn extract_files(
    app_handle: AppHandle,
    archive_path: String,
    output_path: String,
    selected_files: Option<Vec<String>>,
    password: Option<String>,
) -> Result<(), String> {
    // Verify the password before opening the progress window
    {
        let check_path = archive_path.clone();
        let check_password = password.clone();
        tauri::async_runtime::spawn_blocking(move || {
            verify_extraction_password(&check_path, check_password.as_deref())
        })
        .await
        .map_err(|e| e.to_string())??;
    }

    // Unique window label: timestamp + counter, so jobs started in the same
    // millisecond (e.g. multi-select "Extract Here" from Finder) never collide.
    static EXTRACT_SEQ: AtomicU64 = AtomicU64::new(0);
    let label = format!(
        "extract-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
        EXTRACT_SEQ.fetch_add(1, Ordering::Relaxed)
    );
    let webview_url = tauri::WebviewUrl::App(
        format!(
            "index.html?label=progress&name={}&fileName={}",
            label, archive_path
        )
        .into(),
    );

    // // Create the progress window dynamically
    let _progress_window = WebviewWindowBuilder::new(
        &app_handle,
        &label,
        webview_url.clone(), // The window label
    )
    .title("Extraction Progress")
    .resizable(false)
    .inner_size(450.0, 300.0)
    .build()
    .map_err(|e| e.to_string())?;

    // Prepare extraction configuration
    let config = ExtractionConfig {
        source: archive_path.clone(),
        destination: output_path,
        selected_files,
        cancel: Arc::new(AtomicBool::new(false)),
        password,
    };

    // Clone handles for thread communication
    let app_handle_clone = app_handle.clone();
    let config_clone = config.clone();
    let window_label = label.clone();
    thread::spawn(move || {
        app_handle_clone.listen(format!("{}://ready", label), {
            let app_handle_clone = app_handle_clone.clone();
            move |_| {
                println!("Window is ready: {}", window_label);
                let app_handle_clone = app_handle_clone.clone();
                let config_clone = config_clone.clone();

                // Use the archive type detector to identify the archive format
                let archive_type = match detect_archive_type(&config_clone.source) {
                    Some(archive_type) => archive_type,
                    None => {
                        let _ = app_handle
                            .emit("extract-error", "Unsupported archive type".to_string());
                        return;
                    }
                };

                if is_zip_based(archive_type) {
                    match zip::unarchive_zip_file(app_handle_clone, config_clone) {
                        Ok(_) => {
                            // Emit completion event
                            let _ = app_handle.emit("extract-complete", ());
                        }
                        Err(e) => {
                            // Emit error event
                            let _ = app_handle.emit("extract-error", e);
                        }
                    }
                } else if is_tar_based(archive_type) {
                    match tar::unarchive_tar_file(app_handle_clone, config_clone) {
                        Ok(_) => {
                            // Emit completion event
                            let _ = app_handle.emit("extract-complete", ());
                        }
                        Err(e) => {
                            // Emit error event
                            let _ = app_handle.emit("extract-error", e);
                        }
                    }
                } else if is_single_compressed(archive_type) {
                    match unarchive_compressed_file(app_handle_clone, config_clone) {
                        Ok(_) => {
                            let _ = app_handle.emit("extract-complete", ());
                        }
                        Err(e) => {
                            let _ = app_handle.emit("extract-error", e);
                        }
                    }
                } else if is_rar_based(archive_type) {
                    match unarchive_rar_file(app_handle_clone, config_clone) {
                        Ok(_) => {
                            // Emit completion event
                            let _ = app_handle.emit("extract-complete", ());
                        }
                        Err(e) => {
                            // Emit error event
                            let _ = app_handle.emit("extract-error", e);
                        }
                    }
                } else {
                    // Emit error event for unsupported archive type
                    let _ =
                        app_handle.emit("extract-error", "Unsupported archive type".to_string());
                }
            }
        });
    })
    .join()
    .unwrap();

    Ok(())
}

#[tauri::command]
fn view_file_in_archive(
    archive_path: String,
    file_name: String,
    password: Option<String>,
    state: tauri::State<'_, Arc<AppTempDir>>,
) -> Result<(), String> {
    // Use the archive type detector to identify the archive format
    let archive_type = match detect_archive_type(&archive_path) {
        Some(archive_type) => archive_type,
        None => return Err("Unsupported archive type".to_string()),
    };

    if is_zip_based(archive_type) {
        view_file_in_zip(archive_path, file_name, password, state)?;
    } else if is_tar_based(archive_type) {
        view_file_in_tar(archive_path, file_name, state)?;
    } else if is_single_compressed(archive_type) {
        view_file_in_compressed(archive_path, file_name, state)?;
    } else if is_rar_based(archive_type) {
        view_file_in_rar(archive_path, file_name, password, state)?;
    }
    Ok(())
}

#[tauri::command]
fn get_image_preview(archive_path: String, file_path: String, password: Option<String>) -> Result<String, String> {
    // Use the archive type detector to identify the archive format
    let archive_type = match detect_archive_type(&archive_path) {
        Some(archive_type) => archive_type,
        None => {
            return Err("Unsupported archive type. Supported formats: .zip, .jar, .apk, .xapk, .ipa, .tar, .tar.gz, .tgz, .tar.bz2, .tbz2, .gz, .bz2".to_string())
        }
    };

    if is_zip_based(archive_type) {
        get_image_preview_from_zip(archive_path, file_path, password)
    } else if is_tar_based(archive_type) {
        get_image_preview_from_tar(archive_path, file_path)
    } else if is_single_compressed(archive_type) {
        get_image_preview_from_compressed(archive_path, file_path)
    } else if is_rar_based(archive_type) {
        get_image_preview_from_rar(archive_path, file_path, password)
    } else {
        Err("Unsupported archive type. Supported formats: .zip, .jar, .apk, .xapk, .ipa, .tar, .tar.gz, .tgz, .tar.bz2, .tbz2, .gz, .bz2, .rar".to_string())
    }
}

#[tauri::command]
async fn compute_checksum(
    app_handle: AppHandle,
    file_path: String,
    algorithm: String,
) -> Result<SingleChecksum, String> {
    let path = PathBuf::from(&file_path);
    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }
    if !path.is_file() {
        return Err(format!("Path is not a file: {}", file_path));
    }

    let file_size = fs::metadata(&path)
        .map_err(|e| format!("Failed to read file metadata: {}", e))?
        .len();

    let algo = algorithm.to_lowercase();
    let event_name = format!("checksum-progress-{}", algo);

    tauri::async_runtime::spawn_blocking(move || {
        let file = fs::File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut reader = BufReader::with_capacity(131072, file);
        let mut buffer = [0u8; 131072];

        let mut bytes_processed: u64 = 0;
        let mut last_reported_percent: u8 = 0;

        let hash = match algo.as_str() {
            "md5" => {
                let mut hasher = md5::Context::new();
                loop {
                    let n = reader
                        .read(&mut buffer)
                        .map_err(|e| format!("Failed to read file: {}", e))?;
                    if n == 0 {
                        break;
                    }
                    hasher.consume(&buffer[..n]);
                    bytes_processed += n as u64;
                    if file_size > 0 {
                        let pct = ((bytes_processed as f64 / file_size as f64) * 100.0) as u8;
                        if pct >= last_reported_percent + 5 {
                            last_reported_percent = pct;
                            let _ = app_handle.emit(&event_name, pct);
                        }
                    }
                }
                format!("{:x}", hasher.compute())
            }
            "sha1" => {
                use sha1::{Digest, Sha1};
                let mut hasher = Sha1::new();
                loop {
                    let n = reader
                        .read(&mut buffer)
                        .map_err(|e| format!("Failed to read file: {}", e))?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                    bytes_processed += n as u64;
                    if file_size > 0 {
                        let pct = ((bytes_processed as f64 / file_size as f64) * 100.0) as u8;
                        if pct >= last_reported_percent + 5 {
                            last_reported_percent = pct;
                            let _ = app_handle.emit(&event_name, pct);
                        }
                    }
                }
                format!("{:x}", hasher.finalize())
            }
            "sha256" => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                loop {
                    let n = reader
                        .read(&mut buffer)
                        .map_err(|e| format!("Failed to read file: {}", e))?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                    bytes_processed += n as u64;
                    if file_size > 0 {
                        let pct = ((bytes_processed as f64 / file_size as f64) * 100.0) as u8;
                        if pct >= last_reported_percent + 5 {
                            last_reported_percent = pct;
                            let _ = app_handle.emit(&event_name, pct);
                        }
                    }
                }
                format!("{:x}", hasher.finalize())
            }
            "sha512" => {
                use sha2::{Digest, Sha512};
                let mut hasher = Sha512::new();
                loop {
                    let n = reader
                        .read(&mut buffer)
                        .map_err(|e| format!("Failed to read file: {}", e))?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                    bytes_processed += n as u64;
                    if file_size > 0 {
                        let pct = ((bytes_processed as f64 / file_size as f64) * 100.0) as u8;
                        if pct >= last_reported_percent + 5 {
                            last_reported_percent = pct;
                            let _ = app_handle.emit(&event_name, pct);
                        }
                    }
                }
                format!("{:x}", hasher.finalize())
            }
            _ => {
                return Err(format!(
                    "Unsupported algorithm: {}. Use md5, sha1, sha256, or sha512",
                    algo
                ))
            }
        };

        let _ = app_handle.emit(&event_name, 100u8);

        Ok(SingleChecksum {
            algorithm: algo,
            hash,
        })
    })
    .await
    .map_err(|e| format!("Checksum task failed: {}", e))?
}

// =========================
// App State
// =========================
struct PendingFiles {
    files: Mutex<Vec<String>>,
    is_ready: Mutex<bool>,
}
// =========================
// Background launches (Finder / CLI extraction)
// =========================
/// True when this launch was started to extract something. Such a launch shows
/// only the progress windows; the main window stays hidden.
static BACKGROUND_REQUEST: AtomicBool = AtomicBool::new(false);
/// Background jobs (folder picker, extractions) that have not finished yet.
static BACKGROUND_PENDING: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn mark_background_request() {
    BACKGROUND_REQUEST.store(true, Ordering::SeqCst);
}

pub(crate) fn show_main_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub(crate) fn begin_background_job() {
    BACKGROUND_PENDING.fetch_add(1, Ordering::SeqCst);
}

/// Finish a background job, then quit if nothing is left to show.
pub(crate) fn end_background_job(app: &AppHandle) {
    BACKGROUND_PENDING.fetch_sub(1, Ordering::SeqCst);
    exit_if_idle_in_background(app, None);
}

/// Quit a background-only launch once no job is pending, no progress window is
/// open, and the main window was never shown. `closing` is the label of a
/// window that is being destroyed and must not count as open.
pub(crate) fn exit_if_idle_in_background(app: &AppHandle, closing: Option<&str>) {
    if !BACKGROUND_REQUEST.load(Ordering::SeqCst) || BACKGROUND_PENDING.load(Ordering::SeqCst) > 0 {
        return;
    }
    let main_hidden = matches!(
        app.get_webview_window("main").map(|w| w.is_visible()),
        Some(Ok(false))
    );
    let progress_open = app
        .webview_windows()
        .keys()
        .any(|label| label.starts_with("extract-") && Some(label.as_str()) != closing);
    if main_hidden && !progress_open {
        app.exit(0);
    }
}

// =========================
// Helper
// =========================
fn push_file(app: &tauri::AppHandle, path: String) {
    // Opening a file always shows the UI, even during a background launch.
    show_main_window(app);
    let state = app.state::<PendingFiles>();
    let is_ready = *state.is_ready.lock().unwrap();
    if is_ready {
        println!("⚡ Emit immediately: {}", path);
        let _ = app.emit("file-open", path);
    } else {
        println!("📦 Buffering: {}", path);
        state.files.lock().unwrap().push(path);
    }
}

// =========================
// Command: frontend pulls files
// =========================

#[tauri::command]
fn take_files(state: tauri::State<PendingFiles>) -> Vec<String> {
    let mut files = state.files.lock().unwrap();
    println!("Fetching files");
    files.drain(..).collect()
}

// === START: CLI feature ===
// handle_cli is the top-level CLI dispatcher called at app startup.
//
// Supported invocations:
//   app <path>                     → open the archive in the UI (show contents)
//   app extract <path>             → extract to archive's parent directory (default)
//   app extract <path> --here      → extract to archive's parent directory
//   app extract <path> --to <dest> → extract to a specific destination directory
pub fn handle_cli(app: AppHandle) {
    let raw_args: Vec<String> = std::env::args().collect();

    match raw_args.get(1).map(|s| s.as_str()) {
        // 'extract' subcommand → delegate to CLI plugin for structured option parsing.
        // cli().matches() is ONLY called here, so clap never sees bare file paths
        // and can never throw "unexpected argument" for them.
        Some("extract") => {
            use tauri_plugin_cli::CliExt;
            if let Ok(matches) = app.cli().matches() {
                if let Some(sub) = matches.subcommand.as_deref() {
                    if sub.name == "extract" {
                        handle_extract(&app, &sub.matches);
                    }
                }
            }
        }

        // Bare file path → open the archive and show its contents in the UI.
        Some(first_arg) if !first_arg.starts_with('-') => {
            let p = PathBuf::from(first_arg);
            if p.exists() && p.is_file() {
                log::info!("Opening archive from CLI: {}", first_arg);
                push_file(&app, first_arg.to_string());
            } else {
                eprintln!("Archive file not found: {}", first_arg);
            }
        }

        // No arguments or flags only → normal app launch, nothing to do.
        _ => {}
    }
}

// handle_extract performs the fast-extraction flow for `app extract <path> [options]`.
fn handle_extract(app: &AppHandle, sub: &tauri_plugin_cli::Matches) {
    // Retrieve the required positional 'source' argument.
    let archive = match sub.args.get("source").and_then(|a| a.value.as_str()) {
        Some(v) => v.to_string(),
        None => {
            eprintln!(
                "Missing archive path. Usage: app extract <archive> [--here | --to <destination>]"
            );
            return;
        }
    };

    // Verify the archive file exists.
    if !PathBuf::from(&archive).exists() {
        eprintln!("Archive file not found: {}", archive);
        return;
    }

    // Determine the destination directory.
    //
    // Priority:
    //   1. --to <dest>  → extract to the given directory
    //   2. (default)    → extract here (archive's parent directory)
    //
    // --here is provided for discoverability but behaves identically to the
    // default, so we do not need to check it separately.
    let dest = match sub
        .args
        .get("to")
        .and_then(|a| a.value.as_str())
        .map(|s| s.to_string())
    {
        Some(dest_str) => {
            let dest_path = PathBuf::from(&dest_str);
            if !dest_path.exists() {
                eprintln!("Destination directory does not exist: {}", dest_str);
                return;
            }
            if !dest_path.is_dir() {
                eprintln!("Destination is not a directory: {}", dest_str);
                return;
            }
            dest_str
        }
        None => {
            // Default: extract to the archive's parent directory (same as --here).
            match PathBuf::from(&archive).parent() {
                Some(p) => p.to_string_lossy().to_string(),
                None => {
                    eprintln!("Failed to determine parent directory for: {}", archive);
                    return;
                }
            }
        }
    };

    println!("🗜️ Starting CLI extraction: {} -> {}", archive, dest);
    log::info!("Starting CLI extraction: {} -> {}", archive, dest);

    // Validation passed: from here on only the progress window should appear.
    mark_background_request();
    begin_background_job();
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        match extract_files(app.clone(), archive.clone(), dest.clone(), None, None).await {
            Ok(_) => {
                println!("✅ CLI extraction completed: {}", archive);
                log::info!("CLI extraction completed: {}", archive);
            }
            Err(e) => {
                eprintln!("❌ CLI extraction failed for {}: {}", archive, e);
                log::error!("CLI extraction failed for {}: {}", archive, e);
            }
        }
        end_background_job(&app);
    });
}
// === END: CLI feature ===

pub fn run() {
    tauri::Builder::default()
        .manage(PendingFiles {
            files: Mutex::new(vec![]),
            is_ready: Mutex::new(false),
        })
        .plugin(tauri_plugin_cli::init())
        .plugin(tauri_plugin_single_instance::init(
            |app_handle, args, _cwd| {
                println!("🔄 Single instance triggered with {} arguments", args.len());
                log::info!("Single instance triggered with {} arguments", args.len());
                // When a second instance is launched (e.g. double-clicking an archive
                // in the OS file manager on Windows/Linux), forward each argument
                // beyond the executable name as a "file-open" event so the already-
                // running instance can navigate to it.
                for (index, arg) in args.iter().enumerate().skip(1) {
                    println!("📂 Processing argument {}: {}", index, arg);
                    push_file(&app_handle, arg.clone());
                    // match app_handle.emit("file-open", arg.clone()) {
                    //     Ok(_) => println!("✅ Successfully emitted file-open event for: {}", arg),
                    //     Err(e) => println!("❌ Failed to emit file-open event: {}", e),
                    // }
                }
            },
        ))
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout), // Console output for dev
                    Target::new(TargetKind::LogDir { file_name: None }), // Writes to app.log in the log directory
                ])
                .level(log::LevelFilter::Debug) // Capture Debug, Info, Warn, and Error
                .rotation_strategy(RotationStrategy::KeepAll) // Keeps old logs instead of overwriting
                .build(),
        )
        .setup(|app| {
            // Access command-line arguments
            // Create a temporary directory
            log::info!("Creating temporary directory");
            let temp_dir = TempDir::new().map_err(|e| {
                eprintln!("Failed to create temp dir: {}", e);
                e
            })?;

            // Log the temp directory path for debugging
            println!(
                "Temporary directory created at: {}",
                temp_dir.path().display()
            );

            // Store the TempDir in the app state
            app.manage(Arc::new(AppTempDir { temp_dir }));
            // // Register state
            // app.manage(PendingFiles {
            //     files: Mutex::new(vec![]),
            // });
            let handle = app.handle().clone();
            // Safety net: if the frontend never reports ready, still show the main
            // window after 5s (unless this launch is a background extraction).
            let fallback = handle.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(5));
                let hidden = matches!(
                    fallback.get_webview_window("main").map(|w| w.is_visible()),
                    Some(Ok(false))
                );
                if hidden && !BACKGROUND_REQUEST.load(Ordering::SeqCst) {
                    show_main_window(&fallback);
                }
            });
            // === START: CLI feature ===
            // Handle CLI arguments using tauri_plugin_cli.
            // - app <path>                     → open archive in the UI
            // - app extract <path>             → extract here (default)
            // - app extract <path> --here      → extract here
            // - app extract <path> --to <dest> → extract to destination
            handle_cli(handle.clone());
            // === END: CLI feature ===
            // =========================
            // Frontend ready handshake
            // =========================
            app.handle().listen("frontend-ready", move |_| {
                println!("🔥 Frontend ready");
                let state = handle.state::<PendingFiles>();
                *state.is_ready.lock().unwrap() = true;
                if !BACKGROUND_REQUEST.load(Ordering::SeqCst) {
                    show_main_window(&handle);
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| match event {
            // Cleanup when the last window is closed
            tauri::WindowEvent::CloseRequested { .. } => {
                let state = window.state::<Arc<AppTempDir>>();
                let app_handle = window.app_handle();
                let windows = app_handle.windows();
                if windows.len() == 1 {
                    println!(
                        "Cleaning up temporary directory: {}",
                        state.temp_dir.path().display()
                    );
                    let temp_dir = &state.temp_dir.path().to_path_buf();
                    // Delete the directory manually
                    if let Err(e) = fs::remove_dir_all(&temp_dir) {
                        eprintln!("Failed to delete temp directory: {}", e);
                    }
                }
            }
            // A progress window closed: quit if this was a background-only launch.
            tauri::WindowEvent::Destroyed => {
                let label = window.label();
                if label.starts_with("extract-") {
                    exit_if_idle_in_background(window.app_handle(), Some(label));
                }
            }
            _ => {}
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            read_directory,
            open_file,
            archive_file_details,
            extract_files,
            view_file_in_archive,
            get_image_preview,
            compute_checksum,
            path_exists,
            create_archive,
            take_files
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| {
            // Dock click / normal launch while already running: show the main window.
            #[cfg(target_os = "macos")]
            if let RunEvent::Reopen { .. } = event {
                show_main_window(app_handle);
            }
            // macOS: when the app is already running and the user double-clicks
            // an associated file in Finder, the OS sends an Apple Event rather
            // than spawning a second process, so tauri-plugin-single-instance
            // never fires. Handle it via RunEvent::Opened instead.
            #[cfg(target_os = "macos")]
            if let RunEvent::Opened { urls } = event {
                println!("📂 macOS Opened event received with {} URLs", urls.len());
                log::info!("macOS Opened event received with {} URLs", urls.len());
                for (index, url) in urls.iter().enumerate() {
                    println!("📂 Processing URL {}: {:?}", index, url);
                    // Finder extension requests (rustycompress://extract?...)
                    if url.scheme() == finder::URL_SCHEME {
                        match finder::parse_finder_url(url) {
                            Some(request) => finder::handle_finder_request(app_handle, request),
                            None => log::warn!("Ignoring malformed rustycompress URL: {url}"),
                        }
                        continue;
                    }
                    if let Ok(path) = url.to_file_path() {
                        let path_str = path.to_string_lossy().to_string();
                        println!("📂 Emitting file-open event: {}", path_str);
                        push_file(&app_handle, path_str);
                        // let result = app_handle.emit("file-open", path_str);
                        // match result {
                        //     Ok(_) => println!("✅ Successfully emitted file-open event"),
                        //     Err(e) => println!("❌ Failed to emit file-open event: {}", e),
                        // }
                    } else {
                        println!("❌ Failed to convert URL to path: {:?}", url);
                    }
                }
            }
        });
}
