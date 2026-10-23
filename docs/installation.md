---
layout: page
title: Installation
subtitle: Get Rusty Compress running on your system
---

## System Requirements

Before installing Rusty Compress, ensure your system meets these requirements.

### Prerequisites

- **Node.js** (v16 or higher) - [Download Node.js](https://nodejs.org/)
- **npm** or **yarn** - Comes with Node.js
- **Rust** (latest stable) - [Install via rustup](https://www.rust-lang.org/tools/install)

### Platform-Specific Dependencies

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

#### Linux
```bash
# Ubuntu/Debian
sudo apt install build-essential libwebkit2gtk-4.0-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

# Fedora
sudo dnf install webkit2gtk3-devel openssl-devel gtk3-devel libappindicator-gtk3-devel

# Arch Linux
sudo pacman -S base-devel webkit2gtk openssl gtk3 libappindicator-gtk3
```

#### Windows
- Install [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- Install [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

## Installation Methods

### Method 1: Clone from Repository (Recommended for Development)

#### Step 1: Clone the Repository

```bash
git clone https://github.com/vanhonit/rusty-compress.git
cd rusty-compress
```

#### Step 2: Install Dependencies

```bash
npm install
```

#### Step 3: Build and Run

```bash
npm run tauri dev
```

This will:
- Build the Rust backend
- Start the Vite development server
- Launch the application window

### Method 2: Build the macOS App with the Finder Extension

On macOS, `npm run build:mac` builds the app and the right-click Finder extension, then signs them. It needs Xcode Command Line Tools (`xcode-select --install`), which provide `swiftc` and `lipo`.

```bash
npm install
npm run build:mac                 # build and sign the app
npm run build:mac -- --install    # also copy to /Applications and register the extension
```

Flags:

| Flag | Effect |
|------|--------|
| `--skip-tauri` | Reuse the last Tauri build instead of rebuilding |
| `--install` | Copy the app to `/Applications` and register the Finder extension |
| `--restart-finder` | Run `killall Finder` after `--install` |
| `--dmg` | Also create a DMG in `src-tauri/target/release/bundle/dmg/` |

Signing uses a free **Apple Development** certificate if you have one (Xcode > Settings > Accounts). Otherwise it falls back to ad-hoc signing, which works on your own Mac. The first signed build may show a keychain prompt. Click **Always Allow**.

Then turn on the extension in **System Settings > General > Login Items & Extensions > Finder**, or run `pluginkit -e use -i com.rusty-compress.app.FinderExtension`. If the menu does not appear, run `killall Finder`. See the [README](https://github.com/vanhonit/rusty-compress#macos-finder-extension-right-click-extract) for details, including Gatekeeper steps on other Macs.

### Method 3: Using Package Manager (Coming Soon)

We're working on package distribution for easier installation:

- **Homebrew** (macOS): `brew install rusty-compress`
- **Snap** (Linux): `sudo snap install rusty-compress`
- **Chocolatey** (Windows): `choco install rusty-compress`

### Method 4: Download Pre-built Binaries (Coming Soon)

Pre-built installers will be available for:

- **macOS**: `.dmg` installer
- **Windows**: `.exe` installer and `.msi`
- **Linux**: `.AppImage`, `.deb`, and `.rpm` packages

## Verification

After installation, verify Rusty Compress is working correctly:

```bash
# Test the application
npm run tauri dev

# Run backend tests
cd src-tauri
cargo test

# Run linter
cargo clippy --all-targets
```

## Troubleshooting

### Common Issues

#### Rust Installation Issues

**Problem**: Rust compiler not found

**Solution**:
```bash
# Update Rust toolchain
rustup update

# Ensure Rust is in PATH
source $HOME/.cargo/env
```

#### Node.js Version Issues

**Problem**: Node.js version too old

**Solution**:
```bash
# Use nvm (Node Version Manager)
nvm install 18
nvm use 18
```

#### Build Failures on Linux

**Problem**: Missing system dependencies

**Solution**:
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install libwebkit2gtk-4.0-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

#### Windows Build Issues

**Problem**: C++ Build Tools missing

**Solution**:
1. Download [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
2. Install "Desktop development with C++"
3. Restart terminal and try again

#### macOS Finder Menu Missing

**Problem**: "Extract Here with Rusty" does not appear in the Finder context menu

**Solution**:
```bash
pluginkit -e use -i com.rusty-compress.app.FinderExtension
pluginkit -m -i com.rusty-compress.app.FinderExtension   # a leading "+" means enabled
killall Finder
```

#### macOS Permission Issues

**Problem**: Permission denied errors

**Solution**:
```bash
# Fix permissions for node_modules
sudo chown -R $(whoami) ~/.npm
sudo chown -R $(whoami) /usr/local/lib/node_modules

# Alternative: Use Node Version Manager
nvm install 18
nvm use 18
```

### Development Server Issues

#### Port Already in Use

**Problem**: Vite development server port 1420 already in use

**Solution**:
```bash
# Find and kill process using port 1420
lsof -ti:1420 | xargs kill -9

# Or use a different port by modifying vite.config.js
```

#### Hot Module Replacement (HMR) Not Working

**Problem**: Changes not reflected in browser

**Solution**:
```bash
# Clear cache and restart
rm -rf node_modules/.vite
npm run tauri dev
```

## Development Setup

For active development, follow these additional steps:

### IDE Setup

**VS Code** (Recommended):
```bash
# Install extensions
code --install-extension tauri-apps.tauri-vscode
code --install-extension rust-lang.rust-analyzer
code --install-extension dbaeumer.vscode-eslint
```

**Recommended VS Code Settings**:
```json
{
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "rust-analyzer.procMacro.enable": true
}
```

### Environment Variables

Set these in your shell when you need them:

```bash
# Use a LAN address for the dev server (also enables HMR on port 1421)
TAURI_DEV_HOST=192.168.1.10

# Rust logging
RUST_LOG=debug
RUST_BACKTRACE=1
```

## Build Configuration

### Production Build

```bash
# Build React frontend
npm run build

# Build complete application
npm run tauri build
```

Build artifacts will be in `src-tauri/target/release/bundle/`:

- **macOS**: `.dmg` file (or use `npm run build:mac` for the app with the Finder extension)
- **Windows**: `.exe` and `.msi` installers
- **Linux**: `.AppImage`, `.deb`, and `.rpm` packages

### Custom Build Configuration

Edit `tauri.conf.json` to customize:

```json
{
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devPath": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "tauri": {
    "bundle": {
      "identifier": "com.rustycompress.app",
      "icon": ["icons/icon.png"],
      "targets": "all"
    }
  }
}
```

## Next Steps

Once installed:

1. **[Explore Features](features.html)** - Learn about what Rusty Compress can do
2. **[Read Usage Guide](usage.html)** - How to use the application
3. **[Check API Reference](api.html)** - Backend commands and functions
4. **[Development Guide](development.html)** - How to contribute

<div class="card" style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; border: none; margin-top: 2rem;">
    <div class="card-header">
        <h3 class="card-title" style="color: white;">💡 Need Help?</h3>
    </div>
    <p style="color: rgba(255,255,255,0.9);">If you encounter any issues during installation, please check our troubleshooting section or open an issue on GitHub.</p>
    <a href="https://github.com/vanhonit/rusty-compress/issues" class="btn" style="background: white; color: #667eea;">Get Support</a>
</div>