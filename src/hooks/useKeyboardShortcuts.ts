import { useEffect } from "react";
import { usePtzControl } from "./usePtzControl";
import { useAppStore } from "../store/app-store";

export function useKeyboardShortcuts() {
  const {
    moveRelative,
    zoom,
    home,
    stop,
    recallPreset,
  } = usePtzControl();

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const target = e.target;
      if (
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement ||
        target instanceof HTMLSelectElement
      ) {
        return;
      }

      const step = e.shiftKey ? 0.01 : e.ctrlKey || e.metaKey ? 0.15 : 0.05;

      switch (e.key) {
        case "ArrowUp":
          e.preventDefault();
          moveRelative(0, step);
          break;
        case "ArrowDown":
          e.preventDefault();
          moveRelative(0, -step);
          break;
        case "ArrowLeft":
          e.preventDefault();
          moveRelative(-step, 0);
          break;
        case "ArrowRight":
          e.preventDefault();
          moveRelative(step, 0);
          break;
        case "+":
        case "=": {
          const pos = useAppStore.getState().currentPosition;
          zoom(Math.min(1, pos.zoom + 0.05));
          break;
        }
        case "-": {
          const pos = useAppStore.getState().currentPosition;
          zoom(Math.max(0, pos.zoom - 0.05));
          break;
        }
        case " ":
          e.preventDefault();
          stop();
          break;
        case "h":
        case "H":
          home();
          break;
        default:
          if (/^[1-9]$/.test(e.key)) {
            const idx = parseInt(e.key) - 1;
            const presets = useAppStore.getState().presets;
            if (presets[idx]) {
              recallPreset(presets[idx].id);
            }
          }
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [moveRelative, zoom, home, stop, recallPreset]);
}
