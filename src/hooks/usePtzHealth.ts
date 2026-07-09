import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { useAppStore } from "../store/app-store";
import { notifyError } from "../utils/notify";

interface PtzHealthEvent {
  status: "ok" | "error";
  message?: string | null;
}

/**
 * Subscribes to the backend's ptz-health heartbeat so a camera that drops
 * off the network is surfaced without waiting for the next command.
 */
export function usePtzHealth() {
  useEffect(() => {
    const unlisten = listen<PtzHealthEvent>("ptz-health", (event) => {
      const store = useAppStore.getState();
      // Stale probe from an endpoint deactivated mid-flight
      if (!store.activeEndpointId) return;

      const wasError = store.ptzStatus === "error";
      store.setPtzStatus(event.payload.status);
      if (event.payload.status === "error" && !wasError) {
        notifyError(
          "Camera connection lost",
          event.payload.message ?? undefined,
          "ptz-health"
        );
      }
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, []);
}
