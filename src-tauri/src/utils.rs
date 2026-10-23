use std::path::{Component, Path, PathBuf};

/// Error-string prefix the frontend uses to detect that a password is required.
pub const PASSWORD_REQUIRED_PREFIX: &str = "PASSWORD_REQUIRED:";
/// Error-string prefix the frontend uses to detect that a supplied password is wrong.
pub const INVALID_PASSWORD_PREFIX: &str = "INVALID_PASSWORD:";

/// Join an archive entry name onto `base`, refusing anything that could escape it.
///
/// Returns `None` for absolute paths, Windows prefixes/roots, any `..` component,
/// or an entry that reduces to nothing. `.` components are dropped.
pub fn safe_join(base: &Path, entry: &str) -> Option<PathBuf> {
    let mut cleaned = PathBuf::new();
    for component in Path::new(entry).components() {
        match component {
            Component::Normal(part) => cleaned.push(part),
            Component::CurDir => {}
            // RootDir, Prefix, ParentDir are all unsafe
            _ => return None,
        }
    }
    if cleaned.as_os_str().is_empty() {
        return None;
    }
    Some(base.join(cleaned))
}

#[allow(dead_code)]
pub fn remove_last_item_from_path_string(path: &str) -> Result<String, String> {
    // Trim any trailing slashes to normalize the path
    let trimmed_path = path.trim_end_matches('/').trim_end_matches('\\');

    // Find the position of the last separator
    if let Some(pos) = trimmed_path.rfind(|c| c == '/' || c == '\\') {
        // Return the substring up to the last separator
        Ok(trimmed_path[..pos].to_string() + "/")
    } else {
        // No separators found, return an error
        Err("The path has no parent or is invalid.".to_string())
    }
}

#[allow(dead_code)]
pub fn get_file_extension(file_name: &str) -> Option<String> {
    // Split the file name by the last '.' and get the extension
    file_name.split('.').last().map(|ext| ext.to_string())
}

/// Archive type enum for detecting and handling different archive formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveType {
    // ZIP-based archives
    Zip,
    Jar,
    Apk,
    Xapk,
    Ipa,
    // TAR-based archives
    Tar,
    TarGz,
    Tgz,
    TarBz2,
    Tbz2,
    // RAR archives
    Rar,
    // 7z archives
    SevenZ,
    // Single-file compressed streams (one file, not a tar)
    Gz,
    Bz2,
}

/// Detect archive type from filename, handling compound extensions like .tar.gz
#[allow(dead_code)]
pub fn detect_archive_type(file_name: &str) -> Option<ArchiveType> {
    let lower = file_name.to_lowercase();

    // Check compound extensions first (.tar.gz, .tar.bz2)
    if lower.ends_with(".tar.gz") {
        Some(ArchiveType::TarGz)
    } else if lower.ends_with(".tar.bz2") {
        Some(ArchiveType::TarBz2)
    } else if lower.ends_with(".tgz") {
        Some(ArchiveType::Tgz)
    } else if lower.ends_with(".tbz2") {
        Some(ArchiveType::Tbz2)
    } else if lower.ends_with(".tar") {
        Some(ArchiveType::Tar)
    } else if lower.ends_with(".zip") {
        Some(ArchiveType::Zip)
    } else if lower.ends_with(".jar") {
        Some(ArchiveType::Jar)
    } else if lower.ends_with(".apk") {
        Some(ArchiveType::Apk)
    } else if lower.ends_with(".xapk") {
        Some(ArchiveType::Xapk)
    } else if lower.ends_with(".ipa") {
        Some(ArchiveType::Ipa)
    } else if lower.ends_with(".rar") {
        Some(ArchiveType::Rar)
    } else if lower.ends_with(".7z") {
        Some(ArchiveType::SevenZ)
    } else if lower.ends_with(".gz") {
        // Plain .gz only: compound .tar.gz / .tgz were matched above
        Some(ArchiveType::Gz)
    } else if lower.ends_with(".bz2") {
        // Plain .bz2 only: compound .tar.bz2 was matched above
        Some(ArchiveType::Bz2)
    } else {
        None
    }
}

/// Check if the archive type is a single-file compressed stream (.gz / .bz2, not a tar)
#[allow(dead_code)]
pub fn is_single_compressed(archive_type: ArchiveType) -> bool {
    matches!(archive_type, ArchiveType::Gz | ArchiveType::Bz2)
}

/// Check if the archive type is ZIP-based (all ZIP-based formats can use ZIP extraction)
#[allow(dead_code)]
pub fn is_zip_based(archive_type: ArchiveType) -> bool {
    matches!(
        archive_type,
        ArchiveType::Zip | ArchiveType::Jar | ArchiveType::Apk | ArchiveType::Xapk | ArchiveType::Ipa
    )
}

/// Check if the archive type is TAR-based (all TAR-based formats can use TAR extraction)
#[allow(dead_code)]
pub fn is_tar_based(archive_type: ArchiveType) -> bool {
    matches!(
        archive_type,
        ArchiveType::Tar | ArchiveType::TarGz | ArchiveType::Tgz | ArchiveType::TarBz2 | ArchiveType::Tbz2
    )
}

/// Check if the archive type is RAR-based
#[allow(dead_code)]
pub fn is_rar_based(archive_type: ArchiveType) -> bool {
    matches!(archive_type, ArchiveType::Rar)
}

/// Check if the archive type is 7z-based
#[allow(dead_code)]
pub fn is_sevenz_based(archive_type: ArchiveType) -> bool {
    matches!(archive_type, ArchiveType::SevenZ)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_single_file_gz_and_bz2_case_insensitively() {
        assert_eq!(detect_archive_type("kubeconfig.gz"), Some(ArchiveType::Gz));
        assert_eq!(detect_archive_type("a.gz"), Some(ArchiveType::Gz));
        assert_eq!(detect_archive_type("A.BZ2"), Some(ArchiveType::Bz2));
        assert_eq!(detect_archive_type("log.Gz"), Some(ArchiveType::Gz));
    }

    #[test]
    fn compound_extensions_take_precedence_over_plain_gz_bz2() {
        assert_eq!(detect_archive_type("a.tar.gz"), Some(ArchiveType::TarGz));
        assert_eq!(detect_archive_type("A.TAR.GZ"), Some(ArchiveType::TarGz));
        assert_eq!(detect_archive_type("a.tgz"), Some(ArchiveType::Tgz));
        assert_eq!(detect_archive_type("a.tar.bz2"), Some(ArchiveType::TarBz2));
        assert_eq!(detect_archive_type("a.tbz2"), Some(ArchiveType::Tbz2));
    }

    #[test]
    fn single_compressed_is_not_tar_based() {
        assert!(is_single_compressed(ArchiveType::Gz));
        assert!(is_single_compressed(ArchiveType::Bz2));
        assert!(!is_single_compressed(ArchiveType::TarGz));
        assert!(!is_tar_based(ArchiveType::Gz));
        assert!(!is_tar_based(ArchiveType::Bz2));
        assert!(!is_zip_based(ArchiveType::Gz));
        assert!(!is_rar_based(ArchiveType::Bz2));
        assert!(!is_sevenz_based(ArchiveType::Gz));
    }

    #[test]
    fn safe_join_accepts_normal_relative_paths() {
        let base = Path::new("/out");
        assert_eq!(safe_join(base, "a/b.txt"), Some(PathBuf::from("/out/a/b.txt")));
        assert_eq!(safe_join(base, "./a"), Some(PathBuf::from("/out/a")));
    }

    #[test]
    fn safe_join_rejects_traversal_and_absolute_paths() {
        let base = Path::new("/out");
        assert_eq!(safe_join(base, "../x"), None);
        assert_eq!(safe_join(base, "a/../../x"), None);
        assert_eq!(safe_join(base, "/etc/passwd"), None);
        assert_eq!(safe_join(base, ""), None);
    }
}
