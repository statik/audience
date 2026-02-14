/**
 * Preset FOV rectangle estimation for overlay rendering.
 *
 * Each preset stores a PTZ position. To render an overlay rectangle on the
 * current video feed, we project the preset's FOV onto the current view.
 */

import type { Preset } from "@shared/types";

export interface OverlayRect {
  x: number; // left edge in canvas pixels
  y: number; // top edge in canvas pixels
  width: number; // width in canvas pixels
  height: number; // height in canvas pixels
  visible: boolean; // whether overlay is within current view
}

/**
 * Calculate the overlay rectangle for a preset on the current video canvas.
 *
 * @param preset - The preset to calculate overlay for
 * @param currentPan - Current camera pan (-1 to 1)
 * @param currentTilt - Current camera tilt (-1 to 1)
 * @param currentZoom - Current camera zoom (0 to 1)
 * @param cameraFovDegrees - Horizontal FOV at 1x zoom in degrees
 * @param canvasWidth - Canvas width in pixels
 * @param canvasHeight - Canvas height in pixels
 */
export function calculateOverlayRect(
  preset: Preset,
  currentPan: number,
  currentTilt: number,
  currentZoom: number,
  cameraFovDegrees: number,
  canvasWidth: number,
  canvasHeight: number
): OverlayRect {
  // Current FOV narrows as zoom increases
  const currentFov = cameraFovDegrees * (1 - currentZoom * 0.9);
  const presetFov = cameraFovDegrees * (1 - preset.zoom * 0.9);
  const aspectRatio = canvasWidth / canvasHeight;

  // Angular offset between preset and current view
  const panOffset = preset.pan - currentPan;
  const tiltOffset = preset.tilt - currentTilt;

  // Convert angular offset to canvas position (center = 0,0)
  const normalizedX = panOffset / (currentFov / cameraFovDegrees);
  const normalizedY = -tiltOffset / (currentFov / cameraFovDegrees); // Flip Y

  // Preset FOV relative to current FOV determines overlay size
  const relativeWidth = presetFov / currentFov;
  // Height is derived from width divided by aspect ratio (wider canvas = shorter overlay)
  const relativeHeight = relativeWidth / aspectRatio;

  // Convert to canvas pixels
  const centerX = canvasWidth / 2;
  const centerY = canvasHeight / 2;

  const rectWidth = relativeWidth * canvasWidth;
  const rectHeight = relativeHeight * canvasHeight;
  const rectX = centerX + normalizedX * centerX - rectWidth / 2;
  const rectY = centerY + normalizedY * centerY - rectHeight / 2;

  // Check visibility (partially in view)
  const visible =
    rectX + rectWidth > 0 &&
    rectX < canvasWidth &&
    rectY + rectHeight > 0 &&
    rectY < canvasHeight;

  return {
    x: rectX,
    y: rectY,
    width: rectWidth,
    height: rectHeight,
    visible,
  };
}
