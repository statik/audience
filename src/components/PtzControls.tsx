import { useState } from "react";
import { usePtzControl } from "../hooks/usePtzControl";
import { JoystickPad } from "./JoystickPad";

type SpeedPreset = "fine" | "normal" | "fast";

const STEP_MAP: Record<SpeedPreset, number> = {
  fine: 0.01,
  normal: 0.05,
  fast: 0.15,
};

const BTN =
  "px-3 py-2 text-sm bg-[var(--color-bg-card)] text-[var(--color-text)] rounded hover:bg-[var(--color-primary)] transition-colors";

export function PtzControls() {
  const {
    currentPosition,
    moveRelative,
    zoom,
    home,
    focusContinuous,
    focusStop,
    setAutofocus,
  } = usePtzControl();

  const [speed, setSpeed] = useState<SpeedPreset>("normal");
  const [afEnabled, setAfEnabled] = useState(true);
  const [controlMode, setControlMode] = useState<"dpad" | "joystick">("dpad");

  const getStep = (e: React.MouseEvent) => {
    if (e.shiftKey) return STEP_MAP.fine;
    if (e.ctrlKey || e.metaKey) return STEP_MAP.fast;
    return STEP_MAP[speed];
  };

  const toggleAf = async () => {
    const next = !afEnabled;
    setAfEnabled(next);
    await setAutofocus(next);
  };

  return (
    <div className="p-3">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-sm font-semibold text-[var(--color-text)] uppercase tracking-wide">
          PTZ Controls
        </h3>
        <div className="flex gap-1">
          <button
            className={`px-2 py-0.5 text-xs rounded transition-colors ${
              controlMode === "dpad"
                ? "bg-[var(--color-primary)] text-white"
                : "bg-[var(--color-bg-card)] text-[var(--color-text-muted)]"
            }`}
            onClick={() => setControlMode("dpad")}
          >
            D-Pad
          </button>
          <button
            className={`px-2 py-0.5 text-xs rounded transition-colors ${
              controlMode === "joystick"
                ? "bg-[var(--color-primary)] text-white"
                : "bg-[var(--color-bg-card)] text-[var(--color-text-muted)]"
            }`}
            onClick={() => setControlMode("joystick")}
          >
            Stick
          </button>
        </div>
      </div>

      {controlMode === "dpad" && (
        <>
          {/* Speed selector */}
          <div className="flex items-center gap-1 mb-2 justify-center">
            {(["fine", "normal", "fast"] as const).map((s) => (
              <button
                key={s}
                className={`px-2 py-0.5 text-xs rounded transition-colors ${
                  speed === s
                    ? "bg-[var(--color-primary)] text-white"
                    : "bg-[var(--color-bg-card)] text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
                }`}
                onClick={() => setSpeed(s)}
              >
                {s.charAt(0).toUpperCase() + s.slice(1)}
              </button>
            ))}
          </div>

          {/* 8-way D-pad */}
          <div className="grid grid-cols-3 gap-1 w-fit mx-auto mb-4">
            <button className={BTN} onClick={(e) => moveRelative(-getStep(e), getStep(e))} title="Up-Left">
              &#8598;
            </button>
            <button className={BTN} onClick={(e) => moveRelative(0, getStep(e))} title="Tilt Up">
              &#8593;
            </button>
            <button className={BTN} onClick={(e) => moveRelative(getStep(e), getStep(e))} title="Up-Right">
              &#8599;
            </button>
            <button className={BTN} onClick={(e) => moveRelative(-getStep(e), 0)} title="Pan Left">
              &#8592;
            </button>
            <button
              className={`${BTN} text-xs`}
              onClick={() => home()}
              title="Home"
            >
              &#8962;
            </button>
            <button className={BTN} onClick={(e) => moveRelative(getStep(e), 0)} title="Pan Right">
              &#8594;
            </button>
            <button className={BTN} onClick={(e) => moveRelative(-getStep(e), -getStep(e))} title="Down-Left">
              &#8601;
            </button>
            <button className={BTN} onClick={(e) => moveRelative(0, -getStep(e))} title="Tilt Down">
              &#8595;
            </button>
            <button className={BTN} onClick={(e) => moveRelative(getStep(e), -getStep(e))} title="Down-Right">
              &#8600;
            </button>
          </div>
        </>
      )}

      {controlMode === "joystick" && (
        <div className="mb-4">
          <JoystickPad />
        </div>
      )}

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

      {/* Focus controls */}
      <div className="mt-3">
        <div className="flex items-center justify-between mb-1">
          <span className="text-xs text-[var(--color-text-muted)]">Focus</span>
        </div>
        <div className="flex items-center gap-1 justify-center">
          <button
            className="px-2 py-1 text-xs bg-[var(--color-bg-card)] text-[var(--color-text)] rounded hover:bg-[var(--color-primary)] transition-colors"
            onPointerDown={() => focusContinuous(-1)}
            onPointerUp={() => focusStop()}
            onPointerLeave={() => focusStop()}
          >
            Near
          </button>
          <button
            className={`px-2 py-1 text-xs rounded transition-colors ${
              afEnabled
                ? "bg-[var(--color-primary)] text-white"
                : "bg-[var(--color-bg-card)] text-[var(--color-text)]"
            }`}
            onClick={toggleAf}
          >
            AF
          </button>
          <button
            className="px-2 py-1 text-xs bg-[var(--color-bg-card)] text-[var(--color-text)] rounded hover:bg-[var(--color-primary)] transition-colors"
            onPointerDown={() => focusContinuous(1)}
            onPointerUp={() => focusStop()}
            onPointerLeave={() => focusStop()}
          >
            Far
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
