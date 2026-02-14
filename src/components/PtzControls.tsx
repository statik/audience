import { usePtzControl } from "../hooks/usePtzControl";

export function PtzControls() {
  const { currentPosition, moveRelative, zoom } = usePtzControl();

  const step = 0.05;

  return (
    <div className="p-3">
      <h3 className="text-sm font-semibold text-[var(--color-text)] uppercase tracking-wide mb-3">
        PTZ Controls
      </h3>

      {/* Pan/Tilt arrows */}
      <div className="grid grid-cols-3 gap-1 w-fit mx-auto mb-4">
        <div />
        <button
          className="px-3 py-2 text-sm bg-[var(--color-bg-card)] text-[var(--color-text)] rounded hover:bg-[var(--color-primary)] transition-colors"
          onClick={() => moveRelative(0, step)}
          title="Tilt Up"
        >
          Up
        </button>
        <div />
        <button
          className="px-3 py-2 text-sm bg-[var(--color-bg-card)] text-[var(--color-text)] rounded hover:bg-[var(--color-primary)] transition-colors"
          onClick={() => moveRelative(-step, 0)}
          title="Pan Left"
        >
          Left
        </button>
        <button
          className="px-3 py-2 text-sm bg-[var(--color-bg-card)] text-[var(--color-text)] rounded hover:bg-[var(--color-primary)] transition-colors text-xs"
          onClick={() => moveRelative(0, 0)}
          title="Stop"
        >
          Stop
        </button>
        <button
          className="px-3 py-2 text-sm bg-[var(--color-bg-card)] text-[var(--color-text)] rounded hover:bg-[var(--color-primary)] transition-colors"
          onClick={() => moveRelative(step, 0)}
          title="Pan Right"
        >
          Right
        </button>
        <div />
        <button
          className="px-3 py-2 text-sm bg-[var(--color-bg-card)] text-[var(--color-text)] rounded hover:bg-[var(--color-primary)] transition-colors"
          onClick={() => moveRelative(0, -step)}
          title="Tilt Down"
        >
          Down
        </button>
        <div />
      </div>

      {/* Zoom slider */}
      <div>
        <div className="flex items-center justify-between mb-1">
          <span className="text-xs text-[var(--color-text-muted)]">Zoom</span>
          <span className="text-xs text-[var(--color-text-muted)]">
            {Math.round(currentPosition.zoom * 100)}%
          </span>
        </div>
        <div className="flex items-center gap-2">
          <button
            className="px-2 py-1 text-sm bg-[var(--color-bg-card)] text-[var(--color-text)] rounded hover:bg-[var(--color-primary)] transition-colors"
            onClick={() =>
              zoom(Math.max(0, currentPosition.zoom - 0.05))
            }
          >
            -
          </button>
          <input
            type="range"
            min="0"
            max="1"
            step="0.01"
            value={currentPosition.zoom}
            onChange={(e) => zoom(parseFloat(e.target.value))}
            className="flex-1"
          />
          <button
            className="px-2 py-1 text-sm bg-[var(--color-bg-card)] text-[var(--color-text)] rounded hover:bg-[var(--color-primary)] transition-colors"
            onClick={() =>
              zoom(Math.min(1, currentPosition.zoom + 0.05))
            }
          >
            +
          </button>
        </div>
      </div>

      {/* Position readout */}
      <div className="mt-3 text-xs text-[var(--color-text-muted)] space-y-0.5">
        <div>Pan: {currentPosition.pan.toFixed(3)}</div>
        <div>Tilt: {currentPosition.tilt.toFixed(3)}</div>
        <div>Zoom: {currentPosition.zoom.toFixed(3)}</div>
      </div>
    </div>
  );
}
