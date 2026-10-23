//! macOS Finder integration.
//!
//! The FinderSync extension (`macos/FinderExtension`) opens
//! `rustycompress://extract?mode=here|to&path=<p>&path=<p>...` URLs. This module
//! parses those URLs, validates the paths, and starts extraction jobs.
#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

use std::path::PathBuf;

use tauri::{AppHandle, Url};
use tauri_plugin_dialog::DialogExt;

use crate::utils::{
    detect_archive_type, ArchiveType, INVALID_PASSWORD_PREFIX, PASSWORD_REQUIRED_PREFIX,
};

/// URL scheme registered in src-tauri/Info.plist.
pub const URL_SCHEME: &str = "rustycompress";
const URL_HOST: &str = "extract";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractMode {
    /// Extract into each archive's parent directory.
    Here,
    /// Ask the user for a destination folder.
    To,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinderRequest {
    pub mode: ExtractMode,
    pub paths: Vec<String>,
}

/// Parse a `rustycompress://extract?mode=...&path=...` URL.
///
/// Returns `None` for other schemes, other hosts, a missing or unknown `mode`,
/// or a request without any `path`. Query values are percent-decoded.
pub fn parse_finder_url(url: &Url) -> Option<FinderRequest> {
    if url.scheme() != URL_SCHEME || url.host_str() != Some(URL_HOST) {
        return None;
    }

    let mut mode = None;
    let mut paths = Vec::new();
    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "mode" => {
                mode = Some(match value.as_ref() {
                    "here" => ExtractMode::Here,
                    "to" => ExtractMode::To,
                    _ => return None,
                });
            }
            "path" => paths.push(value.into_owned()),
            _ => {}
        }
    }

    if paths.is_empty() {
        return None;
    }
    Some(FinderRequest {
        mode: mode?,
        paths,
    })
}

/// Check that `raw` names an existing, extractable archive file.
/// 7z is rejected because the app cannot extract it yet.
pub fn validate_archive_path(raw: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(raw);
    if !path.is_file() {
        return Err("not an existing file".to_string());
    }
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "file name is not valid UTF-8".to_string())?;
    match detect_archive_type(name) {
        None => Err("not a supported archive".to_string()),
        Some(ArchiveType::SevenZ) => Err("7z archives cannot be extracted".to_string()),
        Some(_) => Ok(path),
    }
}

/// Run a parsed Finder request. Never blocks the calling (event) thread.
pub fn handle_finder_request(app: &AppHandle, request: FinderRequest) {
    // Before validation: even an all-invalid request must not show the main window.
    crate::mark_background_request();

    let archives: Vec<PathBuf> = request
        .paths
        .iter()
        .filter_map(|raw| match validate_archive_path(raw) {
            Ok(path) => Some(path),
            Err(reason) => {
                log::warn!("Finder request: ignoring {raw:?}: {reason}");
                None
            }
        })
        .collect();

    if archives.is_empty() {
        log::warn!("Finder request: no valid archives to extract");
        crate::exit_if_idle_in_background(app, None);
        return;
    }

    match request.mode {
        ExtractMode::Here => {
            for archive in archives {
                let Some(parent) = archive.parent() else {
                    log::error!("Finder request: no parent directory for {}", archive.display());
                    continue;
                };
                let dest = parent.to_string_lossy().into_owned();
                spawn_extract(app.clone(), archive, dest);
            }
        }
        ExtractMode::To => {
            let app = app.clone();
            crate::begin_background_job();
            // The folder picker blocks until the user answers, so run it off the event thread.
            std::thread::spawn(move || {
                let picked = app
                    .dialog()
                    .file()
                    .set_title("Extract to…")
                    .blocking_pick_folder();
                let dest = match picked.map(|folder| folder.into_path()) {
                    Some(Ok(path)) => path.to_string_lossy().into_owned(),
                    Some(Err(e)) => {
                        log::error!("Finder request: invalid destination folder: {e}");
                        crate::end_background_job(&app);
                        return;
                    }
                    None => {
                        log::info!("Finder request: extract-to cancelled");
                        crate::end_background_job(&app);
                        return;
                    }
                };
                for archive in archives {
                    spawn_extract(app.clone(), archive, dest.clone());
                }
                crate::end_background_job(&app);
            });
        }
    }
}

/// Start one extraction. If the archive needs a password, hand it to the main
/// UI instead so the user can enter it there.
fn spawn_extract(app: AppHandle, archive: PathBuf, dest: String) {
    crate::begin_background_job();
    tauri::async_runtime::spawn(async move {
        let archive_str = archive.to_string_lossy().into_owned();
        match crate::extract_files(app.clone(), archive_str.clone(), dest, None, None).await {
            Ok(()) => log::info!("Finder request: extraction started for {archive_str}"),
            Err(e)
                if e.starts_with(PASSWORD_REQUIRED_PREFIX)
                    || e.starts_with(INVALID_PASSWORD_PREFIX) =>
            {
                log::info!("Finder request: password needed, opening {archive_str} in the app");
                // push_file shows the main window itself.
                crate::push_file(&app, archive_str);
            }
            Err(e) => log::error!("Finder request: extraction failed for {archive_str}: {e}"),
        }
        crate::end_background_job(&app);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn parse(s: &str) -> Option<FinderRequest> {
        parse_finder_url(&Url::parse(s).unwrap())
    }

    #[test]
    fn parses_here_with_percent_encoded_path() {
        assert_eq!(
            parse("rustycompress://extract?mode=here&path=%2FUsers%2Fa%2Fb.zip"),
            Some(FinderRequest {
                mode: ExtractMode::Here,
                paths: vec!["/Users/a/b.zip".to_string()],
            })
        );
    }

    #[test]
    fn decodes_spaces_unicode_and_plus_signs() {
        let url = Url::parse_with_params(
            "rustycompress://extract",
            &[("mode", "to"), ("path", "/Users/hon/Tài liệu/a+b (1).zip")],
        )
        .unwrap();
        let req = parse_finder_url(&url).unwrap();
        assert_eq!(req.mode, ExtractMode::To);
        assert_eq!(req.paths, vec!["/Users/hon/Tài liệu/a+b (1).zip".to_string()]);

        // Percent-encoded space and plus (what the Swift side emits).
        let req = parse("rustycompress://extract?mode=here&path=%2Fa%2Bb%20c.zip").unwrap();
        assert_eq!(req.paths, vec!["/a+b c.zip".to_string()]);
    }

    #[test]
    fn keeps_multiple_paths_in_order() {
        let req = parse(
            "rustycompress://extract?mode=here&path=%2Fa.zip&path=%2Fb%20c.tar.gz&path=%2Fd.rar",
        )
        .unwrap();
        assert_eq!(
            req.paths,
            vec![
                "/a.zip".to_string(),
                "/b c.tar.gz".to_string(),
                "/d.rar".to_string()
            ]
        );
    }

    #[test]
    fn rejects_other_schemes_hosts_and_modes() {
        assert_eq!(parse("https://extract?mode=here&path=%2Fa.zip"), None);
        assert_eq!(parse("rustycompress://open?mode=here&path=%2Fa.zip"), None);
        assert_eq!(parse("rustycompress://extract?mode=bogus&path=%2Fa.zip"), None);
        assert_eq!(parse("rustycompress://extract?path=%2Fa.zip"), None);
        assert_eq!(parse("rustycompress://extract?mode=here"), None);
    }

    #[test]
    fn validate_accepts_archives_and_rejects_others() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path();
        let zip = base.join("Photos.ZIP");
        let tgz = base.join("backup.tar.gz");
        let gz = base.join("kubeconfig.gz");
        let sevenz = base.join("x.7z");
        let txt = base.join("notes.txt");
        let folder = base.join("folder.zip");
        fs::write(&zip, b"x").unwrap();
        fs::write(&tgz, b"x").unwrap();
        fs::write(&gz, b"x").unwrap();
        fs::write(&sevenz, b"x").unwrap();
        fs::write(&txt, b"x").unwrap();
        fs::create_dir(&folder).unwrap();

        assert_eq!(validate_archive_path(zip.to_str().unwrap()), Ok(zip.clone()));
        assert_eq!(validate_archive_path(tgz.to_str().unwrap()), Ok(tgz.clone()));
        assert_eq!(validate_archive_path(gz.to_str().unwrap()), Ok(gz.clone()));
        assert!(validate_archive_path(sevenz.to_str().unwrap()).is_err());
        assert!(validate_archive_path(txt.to_str().unwrap()).is_err());
        assert!(validate_archive_path(folder.to_str().unwrap()).is_err());
        assert!(validate_archive_path(base.join("missing.zip").to_str().unwrap()).is_err());
    }
}
