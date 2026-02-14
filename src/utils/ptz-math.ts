/**
 * Click-to-pan/tilt vector calculations.
 *
 * When a user clicks on the video canvas, we calculate normalized deltas
 * from center and apply sensitivity + zoom-aware scaling.
 */

export interface ClickVector {
  panDelta: number;
  tiltDelta: number;
}

/**
 * Calculate pan/tilt adjustment from a canvas click.
 * @param clickX - X coordinate of the click on canvas
 * @param clickY - Y coordinate of the click on canvas
 * @param canvasWidth - Width of the canvas
 * @param canvasHeight - Height of the canvas
 * @param sensitivity - User-configured sensitivity multiplier
 * @param currentZoom - Current zoom level (0-1), used for zoom-aware scaling
 */
export function calculateClickVector(
  clickX: number,
  clickY: number,
  canvasWidth: number,
  canvasHeight: number,
  sensitivity: number,
  currentZoom: number
): ClickVector {
  const centerX = canvasWidth / 2;
  const centerY = canvasHeight / 2;

  // Normalize to -1..+1 range
  const deltaX = (clickX - centerX) / centerX;
  const deltaY = (centerY - clickY) / centerY; // Inverted Y: up = positive

  // Zoom-aware scaling: higher zoom = smaller angular movement per pixel
  const zoomFactor = currentZoom > 0 ? 1 / (1 + currentZoom * 4) : 1;

  return {
    panDelta: deltaX * sensitivity * zoomFactor,
    tiltDelta: deltaY * sensitivity * zoomFactor,
  };
}

/**
 * Clamp a PTZ value to its valid range.
 */
export function clampPtz(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}
