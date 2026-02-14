import { useState } from "react";
import { usePresets } from "../hooks/usePresets";
import { useAppStore } from "../store/app-store";
import { PresetEditor } from "./PresetEditor";

export function PresetList() {
  const { presets, activePresetId, setActivePresetId, createPreset, deletePreset } =
    usePresets();
  const mode = useAppStore((s) => s.mode);
  const currentPosition = useAppStore((s) => s.currentPosition);
  const [showEditor, setShowEditor] = useState(false);
  const [deleteConfirmId, setDeleteConfirmId] = useState<string | null>(null);
  const { recallPreset } = require("../hooks/usePtzControl").usePtzControl();

  const handlePresetClick = (presetId: string) => {
    setActivePresetId(presetId);
    if (mode === "operation") {
      recallPreset(presetId);
    }
  };

  const handleAddPreset = () => {
    setShowEditor(true);
  };

  const handleSaveNewPreset = async (name: string) => {
    await createPreset(
      name,
      currentPosition.pan,
      currentPosition.tilt,
      currentPosition.zoom
    );
    setShowEditor(false);
  };

  const handleDeletePreset = async (presetId: string) => {
    await deletePreset(presetId);
    setDeleteConfirmId(null);
  };

  return (
    <div className="p-3">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-sm font-semibold text-[var(--color-text)] uppercase tracking-wide">
          Presets
        </h3>
        <span className="text-xs text-[var(--color-text-muted)]">
          {presets.length}
        </span>
      </div>

      <div className="space-y-1">
        {presets.map((preset) => (
          <div
            key={preset.id}
            className={`group flex items-center gap-2 px-2 py-1.5 rounded cursor-pointer transition-colors ${
              activePresetId === preset.id
                ? "bg-[var(--color-bg-card)] ring-1 ring-[var(--color-primary)]"
                : "hover:bg-[var(--color-bg-card)]"
            }`}
            onClick={() => handlePresetClick(preset.id)}
          >
            <div
              className="w-3 h-3 rounded-sm shrink-0"
              style={{ backgroundColor: preset.color }}
            />
            <span className="text-sm text-[var(--color-text)] truncate flex-1">
              {preset.name}
            </span>
            {mode === "calibration" && (
              <button
                className="opacity-0 group-hover:opacity-100 text-xs text-[var(--color-danger)] hover:text-red-400 transition-opacity"
                onClick={(e) => {
                  e.stopPropagation();
                  if (deleteConfirmId === preset.id) {
                    handleDeletePreset(preset.id);
                  } else {
                    setDeleteConfirmId(preset.id);
                  }
                }}
              >
                {deleteConfirmId === preset.id ? "Confirm?" : "Delete"}
              </button>
            )}
          </div>
        ))}
      </div>

      {mode === "calibration" && (
        <button
          className="w-full mt-3 px-3 py-2 text-sm rounded border border-dashed border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-primary)] transition-colors"
          onClick={handleAddPreset}
        >
          + Add Preset
        </button>
      )}

      {showEditor && (
        <PresetEditor
          onSave={handleSaveNewPreset}
          onCancel={() => setShowEditor(false)}
        />
      )}
    </div>
  );
}
