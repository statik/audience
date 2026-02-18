import { useEffect } from "react";
import { Toolbar } from "./components/Toolbar";
import { VideoCanvas } from "./components/VideoCanvas";
import { PresetList } from "./components/PresetList";
import { PtzControls } from "./components/PtzControls";
import { EndpointManager } from "./components/EndpointManager";
import { StatusBar } from "./components/StatusBar";
import { SettingsPanel } from "./components/SettingsPanel";
import { useAppStore } from "./store/app-store";
import { usePresets } from "./hooks/usePresets";
import { useEndpoints } from "./hooks/useEndpoints";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";

export default function App() {
  const sidebarCollapsed = useAppStore((s) => s.sidebarCollapsed);
  const settingsOpen = useAppStore((s) => s.settingsOpen);
  const { loadPresets } = usePresets();
  const { loadEndpoints } = useEndpoints();

  useKeyboardShortcuts();

  useEffect(() => {
    loadPresets();
    loadEndpoints();
  }, [loadPresets, loadEndpoints]);

  return (
    <div className="flex flex-col h-screen w-screen bg-[var(--color-bg-dark)]">
      <Toolbar />

      <div className="flex flex-1 min-h-0">
        {/* Main video area */}
        <div className="flex-1 relative min-w-0">
          <VideoCanvas />
        </div>

        {/* Sidebar */}
        {!sidebarCollapsed && (
          <div className="w-72 flex flex-col border-l border-[var(--color-border)] bg-[var(--color-bg-panel)]">
            <div className="flex-1 overflow-y-auto">
              <PresetList />
              <div className="border-t border-[var(--color-border)] p-3">
                <h3 className="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider mb-2">
                  Endpoints
                </h3>
                <EndpointManager />
              </div>
            </div>
            <div className="border-t border-[var(--color-border)]">
              <PtzControls />
            </div>
          </div>
        )}
      </div>

      <StatusBar />

      {settingsOpen && <SettingsPanel />}
    </div>
  );
}
