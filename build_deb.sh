#!/bin/bash
set -e

echo "🦀 Building Svelte frontend..."
cd web
npm install
npm run build
cd ..

echo "🦀 Checking for cargo-deb..."
if ! command -v cargo-deb &> /dev/null; then
    echo "cargo-deb not found. Installing..."
    cargo install cargo-deb
fi

echo "🦀 Compiling binary and generating shell completions..."
cargo build --release
./target/release/crabster --completions bash > crabster.bash
./target/release/crabster --completions zsh > _crabster
./target/release/crabster --completions fish > crabster.fish

echo "🦀 Building Debian package..."
cargo deb

echo "✅ Build complete! You can find the .deb file in target/debian/"
