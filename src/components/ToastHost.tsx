import { useEffect } from "react";
import { useAppStore } from "../store/app-store";
import type { Toast, ToastKind } from "@shared/types";

const DISMISS_MS: Record<ToastKind, number> = {
  error: 8000,
  warning: 5000,
  success: 5000,
  info: 5000,
};

const ACCENT: Record<ToastKind, string> = {
  error: "border-l-[var(--color-danger)]",
  warning: "border-l-[var(--color-warning)]",
  success: "border-l-[var(--color-success)]",
  info: "border-l-[var(--color-primary)]",
};

function ToastItem({ toast }: { toast: Toast }) {
  const dismissToast = useAppStore((s) => s.dismissToast);

  useEffect(() => {
    const timer = setTimeout(
      () => dismissToast(toast.id),
      DISMISS_MS[toast.kind]
    );
    return () => clearTimeout(timer);
  }, [toast.id, toast.kind, dismissToast]);

  return (
    <div
      role={toast.kind === "error" ? "alert" : "status"}
      className={`flex items-start gap-2 w-80 px-3 py-2 rounded border border-[var(--color-border)] border-l-4 ${ACCENT[toast.kind]} bg-[var(--color-bg-panel)] text-sm text-[var(--color-text)] shadow-lg`}
    >
      <span className="flex-1 break-words">{toast.message}</span>
      <button
        className="text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
        onClick={() => dismissToast(toast.id)}
        aria-label="Dismiss notification"
      >
        &#x2715;
      </button>
    </div>
  );
}

export function ToastHost() {
  const toasts = useAppStore((s) => s.toasts);

  if (toasts.length === 0) return null;

  return (
    <div
      className="fixed bottom-8 left-4 z-50 flex flex-col gap-2"
      aria-live="polite"
    >
      {toasts.map((toast) => (
        <ToastItem key={toast.id} toast={toast} />
      ))}
    </div>
  );
}
