import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/app-store";
import type { CameraEndpoint, ProtocolConfig } from "@shared/types";

export function useEndpoints() {
  const endpoints = useAppStore((s) => s.endpoints);
  const setEndpoints = useAppStore((s) => s.setEndpoints);
  const activeEndpointId = useAppStore((s) => s.activeEndpointId);
  const setActiveEndpointId = useAppStore((s) => s.setActiveEndpointId);

  const loadEndpoints = useCallback(async () => {
    try {
      const result = await invoke<CameraEndpoint[]>("get_endpoints");
      setEndpoints(result);
    } catch (err) {
      console.error("Failed to load endpoints:", err);
    }
  }, [setEndpoints]);

  const createEndpoint = useCallback(
    async (endpoint: CameraEndpoint) => {
      try {
        const created = await invoke<CameraEndpoint>("create_endpoint", {
          endpoint,
        });
        const latest = useAppStore.getState().endpoints;
        setEndpoints([...latest, created]);
        return created;
      } catch (err) {
        console.error("Failed to create endpoint:", err);
        throw err;
      }
    },
    [setEndpoints]
  );

  const updateEndpoint = useCallback(
    async (endpoint: CameraEndpoint) => {
      try {
        const updated = await invoke<CameraEndpoint>("update_endpoint", {
          endpoint,
        });
        const latest = useAppStore.getState().endpoints;
        setEndpoints(latest.map((e) => (e.id === updated.id ? updated : e)));
        return updated;
      } catch (err) {
        console.error("Failed to update endpoint:", err);
        throw err;
      }
    },
    [setEndpoints]
  );

  const deleteEndpoint = useCallback(
    async (endpointId: string) => {
      try {
        await invoke("delete_endpoint", { endpointId });
        const latest = useAppStore.getState().endpoints;
        setEndpoints(latest.filter((e) => e.id !== endpointId));
        if (useAppStore.getState().activeEndpointId === endpointId) {
          setActiveEndpointId(null);
        }
      } catch (err) {
        console.error("Failed to delete endpoint:", err);
        throw err;
      }
    },
    [setEndpoints, setActiveEndpointId]
  );

  const setActiveEndpoint = useCallback(
    async (endpointId: string) => {
      try {
        await invoke("set_active_endpoint", { endpointId });
        setActiveEndpointId(endpointId);
      } catch (err) {
        console.error("Failed to set active endpoint:", err);
        throw err;
      }
    },
    [setActiveEndpointId]
  );

  const testConnection = useCallback(
    async (config: ProtocolConfig): Promise<string> => {
      return invoke<string>("test_endpoint_connection", { config });
    },
    []
  );

  return {
    endpoints,
    activeEndpointId,
    loadEndpoints,
    createEndpoint,
    updateEndpoint,
    deleteEndpoint,
    setActiveEndpoint,
    testConnection,
  };
}
