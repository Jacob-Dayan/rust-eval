#!/usr/bin/env bash
set -e

COMPILE_CMD="cargo build --release"

cd "$(dirname "$0")/.."
sudo -v

echo $COMPILE_CMD
$COMPILE_CMD

TARGET_DIR="/usr/local/bin"
echo "Installing binary to $TARGET_DIR..."
if [ -w "$TARGET_DIR" ]; then
    cp -f ./target/release/rs-eval "$TARGET_DIR/rs-eval"
else
    sudo cp -f ./target/release/rs-eval "$TARGET_DIR/rs-eval"
fi

sudo chmod +x "$TARGET_DIR/rs-eval"
echo "Done! Installed rs-eval to $TARGET_DIR/rs-eval."
