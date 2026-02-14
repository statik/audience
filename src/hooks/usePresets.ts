import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/app-store";
import type { Preset } from "@shared/types";

const PRESET_COLORS = [
  "#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#8b5cf6",
  "#ec4899", "#06b6d4", "#f97316", "#14b8a6", "#6366f1",
  "#84cc16", "#e11d48", "#0ea5e9", "#a855f7", "#d946ef",
];

export function usePresets() {
  const presets = useAppStore((s) => s.presets);
  const setPresets = useAppStore((s) => s.setPresets);
  const activePresetId = useAppStore((s) => s.activePresetId);
  const setActivePresetId = useAppStore((s) => s.setActivePresetId);

  const loadPresets = useCallback(async () => {
    try {
      const result = await invoke<Preset[]>("get_all_presets");
      setPresets(result);
    } catch (err) {
      console.error("Failed to load presets:", err);
    }
  }, [setPresets]);

  const createPreset = useCallback(
    async (name: string, pan: number, tilt: number, zoom: number) => {
      const currentPresets = useAppStore.getState().presets;
      const color = PRESET_COLORS[currentPresets.length % PRESET_COLORS.length];
      try {
        const preset = await invoke<Preset>("create_preset", {
          name,
          pan,
          tilt,
          zoom,
          color,
        });
        const latest = useAppStore.getState().presets;
        setPresets([...latest, preset]);
        return preset;
      } catch (err) {
        console.error("Failed to create preset:", err);
        throw err;
      }
    },
    [setPresets]
  );

  const updatePreset = useCallback(
    async (preset: Preset) => {
      try {
        const updated = await invoke<Preset>("update_preset", { preset });
        const latest = useAppStore.getState().presets;
        setPresets(latest.map((p) => (p.id === updated.id ? updated : p)));
        return updated;
      } catch (err) {
        console.error("Failed to update preset:", err);
        throw err;
      }
    },
    [setPresets]
  );

  const deletePreset = useCallback(
    async (presetId: string) => {
      try {
        await invoke("delete_preset", { presetId });
        const latest = useAppStore.getState().presets;
        setPresets(latest.filter((p) => p.id !== presetId));
        if (useAppStore.getState().activePresetId === presetId) {
          setActivePresetId(null);
        }
      } catch (err) {
        console.error("Failed to delete preset:", err);
        throw err;
      }
    },
    [setPresets, setActivePresetId]
  );

  return {
    presets,
    activePresetId,
    setActivePresetId,
    loadPresets,
    createPreset,
    updatePreset,
    deletePreset,
  };
}
