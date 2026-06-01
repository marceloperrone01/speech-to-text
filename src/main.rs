use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rdev::{listen, Event, EventType, Key};
use std::{
    env,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::Duration,
};
use whisper_rs::{
    get_lang_str, FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters,
};

const SAMPLE_RATE: u32 = 16000;
const MIN_AUDIO_SAMPLES: usize = 8000; // 0.5 s at 16 kHz
const PRE_TYPE_SLEEP_MS: u64 = 150;
const SUPPORTED_LANGUAGES: &[&str] = &["pt", "en"];
struct AudioJob {
    frames: Vec<f32>,
    win_id: String,
    win_class: String,
}

const TERMINAL_CLASSES: &[&str] = &[
    "gnome-terminal",
    "xterm",
    "konsole",
    "alacritty",
    "kitty",
    "tilix",
    "terminator",
    "urxvt",
    "foot",
    "org.wezfurlong.wezterm",
];

fn model_path() -> PathBuf {
    match env::var("WHISPER_MODEL_PATH") {
        Ok(p) => PathBuf::from(p),
        Err(_) => {
            eprintln!("error: WHISPER_MODEL_PATH not set. Run install.sh or set the variable manually.");
            eprintln!("  example: WHISPER_MODEL_PATH=~/.cache/whisper/ggml-small.bin cargo run --release");
            std::process::exit(1);
        }
    }
}

fn notify(summary: &str, body: &str, icon: &str, timeout_ms: u32) {
    let _ = Command::new("notify-send")
        .args(["-t", &timeout_ms.to_string(), "-i", icon, summary, body])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

fn get_active_window() -> (String, String) {
    let display = env::var("DISPLAY").unwrap_or_else(|_| ":1".to_string());
    let xauth = env::var("XAUTHORITY").unwrap_or_else(|_| {
        let runtime = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        format!("{runtime}/gdm/Xauthority")
    });

    // getactivewindow reads _NET_ACTIVE_WINDOW set by the WM and works even when
    // rdev/evdev has grabbed the keyboard (getwindowfocus uses XGetInputFocus which
    // returns nothing while an evdev grab is active).
    for cmd in ["getactivewindow", "getwindowfocus"] {
        let Ok(out) = Command::new("xdotool")
            .arg(cmd)
            .env("DISPLAY", &display)
            .env("XAUTHORITY", &xauth)
            .output()
        else {
            eprintln!("[live-dictation] xdotool {cmd}: failed to spawn");
            continue;
        };

        if !out.status.success() {
            eprintln!(
                "[live-dictation] xdotool {cmd} failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
            continue;
        }

        let win_id = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if win_id.is_empty() {
            eprintln!("[live-dictation] xdotool {cmd} returned empty window id");
            continue;
        }

        // A window without a readable WM_CLASS is still a valid injection
        // target, so we keep the id regardless; the class is best-effort.
        let cls = window_class(&win_id, &display, &xauth);
        return (win_id, cls);
    }

    eprintln!("[live-dictation] could not detect active window — assuming terminal");
    (String::new(), "unknown-terminal".to_string())
}

// This xdotool build lacks `getwindowclassname`, so read WM_CLASS via xprop,
// e.g. WM_CLASS(STRING) = "instance", "Class".
fn window_class(win_id: &str, display: &str, xauth: &str) -> String {
    if let Ok(out) = Command::new("xprop")
        .args(["-id", win_id, "WM_CLASS"])
        .env("DISPLAY", display)
        .env("XAUTHORITY", xauth)
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Some(cls) = s.split('"').nth_back(1) {
                let cls = cls.trim().to_lowercase();
                if !cls.is_empty() {
                    return cls;
                }
            }
        } else {
            eprintln!(
                "[live-dictation] xprop WM_CLASS failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
    } else {
        eprintln!("[live-dictation] xprop: failed to spawn");
    }

    eprintln!("[live-dictation] window {win_id} has no readable WM_CLASS — assuming terminal");
    "unknown-terminal".to_string()
}

fn inject_text(text: &str, win_id: &str, win_class: &str) {
    thread::sleep(Duration::from_millis(PRE_TYPE_SLEEP_MS));

    let display = env::var("DISPLAY").unwrap_or_else(|_| ":1".to_string());
    let xauth = env::var("XAUTHORITY").unwrap_or_else(|_| {
        let runtime = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
        format!("{runtime}/gdm/Xauthority")
    });

    let cls = win_class;
    let is_term =
        TERMINAL_CLASSES.contains(&cls) || cls.starts_with("st-") || cls == "unknown-terminal";
    let paste_key = if is_term { "ctrl+shift+v" } else { "ctrl+v" };

    eprintln!("[live-dictation] window class={cls:?} is_term={is_term} paste_key={paste_key}");

    let xclip_result = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .env("DISPLAY", &display)
        .env("XAUTHORITY", &xauth)
        .stdin(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
            child.wait()
        });

    if let Err(e) = xclip_result {
        eprintln!("[live-dictation] xclip failed: {e}");
        return;
    }

    // Send the paste via XTEST (no `--window`): XTEST events are indistinguishable
    // from real input, so GTK/VTE apps (gnome-terminal, gnome-text-editor) accept
    // them — unlike `--window`, which uses XSendEvent and is ignored as synthetic.
    // First raise/focus the target window so XTEST lands in the right place even
    // if focus drifted while transcription ran.
    let mut cmd = Command::new("xdotool");
    cmd.env("DISPLAY", &display).env("XAUTHORITY", &xauth);
    if !win_id.is_empty() {
        cmd.args(["windowactivate", "--sync", win_id]);
    }
    cmd.args(["key", "--clearmodifiers", paste_key]);

    match cmd.status() {
        Ok(_) => eprintln!("[live-dictation] Injected: {text:?}"),
        Err(e) => eprintln!("[live-dictation] xdotool failed: {e}"),
    }
}

fn transcription_worker(rx: mpsc::Receiver<AudioJob>) {
    let path = model_path();
    eprintln!("[live-dictation] Loading model from {}", path.display());

    let ctx = WhisperContext::new_with_params(
        path.to_str().expect("model path is not valid UTF-8"),
        WhisperContextParameters::default(),
    )
    .expect("Failed to load Whisper model — run install.sh to download it");

    let mut state = ctx.create_state().expect("Failed to create Whisper state");
    eprintln!("[live-dictation] Model ready.");

    for job in rx {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("auto"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        params.set_n_threads(4);

        if let Err(e) = state.full(params, &job.frames) {
            eprintln!("[live-dictation] Whisper inference error: {e}");
            continue;
        }

        let n = state.full_n_segments();

        let mut text = String::new();
        for i in 0..n {
            if let Some(seg) = state.get_segment(i) {
                match seg.to_str() {
                    Ok(s) => text.push_str(s),
                    Err(e) => eprintln!("[live-dictation] segment {i} error: {e}"),
                }
            }
        }
        let text = text.trim().to_string();

        let lang_id = state.full_lang_id_from_state();
        if let Some(lang) = get_lang_str(lang_id) {
            eprintln!("[live-dictation] [{lang}] {text:?}");
            if !SUPPORTED_LANGUAGES.contains(&lang) {
                eprintln!(
                    "[live-dictation] Unsupported language {lang:?} detected; \
                     result may be inaccurate."
                );
            }
        } else {
            eprintln!("[live-dictation] {text:?}");
        }

        if !text.is_empty() {
            inject_text(&text, &job.win_id, &job.win_class);
        }
    }

    eprintln!("[live-dictation] Transcription worker stopped.");
}

fn main() -> Result<()> {
    eprintln!("[live-dictation] Starting up…");

    let recording = Arc::new(AtomicBool::new(false));
    let audio_frames: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = mpsc::channel::<AudioJob>();

    let transcription_thread = thread::Builder::new()
        .name("transcription-worker".to_string())
        .spawn(|| transcription_worker(rx))
        .context("Failed to spawn transcription thread")?;

    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("No input audio device found")?;
    eprintln!(
        "[live-dictation] Audio device: {}",
        device.description().as_ref().map(|d| d.name()).unwrap_or("unknown").to_string()
    );

    let config = cpal::StreamConfig {
        channels: 1,
        sample_rate: SAMPLE_RATE,
        buffer_size: cpal::BufferSize::Default,
    };

    let recording_audio = recording.clone();
    let audio_frames_audio = audio_frames.clone();

    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if recording_audio.load(Ordering::Relaxed) {
                    audio_frames_audio.lock().unwrap().extend_from_slice(data);
                }
            },
            |err| eprintln!("[live-dictation] Audio error: {err}"),
            None,
        )
        .context(
            "Failed to build audio input stream.\n\
             Ensure your microphone supports 16 kHz mono capture (most do via PulseAudio).",
        )?;

    stream.play().context("Failed to start audio stream")?;
    eprintln!("[live-dictation] Audio stream open. Hold Right Ctrl to dictate.");

    ctrlc::set_handler(move || {
        eprintln!("[live-dictation] Shutdown signal received, exiting.");
        std::process::exit(0);
    })
    .context("Failed to set signal handler")?;

    let recording_kbd = recording.clone();
    let audio_frames_kbd = audio_frames.clone();
    let tx_kbd = tx.clone();

    eprintln!("[live-dictation] Daemon ready.");

    listen(move |event: Event| match event.event_type {
        EventType::KeyPress(Key::AltGr) => {
            // EventType::KeyPress(Key::AltGr) => {
            if !recording_kbd.swap(true, Ordering::SeqCst) {
                audio_frames_kbd.lock().unwrap().clear();
                eprintln!("[live-dictation] Recording…");
                notify(
                    "live-dictation",
                    "Recording…",
                    "audio-input-microphone",
                    1500,
                );
            }
        }
        EventType::KeyRelease(Key::AltGr) => {
            if recording_kbd.swap(false, Ordering::SeqCst) {
                let frames = {
                    let mut lock = audio_frames_kbd.lock().unwrap();
                    let f = lock.clone();
                    lock.clear();
                    f
                };
                let (win_id, win_class) = get_active_window();
                eprintln!("[live-dictation] Target window class={win_class:?}");
                eprintln!("[live-dictation] Transcribing…");
                notify(
                    "live-dictation",
                    "Transcribing…",
                    "audio-input-microphone-muted",
                    2000,
                );
                if frames.len() >= MIN_AUDIO_SAMPLES {
                    let _ = tx_kbd.send(AudioJob {
                        frames,
                        win_id,
                        win_class,
                    });
                } else {
                    eprintln!("[live-dictation] Audio too short, skipping.");
                }
            }
        }
        _ => {}
    })
    .map_err(|e| anyhow::anyhow!("rdev listen error: {e:?}"))?;

    drop(tx);
    let _ = transcription_thread.join();
    Ok(())
}
