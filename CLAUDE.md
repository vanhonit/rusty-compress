# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rusty Compress is a desktop archive manager built with Tauri 2 (Rust backend) and React 18 (frontend). It browses, previews, extracts, creates, and checksums archives, with multi-tab navigation, password-protected archive support, OS file associations, and a small CLI.

| Format | List / view / preview | Extract | Create | Password |
|---|---|---|---|---|
| ZIP family (zip, jar, apk, xapk, ipa) | yes | yes | yes (`zip`) | yes (ZipCrypto + AES) |
| TAR family (tar, tar.gz, tgz, tar.bz2, tbz2) | yes | yes | yes (`tar`, uncompressed) | n/a |
| Single-file gzip / bzip2 (gz, bz2) | yes (one entry) | yes | no | n/a |
| RAR | yes | yes | no | yes |
| 7z | no (stubs) | no (stubs) | yes, via external `7z` CLI | no |

## Common Commands

- `npm run tauri dev` - full dev mode (Vite on port 1420 + Rust backend)
- `npm run dev` - Vite dev server only (port 1420, `strictPort`; HMR on 1421 only when `TAURI_DEV_HOST` is set)
- `npm run build` - build the React bundle to `/dist`
- `npm run preview` - preview the production bundle
- `npm run tauri build` - build the distributable app
- `cd src-tauri && cargo build` - build the Rust backend
- `cd src-tauri && cargo test` - Rust unit tests
- `cd src-tauri && cargo clippy --all-targets` - lint

## Architecture

### Backend (`src-tauri/src`)

| File | Responsibility |
|---|---|
| `lib.rs` | All Tauri commands, checksum, extraction orchestration, CLI handling, file-open buffering, temp-dir cleanup, plugin setup |
| `utils.rs` | `ArchiveType` detection (compound extensions such as `.tar.gz` checked first), `is_zip_based` / `is_tar_based` / `is_rar_based` / `is_sevenz_based`, password error prefixes, `safe_join` path-traversal guard |
| `data_type.rs` | `FileInfo`, `ExtractionConfig` (with `password`), `CompressionConfig`, `ArchiveFormat {Zip, Tar, Sevenz}`, `ExtractionProgress`, `CompressionProgress`, `AppTempDir`, `SingleChecksum` |
| `file_manager.rs` | `read_directory`, `open_file` |
| `zip.rs` | List, extract, view-in-temp, image preview, create, `check_zip_password` |
| `tar.rs` | List, extract (`unpack_in`), view, preview, create (tar; tar.gz / tar.bz2 creators exist but are not routed) |
| `compressed.rs` | Single-file `.gz` / `.bz2` (not tar): one listed entry named after the stream, streaming extract (multi-member aware), view, preview (50 MiB cap) |
| `rar.rs` | List, extract, view, preview, `check_rar_password` (via the `unrar` crate) |
| `sevenz.rs` | `create_sevenz_archive` shells out to `7z`; read functions are TODO stubs |

All archive commands dispatch with `detect_archive_type` and the `is_*_based` helpers, then call the format module.

### Frontend (`src`)

- `App.jsx` renders `ProgressWindow` when the URL has `?label=progress`; otherwise `<TabProvider><FileManager/></TabProvider>`.
- `contexts/TabContext.jsx` holds multi-tab state. Each tab has `id`, `title`, `type` (`filesystem` or `archive`), `currentPath`, `currentArchive`, `archivePath` (path inside the archive), `archivePassword`, `files`, `selectedFiles`, `searchQuery`, `isLoading`, `error`.
- `components/TabBar.jsx`, `components/Tab.jsx` - tab strip.
- `modules/FileManager/` - `index.jsx` is the main controller (navigation, extraction, password modal state, file-open listener); `FileList`, `FileItem` (image preview), `Toolbar`, `Breadcumb` (sic), `PropertiesModal`.
- `modules/Extract/` - `ProgressWindow` (separate window), `PasswordModal`.
- `modules/Archive/` - `CreateArchiveModal`, `CompressionProgressWindow`.
- `modules/Checksum/ChecksumModal.jsx` - MD5, SHA1, SHA256, SHA512.

Stack: Vite 5, Tailwind 3, Heroicons, Remix Icon, `@tauri-apps/api` `invoke` / `listen` / `emit`.

Styling: a macOS-native look with neutral grays, one blue accent, and automatic Light/Dark mode. Colours are semantic Tailwind tokens (`surface`, `surface-secondary`, `surface-tertiary`, `separator`, `fg`, `fg-secondary`, `fg-tertiary`, `accent`, `accent-hover`, `accent-fg`, `danger`, `success`, `warning`) backed by RGB CSS variables in `src/index.css` and mapped in `tailwind.config.js`. Use them instead of raw palette colours (`blue-500`, `gray-200`), and avoid gradients. The font is the system font.

## Tauri Commands

| Command | Arguments (camelCase from JS) |
|---|---|
| `read_directory` | `path` |
| `open_file` | `path`, `filePath?` |
| `archive_file_details` | `sourcePath`, `filePath?`, `password?` |
| `extract_files` | `archivePath`, `outputPath`, `selectedFiles?`, `password?` |
| `view_file_in_archive` | `archivePath`, `fileName`, `password?` |
| `get_image_preview` | `archivePath`, `filePath`, `password?` (returns a data URL) |
| `compute_checksum` | `filePath`, `algorithm` |
| `path_exists` | `path` |
| `create_archive` | `files`, `outputPath`, `archiveFormat` (`zip`, `tar`, `7z`), `compressionLevel` |
| `take_files` | none (drains buffered file-open paths) |

## Events

| Event | Direction | Payload |
|---|---|---|
| `extract-progress` | backend to UI | `{ files, total, percentage }` |
| `extract-complete` / `extract-error` | backend to UI | `()` / error string |
| `compress-progress` | backend to UI | `{ percentage, files_processed, current_file }` |
| `compress-complete` | backend to UI | `()` |
| `checksum-progress-{algo}` | backend to UI | percent, in 5% steps |
| `file-open` | backend to UI | file path |
| `frontend-ready` | UI to backend | none |
| `{label}://ready` | progress window to backend | none |

## Key Flows

### Extraction
1. `extract_files` first runs `verify_extraction_password` in `spawn_blocking` (zip and rar only). A missing password returns an error prefixed `PASSWORD_REQUIRED:`; a wrong one returns `INVALID_PASSWORD:`. These come back as the `invoke` rejection, so `FileManager` can open `PasswordModal`.
2. It then opens an `extract-<timestamp>` window at `index.html?label=progress&name=<label>&fileName=<archive>`.
3. The backend waits for `{label}://ready` from that window, then extracts.
4. Failures after this point arrive only as `extract-error` events, not as the `invoke` result.

### macOS Finder extension (right-click Extract)
- `macos/FinderExtension/FinderSync.swift` (FinderSync `.appex`, sandboxed, no file access) builds `rustycompress://extract?mode=here|to&path=...` and opens it with `NSWorkspace`.
- `RunEvent::Opened` in `lib.rs` passes `rustycompress://` URLs to `finder::parse_finder_url` and `handle_finder_request`. `here` extracts into each parent directory; `to` shows a folder picker on a std thread. Password errors fall back to `push_file`.
- `scripts/build-macos.sh` (`npm run build:mac`) builds the Tauri app, compiles the universal appex with swiftc, embeds it in `PlugIns/`, signs inside-out, and verifies. `--install` registers it with pluginkit.
- Finder menu items sit in Finder's extension section; there is no API to move them. Finder drops `isTemplate` on extension menu images, so the SF Symbols are pre-tinted white.

### Window visibility (background extraction)
- The `main` window starts hidden (`"visible": false` in `tauri.conf.json`). It is shown on `frontend-ready` unless the launch is a background request, and a 5 s fallback shows it if the frontend never reports ready.
- `finder::handle_finder_request` and the CLI `extract` path call `mark_background_request()`, so only the `extract-*` progress window appears.
- `push_file()` and macOS `RunEvent::Reopen` (Dock click) call `show_main_window()`.
- When an `extract-*` window is destroyed, `exit_if_idle_in_background()` quits the app if `main` is still hidden and no other background job (progress window or Extract To folder picker) is left.

### Passwords
- Supported for zip (ZipCrypto and AES) and rar. The password is stored per tab (`archivePassword`) and reused for browse, view, preview, and extract.
- Backend listing errors get the `PASSWORD_REQUIRED:` prefix when no password was supplied. The frontend `isPasswordError` matches on "password", "encrypted", or "protected".
- For RAR, a bad-data (CRC) error during the password test is treated as a wrong password, because RAR4 reports wrong passwords that way.

### File open (associations, second instance, CLI)
- Paths arrive from three places: CLI args at startup, `tauri-plugin-single-instance` (Windows/Linux), and macOS `RunEvent::Opened`. All call `push_file()`.
- Paths are buffered in `PendingFiles` until the frontend emits `frontend-ready`. The frontend then pulls them with `take_files`. After ready, paths are emitted live as `file-open`.

### CLI (`tauri-plugin-cli`)
- `app <archive>` - open the archive in the UI.
- `app extract <archive> [--here | --to <dest>]` - extract; the default is the archive's parent directory.

### Temp files
- `view_file_in_archive` writes to a `tempfile::TempDir` held in `Arc<AppTempDir>` state and opens it with the system default app.
- The directory is deleted when the last window closes.

## Security Notes

- Use `utils::safe_join` (rar), `ZipFile::enclosed_name` (zip), and `Entry::unpack_in` (tar) for every extraction path. Never join raw entry names onto the destination.
- View and preview reject names containing `..`.
- Capabilities: `capabilities/default.json` applies to windows `main` and `extract-*`; `desktop.json` adds `cli:default`. `tauri.conf.json` has `csp: null`.

## Configuration

- Main window: 900x700, minimum 800x600, not resizable.
- File associations: zip/jar/apk/xapk/ipa, tar/tgz/tar.gz/tbz2/tar.bz2, rar, gz/bz2 ("Compressed File").
- Frontend dist is `../dist`; dev URL is `http://localhost:1420`.

## Testing

- Rust unit tests live in `#[cfg(test)]` modules in `utils.rs`, `zip.rs`, `rar.rs`, `tar.rs`, `compressed.rs`, and `finder.rs`. Run them with `cd src-tauri && cargo test`.
- RAR fixture: `src-tauri/tests/fixtures/crypted.rar` (password `unrar`). Zip, tar, gz, and bz2 test archives are built in temp dirs.
- Tar tests use entries larger than 8 KiB. Never collect `Archive::entries()` into a `Vec` before unpacking: entries share one sequential reader, so the data is already consumed.
- There are no frontend tests.

## Known Issues

- Extract and compress events are broadcast to all windows, so concurrent extractions show each other's progress. The `{label}://ready` listener is never removed.
- 7z read support is stubbed. `view_file_in_archive` silently returns `Ok` for 7z.
- Zip creation adds directories without their contents, reads whole files into memory, and ignores compression level except 0.
- `open_file` only recognises lowercase `.zip` / `.tar`.
- The checksum loop is duplicated four times. A hand-written base64 encoder is copied in three modules although the `base64` crate is a dependency. Password-error helpers are duplicated in `zip.rs` and `rar.rs`.
- Leftover files: `src-tauri/src/lib.rs_corrected.txt`, `src/modules/FileManager/Toolbar_new.jsx`.
