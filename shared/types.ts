/** Normalized PTZ position: pan/tilt in [-1, 1], zoom in [0, 1]. */
export interface PtzPosition {
  pan: number;
  tilt: number;
  zoom: number;
}

/** Supported PTZ protocols. */
export type PtzProtocol = "Ndi" | "Visca" | "PanasonicAw" | "BirdDogRest";

/** Protocol-specific connection configuration. */
export type ProtocolConfig =
  | { type: "Ndi" }
  | { type: "Visca"; host: string; port: number }
  | {
      type: "PanasonicAw";
      host: string;
      port: number;
      username?: string;
      password?: string;
    }
  | { type: "BirdDogRest"; host: string; port: number };

/** A camera endpoint for PTZ control. */
export interface CameraEndpoint {
  id: string;
  name: string;
  protocol: PtzProtocol;
  config: ProtocolConfig;
}

/** A single preset definition. */
export interface Preset {
  id: string;
  name: string;
  pan: number;
  tilt: number;
  zoom: number;
  color: string;
}

/** A named collection of presets for a camera setup. */
export interface PresetProfile {
  id: string;
  name: string;
  camera_fov_degrees: number;
  endpoint_id?: string;
  presets: Preset[];
}

/** Application mode. */
export type AppMode = "calibration" | "operation";

/** Video source configuration. */
export type VideoSourceConfig =
  | { type: "Local"; device_id: string }
  | { type: "Ndi"; source_name: string }
  | { type: "MjpegFallback"; device_path: string };

/** Application settings. */
export interface AppSettings {
  click_sensitivity: number;
  scroll_sensitivity: number;
  overlay_opacity: number;
  camera_fov_degrees: number;
  active_profile_id?: string;
  video_source?: VideoSourceConfig;
}

/** NDI source descriptor. */
export interface NdiSource {
  name: string;
  url: string;
}

/** Local device descriptor. */
export interface LocalDevice {
  id: string;
  label: string;
}

/** Available video source for the source picker. */
export interface VideoSource {
  id: string;
  label: string;
  type: "local" | "ndi";
  deviceId?: string;
  ndiName?: string;
}
