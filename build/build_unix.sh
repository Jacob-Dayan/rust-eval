#!/usr/bin/env bash
set -e

cd "$(dirname "$0")/.."

echo "Building rust-eval (release mode)..."
cargo build --release

TARGET_DIR="/usr/local/bin"
echo "Installing binary to $TARGET_DIR..."
if [ -w "$TARGET_DIR" ]; then
    cp -f ./target/release/rs-eval "$TARGET_DIR/rs-eval"
else
    sudo cp -f ./target/release/rs-eval "$TARGET_DIR/rs-eval"
fi

chmod +x "$TARGET_DIR/rs-eval"
echo "Done! Installed rs-eval to $TARGET_DIR/rs-eval."
