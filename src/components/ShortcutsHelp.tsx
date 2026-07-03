import { useAppStore } from "../store/app-store";
import { Modal } from "./Modal";

// Keep in sync with useKeyboardShortcuts.ts
const SHORTCUTS: Array<[keys: string, action: string]> = [
  ["Arrow keys", "Pan / tilt"],
  ["Shift + Arrow", "Pan / tilt (fine)"],
  ["Ctrl + Arrow", "Pan / tilt (fast)"],
  ["+ / −", "Zoom in / out"],
  ["Space", "Stop all movement"],
  ["H", "Home position"],
  ["1–9", "Recall preset by number"],
  ["?", "Show this help"],
];

export function ShortcutsHelp() {
  const setShortcutsHelpOpen = useAppStore((s) => s.setShortcutsHelpOpen);

  return (
    <Modal
      title="Keyboard Shortcuts"
      onClose={() => setShortcutsHelpOpen(false)}
    >
      <table className="w-full text-sm">
        <tbody>
          {SHORTCUTS.map(([keys, action]) => (
            <tr
              key={keys}
              className="border-b border-[var(--color-border)] last:border-b-0"
            >
              <td className="py-2 pr-6 whitespace-nowrap">
                <kbd className="px-1.5 py-0.5 text-xs rounded bg-[var(--color-bg-card)] border border-[var(--color-border)] text-[var(--color-text)]">
                  {keys}
                </kbd>
              </td>
              <td className="py-2 text-[var(--color-text)]">{action}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <p className="mt-4 text-xs text-[var(--color-text-muted)]">
        Shortcuts are inactive while typing in a text field.
      </p>
    </Modal>
  );
}
