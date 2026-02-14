import { describe, it, expect, beforeEach } from "vitest";
import { useAppStore } from "./app-store";

describe("app-store", () => {
  beforeEach(() => {
    // Reset store to initial state before each test
    useAppStore.setState({
      mode: "calibration",
      videoSources: [],
      activeVideoSource: null,
      presets: [],
      activePresetId: null,
      profiles: [],
      activeProfileId: null,
      endpoints: [],
      activeEndpointId: null,
      currentPosition: { pan: 0, tilt: 0, zoom: 0 },
      settings: {
        click_sensitivity: 0.1,
        scroll_sensitivity: 0.05,
        overlay_opacity: 0.3,
        camera_fov_degrees: 60,
      },
      sidebarCollapsed: false,
      settingsOpen: false,
      isConnected: false,
      fps: 0,
      connectionLabel: "No source selected",
    });
  });

  describe("mode", () => {
    it("initializes in calibration mode", () => {
      expect(useAppStore.getState().mode).toBe("calibration");
    });

    it("sets mode to operation", () => {
      useAppStore.getState().setMode("operation");
      expect(useAppStore.getState().mode).toBe("operation");
    });

    it("toggles back to calibration", () => {
      useAppStore.getState().setMode("operation");
      useAppStore.getState().setMode("calibration");
      expect(useAppStore.getState().mode).toBe("calibration");
    });
  });

  describe("presets", () => {
    it("initializes with empty presets", () => {
      expect(useAppStore.getState().presets).toEqual([]);
    });

    it("sets presets", () => {
      const presets = [
        { id: "1", name: "Front Row", pan: 0, tilt: 0, zoom: 0.5, color: "#3b82f6" },
        { id: "2", name: "Back Row", pan: 0.2, tilt: -0.1, zoom: 0.3, color: "#ef4444" },
      ];
      useAppStore.getState().setPresets(presets);
      expect(useAppStore.getState().presets).toEqual(presets);
    });

    it("sets active preset id", () => {
      useAppStore.getState().setActivePresetId("1");
      expect(useAppStore.getState().activePresetId).toBe("1");
    });

    it("clears active preset id", () => {
      useAppStore.getState().setActivePresetId("1");
      useAppStore.getState().setActivePresetId(null);
      expect(useAppStore.getState().activePresetId).toBeNull();
    });
  });

  describe("currentPosition", () => {
    it("initializes at origin", () => {
      const pos = useAppStore.getState().currentPosition;
      expect(pos).toEqual({ pan: 0, tilt: 0, zoom: 0 });
    });

    it("updates position", () => {
      useAppStore.getState().setCurrentPosition({ pan: 0.5, tilt: -0.3, zoom: 0.8 });
      expect(useAppStore.getState().currentPosition).toEqual({ pan: 0.5, tilt: -0.3, zoom: 0.8 });
    });
  });

  describe("settings", () => {
    it("has sensible defaults", () => {
      const settings = useAppStore.getState().settings;
      expect(settings.click_sensitivity).toBe(0.1);
      expect(settings.scroll_sensitivity).toBe(0.05);
      expect(settings.overlay_opacity).toBe(0.3);
      expect(settings.camera_fov_degrees).toBe(60);
    });

    it("updates settings", () => {
      useAppStore.getState().setSettings({
        click_sensitivity: 0.2,
        scroll_sensitivity: 0.1,
        overlay_opacity: 0.5,
        camera_fov_degrees: 90,
      });
      expect(useAppStore.getState().settings.click_sensitivity).toBe(0.2);
      expect(useAppStore.getState().settings.camera_fov_degrees).toBe(90);
    });
  });

  describe("UI state", () => {
    it("sidebar starts expanded", () => {
      expect(useAppStore.getState().sidebarCollapsed).toBe(false);
    });

    it("toggles sidebar", () => {
      useAppStore.getState().toggleSidebar();
      expect(useAppStore.getState().sidebarCollapsed).toBe(true);
      useAppStore.getState().toggleSidebar();
      expect(useAppStore.getState().sidebarCollapsed).toBe(false);
    });

    it("settings panel starts closed", () => {
      expect(useAppStore.getState().settingsOpen).toBe(false);
    });

    it("opens and closes settings", () => {
      useAppStore.getState().setSettingsOpen(true);
      expect(useAppStore.getState().settingsOpen).toBe(true);
      useAppStore.getState().setSettingsOpen(false);
      expect(useAppStore.getState().settingsOpen).toBe(false);
    });
  });

  describe("connection", () => {
    it("starts disconnected", () => {
      expect(useAppStore.getState().isConnected).toBe(false);
      expect(useAppStore.getState().fps).toBe(0);
    });

    it("sets connection state", () => {
      useAppStore.getState().setIsConnected(true);
      expect(useAppStore.getState().isConnected).toBe(true);
    });

    it("updates fps", () => {
      useAppStore.getState().setFps(30);
      expect(useAppStore.getState().fps).toBe(30);
    });

    it("sets connection label", () => {
      useAppStore.getState().setConnectionLabel("NDI - Camera 1");
      expect(useAppStore.getState().connectionLabel).toBe("NDI - Camera 1");
    });
  });

  describe("endpoints", () => {
    it("initializes with empty endpoints", () => {
      expect(useAppStore.getState().endpoints).toEqual([]);
    });

    it("sets endpoints", () => {
      const endpoints = [
        { id: "1", name: "Main Camera", protocol: "Ndi" as const, config: { type: "Ndi" as const } },
      ];
      useAppStore.getState().setEndpoints(endpoints);
      expect(useAppStore.getState().endpoints).toEqual(endpoints);
    });

    it("sets active endpoint id", () => {
      useAppStore.getState().setActiveEndpointId("ep-1");
      expect(useAppStore.getState().activeEndpointId).toBe("ep-1");
    });
  });
});
