# Rusty Compress

A modern, cross-platform desktop archive manager built with Tauri (Rust backend) and React (frontend). Rusty Compress provides a unified, efficient interface for browsing, extracting, and managing various archive formats including ZIP, TAR, and RAR archives.

![Rusty Compress Logo](src-tauri/icons/icon.png)

## Features

- **Multi-Format Support**: ZIP, JAR, APK, XAPK, IPA, TAR, TAR.GZ, TGZ, TAR.BZ2, TBZ2, single-file GZ / BZ2, and RAR archives. 7z archives can be created (via the external `7z` tool)
- **Password-Protected Archives**: Browse, view, preview, and extract encrypted ZIP (ZipCrypto and AES) and RAR archives. The app asks for the password and remembers it per tab
- **Unified Interface**: Browse both filesystem directories and archive contents seamlessly
- **File Extraction**: Extract individual files or entire archives with progress tracking. Entries whose paths try to escape the destination folder (`..`, absolute paths) are skipped
- **macOS Finder Integration**: Right-click an archive in Finder to choose *Extract Here with Rusty* or *Rusty Extract To…*. Extraction runs in the background with only a small progress window
- **Preview Capabilities**: View images directly from archives without full extraction
- **Checksum Verification**: Compute MD5, SHA1, SHA256, and SHA512 checksums with real-time progress
- **Native macOS Look**: Neutral grays, one blue accent, and the system font (SF Pro). Follows the system Light and Dark modes
- **Cross-Platform**: Works on Windows, macOS, and Linux
- **System Integration**: Automatic file associations for supported archive types
- **Efficient Performance**: Rust backend for fast archive operations

## Screenshots

<!-- Add screenshots here when available -->
*Main Interface*
*Archive Browsing*
*Extraction Progress*

## Prerequisites

Before you begin, ensure you have the following installed:

- **Node.js** (v16 or higher) - [Download](https://nodejs.org/)
- **npm** or **yarn** - Comes with Node.js
- **Rust** (latest stable) - [Install via rustup](https://www.rust-lang.org/tools/install)
- **System Dependencies**:
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Linux**: Build essentials and libwebkit2gtk (see [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites))
  - **Windows**: Microsoft C++ Build Tools and WebView2

## Installation

### Clone the Repository

```bash
git clone https://github.com/vanhonit/rusty-compress.git
cd rusty-compress
```

### Install Dependencies

```bash
npm install
```

## Development

### Start Development Server

Run the full Tauri development environment (recommended):

```bash
npm run tauri dev
```

This will:
- Build the Rust backend
- Start the Vite development server (port 1420)
- Launch the application window with hot-reload enabled

### Frontend-Only Development

For frontend-only changes:

```bash
npm run dev
```

This starts the Vite dev server on port 1420 with HMR on port 1421.

### Backend Development

```bash
cd src-tauri
cargo build
cargo test
cargo clippy --all-targets
```

The Rust unit tests cover ZIP and RAR password checks, TAR extraction (including entries larger than 8 KB), single-file gzip and bzip2 round trips, path-safety helpers, and Finder URL parsing.

## Building for Production

### Build React Frontend

```bash
npm run build
```

### Build Complete Application

```bash
npm run tauri build
```

This creates platform-specific installers in `src-tauri/target/release/bundle/`:
- **macOS**: `.dmg` installer
- **Windows**: `.exe` installer and `.msi`
- **Linux**: `.AppImage`, `.deb`, and `.rpm` packages

### Build the macOS App with the Finder Extension

```bash
npm run build:mac                  # app with the Finder extension, signed
npm run build:mac -- --install     # also copy to /Applications and register the extension
```

See [macOS Finder Extension](#macos-finder-extension-right-click-extract) below. This needs Xcode Command Line Tools.

## Project Structure

```
rusty-compress/
├── src/                          # React frontend
│   ├── components/               # Tab bar and tab components
│   │   ├── Tab.jsx
│   │   └── TabBar.jsx
│   ├── contexts/TabContext.jsx   # Multi-tab state (includes the per-tab archive password)
│   ├── modules/
│   │   ├── FileManager/          # Main file browsing interface
│   │   ├── Extract/              # Extraction progress window and password prompt
│   │   ├── Archive/              # Create-archive modal and progress window
│   │   └── Checksum/             # Checksum computation modal
│   ├── index.css                 # Global styles and semantic color tokens
│   └── main.jsx                  # React entry point
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── lib.rs               # Tauri commands, extraction orchestration, CLI
│   │   ├── data_type.rs         # Shared data structures
│   │   ├── utils.rs             # Archive type detection, safe_join, password error prefixes
│   │   ├── file_manager.rs      # Filesystem operations
│   │   ├── zip.rs               # ZIP archive handler (including passwords)
│   │   ├── tar.rs               # TAR archive handler
│   │   ├── compressed.rs        # Single-file gzip / bzip2 handler
│   │   ├── rar.rs               # RAR archive handler (including passwords)
│   │   ├── sevenz.rs            # 7z creation via the external 7z CLI
│   │   └── finder.rs            # macOS Finder request parsing
│   ├── tests/fixtures/           # Test archives (crypted.rar, password "unrar")
│   ├── Cargo.toml               # Rust dependencies
│   ├── Info.plist               # URL scheme registration (rustycompress://)
│   └── tauri.conf.json          # Tauri configuration
├── macos/FinderExtension/        # FinderSync extension (right-click menu)
├── scripts/build-macos.sh        # Backs `npm run build:mac`
├── package.json                  # Node.js dependencies
└── CLAUDE.md                     # Project documentation
```

## Usage

### Opening Archives

1. **File Association**: Double-click any supported archive file in your file manager
2. **In-App**: Browse to an archive and open it
3. **Command Line**: `rusty-compress <archive>` opens it in the app. `rusty-compress extract <archive> [--here | --to <dest>]` extracts it (see below)

### Password-Protected Archives

- ZIP (ZipCrypto or AES) and RAR archives that have a password open with a password prompt
- The password is kept per tab and reused for browsing, previews, and extraction
- A wrong password shows "incorrect password" and asks again
- Extraction checks the password before the progress window opens

### Extracting Files

1. Browse to an archive and select the files you want to extract
2. Click **Extract…** in the toolbar and choose a destination folder
3. Monitor progress in the extraction window

**Extract Here** extracts into the folder that contains the archive.

From the command line, `rusty-compress extract <archive> --to <dest>` extracts to a folder, and `--here` (or no option) extracts next to the archive.

### Extracting from Finder (macOS)

Right-click an archive in Finder and choose **Extract Here with Rusty** or **Rusty Extract To…**. See the [macOS Finder Extension](#macos-finder-extension-right-click-extract) section.

### Computing Checksums

1. Open an archive (the Checksum button is enabled while an archive is open)
2. Click **Checksum** in the toolbar
3. Click **Compute** next to the algorithm you need (MD5, SHA1, SHA256, SHA512)
4. View real-time progress and the result

### Previewing Images

- Hover over an image inside an archive to see a preview
- Double-click an image to open it in your system viewer. The file is extracted to a temporary folder that is deleted when the last window closes
- Single-file `.gz` and `.bz2` images can be previewed up to 50 MiB

## Tauri Commands

The frontend calls these with `invoke`, using camelCase argument names. `?` marks an optional argument. The full reference is in [docs/api.md](docs/api.md).

| Command | Arguments |
|---------|-----------|
| `read_directory` | `path` |
| `open_file` | `path`, `filePath?` |
| `archive_file_details` | `sourcePath`, `filePath?`, `password?` |
| `extract_files` | `archivePath`, `outputPath`, `selectedFiles?`, `password?` |
| `view_file_in_archive` | `archivePath`, `fileName`, `password?` |
| `get_image_preview` | `archivePath`, `filePath`, `password?` (returns a data URL) |
| `compute_checksum` | `filePath`, `algorithm` |
| `path_exists` | `path` |
| `create_archive` | `files`, `outputPath`, `archiveFormat` (`zip`, `tar`, `7z`), `compressionLevel` |
| `take_files` | none (drains file-open paths buffered before the UI was ready) |

## Development Notes

### Architecture

- **Frontend**: React 18 with Vite, Tailwind CSS for styling
- **Backend**: Rust with Tauri 2.x for native desktop capabilities
- **Communication**: Tauri commands expose Rust functions to React
- **State Management**: React hooks for frontend, Tauri State for backend

### Key Patterns

- **Async Operations**: Long-running tasks use `tauri::async_runtime::spawn_blocking`
- **Progress Reporting**: Events emitted via `app_handle.emit()`
- **Temporary Files**: Managed with `tempfile::TempDir`, auto-cleanup on window close
- **File Associations**: Configured in `tauri.conf.json`

### Testing

```bash
cd src-tauri
cargo test
```

There are no frontend tests.

### Code Quality

```bash
cd src-tauri
cargo clippy --all-targets
```

## Contributing

Contributions are welcome! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests and linting
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Development Guidelines

- Follow existing code style and patterns
- Write tests for new features
- Update documentation as needed
- Ensure all tests pass before submitting PR

## File Type Support

| Format | List / view / preview | Extract | Create | Password |
|--------|-----------------------|---------|--------|----------|
| ZIP family (zip, jar, apk, xapk, ipa) | yes | yes | yes (`zip`) | yes (ZipCrypto + AES) |
| TAR family (tar, tar.gz, tgz, tar.bz2, tbz2) | yes | yes | yes (`tar`, uncompressed) | n/a |
| Single-file gzip / bzip2 (gz, bz2) | yes (one entry) | yes | no | n/a |
| RAR | yes | yes | no | yes |
| 7z | no | no | yes, via external `7z` CLI | no |

- **ZIP family**: `.zip`, `.jar`, `.apk`, `.xapk`, `.ipa`
- **TAR family**: `.tar`, `.tar.gz`, `.tgz`, `.tar.bz2`, `.tbz2` (open as folders)
- **Single-file gzip / bzip2**: `.gz`, `.bz2`. These show as one entry named after the file (`kubeconfig.gz` lists `kubeconfig`). `.bz2` entries show size 0 because the format has no size field
- **RAR**: `.rar`
- **7z**: `.7z`. Creation only; requires the `7z` command-line tool

## macOS Finder Extension (right-click Extract)

Right-click a supported archive in Finder to see **Extract Here with Rusty** and **Rusty Extract To…**. The items appear only when every selected item is a supported archive: zip family (zip, jar, apk, xapk, ipa), tar family (tar, tar.gz, tgz, tar.bz2, tbz2), gz, bz2, and rar. 7z is not offered.

- **Extract Here with Rusty** extracts each selected archive into the folder that contains it.
- **Rusty Extract To…** asks for a destination folder and extracts the selected archives there. Cancelling the picker does nothing.
- Password-protected archives open the main window so you can enter the password.
- Extraction runs in the background and shows only a small progress window. If the main window was never opened, the app quits when the last progress window closes (or when the folder picker is cancelled). Clicking the Dock icon shows the main window.
- macOS places the items in Finder's extension section of the context menu, near Quick Actions. Apps have no API to move them. The icons are white.

The extension is a FinderSync `.appex` in `macos/FinderExtension/`. It sends `rustycompress://extract?mode=here|to&path=<absolute path>` to the app. The app accepts only existing local archive files, and Extract To always asks for a folder.

### Requirements

- Xcode Command Line Tools (`xcode-select --install`) for `swiftc` and `lipo`
- A code-signing identity. A free Apple ID works for your own Mac: create an "Apple Development" certificate in Xcode > Settings > Accounts. Without one, the build falls back to ad-hoc signing, which works locally but may be blocked elsewhere.

### Build and install

```bash
npm run build:mac                          # builds the app with the extension, signed
npm run build:mac -- --install             # also copies to /Applications and registers the extension
npm run build:mac -- --skip-tauri --install --restart-finder   # reuse the last build; restart Finder
npm run build:mac -- --dmg                 # also creates a DMG
```

The first signed run may show a keychain prompt. Click **Always Allow**.

### Enable the extension

Open **System Settings > General > Login Items & Extensions > Finder** and turn on **Rusty Compress**. Or run:

```bash
pluginkit -e use -i com.rusty-compress.app.FinderExtension
```

Verify it is registered; a leading `+` means enabled:

```bash
pluginkit -m -i com.rusty-compress.app.FinderExtension
```

If the menu does not appear, run `killall Finder`.

### Uninstall

```bash
pluginkit -e ignore -i com.rusty-compress.app.FinderExtension
rm -rf "/Applications/Rusty Compress.app"
```

### Limitations

- Free and ad-hoc signed builds are not notarized. On another Mac, remove the quarantine flag with `xattr -dr com.apple.quarantine "/Applications/Rusty Compress.app"`, or use **Privacy & Security > Open Anyway**. Then enable the extension manually as above.
- Distributing to other users needs a paid Developer ID certificate and notarization.
- Any app can open `rustycompress://` URLs. The app only acts on existing local archive files, and Extract To always asks for a folder.

## Troubleshooting

### Build Issues

**Rust compilation fails:**
```bash
rustup update
cargo clean
cargo build
```

**Frontend dependencies fail:**
```bash
rm -rf node_modules package-lock.json
npm install
```

### Runtime Issues

**Archive not opening:**
- Ensure the file format is supported (7z can be created, but not opened)
- Check file permissions
- Verify the file is not corrupted

**Password prompt keeps appearing:**
- The password is wrong. For a damaged encrypted RAR, the app may also report an incorrect password (a RAR4 limitation)

**Extraction fails:**
- Check disk space
- Verify write permissions to the destination
- Entries whose paths try to escape the destination are skipped

## Roadmap

- [ ] 7z read and extract support
- [ ] Archive creation improvements (zip directory contents, compression levels)
- [ ] Split archive handling
- [ ] Batch extraction from multiple archives inside the app
- [ ] Archive search functionality
- [ ] Custom themes and UI customization

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) - Framework for building desktop apps
- [React](https://react.dev/) - UI library
- [Vite](https://vitejs.dev/) - Build tool
- [Rust](https://www.rust-lang.org/) - Backend language

## Contact

- **GitHub Issues**: [Report bugs or request features](https://github.com/your-username/rusty-compress/issues)
- **Discussions**: [Ask questions and share ideas](https://github.com/your-username/rusty-compress/discussions)

---

Built with ❤️ using Tauri and React
