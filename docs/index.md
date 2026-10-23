---
layout: page
title: Home
subtitle: A modern, cross-platform desktop archive manager
---

<div class="hero-section" style="text-align: center; padding: 3rem 0;">
    <div style="max-width: 800px; margin: 0 auto;">
        <h2 style="font-size: 2.5rem; margin-bottom: 1.5rem; color: #667eea;">Welcome to Rusty Compress</h2>
        <p style="font-size: 1.25rem; color: #4a5568; margin-bottom: 2rem;">
            A powerful, cross-platform desktop archive manager built with Tauri and React. 
            Manage ZIP, TAR, RAR, and single-file GZ / BZ2 archives, including password-protected ones, with a native, intuitive interface.
        </p>
        <div style="display: flex; gap: 1rem; justify-content: center; flex-wrap: wrap; margin-top: 2rem;">
            <a href="/installation.html" class="btn">Get Started</a>
            <a href="https://github.com/vanhonit/rusty-compress" class="btn btn-secondary">View on GitHub</a>
        </div>
    </div>
</div>

## Key Features

<div class="grid grid-3" style="margin-top: 3rem;">
    <div class="card">
        <div class="card-header">
            <h3 class="card-title">📦 Multi-Format Support</h3>
        </div>
        <p>Handle ZIP, JAR, APK, XAPK, IPA, TAR, TAR.GZ, TGZ, TAR.BZ2, TBZ2, single-file GZ / BZ2, and RAR archives with a unified interface.</p>
    </div>

    <div class="card">
        <div class="card-header">
            <h3 class="card-title">🔑 Password Support</h3>
        </div>
        <p>Browse, preview, and extract encrypted ZIP (ZipCrypto and AES) and RAR archives. The password is remembered for each tab.</p>
    </div>

    <div class="card">
        <div class="card-header">
            <h3 class="card-title">🖱️ Finder Integration</h3>
        </div>
        <p>On macOS, right-click an archive in Finder to extract it with <em>Extract Here with Rusty</em> or <em>Rusty Extract To…</em>.</p>
    </div>

    <div class="card">
        <div class="card-header">
            <h3 class="card-title">🖥️ Cross-Platform</h3>
        </div>
        <p>Works seamlessly on Windows, macOS, and Linux with consistent functionality and performance across all platforms.</p>
    </div>

    <div class="card">
        <div class="card-header">
            <h3 class="card-title">⚡ High Performance</h3>
        </div>
        <p>Rust backend provides lightning-fast archive operations while React frontend ensures a smooth, responsive user experience.</p>
    </div>

    <div class="card">
        <div class="card-header">
            <h3 class="card-title">🔍 Smart Browsing</h3>
        </div>
        <p>Browse both filesystem directories and archive contents seamlessly with dual-state tracking and intuitive navigation.</p>
    </div>

    <div class="card">
        <div class="card-header">
            <h3 class="card-title">📊 Progress Tracking</h3>
        </div>
        <p>Real-time progress updates for extraction and checksum operations with detailed status reporting.</p>
    </div>

    <div class="card">
        <div class="card-header">
            <h3 class="card-title">🔐 Checksum Verification</h3>
        </div>
        <p>Compute MD5, SHA1, SHA256, and SHA512 checksums with progress reporting to verify file integrity.</p>
    </div>
</div>

## Quick Start

<div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); padding: 2rem; border-radius: 8px; color: white; margin: 2rem 0;">
    <h3 style="color: white; border: none;">Ready to Get Started?</h3>
    <p style="color: rgba(255,255,255,0.9);">Follow these simple steps to get Rusty Compress running on your system:</p>
    <ol style="color: rgba(255,255,255,0.9);">
        <li>Install Node.js, npm, and Rust on your system</li>
        <li>Clone the repository: <code>git clone https://github.com/vanhonit/rusty-compress.git</code></li>
        <li>Navigate to the directory and install dependencies</li>
        <li>Run <code>npm run tauri dev</code> to start the development server</li>
        <li>On macOS, run <code>npm run build:mac -- --install</code> to add the right-click Finder menu</li>
    </ol>
    <a href="/installation.html" class="btn" style="background: white; color: #667eea;">View Full Installation Guide</a>
</div>

## System Integration

Rusty Compress integrates seamlessly with your operating system:

- **File Associations**: Automatically handles supported archive types
- **System Menus**: Right-click integration for quick actions
- **Double-Click**: Open archives directly from your file manager
- **Finder Integration**: Right-click archives in Finder on macOS to extract them without opening the main window
- **Preview**: View images directly from archives without extraction

## Technology Stack

<div class="grid grid-2">
    <div class="card">
        <div class="card-header">
            <h3 class="card-title">Frontend</h3>
        </div>
        <ul>
            <li><strong>React 18</strong> - Modern, component-based UI</li>
            <li><strong>Vite</strong> - Lightning-fast build tool</li>
            <li><strong>Tailwind CSS</strong> - Utility-first styling</li>
            <li><strong>Tauri 2.x</strong> - Desktop framework integration</li>
        </ul>
    </div>

    <div class="card">
        <div class="card-header">
            <h3 class="card-title">Backend</h3>
        </div>
        <ul>
            <li><strong>Rust</strong> - High-performance native code</li>
            <li><strong>Tauri Core</strong> - Secure desktop API</li>
            <li><strong>Archive Libraries</strong> - Native format handlers</li>
            <li><strong>Async Runtime</strong> - Efficient task management</li>
        </ul>
    </div>
</div>

## Get Involved

Rusty Compress is an open-source project, and we welcome contributions from developers of all skill levels!

- 🐛 **Report Bugs**: Help us improve by reporting issues
- 💡 **Suggest Features**: Share your ideas for improvements
- 🔧 **Submit Pull Requests**: Contribute code and documentation
- 📖 **Improve Documentation**: Help make the project more accessible

<div style="display: flex; gap: 1rem; justify-content: center; flex-wrap: wrap; margin: 2rem 0;">
    <a href="https://github.com/vanhonit/rusty-compress/issues" class="btn">Report Issues</a>
    <a href="/contributing.html" class="btn btn-secondary">Contribute</a>
</div>

---

<div style="text-align: center; color: #718096; margin-top: 3rem;">
    <p>Built with ❤️ using Tauri and React</p>
    <p>© {{ 'now' | date: "%Y" }} Hon Nguyen. Licensed under MIT License.</p>
</div>