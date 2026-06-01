#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MODEL_DIR="$HOME/.cache/whisper"
MODEL_FILE="$MODEL_DIR/ggml-small.bin"
SERVICE_DIR="$HOME/.config/systemd/user"
SERVICE_SRC="$SCRIPT_DIR/live-dictation.service"
SERVICE_DST="$SERVICE_DIR/live-dictation.service"

echo "==> Installing system dependencies…"
sudo apt-get install -y \
    xdotool xclip x11-utils libnotify-bin \
    libx11-dev libxtst-dev libasound2-dev libpulse-dev libclang-dev \
    pkg-config build-essential curl

echo "==> Ensuring user is in the 'input' group (needed for keyboard listening)…"
if ! groups | grep -q '\binput\b'; then
    sudo usermod -aG input "$USER"
    echo "    Added $USER to input group. You must log out and back in for this to take effect."
else
    echo "    Already in input group."
fi

echo "==> Downloading GGML Whisper small model (~466 MB)…"
mkdir -p "$MODEL_DIR"
if [[ ! -f "$MODEL_FILE" ]]; then
    curl -L \
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin" \
        -o "$MODEL_FILE"
    echo "    Model saved to $MODEL_FILE"
else
    echo "    Model already present at $MODEL_FILE"
fi

echo "==> Building release binary…"
cd "$SCRIPT_DIR"
cargo build --release --features cuda

BINARY="$SCRIPT_DIR/target/release/dictate"
echo "    Binary: $BINARY"

echo "==> Installing systemd user service…"
mkdir -p "$SERVICE_DIR"
sed "s|%BINARY%|$BINARY|g; s|%MODEL_PATH%|$MODEL_FILE|g" "$SERVICE_SRC" > "$SERVICE_DST"
systemctl --user daemon-reload
systemctl --user enable --now live-dictation

echo ""
echo "Done! Service status:"
systemctl --user status live-dictation --no-pager || true
echo ""
echo "Useful commands:"
echo "  journalctl --user -u live-dictation -f   # live logs"
echo "  systemctl --user restart live-dictation  # restart"
echo ""
if ! groups | grep -q '\binput\b'; then
    echo "IMPORTANT: Log out and back in to activate the 'input' group membership."
fi
