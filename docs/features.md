---
layout: page
title: Features
subtitle: Powerful archive management capabilities
---

## Archive Format Support

Rusty Compress supports a wide range of archive formats, making it your one-stop solution for all archive management needs.

### ZIP-based Formats

| Format | Description | Extensions |
|--------|-------------|------------|
| **ZIP** | Standard ZIP archive format | `.zip` |
| **JAR** | Java Archive files | `.jar` |
| **APK** | Android application packages | `.apk` |
| **XAPK** | Android expanded app packages | `.xapk` |
| **IPA** | iOS application packages | `.ipa` |

### TAR-based Formats

| Format | Description | Extensions |
|--------|-------------|------------|
| **TAR** | Unix tar archive format | `.tar` |
| **TAR.GZ** | Gzip-compressed tar archive | `.tar.gz` |
| **TGZ** | Alternative gzip-compressed tar | `.tgz` |
| **TAR.BZ2** | Bzip2-compressed tar archive | `.tar.bz2` |
| **TBZ2** | Alternative bzip2-compressed tar | `.tbz2` |

### Single-file Compressed Formats

| Format | Description | Extensions |
|--------|-------------|------------|
| **GZ** | Gzip-compressed single file (lists as one entry named after the file) | `.gz` |
| **BZ2** | Bzip2-compressed single file (lists as one entry; size shows as 0) | `.bz2` |

### RAR Format

| Format | Description | Extensions |
|--------|-------------|------------|
| **RAR** | Roshal Archive format | `.rar` |

### 7z Format

| Format | Description | Extensions |
|--------|-------------|------------|
| **7z** | 7-Zip archive. Creation only, through the external `7z` command-line tool; browsing and extraction are not supported yet | `.7z` |

### Format Capabilities

| Format | Browse / view / preview | Extract | Create | Password |
|--------|-------------------------|---------|--------|----------|
| ZIP family (zip, jar, apk, xapk, ipa) | yes | yes | yes (`zip`) | yes (ZipCrypto + AES) |
| TAR family (tar, tar.gz, tgz, tar.bz2, tbz2) | yes | yes | yes (`tar`, uncompressed) | n/a |
| Single-file gz / bz2 | yes (one entry) | yes | no | n/a |
| RAR | yes | yes | no | yes |
| 7z | no | no | yes, via external `7z` CLI | no |

## Core Features

### 🗂️ Unified File System Interface

- **Dual-State Browsing**: Seamlessly switch between filesystem directories and archive contents
- **Breadcrumb Navigation**: Easy navigation with clear path indicators
- **Quick File Selection**: Click to select files, and use the checkboxes to select several
- **File Preview**: View file details and metadata before extraction

### 📤 Smart Extraction

- **Selective Extraction**: Extract specific files or entire archives
- **Password-Protected Archives**: Browse, view, preview, and extract encrypted ZIP and RAR archives. The app asks for the password, remembers it for the open tab, and asks again if it is wrong. Extraction checks the password before the progress window opens
- **Safe Extraction**: Entries whose paths try to escape the destination folder (zip-slip, `..`) are skipped
- **Progress Tracking**: Real-time progress for extraction in a separate window
- **Destination Management**: Choose any location for extracted files, or extract next to the archive
- **Error Handling**: Clear error messages for corrupted files, invalid gzip data, and wrong passwords

### 🔍 File Preview

- **Image Preview**: View images directly from archives without extraction. Images from single-file `.gz` and `.bz2` archives can be previewed up to 50 MiB
- **Metadata Display**: See file sizes, types, and modification dates
- **Quick View**: Open files in your system default viewer

### 🔐 Checksum Verification

- **Multiple Algorithms**: Support for MD5, SHA1, SHA256, and SHA512
- **Progress Updates**: Real-time progress reporting for large files

### 🚀 Performance Optimization

- **Native Rust Backend**: Fast archive operations
- **Async Processing**: Non-blocking interface for large file operations

## User Interface Features

### Intuitive Navigation

- **Tab System**: Multiple tabs for different files/archives (`Ctrl/Cmd+T`, `Ctrl/Cmd+W`, `Ctrl/Cmd+1-9`)
- **Keyboard Shortcuts**: A few common shortcuts, such as Enter to open and Escape to clear the selection
- **Context Menus**: Right-click a file for Open, Properties, and Create Archive

### Visual Feedback

- **Progress Indicators**: Progress bars for extraction, archive creation, and checksums
- **Error Alerts**: Clear error messages, including a password prompt when an archive is encrypted

### Appearance

- **Native macOS Look**: Neutral grays, one blue accent color, and the system font (SF Pro)
- **Light and Dark Mode**: Follows the system appearance automatically

## System Integration

### File Associations

- **Automatic Registration**: Handle supported file types automatically
- **System Integration**: Works with your operating system's file manager
- **Double-Click Support**: Open archives directly from file browser
- **Context Menu Integration**: Quick access from right-click menus

### macOS Finder Integration

- **Right-Click Extract**: In Finder, right-click archives for **Extract Here with Rusty** (each archive into the folder that contains it) or **Rusty Extract To…** (choose a folder). Shown only when every selected item is a supported archive. See the [README](https://github.com/vanhonit/rusty-compress#macos-finder-extension-right-click-extract) for setup
- **Background Extraction**: Finder and CLI extraction show only a small progress window. The main window opens for password-protected archives or from the Dock
- **Command Line**: `rusty-compress extract <archive> [--here | --to <dest>]`

### Cross-Platform Compatibility

- **Windows**: Full Windows integration with native look and feel
- **macOS**: Finder integration, native look, and Light and Dark mode
- **Linux**: Comprehensive Linux distribution support
- **Consistent Behavior**: Same functionality across all platforms

## Advanced Features

### Archive Browsing

- **Hierarchical View**: See nested directory structures clearly
- **Name Filter**: Filter the files in the current folder by name

### File Management

- **Path Preservation**: Extracted files keep their original directory structure
- **Batch Operations**: Extract several selected files at once
- **Archive Creation**: Create ZIP and uncompressed TAR archives from selected files, and 7z through the external `7z` tool

### Customization

- **Theme Support**: Light and dark mode, following the system setting

## Upcoming Features

Our roadmap includes exciting new capabilities:

- **7z Browsing and Extraction**: Read and extract 7z archives
- **Split Archives**: Handle multi-part archive files
- **Batch Extraction**: Extract from multiple archives at once inside the app
- **Archive Search**: Search across multiple archives
- **Custom Themes**: User-defined interface themes

<div class="card" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border: none;">
    <div class="card-header">
        <h3 class="card-title" style="color: white;">🚀 Help Shape the Future</h3>
    </div>
    <p style="color: rgba(255,255,255,0.9);">Have a feature idea or enhancement suggestion? We'd love to hear from you! Your feedback helps us prioritize and improve Rusty Compress.</p>
    <a href="https://github.com/vanhonit/rusty-compress/issues" class="btn" style="background: white; color: #667eea;">Suggest a Feature</a>
</div>