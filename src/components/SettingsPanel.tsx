import { useAppStore } from "../store/app-store";
import { useSettings } from "../hooks/useSettings";
import { Modal } from "./Modal";

export function SettingsPanel() {
  const setSettingsOpen = useAppStore((s) => s.setSettingsOpen);
  const { settings, updateSetting } = useSettings();

  return (
    <Modal title="Settings" onClose={() => setSettingsOpen(false)}>
      <div className="space-y-6">
        {/* Click sensitivity */}
        <div>
          <label className="block text-sm font-medium text-[var(--color-text)] mb-1">
            Click Sensitivity
          </label>
          <p className="text-xs text-[var(--color-text-muted)] mb-2">
            How much the camera moves per click (default: 0.1)
          </p>
          <input
            type="range"
            min="0.01"
            max="0.5"
            step="0.01"
            value={settings.click_sensitivity}
            onChange={(e) =>
              updateSetting("click_sensitivity", parseFloat(e.target.value))
            }
            className="w-full"
          />
          <div className="text-xs text-[var(--color-text-muted)] mt-1">
            {settings.click_sensitivity.toFixed(2)}
          </div>
        </div>

        {/* Scroll sensitivity */}
        <div>
          <label className="block text-sm font-medium text-[var(--color-text)] mb-1">
            Scroll Sensitivity
          </label>
          <p className="text-xs text-[var(--color-text-muted)] mb-2">
            How fast zoom changes on scroll (default: 0.05)
          </p>
          <input
            type="range"
            min="0.01"
            max="0.2"
            step="0.01"
            value={settings.scroll_sensitivity}
            onChange={(e) =>
              updateSetting("scroll_sensitivity", parseFloat(e.target.value))
            }
            className="w-full"
          />
          <div className="text-xs text-[var(--color-text-muted)] mt-1">
            {settings.scroll_sensitivity.toFixed(2)}
          </div>
        </div>

        {/* Overlay opacity */}
        <div>
          <label className="block text-sm font-medium text-[var(--color-text)] mb-1">
            Overlay Opacity
          </label>
          <p className="text-xs text-[var(--color-text-muted)] mb-2">
            Transparency of preset overlays (10%-90%)
          </p>
          <input
            type="range"
            min="0.1"
            max="0.9"
            step="0.05"
            value={settings.overlay_opacity}
            onChange={(e) =>
              updateSetting("overlay_opacity", parseFloat(e.target.value))
            }
            className="w-full"
          />
          <div className="text-xs text-[var(--color-text-muted)] mt-1">
            {Math.round(settings.overlay_opacity * 100)}%
          </div>
        </div>

        {/* Camera FOV */}
        <div>
          <label className="block text-sm font-medium text-[var(--color-text)] mb-1">
            Camera FOV (degrees)
          </label>
          <p className="text-xs text-[var(--color-text-muted)] mb-2">
            Horizontal field of view at 1x zoom
          </p>
          <input
            type="number"
            min="10"
            max="180"
            value={settings.camera_fov_degrees}
            onChange={(e) =>
              updateSetting("camera_fov_degrees", parseFloat(e.target.value))
            }
            className="w-20 px-2 py-1 text-sm bg-[var(--color-bg-dark)] border border-[var(--color-border)] rounded text-[var(--color-text)] focus:outline-none focus:border-[var(--color-primary)]"
          />
        </div>
      </div>
    </Modal>
  );
}
