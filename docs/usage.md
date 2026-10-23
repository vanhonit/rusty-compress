---
layout: page
title: Usage
subtitle: Learn how to use Rusty Compress effectively
---

## Getting Started with Rusty Compress

This guide covers the essential features and workflows of Rusty Compress to help you become productive quickly.

## Interface Overview

### Main Window Components

```
┌─────────────────────────────────────────┐
│ [🔄] Rusty Compress          [Tabs]    │  ← Tab Bar
├─────────────────────────────────────────┤
│ 📂 /home/user/Documents                │  ← Breadcrumb Navigation
├─────────────────────────────────────────┤
│ [Toolbar: ← → ↑ ↓ ⬇️ 🗑️ 🔍 📋]       │  ← Toolbar
├─────────────────────────────────────────┤
│ 📁 Documents    2024-01-15  4.2 GB     │
│ 📁 Pictures     2024-01-10  1.8 GB     │
│ 📄 archive.zip  2024-01-20  256 MB     │  ← File List
│ 📄 notes.txt    2024-01-18   12 KB     │
├─────────────────────────────────────────┤
│ Selected: 1 item  Total: 4 items       │  ← Status Bar
└─────────────────────────────────────────┘
```

### Navigation Elements

- **Tab Bar**: Open multiple files/archives in tabs
- **Breadcrumbs**: Clear path navigation
- **Toolbar**: Quick access to common actions (Open, Extract…, Extract Here, Checksum, Archive, Select All, Deselect)
- **File List**: Browse files and folders. Click a row to select it, and use the checkbox for multiple selection
- **Search**: Filters the files in the current folder by name

## Opening Archives

### Method 1: File Association

1. Locate your archive file in Finder/Explorer/File Manager
2. Double-click the file
3. Rusty Compress automatically opens the archive

On macOS you can also right-click archives in Finder and use **Extract Here with Rusty** or **Rusty Extract To…** to extract without opening the main window. See [Extract from Finder](#extract-from-finder-macos).

### Method 2: In-App Browser

1. Browse to the folder containing your archive in the main window
2. Double-click the archive file to open it

### Supported Archive Formats

- **ZIP-based**: `.zip`, `.jar`, `.apk`, `.xapk`, `.ipa`
- **TAR-based**: `.tar`, `.tar.gz`, `.tgz`, `.tar.bz2`, `.tbz2`
- **Single-file compressed**: `.gz`, `.bz2` (shown as one entry named after the file, e.g. `kubeconfig.gz` shows `kubeconfig`)
- **RAR**: `.rar`

7z archives can be created but not opened (see [Features](features.html)).

### Password-Protected Archives

ZIP (ZipCrypto or AES) and RAR archives can be encrypted:

1. Open the archive. If it is encrypted, a password prompt appears
2. Enter the password. The app remembers it for the current tab, so browsing, previews, and extraction do not ask again
3. A wrong password shows "incorrect password" and asks again
4. Extraction checks the password before the progress window opens
## Browsing Archives

### Navigation Within Archives

- **Double-click a folder**: Open it
- **Backspace**: Go up one folder
- **Breadcrumbs**: Click any part of the path to jump

### File Selection

- **Single Selection**: Click a file
- **Multiple Selection**: Click the checkbox on each file. Use **Select All** and **Deselect** in the toolbar
- **Clear selection**: `Escape`

## Extracting Files

### Extract Selected Files

1. Browse to the archive and select the files you want
2. Click **Extract…** in the toolbar
3. Choose the destination folder
4. Monitor progress in the extraction window

### Extract Entire Archive

1. Open the archive with nothing selected
2. Click **Extract…** and choose the destination folder. With no selection, everything is extracted

### Extract Here

With an archive open, click **Extract Here** in the toolbar to extract into the folder that contains the archive, with no folder picker. It uses the selection if you have one, otherwise the whole archive.

### Extract from the Command Line

```bash
rusty-compress extract archive.zip              # into the archive's folder
rusty-compress extract archive.zip --here       # same as the default
rusty-compress extract archive.zip --to ~/Desktop/out
```

The destination folder must already exist. The progress window shows the extraction, and the main window is not opened.

### Extract from Finder (macOS)

After the [Finder extension is installed](https://github.com/vanhonit/rusty-compress#macos-finder-extension-right-click-extract), right-click one or more archives in Finder:

- **Extract Here with Rusty**: extracts each archive into the folder that contains it
- **Rusty Extract To…**: choose a destination folder

Only a progress window is shown. Password-protected archives open the main window so you can enter the password. See the [README](https://github.com/vanhonit/rusty-compress#macos-finder-extension-right-click-extract) for setup.

### Safety

Entries whose paths try to leave the destination folder (for example `../../etc/file`) are skipped. Everything else is extracted with its folder structure.

## File Preview

### Previewing Images

1. Hover over an image inside an archive to see a preview
2. Double-click the image to open it in your system's default image viewer. The file is extracted to a temporary folder that is deleted when the last window closes
3. Images from single-file `.gz` and `.bz2` archives can be previewed up to 50 MiB

### Viewing File Details

**Right-click** a file and choose **Properties**. For single-file `.bz2` archives the size is shown as 0, because the format does not store the original size.

### Opening Files in Default Applications

1. Select the file
2. Press `Enter` or double-click
3. File opens in your system's default application

## Computing Checksums

The **Checksum** button is available while an archive is open. It computes the checksum of the archive file itself.

1. Open the archive
2. Click **Checksum** in the toolbar
3. Click **Compute** on the algorithm you need:
   - **MD5**: Fast, basic verification
   - **SHA1**: Legacy compatibility
   - **SHA256**: Recommended for most use cases
   - **SHA512**: Highest security

### Understanding Checksum Results

```
File: documents/report.pdf
Algorithm: SHA256
Checksum: a3b2c1d4e5f6...
Status: ✅ Complete
```

**Use Cases**:
- **Verify Downloads**: Ensure downloaded files aren't corrupted
- **File Integrity**: Check if files have been modified
- **Backup Verification**: Confirm backup copies match originals

## Multiple Tabs

- `Ctrl/Cmd + T`: new tab
- `Ctrl/Cmd + W`: close tab (not the last one)
- `Ctrl/Cmd + 1-9`: jump to a tab by position

Each tab keeps its own location, selection, and archive password.

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Open the single selected item |
| `Escape` | Clear the selection |
| `Backspace` | Go up one folder |
| `Ctrl/Cmd + T` | New tab |
| `Ctrl/Cmd + W` | Close tab |
| `Ctrl/Cmd + 1-9` | Switch to tab |
| `Ctrl/Cmd + N` | Create archive from the selection (filesystem only) |

## Common Workflows

### Workflow 1: Download Verification

1. Download an archive file from the internet
2. Open it in Rusty Compress
3. Click **Checksum** and compute SHA256
4. Compare the result with the published checksum
5. ✅ If matches: File is authentic
6. ❌ If differs: File may be corrupted

### Workflow 2: Archive Exploration

1. Download a large archive (e.g., dataset)
2. Open archive in Rusty Compress
3. Browse through directory structure
4. Preview specific files of interest
5. Select only what you need and click **Extract…**

### Workflow 3: System Backup Verification

1. Locate backup archive
2. Open in Rusty Compress
3. Extract small test file
4. Compute checksum of extracted file
5. Compare with original file's checksum
6. Verify backup integrity

### Workflow 4: Android App Inspection

1. Locate `.apk` or `.xapk` file
2. Open in Rusty Compress
3. Browse application structure
4. Preview app icon and resources
5. Extract specific assets if needed

## Tips and Tricks

### Performance Optimization

- **Large Archives**: Use the "Extract Selected" feature for partial extraction
- **Many Small Files**: Consider extracting to SSD for better performance
- **Network Storage**: Extract to local drive first, then move to network location

### Security Best Practices

- **Unknown Archives**: Extract to a quarantine folder first
- **Executable Files**: Don't run executables from untrusted archives
- **Passwords**: The password is kept only for the open tab, in memory

### Workflow Efficiency

- **Extract Here** extracts next to the archive, without a folder picker
- **Finder right-click** on macOS extracts without opening the main window (see the README)

## Troubleshooting

### Archive Won't Open

**Possible causes**:
- File is corrupted
- Unsupported archive format (7z cannot be opened yet)
- File is password-protected and no password was given

**Solutions**:
- Verify file integrity with checksum
- Check supported formats
- Enter the password when prompted. A damaged encrypted RAR may also report "incorrect password" (a RAR4 limitation)

### Extraction Fails

**Common issues**:
- Insufficient disk space
- Permission denied at the destination
- Corrupted archive. Fake or invalid `.gz` files show a clear error

**Fixes**:
- Free up disk space
- Choose a destination you can write to
- Verify archive integrity first

### Checksum Computation Stuck

**Troubleshoot**:
- Try a different algorithm
- Check system resources
- Verify the file is accessible

<div class="card" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border: none; margin-top: 2rem;">
    <div class="card-header">
        <h3 class="card-title" style="color: white;">🎓 Want to Learn More?</h3>
    </div>
    <p style="color: rgba(255,255,255,0.9);">Explore our advanced guides and API documentation to unlock the full potential of Rusty Compress.</p>
    <a href="/api.html" class="btn" style="background: white; color: #667eea;">API Reference</a>
    <a href="/development.html" class="btn" style="background: white; color: #667eea; margin-left: 0.5rem;">Development Guide</a>
</div>