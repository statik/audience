import { useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/app-store";
import { notifyError } from "../utils/notify";
import { calculateClickVector } from "../utils/ptz-math";
import type { PtzPosition } from "@shared/types";

const MIN_COMMAND_INTERVAL_MS = 100;

/**
 * All movement failures share this toast key so rapid-fire commands (held
 * arrow key, joystick drag) coalesce into one toast instead of stacking.
 */
const PTZ_TOAST_KEY = "ptz";

function reportPtzOk() {
  useAppStore.getState().setPtzStatus("ok");
}

function reportPtzError(context: string, err: unknown) {
  console.error(`${context}:`, err);
  useAppStore.getState().setPtzStatus("error");
  notifyError(context, err, PTZ_TOAST_KEY);
}

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
        reportPtzOk();
      } catch (err) {
        reportPtzError("PTZ move failed", err);
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
        reportPtzOk();
      } catch (err) {
        reportPtzError("PTZ move failed", err);
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
        reportPtzOk();
      } catch (err) {
        reportPtzError("PTZ zoom failed", err);
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
        reportPtzOk();
      } catch (err) {
        reportPtzError("Preset recall failed", err);
      }
    },
    [setCurrentPosition]
  );

  const home = useCallback(async () => {
    try {
      await invoke("ptz_home");
      setCurrentPosition({ pan: 0, tilt: 0, zoom: 0 });
      reportPtzOk();
    } catch (err) {
      reportPtzError("PTZ home failed", err);
    }
  }, [setCurrentPosition]);

  const continuousMove = useCallback(
    async (panSpeed: number, tiltSpeed: number) => {
      if (!throttle()) return;
      try {
        await invoke("ptz_continuous_move", { panSpeed, tiltSpeed });
        reportPtzOk();
      } catch (err) {
        reportPtzError("PTZ continuous move failed", err);
      }
    },
    [throttle]
  );

  const stop = useCallback(async () => {
    try {
      await invoke("ptz_stop");
      reportPtzOk();
    } catch (err) {
      reportPtzError("PTZ stop failed", err);
    }
  }, []);

  const focusContinuous = useCallback(async (speed: number) => {
    try {
      await invoke("ptz_focus", { speed });
      reportPtzOk();
    } catch (err) {
      reportPtzError("PTZ focus failed", err);
    }
  }, []);

  const focusStop = useCallback(async () => {
    try {
      await invoke("ptz_focus_stop");
      reportPtzOk();
    } catch (err) {
      reportPtzError("PTZ focus stop failed", err);
    }
  }, []);

  const setAutofocus = useCallback(async (enabled: boolean) => {
    try {
      await invoke("ptz_set_autofocus", { enabled });
      reportPtzOk();
    } catch (err) {
      reportPtzError("PTZ autofocus failed", err);
    }
  }, []);

  const autofocusTrigger = useCallback(async () => {
    try {
      await invoke("ptz_autofocus_trigger");
      reportPtzOk();
    } catch (err) {
      reportPtzError("PTZ autofocus trigger failed", err);
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
      invoke("ptz_zoom", { zoom: newZoom })
        .then(() => {
          const latest = useAppStore.getState().currentPosition;
          setCurrentPosition({ ...latest, zoom: newZoom });
          reportPtzOk();
        })
        .catch((err: unknown) => {
          reportPtzError("PTZ zoom failed", err);
        });
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
