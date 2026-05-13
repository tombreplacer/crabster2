# Crabster 🦀

A lightweight, cross-platform file server in Rust with a beautiful single-file Web UI. 

Crabster is designed to be a single binary that you can drop onto any server or computer to instantly share files via a modern browser interface. It supports uploading, downloading, folder navigation, and basic file management.

## Features

- ⚡️ **Extremely Fast**: Built on Rust and `actix-web`.
- 📦 **Single Binary**: The entire Svelte Web UI (HTML, CSS, JS, SVG) is compiled into a single 77 KB file and embedded directly into the Rust binary. 
- 🎨 **Modern Web UI**: Features a beautiful dark theme, glassmorphism, drag-and-drop uploads, context menus, and breadcrumb navigation.
- 🛡️ **Secure by Default**: Built-in path traversal protection.
- 🔒 **Read-only Mode**: Prevent accidental deletions or uploads by launching in read-only mode.
- 🖥️ **Cross Platform**: Works on Linux, macOS, and Windows.

## Installation / Building for Production

To build Crabster from source, you need Node.js (for the Svelte frontend) and Rust/Cargo (for the backend).

### 1. Build the Frontend
The frontend must be built first so that the Rust compiler can embed `index.html`.

```bash
cd web
npm install
npm run build
```

### 2. Build the Backend
Return to the project root and build the Rust release binary. The `Cargo.toml` is already configured for maximum optimization (`opt-level = "z"`, `strip = true`, `lto = true`).

```bash
cd ..
cargo build --release
```

The resulting binary will be located at `target/release/crabster`. It's a completely standalone file (~3 MB) that you can move and run anywhere!

## Usage

```bash
# Run on the default port (8080) serving the current directory
./crabster

# Serve a specific directory on a custom port
./crabster --port 9090 --dir /path/to/share

# Run in read-only mode (disables uploads, directory creation, and deletions)
./crabster --readonly

# Allow viewing hidden files (files starting with a dot)
./crabster --hidden

# Disable deletions but allow uploads
./crabster --no-delete
```

### Shell Completions

Crabster supports generating shell completions for bash, zsh, fish, and powershell:

```bash
# Example for bash
./crabster --completions bash > ~/.local/share/bash-completion/completions/crabster
```
