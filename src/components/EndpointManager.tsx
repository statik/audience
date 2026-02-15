import { useState } from "react";
import { useEndpoints } from "../hooks/useEndpoints";
import type { CameraEndpoint, ProtocolConfig, PtzProtocol } from "@shared/types";

function generateId(): string {
  return crypto.randomUUID?.() ?? Math.random().toString(36).slice(2);
}

export function EndpointManager() {
  const {
    endpoints,
    activeEndpointId,
    createEndpoint,
    updateEndpoint,
    deleteEndpoint,
    setActiveEndpoint,
    clearActiveEndpoint,
    testConnection,
  } = useEndpoints();

  const [editingEndpoint, setEditingEndpoint] = useState<CameraEndpoint | null>(
    null
  );
  const [isNew, setIsNew] = useState(false);
  const [testResult, setTestResult] = useState<string | null>(null);
  const [testing, setTesting] = useState(false);

  const startNewEndpoint = () => {
    setEditingEndpoint({
      id: generateId(),
      name: "",
      protocol: "Visca",
      config: { type: "Visca", host: "192.168.1.100", port: 52381 },
    });
    setIsNew(true);
    setTestResult(null);
  };

  const getConfigForProtocol = (protocol: PtzProtocol): ProtocolConfig => {
    switch (protocol) {
      case "Ndi":
        return { type: "Ndi" };
      case "Visca":
        return { type: "Visca", host: "192.168.1.100", port: 52381 };
      case "PanasonicAw":
        return { type: "PanasonicAw", host: "192.168.1.100", port: 80 };
      case "BirdDogRest":
        return { type: "BirdDogRest", host: "192.168.1.100", port: 8080 };
      case "Simulated":
        return { type: "Simulated" };
    }
  };

  const handleProtocolChange = (protocol: PtzProtocol) => {
    if (!editingEndpoint) return;
    setEditingEndpoint({
      ...editingEndpoint,
      protocol,
      config: getConfigForProtocol(protocol),
    });
  };

  const handleSave = async () => {
    if (!editingEndpoint || !editingEndpoint.name.trim()) return;
    try {
      if (isNew) {
        await createEndpoint(editingEndpoint);
      } else {
        await updateEndpoint(editingEndpoint);
      }
      setEditingEndpoint(null);
      setIsNew(false);
    } catch (err) {
      setTestResult(`Error saving: ${err}`);
    }
  };

  const handleTest = async () => {
    if (!editingEndpoint) return;
    setTesting(true);
    setTestResult(null);
    try {
      const result = await testConnection(editingEndpoint.config);
      setTestResult(result);
    } catch (err) {
      setTestResult(`Error: ${err}`);
    } finally {
      setTesting(false);
    }
  };

  const handleDelete = async (id: string) => {
    if (confirm("Delete this endpoint?")) {
      await deleteEndpoint(id);
    }
  };

  return (
    <div>
      {/* Endpoint list */}
      <div className="space-y-2 mb-4">
        {endpoints.map((ep) => (
          <div
            key={ep.id}
            className={`flex items-center gap-3 p-3 rounded-lg border transition-colors ${
              activeEndpointId === ep.id
                ? "border-[var(--color-primary)] bg-[var(--color-bg-card)]"
                : "border-[var(--color-border)] hover:bg-[var(--color-bg-card)]"
            }`}
          >
            <div className="flex-1">
              <div className="text-sm font-medium text-[var(--color-text)]">
                {ep.name}
              </div>
              <div className="text-xs text-[var(--color-text-muted)]">
                {ep.protocol}
                {ep.config.type !== "Ndi" && "host" in ep.config
                  ? ` - ${ep.config.host}:${ep.config.port}`
                  : ""}
              </div>
            </div>
            <button
              className="text-xs text-[var(--color-primary)] hover:text-[var(--color-primary-hover)]"
              onClick={() =>
                activeEndpointId === ep.id
                  ? clearActiveEndpoint()
                  : setActiveEndpoint(ep.id)
              }
            >
              {activeEndpointId === ep.id ? "Deactivate" : "Activate"}
            </button>
            <button
              className="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
              onClick={() => {
                setEditingEndpoint(ep);
                setIsNew(false);
                setTestResult(null);
              }}
            >
              Edit
            </button>
            <button
              className="text-xs text-[var(--color-danger)] hover:text-red-400"
              onClick={() => handleDelete(ep.id)}
            >
              Delete
            </button>
          </div>
        ))}
        {endpoints.length === 0 && (
          <p className="text-sm text-[var(--color-text-muted)]">
            No camera endpoints configured.
          </p>
        )}
      </div>

      <button
        className="w-full px-3 py-2 text-sm rounded border border-dashed border-[var(--color-border)] text-[var(--color-text-muted)] hover:text-[var(--color-text)] hover:border-[var(--color-primary)] transition-colors"
        onClick={startNewEndpoint}
      >
        + Add Endpoint
      </button>

      {/* Endpoint editor */}
      {editingEndpoint && (
        <div className="mt-4 p-4 bg-[var(--color-bg-card)] rounded-lg border border-[var(--color-border)] space-y-3">
          <div>
            <label className="block text-xs text-[var(--color-text-muted)] mb-1">
              Name
            </label>
            <input
              type="text"
              value={editingEndpoint.name}
              onChange={(e) =>
                setEditingEndpoint({ ...editingEndpoint, name: e.target.value })
              }
              placeholder="e.g., Stage Left Sony"
              className="w-full px-2 py-1.5 text-sm bg-[var(--color-bg-dark)] border border-[var(--color-border)] rounded text-[var(--color-text)] focus:outline-none focus:border-[var(--color-primary)]"
            />
          </div>

          <div>
            <label className="block text-xs text-[var(--color-text-muted)] mb-1">
              Protocol
            </label>
            <select
              value={editingEndpoint.protocol}
              onChange={(e) =>
                handleProtocolChange(e.target.value as PtzProtocol)
              }
              className="w-full px-2 py-1.5 text-sm bg-[var(--color-bg-dark)] border border-[var(--color-border)] rounded text-[var(--color-text)] focus:outline-none focus:border-[var(--color-primary)]"
            >
              <option value="Ndi">NDI PTZ</option>
              <option value="Visca">VISCA-over-IP</option>
              <option value="PanasonicAw">Panasonic AW (HTTP)</option>
              <option value="BirdDogRest">BirdDog REST API</option>
              <option value="Simulated">Simulated (no hardware)</option>
            </select>
          </div>

          {/* Protocol-specific fields */}
          {editingEndpoint.config.type !== "Ndi" &&
            "host" in editingEndpoint.config && (
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <label className="block text-xs text-[var(--color-text-muted)] mb-1">
                    Host
                  </label>
                  <input
                    type="text"
                    value={editingEndpoint.config.host}
                    onChange={(e) =>
                      setEditingEndpoint({
                        ...editingEndpoint,
                        config: {
                          ...editingEndpoint.config,
                          host: e.target.value,
                        } as ProtocolConfig,
                      })
                    }
                    className="w-full px-2 py-1.5 text-sm bg-[var(--color-bg-dark)] border border-[var(--color-border)] rounded text-[var(--color-text)] focus:outline-none focus:border-[var(--color-primary)]"
                  />
                </div>
                <div>
                  <label className="block text-xs text-[var(--color-text-muted)] mb-1">
                    Port
                  </label>
                  <input
                    type="number"
                    value={editingEndpoint.config.port}
                    onChange={(e) => {
                      const parsed = parseInt(e.target.value, 10);
                      if (!isNaN(parsed)) {
                        setEditingEndpoint({
                          ...editingEndpoint,
                          config: {
                            ...editingEndpoint.config,
                            port: Math.max(1, Math.min(65535, parsed)),
                          } as ProtocolConfig,
                        });
                      }
                    }}
                    className="w-full px-2 py-1.5 text-sm bg-[var(--color-bg-dark)] border border-[var(--color-border)] rounded text-[var(--color-text)] focus:outline-none focus:border-[var(--color-primary)]"
                  />
                </div>
              </div>
            )}

          {/* Panasonic auth fields */}
          {editingEndpoint.config.type === "PanasonicAw" && (
            <div className="grid grid-cols-2 gap-3">
              <div>
                <label className="block text-xs text-[var(--color-text-muted)] mb-1">
                  Username (optional)
                </label>
                <input
                  type="text"
                  value={editingEndpoint.config.username ?? ""}
                  onChange={(e) =>
                    setEditingEndpoint({
                      ...editingEndpoint,
                      config: {
                        ...editingEndpoint.config,
                        username: e.target.value || undefined,
                      } as ProtocolConfig,
                    })
                  }
                  className="w-full px-2 py-1.5 text-sm bg-[var(--color-bg-dark)] border border-[var(--color-border)] rounded text-[var(--color-text)] focus:outline-none focus:border-[var(--color-primary)]"
                />
              </div>
              <div>
                <label className="block text-xs text-[var(--color-text-muted)] mb-1">
                  Password (optional)
                </label>
                <input
                  type="password"
                  value={editingEndpoint.config.password ?? ""}
                  onChange={(e) =>
                    setEditingEndpoint({
                      ...editingEndpoint,
                      config: {
                        ...editingEndpoint.config,
                        password: e.target.value || undefined,
                      } as ProtocolConfig,
                    })
                  }
                  className="w-full px-2 py-1.5 text-sm bg-[var(--color-bg-dark)] border border-[var(--color-border)] rounded text-[var(--color-text)] focus:outline-none focus:border-[var(--color-primary)]"
                />
              </div>
            </div>
          )}

          {/* Test result */}
          {testResult && (
            <div
              className={`text-xs p-2 rounded ${
                testResult.startsWith("Error")
                  ? "bg-red-900/30 text-red-300"
                  : "bg-green-900/30 text-green-300"
              }`}
            >
              {testResult}
            </div>
          )}

          {/* Actions */}
          <div className="flex gap-2">
            <button
              className="px-3 py-1.5 text-sm bg-[var(--color-primary)] text-white rounded hover:bg-[var(--color-primary-hover)] disabled:opacity-50 transition-colors"
              onClick={handleSave}
              disabled={!editingEndpoint.name.trim()}
            >
              Save
            </button>
            <button
              className="px-3 py-1.5 text-sm border border-[var(--color-border)] text-[var(--color-text)] rounded hover:bg-[var(--color-bg-dark)] transition-colors"
              onClick={handleTest}
              disabled={testing}
            >
              {testing ? "Testing..." : "Test Connection"}
            </button>
            <button
              className="px-3 py-1.5 text-sm text-[var(--color-text-muted)] hover:text-[var(--color-text)] transition-colors"
              onClick={() => {
                setEditingEndpoint(null);
                setIsNew(false);
              }}
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
