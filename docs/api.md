---
layout: page
title: API Reference
subtitle: Tauri commands and backend functions
---

## Overview

Rusty Compress exposes a set of Tauri commands that allow the React frontend to communicate with the Rust backend. These commands handle file operations, archive management, and system integration.

## Command Structure

All commands follow the Tauri pattern:

```javascript
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('command_name', { 
  param1: 'value1', 
  param2: 'value2' 
});
```

## Command Reference

The frontend calls commands with `invoke` and camelCase argument names. `?` marks an optional argument. Commands return `Result<T, String>`: the error is the rejection value.

| Command | Arguments | Returns |
|---------|-----------|---------|
| `read_directory` | `path` | `FileInfo[]` |
| `open_file` | `path`, `filePath?` | `FileInfo[] \| null` (archive contents for `.zip` / `.tar`, otherwise `null` after opening in the system app) |
| `archive_file_details` | `sourcePath`, `filePath?`, `password?` | `FileInfo[]` |
| `extract_files` | `archivePath`, `outputPath`, `selectedFiles?`, `password?` | `void` |
| `view_file_in_archive` | `archivePath`, `fileName`, `password?` | `void` |
| `get_image_preview` | `archivePath`, `filePath`, `password?` | data URL `string` |
| `compute_checksum` | `filePath`, `algorithm` | `{ algorithm, hash }` |
| `path_exists` | `path` | `boolean` |
| `create_archive` | `files`, `outputPath`, `archiveFormat` (`zip`, `tar`, `7z`), `compressionLevel` | `void` |
| `take_files` | none | `string[]` (file-open paths buffered before the UI was ready) |

`FileInfo` is `{ name, is_dir, last_modified, size, mimetype, path }`.

## Passwords

Archive commands for ZIP and RAR accept an optional `password`. Errors use these prefixes, which the frontend uses to show the password prompt:

| Prefix | Meaning |
|--------|---------|
| `PASSWORD_REQUIRED:` | The archive is encrypted and no password was given |
| `INVALID_PASSWORD:` | The password was given but is wrong |

```javascript
try {
  await invoke('extract_files', {
    archivePath: '/home/user/secret.zip',
    outputPath: '/home/user/out',
    password: 'hunter2',
  });
} catch (err) {
  if (String(err).startsWith('PASSWORD_REQUIRED:') || String(err).startsWith('INVALID_PASSWORD:')) {
    // show the password prompt and retry
  }
}
```

For `extract_files`, the password is checked before the progress window opens, so a wrong password is returned as the rejection. Failures after that point arrive as `extract-error` events.

## File System Commands

### read_directory

List contents of a filesystem directory.

**Arguments**: `path` (string)

**Example**:
```javascript
const files = await invoke('read_directory', { path: '/home/user/Documents' });
```

### open_file

Open a file with the system's default application. For `.zip` and `.tar` files it returns the archive contents instead.

**Arguments**: `path` (string), `filePath?` (string)

## Archive Management Commands

### archive_file_details

Get the contents of an archive, optionally filtered by a subdirectory.

**Arguments**:
- `sourcePath` (string): path to the archive
- `filePath` (string, optional): path inside the archive (empty or omitted for the root)
- `password` (string, optional): required for encrypted ZIP and RAR archives

**Example**:
```javascript
const rootFiles = await invoke('archive_file_details', {
  sourcePath: '/home/user/archive.zip',
  filePath: '',
});

const subDirFiles = await invoke('archive_file_details', {
  sourcePath: '/home/user/archive.zip',
  filePath: 'subfolder/documents',
  password: 'hunter2',   // only needed for encrypted archives
});
```

**Archive Format Support**:
- ZIP family (zip, jar, apk, xapk, ipa): browse, with password
- TAR family (tar, tar.gz, tgz, tar.bz2, tbz2): browse
- Single-file gz / bz2: one entry named after the file (`.bz2` size is 0)
- RAR: browse, with password
- 7z: not supported for browsing

### extract_files

Extract files from an archive into a destination folder. The progress window opens after the password check. Entries whose paths would leave the destination (`..`, absolute paths) are skipped.

**Arguments**:
- `archivePath` (string)
- `outputPath` (string): destination folder
- `selectedFiles` (string[], optional): entries to extract; omit for everything
- `password` (string, optional)

**Example**:
```javascript
await invoke('extract_files', {
  archivePath: '/home/user/archive.zip',
  outputPath: '/home/user/extracted',
  selectedFiles: ['folder/file1.txt', 'folder/file2.pdf'],
});
```

**Events**: progress and completion are reported by the extraction window through `extract-progress`, `extract-complete`, and `extract-error`. See [Event System](#event-system).

### view_file_in_archive

Extract one file to a temporary folder and open it with the system's default app. The temporary folder is removed when the last window closes.

**Arguments**:
- `archivePath` (string)
- `fileName` (string): path of the entry inside the archive
- `password` (string, optional)

```javascript
await invoke('view_file_in_archive', {
  archivePath: '/home/user/archive.zip',
  fileName: 'documents/report.pdf',
});
```

### get_image_preview

Return an image from an archive as a `data:` URL.

**Arguments**:
- `archivePath` (string)
- `filePath` (string): image path inside the archive
- `password` (string, optional)

**Supported image formats**: jpg, jpeg, png, gif, webp, bmp, tiff, tif. For single-file `.gz` / `.bz2` archives, the decompressed image is limited to 50 MiB.

```javascript
const imageUrl = await invoke('get_image_preview', {
  archivePath: '/home/user/archive.zip',
  filePath: 'images/photo.jpg',
});
```

### create_archive

Create a ZIP, uncompressed TAR, or 7z archive from files. 7z needs the external `7z` command-line tool.

**Arguments**:
- `files` (string[])
- `outputPath` (string)
- `archiveFormat` (`'zip' | 'tar' | '7z'`)
- `compressionLevel` (number)

## Checksum Commands

### compute_checksum

Compute a file checksum, reporting progress in 5% steps.

**Arguments**:
- `filePath` (string)
- `algorithm` (string): `'md5'`, `'sha1'`, `'sha256'`, or `'sha512'`. Case is ignored

**Returns**: `{ algorithm, hash }`

**Events**: `checksum-progress-{algorithm}` with the lowercase algorithm name, e.g. `checksum-progress-sha256`. The payload is a percentage number.

```javascript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen('checksum-progress-sha256', (event) => {
  console.log(`Computing checksum: ${event.payload}%`);
});

const { hash } = await invoke('compute_checksum', {
  filePath: '/home/user/document.pdf',
  algorithm: 'SHA256',
});

unlisten();
```

**Algorithm Comparison**:

| Algorithm | Speed | Security | Common Use |
|-----------|-------|----------|------------|
| MD5 | Fastest | Low | Quick verification |
| SHA1 | Fast | Medium | Legacy compatibility |
| SHA256 | Medium | High | General purpose |
| SHA512 | Slower | Highest | Security-sensitive |

## Event System

### Available Events

| Event | Direction | Payload |
|-------|-----------|---------|
| `extract-progress` | backend to UI | `{ files, total, percentage }` |
| `extract-complete` | backend to UI | none |
| `extract-error` | backend to UI | error string |
| `compress-progress` | backend to UI | `{ percentage, files_processed, current_file }` |
| `compress-complete` | backend to UI | none |
| `checksum-progress-{algorithm}` | backend to UI | percentage, in 5% steps (`{algorithm}` is lowercase, e.g. `sha256`) |
| `file-open` | backend to UI | file path |
| `frontend-ready` | UI to backend | none |
| `{label}://ready` | progress window to backend | none |

Extraction failures after the password check arrive only as `extract-error`, not as the `invoke` result.

### Event Listener Pattern

```javascript
import { listen } from '@tauri-apps/api/event';

// General pattern
const unlisten = await listen('extract-progress', (event) => {
  const { files, total, percentage } = event.payload;
});

// Clean up when the component unmounts
useEffect(() => {
  return () => unlisten();
}, []);
```

## Error Handling

Commands return `Result<T, String>`, so the error reaches JavaScript as a plain string in the `invoke` rejection. There is no structured error object.

Password errors carry a prefix and are handled as described in [Passwords](#passwords).

```javascript
try {
  await invoke('archive_file_details', { sourcePath: archivePath });
} catch (error) {
  const message = String(error);
  if (message.startsWith('PASSWORD_REQUIRED:') || message.startsWith('INVALID_PASSWORD:')) {
    // ask for the password
  } else {
    showErrorNotification(message);
  }
}
```
## Performance Considerations

### Async Operations

All commands are asynchronous and non-blocking:

```javascript
// This won't block the UI
const files = await invoke('read_directory', { path: '/large/directory' });
```

### Memory Management

- Large files are processed in chunks
- Temporary files are cleaned up automatically
- Archive contents are cached for performance

### Optimized Workflows

```javascript
// Bad: Multiple sequential operations
const files1 = await invoke('read_directory', { path: '/dir1' });
const files2 = await invoke('read_directory', { path: '/dir2' });

// Good: Parallel operations
const [files1, files2] = await Promise.all([
  invoke('read_directory', { path: '/dir1' }),
  invoke('read_directory', { path: '/dir2' })
]);
```

## Type Definitions

For TypeScript support, create type definitions:

```typescript
// types/tauri-commands.ts
export interface FileInfo {
  name: string;
  path: string;
  size: number;
  is_directory: boolean;
  modified: string;
  file_type: string;
}

export interface ExtractionProgress {
  current: number;
  total: number;
  filename: string;
}

export interface ChecksumProgress {
  progress: number;
  current_block: number;
  total_blocks: number;
}
```

## Testing Commands

```javascript
// Test file system operations
const testFiles = async () => {
  const files = await invoke('read_directory', { path: '/tmp' });
  console.log('Files in /tmp:', files.length);
};

// Test archive operations
const testArchive = async () => {
  const contents = await invoke('archive_file_details', {
    archive_path: '/test/archive.zip',
    file_path: '',
    state: 'test'
  });
  console.log('Archive contents:', contents);
};
```

<div class="card" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border: none; margin-top: 2rem;">
    <div class="card-header">
        <h3 class="card-title" style="color: white;">🔧 Advanced Usage</h3>
    </div>
    <p style="color: rgba(255,255,255,0.9);">For more advanced usage patterns and integration examples, check out our development guide.</p>
    <a href="/development.html" class="btn" style="background: white; color: #667eea;">Development Guide</a>
</div>