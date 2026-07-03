import { create } from "zustand";
import type {
  AppMode,
  AppSettings,
  CameraEndpoint,
  Preset,
  PresetProfile,
  PtzCapabilities,
  PtzPosition,
  Toast,
  VideoSource,
} from "@shared/types";

/** Health of the active PTZ endpoint, derived from command outcomes. */
export type PtzStatus = "idle" | "ok" | "error";

const MAX_TOASTS = 5;

let nextToastId = 0;

interface AppState {
  // Mode
  mode: AppMode;
  setMode: (mode: AppMode) => void;

  // Video
  videoSources: VideoSource[];
  setVideoSources: (sources: VideoSource[]) => void;
  activeVideoSource: VideoSource | null;
  setActiveVideoSource: (source: VideoSource | null) => void;

  // Presets
  presets: Preset[];
  setPresets: (presets: Preset[]) => void;
  activePresetId: string | null;
  setActivePresetId: (id: string | null) => void;

  // Profiles
  profiles: PresetProfile[];
  setProfiles: (profiles: PresetProfile[]) => void;
  activeProfileId: string | null;
  setActiveProfileId: (id: string | null) => void;

  // Endpoints
  endpoints: CameraEndpoint[];
  setEndpoints: (endpoints: CameraEndpoint[]) => void;
  activeEndpointId: string | null;
  setActiveEndpointId: (id: string | null) => void;

  // PTZ position
  currentPosition: PtzPosition;
  setCurrentPosition: (pos: PtzPosition) => void;

  // PTZ capabilities of the active endpoint
  ptzCapabilities: PtzCapabilities | null;
  setPtzCapabilities: (caps: PtzCapabilities | null) => void;

  // Autofocus state (assumed on; cameras default to AF)
  afEnabled: boolean;
  setAfEnabled: (enabled: boolean) => void;

  // Settings
  settings: AppSettings;
  setSettings: (settings: AppSettings) => void;

  // UI state
  sidebarCollapsed: boolean;
  toggleSidebar: () => void;
  settingsOpen: boolean;
  setSettingsOpen: (open: boolean) => void;
  shortcutsHelpOpen: boolean;
  setShortcutsHelpOpen: (open: boolean) => void;

  // Connection
  isConnected: boolean;
  setIsConnected: (connected: boolean) => void;
  fps: number;
  setFps: (fps: number) => void;
  connectionLabel: string;
  setConnectionLabel: (label: string) => void;
  ptzStatus: PtzStatus;
  setPtzStatus: (status: PtzStatus) => void;

  // Toasts
  toasts: Toast[];
  pushToast: (toast: Omit<Toast, "id">) => void;
  dismissToast: (id: string) => void;
}

export const useAppStore = create<AppState>((set) => ({
  // Mode
  mode: "calibration",
  setMode: (mode) => set({ mode }),

  // Video
  videoSources: [],
  setVideoSources: (videoSources) => set({ videoSources }),
  activeVideoSource: null,
  setActiveVideoSource: (activeVideoSource) => set({ activeVideoSource }),

  // Presets
  presets: [],
  setPresets: (presets) => set({ presets }),
  activePresetId: null,
  setActivePresetId: (activePresetId) => set({ activePresetId }),

  // Profiles
  profiles: [],
  setProfiles: (profiles) => set({ profiles }),
  activeProfileId: null,
  setActiveProfileId: (activeProfileId) => set({ activeProfileId }),

  // Endpoints
  endpoints: [],
  setEndpoints: (endpoints) => set({ endpoints }),
  activeEndpointId: null,
  setActiveEndpointId: (activeEndpointId) => set({ activeEndpointId }),

  // PTZ position
  currentPosition: { pan: 0, tilt: 0, zoom: 0 },
  setCurrentPosition: (currentPosition) => set({ currentPosition }),

  // PTZ capabilities
  ptzCapabilities: null,
  setPtzCapabilities: (ptzCapabilities) => set({ ptzCapabilities }),

  // Autofocus
  afEnabled: true,
  setAfEnabled: (afEnabled) => set({ afEnabled }),

  // Settings
  settings: {
    click_sensitivity: 0.1,
    scroll_sensitivity: 0.05,
    overlay_opacity: 0.3,
    camera_fov_degrees: 60,
  },
  setSettings: (settings) => set({ settings }),

  // UI state
  sidebarCollapsed: false,
  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
  settingsOpen: false,
  setSettingsOpen: (settingsOpen) => set({ settingsOpen }),
  shortcutsHelpOpen: false,
  setShortcutsHelpOpen: (shortcutsHelpOpen) => set({ shortcutsHelpOpen }),

  // Connection
  isConnected: false,
  setIsConnected: (isConnected) => set({ isConnected }),
  fps: 0,
  setFps: (fps) => set({ fps }),
  connectionLabel: "No source selected",
  setConnectionLabel: (connectionLabel) => set({ connectionLabel }),
  ptzStatus: "idle",
  setPtzStatus: (ptzStatus) => set({ ptzStatus }),

  // Toasts
  toasts: [],
  pushToast: (toast) =>
    set((state) => {
      // Same-key toasts replace the existing one (with a fresh id, so the
      // auto-dismiss timer restarts) instead of stacking — rapid-fire
      // command failures coalesce into a single toast.
      const kept = toast.key
        ? state.toasts.filter((t) => t.key !== toast.key)
        : state.toasts;
      const id = `toast-${++nextToastId}`;
      return { toasts: [...kept, { ...toast, id }].slice(-MAX_TOASTS) };
    }),
  dismissToast: (id) =>
    set((state) => ({ toasts: state.toasts.filter((t) => t.id !== id) })),
}));
