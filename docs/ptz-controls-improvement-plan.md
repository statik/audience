# PTZ Controls Improvement Plan

Comparative analysis of **Audience** vs **obs-ptz** (OBS PTZ Controls plugin) with
actionable improvement recommendations prioritized by impact.

---

## Executive Summary

Audience has strong architectural fundamentals: a clean protocol-agnostic trait
(`PtzController`), normalized coordinate system, multi-protocol support, and a
polished preset-overlay UI. However, comparing against obs-ptz reveals significant
gaps in **camera feedback**, **real-time control responsiveness**, and **operator
ergonomics** that would meaningfully improve the production workflow.

---

## Current State Comparison

| Capability                     | Audience              | obs-ptz              |
| ------------------------------ | --------------------- | -------------------- |
| Protocol abstraction           | Trait-based, clean    | Class hierarchy      |
| VISCA over UDP                 | Yes                   | Yes                  |
| VISCA over TCP                 | No                    | Yes (PTZOptics)      |
| VISCA over serial              | No                    | Yes (RS-232/422)     |
| Panasonic AW (HTTP)            | Yes                   | No                   |
| BirdDog REST                   | Yes                   | No                   |
| ONVIF                          | No                    | Experimental         |
| Position feedback from camera  | Stubbed (returns 0,0,0) | Full parsing      |
| Continuous movement (joystick) | No                    | Yes (touch + physical) |
| Diagonal movement              | No (4-way only)       | Yes (8-way + analog) |
| Focus control                  | No                    | Yes (AF, manual, OT-AF) |
| Speed control (variable)       | Hardcoded step        | Modifier keys + slider |
| Keyboard shortcuts             | No                    | OBS hotkey system    |
| Command queue with retry       | Basic 100ms throttle  | Queue with 8 slots   |
| Speed ramping                  | No                    | Yes                  |
| Live-move safety lockout       | No                    | Yes (studio mode)    |
| Camera power control           | No                    | Yes                  |
| WB / Exposure / Gain           | No                    | Yes                  |
| Preset overlay on video        | Yes (unique strength) | No                   |
| Click-to-pan on video          | Yes (unique strength) | No                   |

---

## Priority 1 — Critical Gaps

### 1.1 Implement VISCA Response Parsing (Position Feedback)

**Problem:** `ViscaClient::get_position()` sends inquiry commands but returns
hardcoded `(0, 0, 0)`. The frontend tracks position optimistically by accumulating
deltas, which drifts from reality over time. Preset overlay positions become
inaccurate.

**What obs-ptz does:** Fully decodes VISCA nibble-encoded responses for pan/tilt
(4 nibbles each, 16-bit signed) and zoom (4 nibbles, 16-bit unsigned). Uses a
stale-property cache that re-queries only when values may have changed.

**Implementation:**

- `src-tauri/src/visca/commands.rs` — Add response parsing functions:
  - `parse_pan_tilt_response(&[u8]) -> Option<(i16, i16)>` — Extract 4+4 nibbles
    from bytes 2-9 of response payload `90 50 0p 0p 0p 0p 0t 0t 0t 0t FF`
  - `parse_zoom_response(&[u8]) -> Option<u16>` — Extract 4 nibbles from
    `90 50 0z 0z 0z 0z FF`
  - Reverse-normalization functions: `visca_pan_to_normalized(i16) -> f64`, etc.
- `src-tauri/src/visca/client.rs` — Update `get_position()` to strip the 8-byte
  VISCA-over-IP header and call the parsers.
- Do the same for `PanasonicClient::get_position()` (parse `aPC` hex responses).

**Impact:** Fixes position drift, makes overlay geometry accurate, enables reliable
preset workflows. This is the single highest-value change.

### 1.2 Continuous Movement (Virtual Joystick)

**Problem:** PTZ controls use discrete fixed-step buttons (`step = 0.05`). The
VISCA `move_relative` implementation sends a command, sleeps 200ms, then sends
stop — producing jerky, imprecise movement. There is no way to perform smooth,
variable-speed continuous movement.

**What obs-ptz does:** A `TouchControl` widget acts as a 2D virtual joystick.
Mouse drag position maps to continuous velocity via `atan2()` polar coordinates.
Releasing the mouse snaps to center (stop). Speed is proportional to distance
from center.

**Implementation:**

- Add `continuous_move` to the `PtzController` trait:
  ```rust
  async fn continuous_move(&self, pan_speed: f64, tilt_speed: f64) -> Result<(), PtzError>;
  async fn stop(&self) -> Result<(), PtzError>;
  ```
- VISCA: Use the existing `pan_tilt_relative` command (which is actually
  continuous drive, not relative positioning) without the 200ms sleep + stop.
  Send stop only when the user releases the joystick.
- Panasonic: Map to `#PTS` continuous command.
- Frontend: New `<JoystickPad>` component using pointer events:
  - `pointerdown` + `pointermove`: Calculate normalized velocity from center offset
  - `pointerup`: Send stop command
  - Visual feedback: concentric circles with position indicator (like obs-ptz)
- Show in both Calibration and Operation modes.

**Impact:** Transforms camera positioning from tedious click-click-click to fluid,
professional-grade control. Essential for live production.

### 1.3 Make PTZ Controls Available in Operation Mode

**Problem:** `PtzControls` component (directional buttons, zoom slider, position
readout) is only rendered in Calibration mode. During live operation, the only
controls are click-to-pan on video and scroll-to-zoom — no way to fine-tune
position or see current coordinates.

**What obs-ptz does:** Controls are always visible in the dock panel regardless
of mode.

**Implementation:**

- Show the PTZ controls sidebar section in both modes.
- Consider a collapsible section so operators can minimize when not needed.

**Impact:** Low-effort change with immediate usability benefit. Operators need
fine control during live events.

---

## Priority 2 — Significant Improvements

### 2.1 Diagonal Movement Support

**Problem:** The 3x3 grid has empty corners. Only 4 cardinal directions are
available. Diagonal movement (up-left, up-right, down-left, down-right) requires
two separate commands.

**What obs-ptz does:** 8-way directional pad with diagonal buttons. The VISCA
protocol natively supports simultaneous pan+tilt in a single command.

**Implementation:**

- Add 4 diagonal buttons to the 3x3 grid (filling the empty corners)
- Each sends `moveRelative(±step, ±step)`
- The VISCA command already encodes pan and tilt direction independently, so this
  works at the protocol level with no backend changes.

### 2.2 Variable Speed Control

**Problem:** Movement speed is hardcoded to `step = 0.05`. No way to make fine
adjustments or fast repositioning without repeated clicks.

**What obs-ptz does:** Ctrl+click = fast, Shift+click = slow. Configurable speed
limits per device. Touch control provides analog speed proportional to drag distance.

**Implementation:**

- Add a speed selector to PtzControls (e.g., 3 presets: Fine/Normal/Fast mapping
  to step values 0.01/0.05/0.15)
- Support modifier keys on the directional buttons:
  - `Shift+click` = fine step (0.01)
  - `Ctrl+click` = coarse step (0.15)
- Store speed preference in settings.

### 2.3 Focus Control

**Problem:** No focus controls at all. Operators must use camera hardware or
separate software to adjust focus.

**What obs-ptz does:** Autofocus toggle, manual focus near/far, one-touch AF.
VISCA commands: `81 01 04 38 02 FF` (AF on), `81 01 04 38 03 FF` (AF off),
`81 01 04 08 02 FF` (focus far), `81 01 04 08 03 FF` (focus near),
`81 01 04 18 01 FF` (one-push trigger).

**Implementation:**

- Add to `PtzController` trait:
  ```rust
  async fn focus_continuous(&self, speed: f64) -> Result<(), PtzError>;
  async fn autofocus(&self, enabled: bool) -> Result<(), PtzError>;
  ```
- Add VISCA focus commands to `commands.rs`.
- Frontend: Add a Focus section below Zoom in PtzControls with AF toggle, Near/Far
  buttons.
- Default implementations that return `Ok(())` for protocols that don't support
  focus (BirdDog REST handles it differently).

### 2.4 VISCA over TCP Transport

**Problem:** Only VISCA-over-IP (UDP, port 52381) is supported. PTZOptics and
many other cameras use VISCA over TCP on port 5678, which is a different framing
format.

**What obs-ptz does:** Separate `PTZViscaOverTCP` class with TCP keep-alive,
auto-reconnection every 1.9s, and UART-style initialization over TCP.

**Implementation:**

- New `ViscaTcpClient` in `src-tauri/src/visca/` using `tokio::net::TcpStream`.
- TCP VISCA does not use the 8-byte IP header — commands are sent raw
  (same as serial VISCA framing).
- Add reconnection logic on disconnect.
- Add `ViscaTcp` variant to the `PtzProtocol` enum with `host` and `port` config.
- Update EndpointManager UI to offer "VISCA (UDP)" vs "VISCA (TCP)" selection.

### 2.5 Keyboard Shortcuts for PTZ Operations

**Problem:** No keyboard shortcuts. All PTZ operations require mouse interaction.

**Implementation:**

- Arrow keys: Pan/tilt (with Shift for fine, Ctrl for coarse)
- `+` / `-`: Zoom in/out
- Number keys `1-9`: Recall presets by index
- `Space`: Stop all movement
- `F`: Toggle autofocus
- `H`: Home position
- Register via Tauri's global shortcut API or React `useEffect` with
  `keydown`/`keyup` listeners.

---

## Priority 3 — Nice-to-Have Enhancements

### 3.1 Command Queue with Retry Logic

**Problem:** Commands are fire-and-forget with a basic 100ms throttle. If a UDP
packet is lost, the command silently fails. The `send_command` method locks the
socket mutex for the entire send+receive cycle, blocking concurrent commands.

**What obs-ptz does:** 8-slot command queue matching the VISCA spec, with
per-command timeouts and retry. Sequence number tracking for reliable delivery.
Throttled `do_update()` batches rapid changes.

**Implementation:**

- Add a command queue in `ViscaClient` that tracks in-flight commands by sequence
  number.
- Implement single-retry on timeout (2s timeout, 1 retry).
- Allow concurrent pan/tilt + zoom commands (VISCA supports parallel commands in
  different categories).
- Consider moving from `Mutex<Option<UdpSocket>>` to a dedicated async task that
  owns the socket and processes a command channel.

### 3.2 Speed Ramping (Acceleration Curves)

**Problem:** Movement starts at full speed instantly and stops instantly, which
looks jarring on camera.

**What obs-ptz does:** Configurable speed ramping that gradually increases velocity
over time for smoother on-air camera movements.

**Implementation:**

- For continuous movement (after implementing the joystick), apply an easing
  function to the velocity over the first ~500ms of movement.
- Store ramping preference in settings (off / gentle / aggressive).

### 3.3 Periodic Position Polling

**Problem:** Position is only updated locally via delta accumulation. Even with
response parsing (1.1), position is only queried on explicit `get_position()` calls.

**Implementation:**

- After implementing response parsing, add an optional polling interval (e.g.,
  every 2-5 seconds) that queries camera position and updates the store.
- This keeps the overlay accurate even if the camera is moved by another controller.
- Make polling interval configurable; disable when no endpoint is active.

### 3.4 Camera Image Settings (White Balance, Exposure)

**What obs-ptz does:** VISCA commands for white balance mode, exposure mode,
gain, shutter speed, iris.

**Implementation:**

- Add an "Image" or "Camera Settings" panel to the settings area.
- VISCA commands are well-documented; add encoding functions to `commands.rs`.
- Only show for protocols that support it (VISCA, some BirdDog endpoints).

### 3.5 Home Position Command

**What obs-ptz does:** Dedicated home button that sends VISCA `81 01 06 04 FF`.

**Implementation:**

- Add `home()` to `PtzController` trait.
- Add VISCA home command.
- Add Home button to PtzControls (the current "Stop" button at center of the
  d-pad could become Home, or add it separately).

---

## Priority 4 — Future Considerations

These are lower priority but worth tracking:

- **ONVIF protocol support** — Would significantly expand compatible camera range.
  obs-ptz has an experimental implementation that could serve as reference.
- **VISCA over serial (RS-232/RS-422)** — Relevant for legacy installations with
  daisy-chained cameras. Requires `serialport` crate.
- **Pelco-D/P protocol** — Common in security/surveillance PTZ cameras.
- **Gamepad/joystick input** — Physical joystick support for professional operators.
  Would build on the continuous movement API from 1.2.
- **Camera power control** — Power on/off commands for VISCA cameras.
- **Multi-camera simultaneous control** — Move multiple cameras at once (not in
  obs-ptz either, but relevant for Audience's use case).

---

## Recommended Implementation Order

```
Phase 1 (Foundation):
  1.1 VISCA response parsing
  1.3 PTZ controls in operation mode
  2.1 Diagonal movement

Phase 2 (Control Quality):
  1.2 Continuous movement / virtual joystick
  2.2 Variable speed control
  2.5 Keyboard shortcuts
  3.5 Home position command

Phase 3 (Protocol Expansion):
  2.4 VISCA over TCP
  2.3 Focus control
  3.1 Command queue with retry

Phase 4 (Polish):
  3.2 Speed ramping
  3.3 Periodic position polling
  3.4 Camera image settings
```

Each phase builds on the previous one. Phase 1 items are mostly independent and
could be developed in parallel. Phase 2 depends on the `continuous_move` trait
method being added. Phase 3 expands protocol coverage. Phase 4 adds refinement.

---

## Key Architectural Observations

**What Audience does better than obs-ptz:**
- Preset overlay projected onto the video feed (obs-ptz has no visual preset
  representation on the camera image)
- Click-to-pan on video canvas with zoom-aware sensitivity
- Clean Rust async trait abstraction vs C++ class hierarchy
- Multi-protocol support beyond VISCA (Panasonic AW, BirdDog) that obs-ptz lacks
- Profile system for saving/loading preset sets per venue or show

**What obs-ptz does better:**
- Mature VISCA implementation with full response parsing and vendor quirk handling
- Smooth continuous movement via virtual joystick
- Comprehensive camera control (focus, WB, exposure, power)
- Safety mechanisms (studio mode lockout)
- Physical joystick/gamepad support
- Automated camera control via scene-triggered actions

The two projects serve different use cases — obs-ptz is tightly integrated with
OBS Studio's live production workflow, while Audience is a standalone controller
focused on preset-based event camera management. The improvements above borrow
obs-ptz's strengths while preserving Audience's unique value proposition.
