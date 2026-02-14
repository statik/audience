import { useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/app-store";
import { calculateClickVector } from "../utils/ptz-math";
import type { PtzPosition } from "@shared/types";

const MIN_COMMAND_INTERVAL_MS = 100;

export function usePtzControl() {
  const settings = useAppStore((s) => s.settings);
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
        // Read current state at call time to avoid stale closures
        const pos = useAppStore.getState().currentPosition;
        setCurrentPosition({
          pan: Math.max(-1, Math.min(1, pos.pan + panDelta)),
          tilt: Math.max(-1, Math.min(1, pos.tilt + tiltDelta)),
          zoom: pos.zoom,
        });
      } catch (err) {
        console.error("PTZ move failed:", err);
      }
    },
    [throttle, setCurrentPosition]
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
        const pos = useAppStore.getState().currentPosition;
        setCurrentPosition({ ...pos, zoom: zoomLevel });
      } catch (err) {
        console.error("PTZ zoom failed:", err);
      }
    },
    [throttle, setCurrentPosition]
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
      const pos = useAppStore.getState().currentPosition;
      const { panDelta, tiltDelta } = calculateClickVector(
        clickX, clickY, canvasWidth, canvasHeight,
        settings.click_sensitivity, pos.zoom
      );
      moveRelative(panDelta, tiltDelta);
    },
    [settings.click_sensitivity, moveRelative]
  );

  /** Handle scroll on the video canvas to adjust zoom. */
  const handleVideoScroll = useCallback(
    (deltaY: number) => {
      if (!throttle()) return;
      const pos = useAppStore.getState().currentPosition;
      const zoomDelta = -deltaY * settings.scroll_sensitivity * 0.01;
      const newZoom = Math.max(0, Math.min(1, pos.zoom + zoomDelta));
      // Set position directly to avoid double-throttle from calling zoom()
      invoke("ptz_zoom", { zoom: newZoom }).catch((err: unknown) =>
        console.error("PTZ zoom failed:", err)
      );
      setCurrentPosition({ ...pos, zoom: newZoom });
    },
    [throttle, settings.scroll_sensitivity, setCurrentPosition]
  );

  // Read currentPosition from store for components that need it reactively
  const currentPosition = useAppStore((s) => s.currentPosition);

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
