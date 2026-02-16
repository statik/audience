# E2E Testing Design: Automated UI Testing in GitHub Actions

## Overview

This document describes an end-to-end testing strategy for the Audience PTZ Camera Controller UI. The approach uses **Playwright** to test the React frontend against a mocked Tauri IPC layer, using the app's built-in **Simulated camera** as a live video stand-in. This enables fast, reliable UI automation in GitHub Actions without requiring Rust compilation, system-level dependencies, or real camera hardware.

## Why Playwright + Mocked Tauri IPC

### The core problem

Audience is a Tauri v2 desktop app. Full-stack E2E testing would require:
- Building the Rust backend (Rust toolchain + system deps like webkit2gtk)
- Launching the native window
- Driving it via WebDriver (tauri-driver)
- Platform-specific CI runners

This is slow, fragile, and expensive. The Rust backend already has its own test suite via `cargo test`.

### The approach

Test the React frontend in a real browser (Chromium) by intercepting `@tauri-apps/api/core` `invoke()` calls with controlled mock responses. The mock automatically provisions a **Simulated endpoint** so the app boots into a connected state with a live canvas-based camera feed. This covers:
- All UI rendering and interactions
- Video canvas interactions (click-to-pan, scroll-to-zoom)
- Preset overlay rendering and click-to-recall
- SimulatedFeed visual output responding to PTZ position changes
- State management (Zustand store)
- Component behavior across modes (calibration/operation)
- User flows (create presets, manage endpoints, adjust settings)
- Responsive layout behavior

What it does **not** cover (and doesn't need to):
- Actual VISCA/NDI/Panasonic protocol communication (covered by Rust tests)
- Real `getUserMedia` camera capture (requires hardware)
- MJPEG streaming from the Rust backend (requires the Rust process)
- NDI source discovery (requires NDI SDK + network sources)
- Native window chrome behavior

### Why Playwright over alternatives

| Criteria | Playwright | Cypress | Tauri WebDriver |
|---|---|---|---|
| GitHub Actions support | Built-in, first-class | Good | Requires native build |
| Headless execution | Yes | Yes | Linux only (WebKitGTK) |
| Speed | Fast (no Rust build) | Fast | Slow (full build) |
| Multi-browser | Chromium, Firefox, WebKit | Chromium only (free) | WebKit only |
| TypeScript | Native | Native | Limited |
| System deps in CI | None (auto-installs browsers) | None | webkit2gtk, etc. |
| IPC mocking | Easy via route/addInitScript | Possible | Not applicable |

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  Playwright Test Runner                             │
│                                                     │
│  ┌───────────────────────────────────────────────┐  │
│  │  Chromium (headless)                          │  │
│  │                                               │  │
│  │  ┌─────────────────────────────────────────┐  │  │
│  │  │  Vite Dev Server (localhost:1420)        │  │  │
│  │  │                                         │  │  │
│  │  │  React App                              │  │  │
│  │  │    ├── Components                       │  │  │
│  │  │    │     ├── VideoCanvas                │  │  │
│  │  │    │     │     └── SimulatedFeed ◄──────│──│──│── Canvas renders PTZ grid
│  │  │    │     └── PresetOverlay ◄────────────│──│──│── Positioned rectangles
│  │  │    ├── Zustand Store                    │  │  │
│  │  │    └── Hooks → invoke() ──┐             │  │  │
│  │  │                           │             │  │  │
│  │  │  ┌────────────────────────▼──────────┐  │  │  │
│  │  │  │  Tauri IPC Mock Layer             │  │  │  │
│  │  │  │  (Vite alias in E2E mode)         │  │  │  │
│  │  │  │                                   │  │  │  │
│  │  │  │  • Preseeded Simulated endpoint   │  │  │  │
│  │  │  │  • In-memory preset/endpoint CRUD │  │  │  │
│  │  │  │  • PTZ position state tracking    │  │  │  │
│  │  │  │  • Preset recall → position snap  │  │  │  │
│  │  │  └───────────────────────────────────┘  │  │  │
│  │  └─────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## Testing Video: The Simulated Feed Strategy

### The problem

The video path is the core of the app but doesn't go through Tauri `invoke()`:
- Local cameras use `navigator.mediaDevices.getUserMedia()` (browser API, requires hardware)
- NDI uses an MJPEG stream served by the Rust backend
- Neither is available in a headless CI browser

### The solution

The app already has a `SimulatedFeed` component (`src/components/SimulatedFeed.tsx`) that renders a canvas-based coordinate grid reacting to `currentPosition` from the Zustand store. When the active endpoint's protocol is `"Simulated"`, `VideoCanvas` detects `isSimulated`, sets `isConnected = true`, and renders `SimulatedFeed` instead of relying on a real video stream.

The IPC mock preseeds a Simulated endpoint and activates it, so the app boots into a "connected" state with a live, interactive canvas. No real camera hardware needed.

### What this enables

| Interaction | How it works in tests |
|---|---|
| **Click-to-pan** | Click on the canvas overlay → `handleVideoClick` → `calculateClickVector` → `invoke("ptz_move_relative")` → mock updates `currentPosition` → `SimulatedFeed` re-renders with shifted grid |
| **Scroll-to-zoom** | Wheel event on canvas → `handleVideoScroll` → `invoke("ptz_zoom")` → mock updates zoom → `SimulatedFeed` re-renders with scaled grid |
| **PTZ button controls** | Click Up/Down/Left/Right → `moveRelative` → mock updates position → grid shifts, position readout updates |
| **Preset overlays** | Create presets at known positions → `PresetOverlay` calls `calculateOverlayRect` → colored rectangles render at computed pixel coordinates |
| **Preset recall** | Click preset in operation mode → `recallPreset` → mock snaps `currentPosition` to preset → grid jumps, overlays reposition |
| **Visual regression** | `toHaveScreenshot()` captures the SimulatedFeed grid + overlays, catches rendering regressions |

### Mock setup for Simulated endpoint

The IPC mock starts with a preseeded Simulated endpoint already activated:

```typescript
const SIMULATED_ENDPOINT: CameraEndpoint = {
  id: "e2e-simulated",
  name: "E2E Test Camera",
  protocol: "Simulated",
  config: { type: "Simulated" },
};

let endpoints: CameraEndpoint[] = [SIMULATED_ENDPOINT];
let activeEndpointId: string | null = SIMULATED_ENDPOINT.id;
```

This means:
- `get_endpoints` returns the Simulated endpoint on first load
- `VideoCanvas` sees `isSimulated === true` and renders `SimulatedFeed`
- The app shows "Simulated Camera" in the status bar, connected state
- Tests can immediately interact with the canvas without setup boilerplate

### What remains untestable (and that's OK)

- **Real camera capture**: `getUserMedia` requires hardware — not testable in headless CI
- **MJPEG streaming**: Requires the Rust backend process running
- **Auto-reconnect**: The `ended`/`error` event reconnect loop on `<video>` only applies to real streams. Could be unit-tested by dispatching synthetic events on the video element, but this is better as a targeted unit test than an E2E scenario.
- **FPS counter accuracy**: The `requestAnimationFrame`-based FPS counter in `useVideoFeed` measures real video frame delivery. In E2E with SimulatedFeed the `<video>` element has no stream, so `currentTime` doesn't change and FPS stays at 0. This is expected and correct.

## Tauri IPC Mock Strategy

The frontend calls `invoke()` from `@tauri-apps/api/core` for all backend communication. In the E2E environment, we intercept this at the module level.

### Mock implementation

Create a mock module that replaces `@tauri-apps/api/core` when running under Playwright. The Vite dev server is configured to alias the import in test mode.

```typescript
// e2e/mocks/tauri-ipc.ts
import type { Preset, CameraEndpoint, AppSettings, PtzPosition } from "@shared/types";

const SIMULATED_ENDPOINT: CameraEndpoint = {
  id: "e2e-simulated",
  name: "E2E Test Camera",
  protocol: "Simulated",
  config: { type: "Simulated" },
};

// In-memory state for the mock backend
let presets: Preset[] = [];
let endpoints: CameraEndpoint[] = [SIMULATED_ENDPOINT];
let activeEndpointId: string | null = SIMULATED_ENDPOINT.id;
let settings: AppSettings = {
  click_sensitivity: 0.1,
  scroll_sensitivity: 0.05,
  overlay_opacity: 0.3,
  camera_fov_degrees: 65,
};
let currentPosition: PtzPosition = { pan: 0.0, tilt: 0.0, zoom: 0.0 };

const handlers: Record<string, (args: any) => any> = {
  // Presets
  get_all_presets: () => presets,
  create_preset: ({ name, pan, tilt, zoom, color }) => {
    const preset = { id: crypto.randomUUID(), name, pan, tilt, zoom, color };
    presets.push(preset);
    return preset;
  },
  update_preset: ({ preset }) => {
    presets = presets.map((p) => (p.id === preset.id ? preset : p));
    return preset;
  },
  delete_preset: ({ presetId }) => {
    presets = presets.filter((p) => p.id !== presetId);
  },

  // Endpoints
  get_endpoints: () => endpoints,
  create_endpoint: ({ endpoint }) => {
    endpoints.push(endpoint);
    return endpoint;
  },
  update_endpoint: ({ endpoint }) => {
    endpoints = endpoints.map((e) => (e.id === endpoint.id ? endpoint : e));
    return endpoint;
  },
  delete_endpoint: ({ endpointId }) => {
    endpoints = endpoints.filter((e) => e.id !== endpointId);
    if (activeEndpointId === endpointId) activeEndpointId = null;
  },
  set_active_endpoint: ({ endpointId }) => {
    activeEndpointId = endpointId;
  },
  clear_active_endpoint: () => {
    activeEndpointId = null;
  },
  test_endpoint_connection: () => "Connection successful (simulated)",

  // PTZ
  ptz_get_position: () => currentPosition,
  ptz_move_relative: ({ panDelta, tiltDelta }) => {
    currentPosition = {
      pan: Math.max(-1, Math.min(1, currentPosition.pan + panDelta)),
      tilt: Math.max(-1, Math.min(1, currentPosition.tilt + tiltDelta)),
      zoom: currentPosition.zoom,
    };
  },
  ptz_move_absolute: ({ pan, tilt, zoom }) => {
    currentPosition = { pan, tilt, zoom };
  },
  ptz_zoom: ({ zoom }) => {
    currentPosition = { ...currentPosition, zoom: Math.max(0, Math.min(1, zoom)) };
  },
  ptz_recall_preset: ({ presetId }) => {
    const preset = presets.find((p) => p.id === presetId);
    if (preset) {
      currentPosition = { pan: preset.pan, tilt: preset.tilt, zoom: preset.zoom };
    }
  },
  ptz_store_preset: () => {},

  // Settings
  get_settings: () => settings,
  update_settings: (newSettings) => {
    settings = { ...settings, ...newSettings };
    return settings;
  },

  // Video (no-ops — SimulatedFeed doesn't use these)
  list_ndi_sources: () => [],
  list_local_devices: () => [],
  start_mjpeg_stream: () => 0,
  stop_mjpeg_stream: () => {},
  get_mjpeg_port: () => null,

  // Profiles
  get_profiles: () => [],
  save_profile: ({ profile }) => profile,
  load_profile: () => null,
  delete_profile: () => {},
};

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  const handler = handlers[cmd];
  if (!handler) {
    console.warn(`[e2e mock] unhandled invoke: ${cmd}`, args);
    return undefined as T;
  }
  return handler(args ?? {}) as T;
}
```

### Injection method

Use Vite's conditional aliasing to swap the Tauri API module in E2E mode:

```typescript
// vite.config.ts (E2E additions)
export default defineConfig(async () => ({
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@shared": path.resolve(__dirname, "./shared"),
      // When VITE_E2E=true, replace Tauri API with mock
      ...(process.env.VITE_E2E && {
        "@tauri-apps/api/core": path.resolve(__dirname, "./e2e/mocks/tauri-ipc.ts"),
        "@tauri-apps/plugin-shell": path.resolve(__dirname, "./e2e/mocks/tauri-shell.ts"),
      }),
    },
  },
}));
```

This approach is clean because:
- No runtime feature flags in production code
- The mock is a full in-memory backend with realistic behavior
- The Simulated endpoint is preseeded — tests start with a "connected" camera
- Tests interact with the real React components, real Zustand store, real hooks
- Only the Tauri IPC boundary is replaced
- `SimulatedFeed` renders a real interactive canvas driven by the same store

## Test Data IDs

Add `data-testid` attributes to key interactive elements for reliable selectors. These are stable across styling changes and refactors.

### Required additions

| Component | Element | data-testid |
|---|---|---|
| **Toolbar** | Calibration button | `mode-calibration` |
| | Operation button | `mode-operation` |
| | Sidebar toggle | `toggle-sidebar` |
| | Settings button | `open-settings` |
| **PresetList** | Container | `preset-list` |
| | Each preset item | `preset-item-{id}` |
| | Add preset button | `add-preset` |
| | Delete button | `delete-preset-{id}` |
| **PresetEditor** | Name input | `preset-name-input` |
| | Save button | `preset-save` |
| | Cancel button | `preset-cancel` |
| **EndpointManager** | Container | `endpoint-list` |
| | Add endpoint button | `add-endpoint` |
| | Name input | `endpoint-name-input` |
| | Protocol select | `endpoint-protocol` |
| | Save button | `endpoint-save` |
| | Activate/Deactivate | `endpoint-activate-{id}` |
| | Delete button | `endpoint-delete-{id}` |
| **PtzControls** | Pan/Tilt buttons | `ptz-up`, `ptz-down`, `ptz-left`, `ptz-right` |
| | Zoom controls | `ptz-zoom-in`, `ptz-zoom-out`, `ptz-zoom-slider` |
| | Position readout | `ptz-position` |
| **SettingsPanel** | Panel container | `settings-panel` |
| | Close button | `settings-close` |
| | Sliders | `setting-click-sensitivity`, `setting-scroll-sensitivity`, `setting-overlay-opacity` |
| | FOV input | `setting-camera-fov` |
| **StatusBar** | Connection indicator | `status-connection` |
| | Mode display | `status-mode` |
| | FPS display | `status-fps` |
| **VideoCanvas** | Container | `video-canvas` |
| | No signal message | `no-signal` |
| **SimulatedFeed** | Canvas element | `simulated-feed` |
| **PresetOverlay** | Each overlay rect | `preset-overlay-{id}` |

## Directory Structure

```
e2e/
├── mocks/
│   ├── tauri-ipc.ts          # Mock invoke() with in-memory state + preseeded Simulated endpoint
│   └── tauri-shell.ts        # Mock shell plugin (no-op)
├── fixtures/
│   └── test-data.ts          # Shared preset/endpoint factory helpers
├── tests/
│   ├── app-layout.spec.ts    # Basic layout, sidebar toggle, connected state
│   ├── mode-switching.spec.ts # Calibration ↔ Operation mode
│   ├── presets.spec.ts       # Create, select, delete presets
│   ├── endpoints.spec.ts     # CRUD endpoints, protocol switching
│   ├── ptz-controls.spec.ts  # Pan/tilt/zoom buttons + position readout
│   ├── video-interaction.spec.ts  # Click-to-pan, scroll-to-zoom on canvas
│   ├── preset-overlays.spec.ts    # Overlay rendering, click-to-recall
│   ├── settings.spec.ts      # Settings panel, slider interactions
│   └── status-bar.spec.ts    # Connection status, mode display
└── playwright.config.ts      # Playwright configuration
```

## Test Scenarios

### 1. App Layout (`app-layout.spec.ts`)

- App renders with toolbar, video area, sidebar, and status bar
- Sidebar is visible by default
- Clicking "Hide Panel" hides the sidebar
- Clicking "Show Panel" restores the sidebar
- SimulatedFeed canvas is rendered (app boots connected via preseeded endpoint)
- Status bar shows "Simulated Camera" connection label
- "No Signal" message is NOT displayed (because Simulated endpoint is active)

### 2. Mode Switching (`mode-switching.spec.ts`)

- App starts in calibration mode
- Calibration button is highlighted in calibration mode
- Clicking Operation switches to operation mode
- Operation button is highlighted in operation mode
- Switching to calibration shows PTZ controls in sidebar
- Switching to operation hides PTZ controls
- Status bar reflects current mode
- SimulatedFeed remains rendered across mode switches

### 3. Presets (`presets.spec.ts`)

- Preset list shows "0" count when empty
- Clicking "+ Add Preset" opens the preset editor (calibration mode)
- Entering a name and saving creates a preset
- New preset appears in the list with correct name
- Preset count updates after adding
- Clicking a preset selects it (visual highlight)
- Delete button appears on hover in calibration mode
- Delete requires confirmation (click once = "Confirm?", click again = delete)
- Deleted preset is removed from the list
- "+ Add Preset" button is not visible in operation mode
- Delete button is not visible in operation mode

### 4. Endpoints (`endpoints.spec.ts`)

- Preseeded "E2E Test Camera" Simulated endpoint is listed on load
- Clicking "+ Add Endpoint" opens the editor form
- Default protocol is VISCA with default host/port
- Changing protocol updates the form fields
- NDI protocol hides host/port fields
- Simulated protocol hides host/port fields
- VISCA/Panasonic/BirdDog show host and port fields
- Panasonic shows username/password fields
- Saving an endpoint adds it to the list
- Activate/Deactivate toggle works
- Edit button opens the editor with existing values
- Delete button removes the endpoint (with confirmation)
- Test Connection button shows "Connection successful (simulated)"

### 5. PTZ Controls (`ptz-controls.spec.ts`)

- PTZ controls visible only in calibration mode
- Position readout shows initial position (P:0.000, T:0.000, Z:0.000)
- Clicking Up updates tilt in position readout
- Clicking Down updates tilt in the opposite direction
- Clicking Left/Right updates pan in position readout
- SimulatedFeed grid visually shifts after pan/tilt (verified via screenshot or readout text)
- Zoom +/- buttons adjust zoom, readout reflects change
- Zoom slider drag updates zoom level
- Rapid clicks are throttled (100ms interval) — second immediate click is ignored

### 6. Video Interaction (`video-interaction.spec.ts`)

These tests verify the click-to-pan and scroll-to-zoom pipeline end-to-end through the SimulatedFeed.

- **Click-to-pan in operation mode**: Click on the right side of the canvas → pan increases (readout or store check)
- **Click-to-pan in calibration mode**: Click on the canvas → no pan change (clicks are ignored in calibration mode per `VideoCanvas.onCanvasClick`)
- **Scroll-to-zoom**: Wheel event with negative deltaY on canvas → zoom increases
- **Scroll-to-zoom**: Wheel event with positive deltaY → zoom decreases
- **Zoom-aware sensitivity**: At high zoom, a click produces a smaller pan delta than the same click at low zoom (verified via `calculateClickVector` behavior reflected in position readout)
- **SimulatedFeed re-renders**: After PTZ changes, the SimulatedFeed's position readout text (`P:... T:... Z:...`) rendered on the canvas reflects the new position

### 7. Preset Overlays (`preset-overlays.spec.ts`)

These tests verify that preset overlay rectangles render correctly and respond to clicks.

- Create a preset at pan=0, tilt=0, zoom=0 → overlay rectangle is visible and centered on the canvas
- Create a preset at a different position → overlay renders at the correct offset
- Overlay rectangle shows the preset name label
- Overlay rectangle uses the preset's assigned color
- Clicking an overlay in operation mode triggers preset recall → camera position snaps to preset, SimulatedFeed grid jumps
- Multiple presets render multiple non-overlapping overlay rectangles (at different positions)
- After zooming in, overlay rectangles scale and reposition correctly
- Overlay respects `overlay_opacity` setting (visual regression test)

### 8. Settings (`settings.spec.ts`)

- Clicking Settings button opens the settings panel
- Settings panel displays as a modal overlay
- Click sensitivity slider has correct default (0.1)
- Moving slider updates the displayed value
- Scroll sensitivity slider works
- Overlay opacity slider works
- Camera FOV input accepts numeric values
- Closing settings panel removes the modal
- Settings persist after closing and reopening the panel

### 9. Status Bar (`status-bar.spec.ts`)

- Status bar shows "Simulated Camera" as connection label (preseeded endpoint)
- Connection indicator shows connected state (green dot)
- Mode display shows "Calibration" or "Operation"
- Mode display updates when mode changes
- Deactivating the Simulated endpoint → connection indicator shows disconnected, "No Signal" message appears

## Playwright Configuration

```typescript
// e2e/playwright.config.ts
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? [["github"], ["html", { open: "never" }]] : "list",
  use: {
    baseURL: "http://localhost:1420",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
  webServer: {
    command: "VITE_E2E=true npm run dev",
    url: "http://localhost:1420",
    reuseExistingServer: !process.env.CI,
  },
});
```

Key configuration choices:
- **Single browser (Chromium)**: The app runs in Tauri's WebView (WebKit-based), but Chromium tests catch virtually all frontend bugs. WebKit can be added later if needed.
- **Serial in CI (`workers: 1`)**: Avoids port conflicts with the single Vite dev server.
- **Retries in CI**: Handles transient failures (2 retries).
- **Trace on first retry**: Captures full traces for debugging flaky tests.
- **Screenshots on failure**: Automatically captured for CI debugging.
- **GitHub reporter**: Native annotations on PR checks.

## GitHub Actions Workflow

Add an `e2e` job to the existing `ci.yml`:

```yaml
  e2e:
    needs: changes
    if: needs.changes.outputs.frontend == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          persist-credentials: false

      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: npm

      - run: npm ci --ignore-scripts

      - name: Install Playwright browsers
        run: npx playwright install --with-deps chromium

      - name: Run E2E tests
        run: npx playwright test --config e2e/playwright.config.ts

      - name: Upload test report
        uses: actions/upload-artifact@v4
        if: ${{ !cancelled() }}
        with:
          name: playwright-report
          path: playwright-report/
          retention-days: 14
```

The `changes` filter should be updated to trigger E2E tests when `e2e/**` files change:

```yaml
frontend:
  - 'src/**'
  - 'shared/**'
  - 'e2e/**'          # <-- add this
  - 'package.json'
  # ... rest unchanged
```

The `ci-ok` job should also be updated to include the `e2e` job result.

## Package.json Changes

```json
{
  "scripts": {
    "test:e2e": "playwright test --config e2e/playwright.config.ts",
    "test:e2e:ui": "playwright test --config e2e/playwright.config.ts --ui"
  },
  "devDependencies": {
    "@playwright/test": "^1.50.0"
  }
}
```

## Justfile Changes

```just
# Run E2E tests
test-e2e:
    npx playwright test --config e2e/playwright.config.ts

# Run E2E tests with interactive UI
test-e2e-ui:
    npx playwright test --config e2e/playwright.config.ts --ui
```

## Implementation Plan

### Phase 1: Foundation

1. Install Playwright: `npm install -D @playwright/test`
2. Create `e2e/` directory structure
3. Write the Tauri IPC mock with preseeded Simulated endpoint (`e2e/mocks/tauri-ipc.ts`)
4. Write the shell plugin no-op mock (`e2e/mocks/tauri-shell.ts`)
5. Add the `VITE_E2E` conditional alias to `vite.config.ts`
6. Create `e2e/playwright.config.ts`
7. Add `data-testid` attributes to all components listed in the Test Data IDs table
8. Write a smoke test (`app-layout.spec.ts`) that validates:
   - App boots with SimulatedFeed visible
   - Status bar shows "Simulated Camera"
   - No "No Signal" message

### Phase 2: Core Test Suites

9. Write mode switching tests
10. Write preset CRUD tests
11. Write endpoint management tests
12. Write PTZ controls tests (button clicks → position readout changes)
13. Write video interaction tests (click-to-pan, scroll-to-zoom on canvas)
14. Write preset overlay tests (rendering, click-to-recall, position snap)
15. Write settings panel tests
16. Write status bar tests

### Phase 3: CI Integration

17. Add `e2e` job to `ci.yml`
18. Update `changes` filter to include `e2e/**`
19. Update `ci-ok` job to check `e2e` result
20. Add scripts to `package.json` and `justfile`

### Phase 4: Future Enhancements (optional, not in initial scope)

- **Visual regression testing**: Use `toHaveScreenshot()` to snapshot the SimulatedFeed grid + overlays and catch rendering regressions in preset overlay positioning, color, opacity
- **Firefox/WebKit browsers**: Add browser projects to catch cross-engine rendering bugs
- **Accessibility testing**: Integrate `@axe-core/playwright` for automated a11y audits
- **Full-stack Tauri WebDriver tests**: For critical integration paths that must exercise the real Rust backend (e.g., VISCA command serialization, persistence to disk)
