import { describe, it, expect } from "vitest";
import { calculateClickVector, clampPtz } from "./ptz-math";

describe("calculateClickVector", () => {
  const canvasWidth = 800;
  const canvasHeight = 600;

  it("returns zero deltas when clicking the center", () => {
    const result = calculateClickVector(400, 300, canvasWidth, canvasHeight, 0.1, 0);
    expect(result.panDelta).toBeCloseTo(0);
    expect(result.tiltDelta).toBeCloseTo(0);
  });

  it("returns positive panDelta for right-of-center clicks", () => {
    const result = calculateClickVector(600, 300, canvasWidth, canvasHeight, 0.1, 0);
    expect(result.panDelta).toBeGreaterThan(0);
    expect(result.tiltDelta).toBeCloseTo(0);
  });

  it("returns negative panDelta for left-of-center clicks", () => {
    const result = calculateClickVector(200, 300, canvasWidth, canvasHeight, 0.1, 0);
    expect(result.panDelta).toBeLessThan(0);
    expect(result.tiltDelta).toBeCloseTo(0);
  });

  it("returns positive tiltDelta for above-center clicks (inverted Y)", () => {
    const result = calculateClickVector(400, 100, canvasWidth, canvasHeight, 0.1, 0);
    expect(result.panDelta).toBeCloseTo(0);
    expect(result.tiltDelta).toBeGreaterThan(0);
  });

  it("returns negative tiltDelta for below-center clicks", () => {
    const result = calculateClickVector(400, 500, canvasWidth, canvasHeight, 0.1, 0);
    expect(result.panDelta).toBeCloseTo(0);
    expect(result.tiltDelta).toBeLessThan(0);
  });

  it("scales deltas by sensitivity", () => {
    const low = calculateClickVector(600, 300, canvasWidth, canvasHeight, 0.05, 0);
    const high = calculateClickVector(600, 300, canvasWidth, canvasHeight, 0.2, 0);
    expect(high.panDelta).toBeCloseTo(low.panDelta * 4);
  });

  it("reduces deltas at higher zoom levels", () => {
    const noZoom = calculateClickVector(600, 300, canvasWidth, canvasHeight, 0.1, 0);
    const highZoom = calculateClickVector(600, 300, canvasWidth, canvasHeight, 0.1, 0.5);
    expect(Math.abs(highZoom.panDelta)).toBeLessThan(Math.abs(noZoom.panDelta));
  });

  it("returns maximal deltas at the canvas corners", () => {
    const result = calculateClickVector(800, 0, canvasWidth, canvasHeight, 1, 0);
    expect(result.panDelta).toBeCloseTo(1);
    expect(result.tiltDelta).toBeCloseTo(1);
  });

  it("applies zoom factor of 1 when zoom is 0", () => {
    const result = calculateClickVector(800, 300, canvasWidth, canvasHeight, 1, 0);
    // deltaX = (800 - 400) / 400 = 1, sensitivity = 1, zoomFactor = 1
    expect(result.panDelta).toBeCloseTo(1);
  });

  it("correctly computes zoom factor for max zoom", () => {
    const result = calculateClickVector(800, 300, canvasWidth, canvasHeight, 1, 1);
    // zoomFactor = 1 / (1 + 1 * 4) = 0.2
    expect(result.panDelta).toBeCloseTo(0.2);
  });
});

describe("clampPtz", () => {
  it("returns value when within range", () => {
    expect(clampPtz(0.5, 0, 1)).toBe(0.5);
  });

  it("clamps to min when below range", () => {
    expect(clampPtz(-2, -1, 1)).toBe(-1);
  });

  it("clamps to max when above range", () => {
    expect(clampPtz(5, 0, 1)).toBe(1);
  });

  it("returns min when value equals min", () => {
    expect(clampPtz(0, 0, 1)).toBe(0);
  });

  it("returns max when value equals max", () => {
    expect(clampPtz(1, 0, 1)).toBe(1);
  });
});
