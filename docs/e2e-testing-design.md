# E2E Testing Design: Automated UI Testing in GitHub Actions

## Overview

This document describes an end-to-end testing strategy for the Audience PTZ Camera Controller UI. The approach uses **Playwright** to test the React frontend against a mocked Tauri IPC layer, enabling fast, reliable UI automation in GitHub Actions without requiring Rust compilation or system-level dependencies.

## Why Playwright + Mocked Tauri IPC

### The core problem

Audience is a Tauri v2 desktop app. Full-stack E2E testing would require:
- Building the Rust backend (Rust toolchain + system deps like webkit2gtk)
- Launching the native window
- Driving it via WebDriver (tauri-driver)
- Platform-specific CI runners

This is slow, fragile, and expensive. The Rust backend already has its own test suite via `cargo test`.

### The approach

Test the React frontend in a real browser (Chromium) by intercepting `@tauri-apps/api/core` `invoke()` calls with controlled mock responses. This covers:
- All UI rendering and interactions
- State management (Zustand store)
- Component behavior across modes (calibration/operation)
- User flows (create presets, manage endpoints, adjust settings)
- Responsive layout behavior

What it does **not** cover (and doesn't need to):
- Actual VISCA/NDI/Panasonic protocol communication (covered by Rust tests)
- Native window chrome behavior
- Real camera hardware interaction

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
│  │  │    ├── Zustand Store                    │  │  │
│  │  │    └── Hooks → invoke() ──┐             │  │  │
│  │  │                           │             │  │  │
│  │  │  ┌────────────────────────▼──────────┐  │  │  │
│  │  │  │  Tauri IPC Mock Layer             │  │  │  │
│  │  │  │  (injected via addInitScript)     │  │  │  │
│  │  │  │                                   │  │  │  │
│  │  │  │  invoke("get_all_presets") → []    │  │  │  │
│  │  │  │  invoke("get_endpoints") → []     │  │  │  │
│  │  │  │  invoke("create_preset") → {...}  │  │  │  │
│  │  │  │  ...                              │  │  │  │
│  │  │  └───────────────────────────────────┘  │  │  │
│  │  └─────────────────────────────────────────┘  │  │
│  └───────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

## Tauri IPC Mock Strategy

The frontend calls `invoke()` from `@tauri-apps/api/core` for all backend communication. In the E2E environment, we intercept this at the module level.

### Mock implementation

Create a mock module that replaces `@tauri-apps/api/core` when running under Playwright. The Vite dev server is configured to alias the import in test mode.

```typescript
// e2e/mocks/tauri-ipc.ts
// In-memory state for the mock backend
let presets: Preset[] = [];
let endpoints: CameraEndpoint[] = [];
let settings: AppSettings = {
  click_sensitivity: 0.1,
  scroll_sensitivity: 0.05,
  overlay_opacity: 0.3,
  camera_fov_degrees: 65,
};
let currentPosition: PtzPosition = { pan: 0.5, tilt: 0.5, zoom: 0.0 };

const handlers: Record<string, (args: any) => any> = {
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
  get_endpoints: () => endpoints,
  create_endpoint: ({ endpoint }) => {
    endpoints.push(endpoint);
    return endpoint;
  },
  // ... remaining handlers
  ptz_get_position: () => currentPosition,
  ptz_move_relative: ({ panDelta, tiltDelta }) => {
    currentPosition.pan += panDelta;
    currentPosition.tilt += tiltDelta;
  },
  ptz_zoom: ({ zoom }) => {
    currentPosition.zoom = zoom;
  },
  get_settings: () => settings,
  update_settings: (newSettings) => {
    settings = { ...settings, ...newSettings };
    return settings;
  },
};
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
- Tests interact with the real React components, real Zustand store, real hooks
- Only the Tauri IPC boundary is replaced

## Test Data IDs

Add `data-testid` attributes to key interactive elements for reliable selectors. These are stable across styling changes and refactors.

### Required additions

| Component | Element | data-testid |
|---|---|---|
| Toolbar | Calibration button | `mode-calibration` |
| Toolbar | Operation button | `mode-operation` |
| Toolbar | Sidebar toggle | `toggle-sidebar` |
| Toolbar | Settings button | `open-settings` |
| PresetList | Container | `preset-list` |
| PresetList | Each preset item | `preset-item-{id}` |
| PresetList | Add preset button | `add-preset` |
| PresetList | Delete button | `delete-preset-{id}` |
| PresetEditor | Name input | `preset-name-input` |
| PresetEditor | Save button | `preset-save` |
| PresetEditor | Cancel button | `preset-cancel` |
| EndpointManager | Container | `endpoint-list` |
| EndpointManager | Add endpoint button | `add-endpoint` |
| EndpointManager | Name input | `endpoint-name-input` |
| EndpointManager | Protocol select | `endpoint-protocol` |
| EndpointManager | Save button | `endpoint-save` |
| EndpointManager | Activate/Deactivate | `endpoint-activate-{id}` |
| EndpointManager | Delete button | `endpoint-delete-{id}` |
| PtzControls | Pan/Tilt buttons | `ptz-up`, `ptz-down`, `ptz-left`, `ptz-right` |
| PtzControls | Zoom controls | `ptz-zoom-in`, `ptz-zoom-out`, `ptz-zoom-slider` |
| PtzControls | Position readout | `ptz-position` |
| SettingsPanel | Panel container | `settings-panel` |
| SettingsPanel | Close button | `settings-close` |
| SettingsPanel | Click sensitivity slider | `setting-click-sensitivity` |
| SettingsPanel | Scroll sensitivity slider | `setting-scroll-sensitivity` |
| SettingsPanel | Overlay opacity slider | `setting-overlay-opacity` |
| SettingsPanel | FOV input | `setting-camera-fov` |
| StatusBar | Connection indicator | `status-connection` |
| StatusBar | Mode display | `status-mode` |
| StatusBar | FPS display | `status-fps` |
| VideoCanvas | Container | `video-canvas` |
| VideoCanvas | No signal message | `no-signal` |

## Directory Structure

```
e2e/
├── mocks/
│   ├── tauri-ipc.ts          # Mock invoke() with in-memory state
│   └── tauri-shell.ts        # Mock shell plugin (no-op)
├── fixtures/
│   └── test-data.ts          # Shared preset/endpoint fixtures
├── tests/
│   ├── app-layout.spec.ts    # Basic layout, sidebar toggle
│   ├── mode-switching.spec.ts # Calibration ↔ Operation mode
│   ├── presets.spec.ts       # Create, select, delete presets
│   ├── endpoints.spec.ts     # CRUD endpoints, protocol switching
│   ├── ptz-controls.spec.ts  # Pan/tilt/zoom button interactions
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
- No signal message is displayed when no video source is connected

### 2. Mode Switching (`mode-switching.spec.ts`)

- App starts in calibration mode
- Calibration button is highlighted in calibration mode
- Clicking Operation switches to operation mode
- Operation button is highlighted in operation mode
- Switching to calibration shows PTZ controls in sidebar
- Switching to operation hides PTZ controls
- Status bar reflects current mode

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
- Clicking a preset in operation mode triggers recall

### 4. Endpoints (`endpoints.spec.ts`)

- "No camera endpoints configured" message when empty
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
- Test Connection button shows result

### 5. PTZ Controls (`ptz-controls.spec.ts`)

- PTZ controls visible only in calibration mode
- Up/Down/Left/Right buttons exist
- Clicking Up calls moveRelative with positive tilt delta
- Zoom slider reflects current zoom level
- Zoom +/- buttons adjust zoom
- Position readout shows current pan/tilt/zoom

### 6. Settings (`settings.spec.ts`)

- Clicking Settings button opens the settings panel
- Settings panel displays as a modal overlay
- Click sensitivity slider has correct default (0.1)
- Moving slider updates the displayed value
- Scroll sensitivity slider works
- Overlay opacity slider works
- Camera FOV input accepts numeric values
- Closing settings panel removes the modal
- Settings persist after closing and reopening the panel

### 7. Status Bar (`status-bar.spec.ts`)

- Status bar shows "Not connected" initially (or equivalent)
- Mode display shows "Calibration" or "Operation"
- Mode display updates when mode changes

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
3. Write the Tauri IPC mock (`e2e/mocks/tauri-ipc.ts`)
4. Add the `VITE_E2E` alias to `vite.config.ts`
5. Create `e2e/playwright.config.ts`
6. Add `data-testid` attributes to components
7. Write a smoke test (`app-layout.spec.ts`) to validate the setup

### Phase 2: Core Test Suites

8. Write mode switching tests
9. Write preset CRUD tests
10. Write endpoint management tests
11. Write PTZ controls tests
12. Write settings panel tests
13. Write status bar tests

### Phase 3: CI Integration

14. Add `e2e` job to `ci.yml`
15. Update `changes` filter to include `e2e/**`
16. Update `ci-ok` job to check `e2e` result
17. Add scripts to `package.json` and `justfile`

### Phase 4: Future Enhancements (optional, not in initial scope)

- Add Firefox/WebKit browser projects
- Visual regression testing with `toHaveScreenshot()`
- Accessibility testing with `@axe-core/playwright`
- Full-stack Tauri WebDriver tests for critical integration paths
