import { useAppStore } from "../store/app-store";
import type { AppMode } from "@shared/types";

export function useAppMode() {
  const mode = useAppStore((s) => s.mode);
  const setMode = useAppStore((s) => s.setMode);

  const isCalibration = mode === "calibration";
  const isOperation = mode === "operation";

  const toggleMode = () => {
    setMode(isCalibration ? "operation" : "calibration");
  };

  return {
    mode,
    setMode: (m: AppMode) => setMode(m),
    isCalibration,
    isOperation,
    toggleMode,
  };
}
