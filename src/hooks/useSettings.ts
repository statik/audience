import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/app-store";
import type { AppSettings } from "@shared/types";

export function useSettings() {
  const settings = useAppStore((s) => s.settings);
  const setSettings = useAppStore((s) => s.setSettings);

  const loadSettings = useCallback(async () => {
    try {
      const result = await invoke<AppSettings>("get_settings");
      setSettings(result);
    } catch (err) {
      console.error("Failed to load settings:", err);
    }
  }, [setSettings]);

  const updateSetting = useCallback(
    async (key: keyof AppSettings, value: number) => {
      if (!Number.isFinite(value)) return;
      const previous = useAppStore.getState().settings;
      // Optimistic update keeps sliders responsive while dragging.
      setSettings({ ...previous, [key]: value });
      try {
        // The backend clamps and persists, then returns the full config —
        // adopt it as the source of truth.
        const saved = await invoke<AppSettings>("update_settings", {
          [key]: value,
        });
        setSettings(saved);
      } catch (err) {
        console.error("Failed to save settings:", err);
        setSettings(previous);
      }
    },
    [setSettings]
  );

  return { settings, loadSettings, updateSetting };
}
