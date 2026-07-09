import { useAppStore } from "../store/app-store";
import type { PtzStatus } from "../store/app-store";

const PTZ_DOT: Record<PtzStatus, string> = {
  idle: "bg-[var(--color-text-muted)]",
  ok: "bg-[var(--color-success)]",
  error: "bg-[var(--color-danger)]",
};

const PTZ_TEXT: Record<PtzStatus, string> = {
  idle: "Idle",
  ok: "OK",
  error: "Error",
};

export function StatusBar() {
  const isConnected = useAppStore((s) => s.isConnected);
  const connectionLabel = useAppStore((s) => s.connectionLabel);
  const fps = useAppStore((s) => s.fps);
  const mode = useAppStore((s) => s.mode);
  const ptzStatus = useAppStore((s) => s.ptzStatus);
  const endpoints = useAppStore((s) => s.endpoints);
  const activeEndpointId = useAppStore((s) => s.activeEndpointId);
  const setShortcutsHelpOpen = useAppStore((s) => s.setShortcutsHelpOpen);

  const activeEndpoint = endpoints.find((e) => e.id === activeEndpointId);

  return (
    <div className="flex items-center justify-between px-4 py-1 bg-[var(--color-bg-panel)] border-t border-[var(--color-border)] text-xs text-[var(--color-text-muted)] no-select">
      <div
        className="flex items-center gap-4"
        role="status"
        aria-live="polite"
      >
        <div className="flex items-center gap-1.5">
          <div
            aria-hidden="true"
            className={`w-2 h-2 rounded-full ${
              isConnected ? "bg-[var(--color-success)]" : "bg-[var(--color-danger)]"
            }`}
          />
          <span>Video: {connectionLabel}</span>
        </div>
        <div className="flex items-center gap-1.5">
          <div
            aria-hidden="true"
            className={`w-2 h-2 rounded-full ${PTZ_DOT[ptzStatus]}`}
          />
          <span>
            PTZ:{" "}
            {activeEndpoint
              ? `${activeEndpoint.name} — ${PTZ_TEXT[ptzStatus]}`
              : "No endpoint active"}
          </span>
        </div>
      </div>

      <div className="flex items-center gap-4">
        <span>
          Mode: {mode === "calibration" ? "Calibration" : "Operation"}
        </span>
        {isConnected && <span>FPS: {fps}</span>}
        <button
          className="hover:text-[var(--color-text)] transition-colors"
          onClick={() => setShortcutsHelpOpen(true)}
          title="Keyboard shortcuts"
        >
          Shortcuts (?)
        </button>
      </div>
    </div>
  );
}
