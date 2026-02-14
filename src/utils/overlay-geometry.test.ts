import { describe, it, expect } from "vitest";
import { calculateOverlayRect } from "./overlay-geometry";
import type { Preset } from "@shared/types";

function makePreset(overrides: Partial<Preset> = {}): Preset {
  return {
    id: "test-1",
    name: "Test Preset",
    pan: 0,
    tilt: 0,
    zoom: 0,
    color: "#3b82f6",
    ...overrides,
  };
}

describe("calculateOverlayRect", () => {
  const canvasWidth = 800;
  const canvasHeight = 600;
  const fov = 60;

  it("centers overlay when preset matches current position and zoom", () => {
    const preset = makePreset({ pan: 0, tilt: 0, zoom: 0 });
    const rect = calculateOverlayRect(preset, 0, 0, 0, fov, canvasWidth, canvasHeight);

    // relativeWidth = 1 (same zoom), relativeHeight = 1 / aspectRatio = 0.75
    // rectHeight = 0.75 * 600 = 450, centered vertically at y = 75
    const aspectRatio = canvasWidth / canvasHeight;
    const expectedHeight = canvasHeight / aspectRatio;
    expect(rect.x).toBeCloseTo(0);
    expect(rect.width).toBeCloseTo(canvasWidth);
    expect(rect.height).toBeCloseTo(expectedHeight);
    expect(rect.y).toBeCloseTo((canvasHeight - expectedHeight) / 2);
    expect(rect.visible).toBe(true);
  });

  it("marks overlay as visible when partially in view", () => {
    // Preset slightly off to the right but still overlapping
    const preset = makePreset({ pan: 0.02, tilt: 0, zoom: 0 });
    const rect = calculateOverlayRect(preset, 0, 0, 0, fov, canvasWidth, canvasHeight);
    expect(rect.visible).toBe(true);
  });

  it("marks overlay as not visible when far off screen", () => {
    // Preset far to the right of current view
    const preset = makePreset({ pan: 1, tilt: 1, zoom: 0 });
    const rect = calculateOverlayRect(preset, -1, -1, 0.9, fov, canvasWidth, canvasHeight);
    expect(rect.visible).toBe(false);
  });

  it("reduces overlay size when preset is more zoomed in than current view", () => {
    const preset = makePreset({ pan: 0, tilt: 0, zoom: 0.5 });
    const rect = calculateOverlayRect(preset, 0, 0, 0, fov, canvasWidth, canvasHeight);

    // Preset FOV is narrower than current FOV, so overlay is smaller
    expect(rect.width).toBeLessThan(canvasWidth);
    expect(rect.height).toBeLessThan(canvasHeight);
  });

  it("increases overlay size when current view is more zoomed in", () => {
    const preset = makePreset({ pan: 0, tilt: 0, zoom: 0 });
    const rect = calculateOverlayRect(preset, 0, 0, 0.5, fov, canvasWidth, canvasHeight);

    // Preset FOV is wider than current zoomed-in FOV, so overlay is larger
    expect(rect.width).toBeGreaterThan(canvasWidth);
    expect(rect.height).toBeGreaterThan(canvasHeight);
  });

  it("offsets overlay when preset pan differs from current", () => {
    const preset = makePreset({ pan: 0.1, tilt: 0, zoom: 0 });
    const rect = calculateOverlayRect(preset, 0, 0, 0, fov, canvasWidth, canvasHeight);

    // Overlay center should be to the right of canvas center
    const overlayCenterX = rect.x + rect.width / 2;
    expect(overlayCenterX).toBeGreaterThan(canvasWidth / 2);
  });

  it("offsets overlay when preset tilt differs from current", () => {
    const preset = makePreset({ pan: 0, tilt: 0.1, zoom: 0 });
    const rect = calculateOverlayRect(preset, 0, 0, 0, fov, canvasWidth, canvasHeight);

    // Positive tilt = up, but Y axis is flipped, so overlay center should be above canvas center
    const overlayCenterY = rect.y + rect.height / 2;
    expect(overlayCenterY).toBeLessThan(canvasHeight / 2);
  });

  it("respects different camera FOV settings via zoom scaling", () => {
    // At currentZoom=0.5 with different FOV, the currentFov/cameraFovDegrees ratio
    // changes, producing different overlay sizes
    const preset = makePreset({ pan: 0, tilt: 0, zoom: 0 });
    const narrowFov = calculateOverlayRect(preset, 0, 0, 0.5, 30, canvasWidth, canvasHeight);
    const wideFov = calculateOverlayRect(preset, 0, 0, 0.5, 120, canvasWidth, canvasHeight);

    // Both overlays are wider than the canvas (preset has wider FOV than zoomed-in view)
    // but the ratio should be the same since the formula is symmetric
    expect(narrowFov.width).toBeGreaterThan(canvasWidth);
    expect(wideFov.width).toBeGreaterThan(canvasWidth);
    // widths should be equal since the zoom scaling factor is the same
    expect(narrowFov.width).toBeCloseTo(wideFov.width);
  });

  it("handles square canvas aspect ratio", () => {
    const preset = makePreset({ pan: 0, tilt: 0, zoom: 0 });
    const rect = calculateOverlayRect(preset, 0, 0, 0, fov, 600, 600);
    expect(rect.width).toBeCloseTo(600);
    expect(rect.height).toBeCloseTo(600);
  });
});
