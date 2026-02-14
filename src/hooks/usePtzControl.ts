import { useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/app-store";
import type { PtzPosition } from "@shared/types";

const MIN_COMMAND_INTERVAL_MS = 100;

export function usePtzControl() {
  const settings = useAppStore((s) => s.settings);
  const currentPosition = useAppStore((s) => s.currentPosition);
  const setCurrentPosition = useAppStore((s) => s.setCurrentPosition);
  const lastCommandTime = useRef(0);

  const throttle = useCallback((): boolean => {
    const now = Date.now();
    if (now - lastCommandTime.current < MIN_COMMAND_INTERVAL_MS) {
      return false;
    }
    lastCommandTime.current = now;
    return true;
  }, []);

  const moveRelative = useCallback(
    async (panDelta: number, tiltDelta: number) => {
      if (!throttle()) return;
      try {
        await invoke("ptz_move_relative", {
          panDelta,
          tiltDelta,
        });
        setCurrentPosition({
          ...currentPosition,
          pan: Math.max(-1, Math.min(1, currentPosition.pan + panDelta)),
          tilt: Math.max(-1, Math.min(1, currentPosition.tilt + tiltDelta)),
        });
      } catch (err) {
        console.error("PTZ move failed:", err);
      }
    },
    [throttle, currentPosition, setCurrentPosition]
  );

  const moveAbsolute = useCallback(
    async (pan: number, tilt: number, zoom: number) => {
      if (!throttle()) return;
      try {
        await invoke("ptz_move_absolute", { pan, tilt, zoom });
        setCurrentPosition({ pan, tilt, zoom });
      } catch (err) {
        console.error("PTZ move failed:", err);
      }
    },
    [throttle, setCurrentPosition]
  );

  const zoom = useCallback(
    async (zoomLevel: number) => {
      if (!throttle()) return;
      try {
        await invoke("ptz_zoom", { zoom: zoomLevel });
        setCurrentPosition({ ...currentPosition, zoom: zoomLevel });
      } catch (err) {
        console.error("PTZ zoom failed:", err);
      }
    },
    [throttle, currentPosition, setCurrentPosition]
  );

  const recallPreset = useCallback(
    async (presetId: string) => {
      try {
        await invoke("ptz_recall_preset", { presetId });
        const pos = await invoke<PtzPosition>("ptz_get_position");
        setCurrentPosition(pos);
      } catch (err) {
        console.error("Preset recall failed:", err);
      }
    },
    [setCurrentPosition]
  );

  /** Handle a click on the video canvas to adjust pan/tilt. */
  const handleVideoClick = useCallback(
    (clickX: number, clickY: number, canvasWidth: number, canvasHeight: number) => {
      const centerX = canvasWidth / 2;
      const centerY = canvasHeight / 2;
      const deltaX = (clickX - centerX) / centerX; // -1 to +1
      const deltaY = (centerY - clickY) / centerY; // -1 to +1 (inverted Y)

      const zoomFactor = currentPosition.zoom > 0 ? 1 / (1 + currentPosition.zoom * 4) : 1;
      const panAdjustment = deltaX * settings.click_sensitivity * zoomFactor;
      const tiltAdjustment = deltaY * settings.click_sensitivity * zoomFactor;

      moveRelative(panAdjustment, tiltAdjustment);
    },
    [currentPosition, settings.click_sensitivity, moveRelative]
  );

  /** Handle scroll on the video canvas to adjust zoom. */
  const handleVideoScroll = useCallback(
    (deltaY: number) => {
      if (!throttle()) return;
      const zoomDelta = -deltaY * settings.scroll_sensitivity * 0.01;
      const newZoom = Math.max(0, Math.min(1, currentPosition.zoom + zoomDelta));
      zoom(newZoom);
    },
    [throttle, settings.scroll_sensitivity, currentPosition.zoom, zoom]
  );

  return {
    currentPosition,
    moveRelative,
    moveAbsolute,
    zoom,
    recallPreset,
    handleVideoClick,
    handleVideoScroll,
  };
}
