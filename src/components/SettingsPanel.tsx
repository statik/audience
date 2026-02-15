import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/app-store";

export function SettingsPanel() {
  const settings = useAppStore((s) => s.settings);
  const setSettings = useAppStore((s) => s.setSettings);
  const setSettingsOpen = useAppStore((s) => s.setSettingsOpen);

  const updateSetting = async (key: string, value: number) => {
    if (!Number.isFinite(value)) return;
    const previousSettings = settings;
    const newSettings = { ...settings, [key]: value };
    setSettings(newSettings);
    try {
      await invoke("update_settings", { [key]: value });
    } catch (err) {
      console.error("Failed to save settings:", err);
      setSettings(previousSettings);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="w-[600px] max-h-[80vh] bg-[var(--color-bg-panel)] rounded-xl border border-[var(--color-border)] shadow-2xl flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-[var(--color-border)]">
          <h2 className="text-lg font-semibold text-[var(--color-text)]">
            Settings
          </h2>
          <button
            className="text-[var(--color-text-muted)] hover:text-[var(--color-text)] text-lg transition-colors"
            onClick={() => setSettingsOpen(false)}
          >
            X
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6">
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
                  updateSetting(
                    "click_sensitivity",
                    parseFloat(e.target.value)
                  )
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
                  updateSetting(
                    "scroll_sensitivity",
                    parseFloat(e.target.value)
                  )
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
                  updateSetting(
                    "overlay_opacity",
                    parseFloat(e.target.value)
                  )
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
                  updateSetting(
                    "camera_fov_degrees",
                    parseFloat(e.target.value)
                  )
                }
                className="w-20 px-2 py-1 text-sm bg-[var(--color-bg-dark)] border border-[var(--color-border)] rounded text-[var(--color-text)] focus:outline-none focus:border-[var(--color-primary)]"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
