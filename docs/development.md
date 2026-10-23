---
layout: page
title: Development
subtitle: Contribute to Rusty Compress development
---

## Development Environment

This guide will help you set up a development environment and contribute to Rusty Compress.

## Prerequisites

Before starting development, ensure you have:

- **Node.js** (v16 or higher)
- **npm** or **yarn**
- **Rust** (latest stable)
- **Git**
- **Code Editor** (VS Code recommended)

### Recommended Tools

- **VS Code** with extensions:
  - Tauri Extension Pack
  - Rust Analyzer
  - ESLint
  - Prettier

## Setting Up Development Environment

### Step 1: Fork and Clone

```bash
# Fork the repository on GitHub
# Clone your fork
git clone https://github.com/YOUR_USERNAME/rusty-compress.git
cd rusty-compress

# Add upstream repository
git remote add upstream https://github.com/vanhonit/rusty-compress.git
```

### Step 2: Install Dependencies

```bash
# Install Node.js dependencies
npm install

# Install Rust tools (if not already installed)
rustup update
rustup component add rustfmt clippy
```

### Step 3: Configure Development

```bash
# Install VS Code extensions (optional)
code --install-extension tauri-apps.tauri-vscode
code --install-extension rust-lang.rust-analyzer
```

## Project Structure

```
rusty-compress/
├── src/                      # React Frontend
│   ├── components/           # Reusable components
│   ├── modules/              # Feature modules
│   └── main.jsx              # Entry point
├── src-tauri/                # Rust Backend
│   ├── src/
│   │   ├── lib.rs           # Tauri commands, CLI, file-open handling
│   │   ├── data_type.rs     # Shared types
│   │   ├── utils.rs         # Format detection, password errors, safe_join
│   │   ├── file_manager.rs  # File operations
│   │   ├── zip.rs           # ZIP handler (incl. password)
│   │   ├── tar.rs           # TAR handler
│   │   ├── rar.rs           # RAR handler (incl. password)
│   │   ├── compressed.rs    # Single-file .gz / .bz2
│   │   ├── finder.rs        # Finder right-click URL parsing and requests
│   │   └── sevenz.rs        # 7z creation via external 7z CLI
│   ├── tests/fixtures/      # Test fixtures (crypted.rar)
│   ├── Info.plist           # macOS bundle info, including the rustycompress:// URL scheme
│   ├── Cargo.toml           # Rust dependencies
│   └── tauri.conf.json      # Tauri configuration
├── macos/FinderExtension/   # Finder right-click extension (FinderSync.swift)
├── scripts/build-macos.sh   # macOS build, sign, and install (npm run build:mac)
└── docs/                    # Jekyll documentation
    ├── _layouts/
    ├── _includes/
    └── assets/
```

## Development Workflow

### Running Development Server

```bash
# Full Tauri development mode
npm run tauri dev

# Frontend only (faster for UI changes)
npm run dev
```

### Building for Testing

```bash
# Build React frontend
npm run build

# Build Rust backend
cd src-tauri
cargo build
```

### Running Tests

There are no frontend tests. Backend tests are Rust unit tests:

```bash
# Backend tests
cd src-tauri
cargo test

# Run a single test by name
cargo test <test_name>
```

The tests cover ZIP and RAR password handling (RAR uses `tests/fixtures/crypted.rar`, password `unrar`), TAR extraction (including files larger than 8 KB), gz / bz2 handling, and Finder URL parsing. Zip test archives are built in temporary folders.

## Frontend Development

### React Component Structure

```jsx
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

const MyComponent = () => {
  const [files, setFiles] = useState([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    loadFiles();
  }, []);

  const loadFiles = async () => {
    setLoading(true);
    try {
      const result = await invoke('read_directory', { path: '/tmp' });
      setFiles(result);
    } catch (error) {
      console.error('Failed to load files:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      {loading ? <div>Loading...</div> : (
        <ul>
          {files.map(file => (
            <li key={file.path}>{file.name}</li>
          ))}
        </ul>
      )}
    </div>
  );
};

export default MyComponent;
```

### State Management Patterns

```jsx
// Local component state
const [currentPath, setCurrentPath] = useState('/');
const [selectedFiles, setSelectedFiles] = useState([]);

// Derived state
const selectedCount = selectedFiles.length;
const isDirectoryEmpty = files.length === 0;
```

### Event Handling

```jsx
import { listen } from '@tauri-apps/api/event';

const useProgressListener = (eventName, callback) => {
  useEffect(() => {
    let unlisten;

    const setupListener = async () => {
      unlisten = await listen(eventName, (event) => {
        callback(event.payload);
      });
    };

    setupListener();
    return () => unlisten?.();
  }, [eventName, callback]);
};

// Usage in component
const MyComponent = () => {
  const [progress, setProgress] = useState(0);

  useProgressListener('extract-progress', (data) => {
    const percentage = (data.current / data.total) * 100;
    setProgress(percentage);
  });

  return <progress value={progress} max={100} />;
};
```

## Backend Development

### Adding New Tauri Commands

```rust
// src-tauri/src/lib.rs

use tauri::State;

#[tauri::command]
async fn my_new_command(param: String) -> Result<String, String> {
    // Your logic here
    Ok(format!("Processed: {}", param))
}

// Register the command
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            my_new_command,
            read_directory,
            // ... other commands
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Data Types

```rust
// src-tauri/src/data_type.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_directory: bool,
    pub modified: String,
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionProgress {
    pub current: u64,
    pub total: u64,
    pub filename: String,
}
```

### Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArchiveError {
    #[error("File not found: {0}")]
    FileNotFound(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Archive corrupted: {0}")]
    ArchiveCorrupted(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

// Convert to Tauri-compatible error
impl From<ArchiveError> for String {
    fn from(error: ArchiveError) -> Self {
        error.to_string()
    }
}
```

## Testing

### Frontend Testing

There is no frontend test setup. Check UI changes by running `npm run tauri dev` and testing the flow manually.

### Backend Testing

Unit tests live in `#[cfg(test)]` modules in `utils.rs`, `zip.rs`, `rar.rs`, `tar.rs`, `compressed.rs`, and `finder.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finder_url_parsing() {
        // see src-tauri/src/finder.rs for the real tests
    }
}
```

## Code Quality

### Linting

```bash
# Backend linting (all targets, including tests)
cd src-tauri
cargo clippy --all-targets

# Fix issues automatically
cargo clippy --all-targets --fix
```

### Formatting

```bash
# Format Rust code
cd src-tauri
cargo fmt
```

There is no JavaScript linter or formatter configured in `package.json`.

### macOS Finder Extension

```bash
npm run build:mac                   # build, embed, and sign the app with the Finder extension
npm run build:mac -- --install      # also install to /Applications and register the extension
npm run build:mac -- --skip-tauri   # reuse the last Tauri build
```

See the README section on the Finder extension for the flags and macOS setup.

## Performance Optimization

### Rust Performance

```rust
// Use async for blocking operations
#[tauri::command]
async fn heavy_computation() -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        // CPU-intensive work here
        heavy_operation()
    }).await.map_err(|e| e.to_string())?
}

// Use efficient data structures
use std::collections::HashMap;

// Avoid unnecessary clones
fn process_data(data: &Vec<String>) -> String {
    data.iter()
        .map(|s| s.len())
        .sum::<usize>()
        .to_string()
}
```

### React Performance

```jsx
// Memoize expensive calculations
import { useMemo } from 'react';

const expensiveValue = useMemo(() => {
  return heavyComputation(data);
}, [data]);

// Memoize callbacks
const handleClick = useCallback(() => {
  doSomething(value);
}, [value]);
```

## Debugging

### Frontend Debugging

```jsx
// Add console logging
useEffect(() => {
  console.log('Component mounted', { files, path });
}, [files, path]);

// Debug Tauri commands
const result = await invoke('my_command', { param: 'test' });
console.log('Command result:', result);
```

### Backend Debugging

```rust
// Use logging
use log::{info, debug, error};

#[tauri::command]
async fn my_function() -> Result<String, String> {
    info!("Starting my_function");
    debug!("Processing data");
    
    let result = process_data();
    
    info!("Function completed successfully");
    Ok(result)
}

// Add logging to Cargo.toml
[dependencies]
log = "0.4"
env_logger = "0.10"
```

### Tauri DevTools

```bash
# Enable detailed logging
RUST_LOG=debug npm run tauri dev

# View logs in terminal
# Inspect React DevTools in browser
# Use Tauri API for native debugging
```

## Building for Distribution

```bash
# Build for production
npm run tauri build

# Build for specific platform
npm run tauri build -- --target x86_64-pc-windows-msvc
npm run tauri build -- --target x86_64-apple-darwin
npm run tauri build -- --target x86_64-unknown-linux-gnu
```

## Contributing Guidelines

### Before You Start

1. Check existing issues and pull requests
2. Discuss major changes in an issue first
3. Follow the existing code style
4. Write tests for new features

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add support for 7z archives
fix: resolve extraction progress issue
docs: update installation guide
refactor: simplify file manager component
test: add integration tests for checksum computation
```

### Pull Request Process

1. Create a feature branch
   ```bash
   git checkout -b feature/my-feature
   ```

2. Make your changes
   ```bash
   # Make changes
   cd src-tauri && cargo test && cargo clippy --all-targets
   ```

3. Commit your changes
   ```bash
   git add .
   git commit -m "feat: add my new feature"
   ```

4. Push to your fork
   ```bash
   git push origin feature/my-feature
   ```

5. Create pull request
   - Describe your changes
   - Link related issues
   - Add screenshots for UI changes

## Documentation

### Code Comments

```rust
/// Reads the contents of a directory
///
/// # Arguments
/// * `path` - The absolute path to the directory
///
/// # Returns
/// A vector of `FileInfo` structs representing directory contents
///
/// # Errors
/// Returns an error if the directory doesn't exist or lacks permissions
#[tauri::command]
async fn read_directory(path: String) -> Result<Vec<FileInfo>, String> {
    // Implementation
}
```

### API Documentation

Keep the API documentation up to date:
- Add examples for new commands
- Document parameter types
- Describe error cases
- Update type definitions

## Getting Help

### Resources

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [React Documentation](https://react.dev/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [Project Issues](https://github.com/vanhonit/rusty-compress/issues)

### Community

- GitHub Discussions
- Stack Overflow (tauri tag)
- Discord server (if available)

<div class="card" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border: none; margin-top: 2rem;">
    <div class="card-header">
        <h3 class="card-title" style="color: white;">🚀 Ready to Contribute?</h3>
    </div>
    <p style="color: rgba(255,255,255,0.9);">We welcome contributions from developers of all skill levels. Check out our open issues and start making a difference!</p>
    <a href="https://github.com/vanhonit/rusty-compress/issues" class="btn" style="background: white; color: #667eea;">Find Issues to Work On</a>
    <a href="/contributing.html" class="btn" style="background: white; color: #667eea; margin-left: 0.5rem;">Contributing Guide</a>
</div>