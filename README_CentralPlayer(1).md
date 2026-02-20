# 🎵 Central Player Pro

> **This is not your average music player.**
> Built from scratch in Rust. No Electron. No Python. No bloat. Just raw performance.

![Platform](https://img.shields.io/badge/platform-Linux-blue)
![Language](https://img.shields.io/badge/language-Rust-orange)
![UI](https://img.shields.io/badge/UI-egui%20%2F%20eframe-purple)
![Audio](https://img.shields.io/badge/audio-GStreamer-green)
![Version](https://img.shields.io/badge/version-v2.5%20Stable-brightgreen)

---

## 🚀 Why Central Player Pro?

Most music players on Linux are either:
- Bloated Electron apps that eat 500MB of RAM just to play an MP3
- Old GTK2 relics that haven't been touched since 2009
- Ports of Windows software that feel completely out of place

**Central Player Pro is different.**

It's written entirely in **Rust** — which means it starts in milliseconds, uses almost no memory, never crashes, and runs natively on your hardware. The audio pipeline is powered by **GStreamer** — the same engine used by GNOME, Firefox, and professional broadcast software. The UI is rendered with **egui**, an immediate-mode GPU-accelerated framework.

This is a music player that respects your machine and your time.

---

## ✨ Features

### 🎚️ Real 10-Band Parametric Equalizer
Not a fake slider that just changes volume. A genuine GStreamer `equalizer-10bands` element sitting inside the audio pipeline, processing every sample in real time. Frequencies: 29Hz → 59Hz → 119Hz → 237Hz → 474Hz → 947Hz → 1.8kHz → 3.7kHz → 7.5kHz → 15kHz.

### 🎛️ EQ Presets That Actually Sound Good
Flat, Rock, Jazz, Classical, Pop, Bass Boost — one click, instant change. No restart, no lag.

### 📊 Live Waveform Visualizer
40-bar animated spectrum rendered directly on the GPU via egui's painter API. Smooth interpolation (80/20 lerp per frame), graceful fade-out when playback stops. Accent color matches your current theme automatically.

### 🎨 Full Theme Engine
- Dark / Light mode toggle
- Custom accent color picker (full RGB, per-pixel)
- Custom background & text colors
- Live preview — changes apply instantly without restarting

### 💾 Persistent App State
Every time you close the app, it saves: your volume, your entire playlist, the last track you were on, your theme, and your accent color. Next time you open — everything is exactly where you left it.

### ⌨️ Keyboard Navigation
- `↑` / `↓` — move through the playlist
- `Enter` — play the selected track
- No mouse required once you're in the zone

### 📁 Smart Library Import
- Add individual files via file picker
- **Recursive folder scan** — drop your entire music folder and it finds every track automatically, including nested subfolders

### 🔊 Format Support
MP3, FLAC, OGG, WAV, M4A, MP4 — anything GStreamer can decode, Central Player Pro can play. Which is basically everything.

---

## 🏗️ Architecture

```
┌──────────────────────────────────────────────┐
│              egui / eframe (GPU UI)          │
│  Header | Visualizer | Playlist | EQ Panel  │
│  ThemeManager | AppState | Components        │
└─────────────────────┬────────────────────────┘
                      │ mpsc::channel
                      │ AudioCommand enum
                      ▼
┌──────────────────────────────────────────────┐
│              AudioEngine (Rust)              │
│  update() called every frame                 │
│  drains AudioStatus events                   │
│  updates spectrum_data (Arc<Mutex<Vec>>)     │
└─────────────────────┬────────────────────────┘
                      │ dedicated OS thread
                      ▼
┌──────────────────────────────────────────────┐
│         GStreamer Pipeline (native C)        │
│  playbin → equalizer-10bands → audio sink   │
│  bus polling @ 30ms                          │
│  position updates @ 100ms                   │
│  fakesink for video (no popup windows)       │
└──────────────────────────────────────────────┘
```

The UI thread and the audio thread are **completely decoupled**. The UI never waits for audio. Audio never waits for the UI. Zero blocking, zero stuttering.

---

## 🚀 Installation

### Prerequisites

```bash
# Debian / Ubuntu
sudo apt install libgstreamer1.0-dev \
                 gstreamer1.0-plugins-good \
                 gstreamer1.0-plugins-bad \
                 gstreamer1.0-plugins-ugly \
                 gstreamer1.0-libav

# Fedora
sudo dnf install gstreamer1-devel \
                 gstreamer1-plugins-good \
                 gstreamer1-plugins-bad-free \
                 gstreamer1-plugins-ugly
```

### Build & Run

```bash
git clone https://github.com/yourname/central-player-pro
cd central-player-pro
cargo build --release
./target/release/central-player-pro
```

No pip. No npm. No AppImage drama. Just Cargo.

---

## 📦 Dependencies

| Crate | Purpose |
|-------|---------|
| `eframe` / `egui` | GPU-accelerated immediate-mode UI |
| `gstreamer` | Professional audio pipeline |
| `rfd` | Native file/folder picker dialogs |
| `image` | Icon loading from embedded bytes |
| `std::sync::mpsc` | Command/event channels between threads |

---

## 🎚️ EQ Band Reference

| Band | Frequency | Good for |
|------|-----------|----------|
| 0 | 29 Hz | Sub-bass rumble |
| 1 | 59 Hz | Bass punch |
| 2 | 119 Hz | Low warmth |
| 3 | 237 Hz | Body / muddiness |
| 4 | 474 Hz | Low mids |
| 5 | 947 Hz | Mids / presence |
| 6 | 1.8 kHz | Upper mids / vocals |
| 7 | 3.7 kHz | Clarity / attack |
| 8 | 7.5 kHz | Air / brightness |
| 9 | 15 kHz | Shimmer / hiss |

Gain range: **-24 dB to +12 dB** per band, hardware-clamped before hitting GStreamer.

---

## 🛠️ Roadmap

- [ ] Real spectrum analyzer via GStreamer `spectrum` element
- [ ] Shuffle & repeat modes
- [ ] MPRIS2 integration (control from taskbar / notifications)
- [ ] Media key support (hardware play/pause/next)
- [ ] Save & load custom EQ presets to disk
- [ ] Search / filter in playlist

---

## 👤 About

Developed by **Shay Kadosh** — Software Engineer from Ashkelon 🇮🇱

Built because the alternatives weren't good enough.

---

## 📄 License

MIT — free to use, free to modify, free to ship.
