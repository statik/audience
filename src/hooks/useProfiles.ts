import { useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAppStore } from "../store/app-store";
import { notifyError } from "../utils/notify";
import { usePresets } from "./usePresets";
import { useEndpoints } from "./useEndpoints";
import type { PresetProfile } from "@shared/types";

function generateId(): string {
  return crypto.randomUUID?.() ?? Math.random().toString(36).slice(2);
}

export function useProfiles() {
  const profiles = useAppStore((s) => s.profiles);
  const setProfiles = useAppStore((s) => s.setProfiles);
  const activeProfileId = useAppStore((s) => s.activeProfileId);
  const setActiveProfileId = useAppStore((s) => s.setActiveProfileId);
  const { loadPresets } = usePresets();
  const { setActiveEndpoint } = useEndpoints();

  const loadProfiles = useCallback(async () => {
    try {
      // get_active_profile_id first: it creates the default profile on
      // first run, so the subsequent list is never empty.
      const activeId = await invoke<string | null>("get_active_profile_id");
      const profileList = await invoke<PresetProfile[]>("get_profiles");
      setProfiles(profileList);
      setActiveProfileId(activeId);
    } catch (err) {
      console.error("Failed to load profiles:", err);
      notifyError("Failed to load profiles", err);
    }
  }, [setProfiles, setActiveProfileId]);

  const switchProfile = useCallback(
    async (profileId: string) => {
      try {
        await invoke("load_profile", { profileId });
        setActiveProfileId(profileId);
        await loadPresets();
        // Per-venue camera binding: activate the profile's endpoint if it
        // still exists.
        const state = useAppStore.getState();
        const profile = state.profiles.find((p) => p.id === profileId);
        const boundEndpoint = profile?.endpoint_id;
        if (
          boundEndpoint &&
          boundEndpoint !== state.activeEndpointId &&
          state.endpoints.some((e) => e.id === boundEndpoint)
        ) {
          await setActiveEndpoint(boundEndpoint);
        }
      } catch (err) {
        console.error("Failed to switch profile:", err);
        notifyError("Failed to switch profile", err);
      }
    },
    [setActiveProfileId, loadPresets, setActiveEndpoint]
  );

  const createProfile = useCallback(
    async (name: string) => {
      const state = useAppStore.getState();
      const profile: PresetProfile = {
        id: generateId(),
        name,
        camera_fov_degrees: state.settings.camera_fov_degrees,
        endpoint_id: state.activeEndpointId ?? undefined,
        presets: [],
      };
      try {
        await invoke("save_profile", { profile });
        await invoke("load_profile", { profileId: profile.id });
        setProfiles([...useAppStore.getState().profiles, profile]);
        setActiveProfileId(profile.id);
        await loadPresets();
        return profile;
      } catch (err) {
        console.error("Failed to create profile:", err);
        notifyError("Failed to create profile", err);
        throw err;
      }
    },
    [setProfiles, setActiveProfileId, loadPresets]
  );

  const deleteProfile = useCallback(
    async (profileId: string) => {
      try {
        await invoke("delete_profile", { profileId });
        // The backend picks the next active profile (or recreates the
        // default), so re-sync everything from it.
        await loadProfiles();
        await loadPresets();
      } catch (err) {
        console.error("Failed to delete profile:", err);
        notifyError("Failed to delete profile", err);
      }
    },
    [loadProfiles, loadPresets]
  );

  return {
    profiles,
    activeProfileId,
    loadProfiles,
    switchProfile,
    createProfile,
    deleteProfile,
  };
}
