import { useAppStore } from "../store/app-store";

export function StatusBar() {
  const isConnected = useAppStore((s) => s.isConnected);
  const connectionLabel = useAppStore((s) => s.connectionLabel);
  const fps = useAppStore((s) => s.fps);
  const mode = useAppStore((s) => s.mode);

  return (
    <div className="flex items-center justify-between px-4 py-1 bg-[var(--color-bg-panel)] border-t border-[var(--color-border)] text-xs text-[var(--color-text-muted)] no-select">
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-1.5">
          <div
            className={`w-2 h-2 rounded-full ${
              isConnected ? "bg-[var(--color-success)]" : "bg-[var(--color-danger)]"
            }`}
          />
          <span>{connectionLabel}</span>
        </div>
      </div>

      <div className="flex items-center gap-4">
        <span>
          Mode: {mode === "calibration" ? "Calibration" : "Operation"}
        </span>
        {isConnected && <span>FPS: {fps}</span>}
      </div>
    </div>
  );
}
