Correct the error

# error "could not detect active window"
"could not detect active window"

## log from "journalctl --user -u live-dictation -f"
May 15 09:07:46 mperrone-LOQ-15IRX9 live-dictation[108434]: [live-dictation] Recording…
May 15 09:07:49 mperrone-LOQ-15IRX9 live-dictation[108434]: [live-dictation] could not detect active window — assuming terminal
May 15 09:07:49 mperrone-LOQ-15IRX9 live-dictation[108434]: [live-dictation] Target window class="unknown-terminal"
May 15 09:07:49 mperrone-LOQ-15IRX9 live-dictation[108434]: [live-dictation] Transcribing…
May 15 09:07:49 mperrone-LOQ-15IRX9 live-dictation[108434]: whisper_full_with_state: auto-detected language: en (p = 0.565943)
May 15 09:07:49 mperrone-LOQ-15IRX9 live-dictation[108434]: [live-dictation] [en] "Testing on terminal."
May 15 09:07:49 mperrone-LOQ-15IRX9 live-dictation[108434]: [live-dictation] window class="unknown-terminal" is_term=true paste_key=ctrl+shift+v
May 15 09:07:49 mperrone-LOQ-15IRX9 live-dictation[108434]: [live-dictation] Injected: "Testing on terminal."

---
# System Details Report
---

## Report details
- **Date generated:**                              2026-05-15 09:09:32

## Hardware Information:
- **Hardware Model:**                              Lenovo LOQ 15IRX9
- **Memory:**                                      16.0 GiB
- **Processor:**                                   13th Gen Intel® Core™ i7-13650HX × 20
- **Graphics:**                                    Intel® Graphics (RPL-S)
- **Graphics 1:**                                  NVIDIA GeForce RTX™ 4050 Laptop GPU
- **Disk Capacity:**                               512.1 GB

## Software Information:
- **Firmware Version:**                            NECN43WW
- **OS Name:**                                     Ubuntu 24.04.4 LTS
- **OS Build:**                                    (null)
- **OS Type:**                                     64-bit
- **GNOME Version:**                               46
- **Windowing System:**                            X11
- **Kernel Version:**                              Linux 6.17.0-23-generic
---
testando no editor.


2.

# Error "getwindowclassname failed: xdotool: Unknown command: getwindowclassname"

## log from "journalctl --user -u live-dictation -f"
May 15 09:22:28 mperrone-LOQ-15IRX9 live-dictation[110268]: [live-dictation] getwindowclassname failed: xdotool: Unknown command: getwindowclassname
May 15 09:22:28 mperrone-LOQ-15IRX9 live-dictation[110268]: Run 'xdotool help' if you want a command list

## commands available in xdotool
xdotool
Usage: xdotool <cmd> <args>
Available commands:
  getactivewindow
  getwindowfocus
  getwindowname
  getwindowpid
  getwindowgeometry
  getdisplaygeometry
  search
  selectwindow
  help
  version
  behave
  behave_screen_edge
  click
  getmouselocation
  key
  keydown
  keyup
  mousedown
  mousemove
  mousemove_relative
  mouseup
  set_window
  type
  windowactivate
  windowfocus
  windowkill
  windowclose
  windowmap
  windowminimize
  windowmove
  windowraise
  windowreparent
  windowsize
  windowunmap
  set_num_desktops
  get_num_desktops
  set_desktop
  get_desktop
  set_desktop_for_window
  get_desktop_for_window
  get_desktop_viewport
  set_desktop_viewport
  exec
  sleep



testando aqui no ZED

# everything is write on the log but 
## failed to Inject in 
1. Target window class="gnome-terminal"
## log entries 'Target window class="gnome-terminal"'
May 15 09:30:45 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Recording…
May 15 09:30:47 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Target window class="gnome-terminal"
May 15 09:30:47 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Transcribing…
May 15 09:30:47 mperrone-LOQ-15IRX9 live-dictation[111085]: whisper_full_with_state: auto-detected language: en (p = 0.662818)
May 15 09:30:47 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] [en] "There will be no detection."
May 15 09:30:47 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] window class="gnome-terminal" is_term=true paste_key=ctrl+shift+v
May 15 09:30:47 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Injected: "There will be no detection."
2. Target window class="gnome-text-editor"
## log entries 'Target window class="gnome-text-editor"'
May 15 09:38:48 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Recording…
May 15 09:38:50 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Target window class="gnome-text-editor"
May 15 09:38:50 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Transcribing…
May 15 09:38:51 mperrone-LOQ-15IRX9 live-dictation[111085]: whisper_full_with_state: auto-detected language: pt (p = 0.978193)
May 15 09:38:51 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] [pt] "Testando no que nome é dito."
May 15 09:38:51 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] window class="gnome-text-editor" is_term=false paste_key=ctrl+v
May 15 09:38:51 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Injected: "Testando no que nome é dito."
## Works fine
1. Target window class="dev.zed.zed"
## log entries 'Target window class="dev.zed.zed"'
May 15 09:42:08 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Recording…
May 15 09:42:10 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Target window class="dev.zed.zed"
May 15 09:42:10 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Transcribing…
May 15 09:42:10 mperrone-LOQ-15IRX9 live-dictation[111085]: whisper_full_with_state: auto-detected language: pt (p = 0.998308)
May 15 09:42:10 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] [pt] "testando aqui no ZED"
May 15 09:42:10 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] window class="dev.zed.zed" is_term=false paste_key=ctrl+v
May 15 09:42:10 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Injected: "testando aqui no ZED"
2. Target window class="google-chrome"
## log entries 'Target window class="google-chrome"'
May 15 09:35:48 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Recording…
May 15 09:35:49 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Target window class="google-chrome"
May 15 09:35:49 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Transcribing…
May 15 09:35:49 mperrone-LOQ-15IRX9 live-dictation[111085]: whisper_full_with_state: auto-detected language: pt (p = 0.990719)
May 15 09:35:49 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] [pt] "testando no Chrome"
May 15 09:35:50 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] window class="google-chrome" is_term=false paste_key=ctrl+v
May 15 09:35:50 mperrone-LOQ-15IRX9 live-dictation[111085]: [live-dictation] Injected: "testando no Chrome"

testando no ZED

testando o museu. Testando a transcrição de texto para ver se está tudo funcionando.
