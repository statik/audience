import { useAppStore } from "../store/app-store";

/**
 * Toast helpers callable from plain async callbacks (no hook plumbing).
 * Tauri command errors arrive as plain strings; anything else is stringified.
 */
function toDetail(err: unknown): string {
  if (err === undefined || err === null) return "";
  if (typeof err === "string") return `: ${err}`;
  if (err instanceof Error) return `: ${err.message}`;
  return `: ${String(err)}`;
}

export function notifyError(message: string, err?: unknown, key?: string) {
  useAppStore.getState().pushToast({
    key,
    kind: "error",
    message: `${message}${toDetail(err)}`,
  });
}

export function notifyWarning(message: string, key?: string) {
  useAppStore.getState().pushToast({ key, kind: "warning", message });
}

export function notifySuccess(message: string, key?: string) {
  useAppStore.getState().pushToast({ key, kind: "success", message });
}
