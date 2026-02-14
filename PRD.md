# Product Requirements Document: PTZ Camera Controller

## 1. Overview

**Product Name:** PTZ Cam Controller
**Version:** 1.0
**Last Updated:** 2026-02-14

PTZ Cam Controller is a cross-platform desktop application for controlling PTZ (Pan-Tilt-Zoom) cameras via a live video feed. Operators define named audience-location presets by drawing overlay regions on a live video preview, then recall those presets during live operation with a single click. After recalling a preset, additional clicks on the video feed fine-tune pan and tilt relative to the current camera position.

-----

## 2. Framework & Technology Stack

### 2.1 Recommendation: Tauri 2 + React + TypeScript (frontend) / Rust (backend)

**Tauri 2** is the recommended application shell for the following reasons:

- **Cross-platform:** Single codebase targets Windows and macOS with native installers. Tauri 2 uses the OS-native webview (WebView2 on Windows, WKWebView on macOS), resulting in dramatically smaller binaries (~5-10 MB vs ~150+ MB for Electron).
- **Lower resource usage:** No bundled Chromium means significantly less baseline memory consumption, leaving more headroom for video frame buffers and processing.
- **Rust backend:** The Rust core provides direct FFI access to the NDI SDK's C API via `bindgen` or manual bindings, with strong safety guarantees around memory management and threading. Rust's `unsafe` FFI blocks are well-scoped and auditable.
- **Fast startup:** Native binary + system webview yields sub-second cold starts on modern hardware.
- **Mature v2 ecosystem:** Tauri 2 offers stable plugin APIs, multi-window support, and solid packaging/signing tooling for both platforms.

| Layer | Technology | Purpose |
|---|---|---|
| Shell | Tauri 2 | Cross-platform desktop shell using native OS webview |
| Backend | Rust | Core logic, video pipeline, PTZ protocol integration |
| Frontend UI | React 19 + TypeScript | Component-based UI with strong typing |
| Styling | Tailwind CSS | Utility-first styling |
| Local video capture | `navigator.mediaDevices.getUserMedia()` in webview | USB/HDMI capture devices rendered natively in `<video>` element |
| NDI video receive | NDI SDK 6 via Rust FFI (`bindgen`) -> localhost MJPEG stream | NDI source discovery, video receive, PTZ control |
| Video display | `<video>` element (native webview) | Hardware-accelerated video decode and rendering |
| Overlay rendering | HTML5 Canvas 2D, layered over `<video>` | Semi-transparent preset overlays |
| PTZ protocols | NDI PTZ, VISCA-over-IP, Panasonic AW (HTTP CGI), BirdDog REST API | Camera movement commands |
| State management | Zustand | Lightweight frontend store |
| Persistence | `serde` + `serde_json` writing to app data directory | Save/load preset configurations per camera profile |
| Build / package | Tauri Bundler | Installers for Windows (.msi / NSIS) and macOS (.dmg) |

### 2.2 Alternatives Considered

| Framework | Why Not |
|---|---|
| Electron + Node.js | Larger binary size and memory footprint (~150 MB+ bundled Chromium) |
| Qt (C++ or Python) | Excellent video handling but slower UI iteration cycle |
| Flutter Desktop | Video texture support on desktop is limited |

### 2.3 Risks & Mitigations (Tauri-specific)

| Risk | Mitigation |
|---|---|
| NDI SDK has no official Rust crate | Use `bindgen` to auto-generate bindings from NDI C headers |
| NDI video frames must transit from Rust to webview | Serve NDI frames as a localhost MJPEG HTTP stream from Rust |
| WebView2/WKWebView rendering differences | Use `<canvas>` overlay for interactive rendering, standard `<video>` for display |
| `getUserMedia` may not enumerate all capture devices | Provide FFmpeg subprocess fallback path |

-----

## 3. User Roles

| Role | Description |
|---|---|
| Operator | The primary user. Configures presets and controls the camera during live events. |

-----

## 4. Application Modes

### 4.1 Calibration Mode
Used before or between events to define where the camera should point for each audience location.

### 4.2 Operation Mode
Used during a live event to recall presets and fine-tune camera aim in real time.

-----

## 5. Functional Requirements

### 5.1 Video Feed

| ID | Requirement |
|---|---|
| VF-1 | Display real-time video feed in the main content area |
| VF-2 | Support receiving video from NDI sources |
| VF-3 | Support receiving video from local capture devices |
| VF-4 | User can select video source from dropdown |
| VF-5 | Video renders at native resolution, scaled to fit with aspect ratio preserved |
| VF-6 | Target latency <= 3 frames |
| VF-7 | Display "No Signal" indicator on disconnect, auto-reconnect every 5 seconds |

### 5.2 Calibration Mode

| ID | Requirement |
|---|---|
| CM-1 | Create new preset via "Add Preset" button |
| CM-2 | Enter human-readable name for preset |
| CM-3 | Use on-screen PTZ controls or mouse to position camera |
| CM-4 | Save current PTZ position as preset |
| CM-5 | Draw semi-transparent rectangular overlay for each preset |
| CM-6 | Distinct colors/border styles for preset overlays |
| CM-7 | Select and update existing presets |
| CM-8 | Delete preset with confirmation |
| CM-9 | Support at least 50 presets |
| CM-10 | Persist preset data to disk |
| CM-11 | Save and load named preset profiles |

### 5.3 Operation Mode

| ID | Requirement |
|---|---|
| OM-1 | Display all preset overlays on video feed |
| OM-2 | Click preset overlay to recall PTZ position |
| OM-3 | Send PTZ command via configured protocol |
| OM-4 | Click-to-adjust pan/tilt relative to center |
| OM-5 | Scroll wheel for zoom control |
| OM-6 | Highlight active preset |
| OM-7 | No preset editing in operation mode |
| OM-8 | Debounce PTZ commands (100ms minimum interval) |

### 5.4 Camera Endpoints & PTZ Control

#### 5.4.1 Camera Endpoint Management

| ID | Requirement |
|---|---|
| EP-1 | Maintain list of configured camera endpoints |
| EP-2 | Add, edit, delete camera endpoints |
| EP-3 | Assign endpoint to current session |
| EP-4 | Persist endpoint configurations |

#### 5.4.2 Supported PTZ Protocols

| ID | Protocol | Description |
|---|---|---|
| PTZ-1 | NDI PTZ | NDI SDK built-in PTZ control functions |
| PTZ-2 | VISCA-over-IP | Sony VISCA protocol over UDP (port 52381) |
| PTZ-3 | Panasonic AW | HTTP CGI interface for Panasonic cameras |
| PTZ-4 | BirdDog REST | RESTful HTTP API (port 8080) |

#### 5.4.3 Protocol-Agnostic Behavior

| ID | Requirement |
|---|---|
| PTZ-5 | Normalize PTZ values: pan/tilt (-1.0 to +1.0), zoom (0.0 to 1.0) |
| PTZ-6 | Use native preset recall commands where available |
| PTZ-7 | Provide "Test Connection" button for endpoints |

### 5.5 Settings & Configuration

| ID | Requirement |
|---|---|
| SC-1 | Settings panel accessible from toolbar |
| SC-2 | Video source, endpoint management, sensitivity, overlay opacity, profile management |
| SC-3 | Persist settings to disk |

### 5.6 CI/CD, Versioning & Releases

| ID | Requirement |
|---|---|
| CI-1 | Semantic Versioning 2.0.0 |
| CI-2 | Version synced across package.json, Cargo.toml, tauri.conf.json |
| CI-3 | Releases triggered by git tags (v*.*.*) |
| CI-4 | CI workflow on push/PR to main |
| CI-5 | Release workflow with signing and changelog |
| CI-6 | Platform artifacts: .msi/.exe for Windows, .dmg for macOS |
| CI-7 | Conventional Commits |
| CI-8 | Automated changelog via git-cliff |
| CI-9 | Code signing via GitHub Actions secrets |
| CI-10 | macOS notarization |
| CI-11 | NDI SDK caching in CI |
| CI-12 | Pre-release support |

-----

## 6. Non-Functional Requirements

| ID | Requirement |
|---|---|
| NF-1 | Windows 10/11 (x64) and macOS 12+ |
| NF-2 | >= 24 fps overlay compositing |
| NF-3 | Cold start under 2 seconds |
| NF-4 | Signed installers (.msi/.exe, .dmg) |
| NF-5 | Clear error messages for common failures |
| NF-6 | English only for v1 |
| NF-7 | Installer under 20 MB (excluding NDI runtime) |

-----

## 7. UI / Layout Specification

```
+--------------------------------------------------------------+
|  Toolbar                                                     |
|  [Mode: Calibration | Operation]  [Source: v]  [Settings]    |
+------------------------------------------+-------------------+
|                                          |  Preset List      |
|                                          |                   |
|          Live Video Feed                 |  Front Row C      |
|          with Overlay Presets            |  Balcony L        |
|                                          |  Balcony R        |
|     +---------+                          |  Stage Left       |
|     |Preset A | (semi-transparent)       |  + Add Preset     |
|     +---------+                          |                   |
|              +---------+                 |  PTZ Controls     |
|              |Preset B |                 |  (calibration     |
|              +---------+                 |   mode only)      |
|                                          |  [arrows]         |
|                                          |  [Zoom - / +]     |
+------------------------------------------+-------------------+
|  Status Bar: [Connected to: Source] [FPS: 30]                |
+--------------------------------------------------------------+
```

-----

## 8. Development Phases

### Phase 0 - Project Setup & CI
### Phase 1 - Video Feed
### Phase 2 - NDI Integration
### Phase 3 - Calibration Mode
### Phase 4 - Operation Mode
### Phase 5 - Additional PTZ Protocols & Polish
### Phase 6 - Release Pipeline

-----

## 9. Out of Scope (v1)

- Multi-camera control
- Video recording or streaming output
- Remote/networked operator access
- Touchscreen or mobile interfaces
- Audio monitoring
- Internationalization / localization
- Auto-tracking or AI-based framing
