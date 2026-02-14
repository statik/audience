import { useAppMode } from "../hooks/useAppMode";
import { useAppStore } from "../store/app-store";
import { SourcePicker } from "./SourcePicker";

export function Toolbar() {
  const { mode, setMode } = useAppMode();
  const toggleSidebar = useAppStore((s) => s.toggleSidebar);
  const sidebarCollapsed = useAppStore((s) => s.sidebarCollapsed);
  const setSettingsOpen = useAppStore((s) => s.setSettingsOpen);

  return (
    <div className="flex items-center gap-3 px-4 py-2 bg-[var(--color-bg-panel)] border-b border-[var(--color-border)] no-select">
      {/* Mode toggle */}
      <div className="flex rounded-lg overflow-hidden border border-[var(--color-border)]">
        <button
          className={`px-3 py-1.5 text-sm font-medium transition-colors ${
            mode === "calibration"
              ? "bg-[var(--color-primary)] text-white"
              : "bg-transparent text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
          }`}
          onClick={() => setMode("calibration")}
        >
          Calibration
        </button>
        <button
          className={`px-3 py-1.5 text-sm font-medium transition-colors ${
            mode === "operation"
              ? "bg-[var(--color-primary)] text-white"
              : "bg-transparent text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
          }`}
          onClick={() => setMode("operation")}
        >
          Operation
        </button>
      </div>

      {/* Source picker */}
      <SourcePicker />

      {/* Spacer */}
      <div className="flex-1" />

      {/* Sidebar toggle */}
      <button
        className="px-2 py-1.5 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
        onClick={toggleSidebar}
        title={sidebarCollapsed ? "Show sidebar" : "Hide sidebar"}
      >
        {sidebarCollapsed ? "Show Panel" : "Hide Panel"}
      </button>

      {/* Settings */}
      <button
        className="px-2 py-1.5 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
        onClick={() => setSettingsOpen(true)}
        title="Settings"
      >
        Settings
      </button>
    </div>
  );
}
