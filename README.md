# Audience

Cross-platform desktop application for controlling PTZ (Pan-Tilt-Zoom) cameras via a live video feed. Operators define named presets by positioning the camera over audience locations, then recall those presets during live events with a single click.

Built with **Tauri 2** (Rust backend + React/TypeScript frontend).

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Tauri 2 Desktop Shell                        │
│                     (OS-native WebView + Rust)                      │
├─────────────────────────────────┬───────────────────────────────────┤
│       Frontend (WebView)        │         Backend (Rust)            │
│                                 │                                   │
│  ┌───────────┐  ┌───────────┐  │  ┌─────────────────────────────┐  │
│  │  Toolbar   │  │  Status   │  │  │       Tauri Commands        │  │
│  │  (mode,    │  │  Bar      │  │  │  video · ptz · presets      │  │
│  │  source)   │  │  (fps,    │  │  │  endpoints · settings       │  │
│  └───────────┘  │  conn)     │  │  └──────────┬──────────────────┘  │
│                  └───────────┘  │             │                     │
│  ┌──────────────────────────┐   │  ┌──────────▼──────────────────┐  │
│  │      Video Canvas        │   │  │       AppState              │  │
│  │  ┌────────────────────┐  │   │  │  config · profiles          │  │
│  │  │  <video> element   │  │   │  │  endpoints · position       │  │
│  │  │  (local capture or │◄─┼───┼──┤  ptz_dispatcher             │  │
│  │  │   MJPEG stream)    │  │   │  │  mjpeg_port                 │  │
│  │  └────────────────────┘  │   │  └──────────┬──────────────────┘  │
│  │  ┌────────────────────┐  │   │             │                     │
│  │  │  Preset Overlays   │  │   │  ┌──────────▼──────────────────┐  │
│  │  │  (HTML divs over   │  │   │  │     PtzDispatcher           │  │
│  │  │   video, projected │  │   │  │  routes commands to the     │  │
│  │  │   from PTZ coords) │  │   │  │  active PtzController       │  │
│  │  └────────────────────┘  │   │  └──────────┬──────────────────┘  │
│  └──────────────────────────┘   │             │                     │
│                                 │  ┌──────────▼──────────────────┐  │
│  ┌──────────────────────────┐   │  │  PtzController (trait)      │  │
│  │     Sidebar              │   │  │                             │  │
│  │  ┌────────────────────┐  │   │  │  ┌─────────┐ ┌──────────┐  │  │
│  │  │  Preset List       │  │   │  │  │  VISCA  │ │Panasonic │  │  │
│  │  │  (add/recall/      │  │   │  │  │  (UDP)  │ │ AW (HTTP)│  │  │
│  │  │   delete)          │  │   │  │  └─────────┘ └──────────┘  │  │
│  │  └────────────────────┘  │   │  │  ┌─────────┐ ┌──────────┐  │  │
│  │  ┌────────────────────┐  │   │  │  │ BirdDog │ │ NDI PTZ  │  │  │
│  │  │  PTZ Controls      │  │   │  │  │ (REST)  │ │ (FFI)*   │  │  │
│  │  │  (calibration only)│  │   │  │  └─────────┘ └──────────┘  │  │
│  │  └────────────────────┘  │   │  └─────────────────────────────┘  │
│  └──────────────────────────┘   │                                   │
│                                 │  ┌─────────────────────────────┐  │
│  ┌──────────────────────────┐   │  │     Video Pipeline          │  │
│  │    Zustand Store         │   │  │  MJPEG server (Axum)        │  │
│  │  mode · presets · ptz    │   │  │  localhost:port/stream       │  │
│  │  endpoints · settings    │   │  │  NDI receiver (stub)*       │  │
│  └──────────────────────────┘   │  └─────────────────────────────┘  │
│                                 │                                   │
│         invoke() ──────────────►│  ┌─────────────────────────────┐  │
│         (Tauri IPC)             │  │     Persistence             │  │
│                                 │  │  profiles.json              │  │
│                                 │  │  endpoints.json             │  │
│                                 │  │  config.json                │  │
│                                 │  └─────────────────────────────┘  │
├─────────────────────────────────┴───────────────────────────────────┤
│                         * NDI requires SDK                          │
└─────────────────────────────────────────────────────────────────────┘

                        External Devices
          ┌──────────────────────────────────────────┐
          │                                          │
          │   ┌──────────┐  ┌──────────┐             │
          │   │ PTZ      │  │ PTZ      │             │
          │   │ Camera A │  │ Camera B │  ...        │
          │   │ (VISCA)  │  │ (HTTP)   │             │
          │   └──────────┘  └──────────┘             │
          │                                          │
          │   ┌──────────┐  ┌──────────┐             │
          │   │ Capture   │  │ NDI      │             │
          │   │ Device    │  │ Source   │             │
          │   │ (USB/HDMI)│  │ (network)│             │
          │   └──────────┘  └──────────┘             │
          └──────────────────────────────────────────┘
```

### Data Flow

**Preset recall (Operation Mode):**

```
User clicks preset overlay
  → Frontend invoke("ptz_recall_preset")
    → PtzDispatcher.move_absolute(pan, tilt, zoom)
      → Active PtzController encodes protocol command
        → UDP/HTTP to camera hardware
```

**Video feed (NDI source):**

```
NDI Source → Rust NDI receiver → Broadcast channel
  → Axum MJPEG server (127.0.0.1:port/stream)
    → <video> element in WebView
```

**Video feed (local capture):**

```
USB/HDMI device → navigator.mediaDevices.getUserMedia()
  → <video srcObject={stream}> (direct WebView rendering)
```

## Features

- **Two operating modes** — Calibration mode for defining presets; Operation mode for recalling them live
- **Click-to-adjust** — Click anywhere on the video to nudge pan/tilt relative to center; scroll to zoom
- **Preset overlays** — Color-coded rectangles projected onto the video feed based on PTZ coordinates and camera FOV
- **Multiple PTZ protocols** — VISCA-over-IP (UDP), Panasonic AW (HTTP CGI), BirdDog REST API, NDI PTZ (requires SDK)
- **Profile management** — Save and load named preset profiles per camera
- **Persistent configuration** — Endpoints, presets, and settings stored as JSON in the app data directory

## Project Structure

```
src/                          React/TypeScript frontend
├── components/               UI components (VideoCanvas, PresetList, Toolbar, etc.)
├── hooks/                    Tauri IPC hooks (usePresets, usePtzControl, useVideoFeed, etc.)
├── store/                    Zustand global state
├── utils/                    Geometry projection, PTZ math
└── styles/                   Tailwind CSS

src-tauri/                    Rust backend
├── src/
│   ├── commands/             Tauri command handlers (video, ptz, presets, endpoints, settings)
│   ├── ptz/                  PtzController trait + PtzDispatcher + EndpointManager
│   ├── visca/                VISCA-over-IP protocol (UDP, port 52381)
│   ├── panasonic/            Panasonic AW protocol (HTTP CGI)
│   ├── birddog/              BirdDog REST API (HTTP, port 8080)
│   ├── ndi/                  NDI SDK stubs (FFI-ready)
│   ├── video/                MJPEG streaming server (Axum)
│   └── persistence/          JSON file storage (profiles, endpoints, config)
└── Cargo.toml

shared/                       TypeScript types shared between frontend modules
```

## Supported PTZ Protocols

| Protocol | Transport | Default Port | Status |
|---|---|---|---|
| VISCA-over-IP | UDP | 52381 | Implemented |
| Panasonic AW | HTTP CGI | 80 | Implemented |
| BirdDog REST | HTTP JSON | 8080 | Implemented |
| NDI PTZ | NDI SDK (FFI) | — | Stub (requires NDI SDK) |

All protocols use normalized coordinates: pan/tilt in the range **-1.0 to +1.0**, zoom in **0.0 to 1.0**. Each controller translates these to protocol-specific values.

## Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install) (stable)
- [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/) for your platform
- [just](https://github.com/casey/just) command runner (optional, for convenience commands)

## Getting Started

```bash
# Install frontend dependencies
npm install

# Start the dev server (Vite + Tauri)
npm run tauri dev
```

## Building

```bash
# Build the frontend
npm run build

# Build the Tauri application (produces platform installer)
npm run tauri build
```

## Development Commands

```bash
just check            # Run all lint and type checks (frontend + Rust)
just check-frontend   # ESLint + TypeScript typecheck
just check-rust       # cargo fmt --check + clippy
just fmt              # Auto-fix formatting (both frontend and Rust)
just test             # Run all tests
```

Or without `just`:

```bash
npm run lint          # ESLint
npm run typecheck     # TypeScript type checking
npm run test          # Frontend tests (Vitest)
cargo test --manifest-path src-tauri/Cargo.toml   # Rust tests
```

## Application Modes

### Calibration Mode

Used before or between events to define camera presets:
- Position the camera using on-screen PTZ controls
- Save the current position as a named preset
- Preset overlays appear on the video feed as colored rectangles
- Add, update, or delete presets

### Operation Mode

Used during a live event:
- Click a preset overlay or list item to recall that camera position
- Click on the video to fine-tune pan/tilt relative to center
- Scroll to adjust zoom
- Preset editing is disabled

## License

MIT
