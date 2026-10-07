# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## What this is

Rust port of the push-to-talk dictation daemon. Hold Right Alt (AltGr) → audio is recorded → release → whisper.cpp (via whisper-rs) transcribes → xclip + a uinput virtual keyboard paste the result into the focused window. Works on Wayland (GNOME 50 / Ubuntu 26.04) and X11. Supports pt-BR and en-US (auto-detected per utterance).

The original Python version lives at `../voice-to-text/`.

## Running and managing the service

```bash
# Service lifecycle
systemctl --user status live-dictation
systemctl --user restart live-dictation
systemctl --user stop live-dictation
systemctl --user start live-dictation

# Live logs
journalctl --user -u live-dictation -f

# Run directly (stop service first to avoid mic conflicts)
systemctl --user stop live-dictation
WHISPER_MODEL_PATH=~/.cache/whisper/ggml-small.bin cargo run --release
```

## Changing the Whisper model

`install.sh` is the single source of truth for the model. To switch models:

1. Edit `MODEL_FILE` at the top of `install.sh` (e.g. `ggml-large-v3-turbo.bin`)
2. Run `./install.sh` — it downloads the model if needed, rebuilds, and restarts the service

Available models (GGML format from `huggingface.co/ggerganov/whisper.cpp`):
`tiny` (~75 MB) · `base` (~142 MB) · `small` (~466 MB) · `medium` (~1.5 GB) · `large-v3-turbo` (~809 MB) · `large-v3` (~2.9 GB)

`WHISPER_MODEL_PATH` is required — the binary exits with an error if it's not set.

## Installation (first time or after rebuilding)

```bash
./install.sh
```

This installs apt packages, adds user to `input` group (needed by evdev to read keyboards), installs a udev rule giving the `input` group access to `/dev/uinput` (needed to paste on Wayland), downloads the configured GGML model to `~/.cache/whisper/`, builds the release binary, and enables the systemd user service.

**Important:** After `install.sh`, log out and back in if the `input` group was newly added.

## Building

```bash
cargo build           # debug build
cargo build --release # optimised release build (used by service)
```

Optional CUDA support (requires CUDA toolkit):
```bash
cargo build --release --features cuda
```

## Architecture (`src/main.rs`)

Three concurrent execution contexts:

| Context | Role |
|---|---|
| Main thread | Runs `listen_evdev` (blocks on one reader thread per keyboard in `/dev/input`); fires key-press/release callbacks |
| `cpal` audio thread | Fires per audio block; appends to `audio_frames` while `recording` is true |
| `transcription-worker` thread | Blocks on `mpsc::Receiver`; calls whisper-rs then `inject_text()` |

**Data flow:** `KEY_RIGHTALT` press sets `recording` → cpal callback accumulates f32 PCM frames → `KEY_RIGHTALT` release clears `recording`, snapshots frames, sends `Vec<f32>` over channel → worker calls `state.full()`, calls `inject_text()` → clipboard + paste keystroke.

**Key globals:** `recording` (AtomicBool), `audio_frames` (Mutex<Vec<f32>>), mpsc channel.

**Model format:** GGML `.bin` file (whisper.cpp format), not CTranslate2. Currently using `ggml-small.bin` (~466 MB). Downloaded from `huggingface.co/ggerganov/whisper.cpp`. Path set via `WHISPER_MODEL_PATH` env var (required — binary exits if unset). To change model, see "Changing the Whisper model" above.

**Keyboard listener:** the `evdev` crate reads `/dev/input/event*` directly (works on Wayland and X11) — requires membership in the `input` group. `rdev::listen` was dropped because it uses X11 XRecord, which sees no keys on Wayland. Hotplugged keyboards need a service restart.

**Text injection:** `xclip -selection clipboard` (via XWayland, bridged to the Wayland clipboard), then `uinput_paste()` emits Ctrl+V (Ctrl+Shift+V for terminals) from a virtual keyboard; needs `/dev/uinput` access (udev rule from `install.sh`). If uinput fails, it falls back to `xdotool key` (X11/XWayland only). On Wayland the active window can't be detected for native windows, so the class defaults to `unknown-terminal` → Ctrl+Shift+V.

**Pre-paste delay:** `PRE_TYPE_SLEEP_MS = 50` lets the push-to-talk key-up be processed before the paste fires.

**Minimum audio gate:** `MIN_AUDIO_SAMPLES = 8000` (0.5 s at 16 kHz) prevents hallucination on accidental brief presses.

## systemd service notes

`live-dictation.service` uses `%BINARY%` and `%MODEL_PATH%` as placeholders substituted by `install.sh`. The service passes `DISPLAY`, `XAUTHORITY`, `DBUS_SESSION_BUS_ADDRESS`, and `XDG_RUNTIME_DIR` from the user environment. No `LD_LIBRARY_PATH` needed. Memory capped at 2 GB.
