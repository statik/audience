import { useAppStore } from "../store/app-store";
import { usePtzControl } from "../hooks/usePtzControl";
import { calculateOverlayRect } from "../utils/overlay-geometry";

interface PresetOverlayProps {
  canvasWidth: number;
  canvasHeight: number;
}

export function PresetOverlay({ canvasWidth, canvasHeight }: PresetOverlayProps) {
  const presets = useAppStore((s) => s.presets);
  const activePresetId = useAppStore((s) => s.activePresetId);
  const currentPosition = useAppStore((s) => s.currentPosition);
  const settings = useAppStore((s) => s.settings);
  const mode = useAppStore((s) => s.mode);
  const { recallPreset } = usePtzControl();
  const setActivePresetId = useAppStore((s) => s.setActivePresetId);

  const handlePresetClick = (presetId: string) => {
    if (mode === "operation") {
      setActivePresetId(presetId);
      recallPreset(presetId);
    } else {
      setActivePresetId(presetId);
    }
  };

  return (
    <div className="absolute inset-0 pointer-events-none">
      {presets.map((preset) => {
        const rect = calculateOverlayRect(
          preset,
          currentPosition.pan,
          currentPosition.tilt,
          currentPosition.zoom,
          settings.camera_fov_degrees,
          canvasWidth,
          canvasHeight
        );

        if (!rect.visible) return null;

        const isActive = activePresetId === preset.id;

        return (
          <div
            key={preset.id}
            className="absolute pointer-events-auto cursor-pointer transition-all duration-150"
            style={{
              left: `${rect.x}px`,
              top: `${rect.y}px`,
              width: `${rect.width}px`,
              height: `${rect.height}px`,
              backgroundColor: `${preset.color}${Math.round(settings.overlay_opacity * 255)
                .toString(16)
                .padStart(2, "0")}`,
              border: `2px solid ${preset.color}`,
              borderWidth: isActive ? "3px" : "2px",
              boxShadow: isActive
                ? `0 0 12px ${preset.color}80`
                : "none",
            }}
            onClick={(e) => {
              e.stopPropagation();
              handlePresetClick(preset.id);
            }}
          >
            <div
              className="absolute top-0 left-0 px-1.5 py-0.5 text-xs font-medium text-white truncate max-w-full"
              style={{ backgroundColor: preset.color }}
            >
              {preset.name}
            </div>
          </div>
        );
      })}
    </div>
  );
}
