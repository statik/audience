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

  const home = useCallback(async () => {
    try {
      await invoke("ptz_home");
      setCurrentPosition({ pan: 0, tilt: 0, zoom: 0 });
    } catch (err) {
      console.error("PTZ home failed:", err);
    }
  }, [setCurrentPosition]);

  const continuousMove = useCallback(
    async (panSpeed: number, tiltSpeed: number) => {
      if (!throttle()) return;
      try {
        await invoke("ptz_continuous_move", { panSpeed, tiltSpeed });
      } catch (err) {
        console.error("PTZ continuous move failed:", err);
      }
    },
    [throttle]
  );

  const stop = useCallback(async () => {
    try {
      await invoke("ptz_stop");
    } catch (err) {
      console.error("PTZ stop failed:", err);
    }
  }, []);

  const focusContinuous = useCallback(
    async (speed: number) => {
      try {
        await invoke("ptz_focus", { speed });
      } catch (err) {
        console.error("PTZ focus failed:", err);
      }
    },
    []
  );

  const focusStop = useCallback(async () => {
    try {
      await invoke("ptz_focus_stop");
    } catch (err) {
      console.error("PTZ focus stop failed:", err);
    }
  }, []);

  const setAutofocus = useCallback(async (enabled: boolean) => {
    try {
      await invoke("ptz_set_autofocus", { enabled });
    } catch (err) {
      console.error("PTZ autofocus failed:", err);
    }
  }, []);

  const autofocusTrigger = useCallback(async () => {
    try {
      await invoke("ptz_autofocus_trigger");
    } catch (err) {
      console.error("PTZ autofocus trigger failed:", err);
    }
  }, []);

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

  const handleVideoScroll = useCallback(
    (deltaY: number) => {
      if (!throttle()) return;
      const pos = useAppStore.getState().currentPosition;
      const zoomDelta = -deltaY * settings.scroll_sensitivity * 0.01;
      const newZoom = Math.max(0, Math.min(1, pos.zoom + zoomDelta));
      invoke("ptz_zoom", { zoom: newZoom }).catch((err: unknown) =>
        console.error("PTZ zoom failed:", err)
      );
      setCurrentPosition({ ...pos, zoom: newZoom });
    },
    [throttle, settings.scroll_sensitivity, setCurrentPosition]
  );

  const currentPosition = useAppStore((s) => s.currentPosition);

  return {
    currentPosition,
    moveRelative,
    moveAbsolute,
    zoom,
    recallPreset,
    home,
    continuousMove,
    stop,
    focusContinuous,
    focusStop,
    setAutofocus,
    autofocusTrigger,
    handleVideoClick,
    handleVideoScroll,
  };
}
