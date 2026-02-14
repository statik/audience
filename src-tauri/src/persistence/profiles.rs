use crate::ptz::types::{Preset, PresetProfile};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProfileData {
    profiles: Vec<PresetProfile>,
    active_profile_id: Option<String>,
}

impl Default for ProfileData {
    fn default() -> Self {
        Self {
            profiles: Vec::new(),
            active_profile_id: None,
        }
    }
}

/// Manages preset profiles and their persistence.
pub struct ProfileStore {
    data: ProfileData,
    file_path: PathBuf,
}

impl ProfileStore {
    pub fn load_or_default(data_dir: &Path) -> Self {
        let file_path = data_dir.join("profiles.json");
        let data = if file_path.exists() {
            std::fs::read_to_string(&file_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            ProfileData::default()
        };
        Self { data, file_path }
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.data).map_err(|e| e.to_string())?;
        std::fs::write(&self.file_path, json).map_err(|e| e.to_string())
    }

    // --- Profile operations ---

    pub fn get_profiles(&self) -> Vec<PresetProfile> {
        self.data.profiles.clone()
    }

    pub fn get_active_profile(&self) -> Option<&PresetProfile> {
        self.data
            .active_profile_id
            .as_ref()
            .and_then(|id| self.data.profiles.iter().find(|p| p.id == *id))
    }

    pub fn get_active_profile_mut(&mut self) -> Option<&mut PresetProfile> {
        let active_id = self.data.active_profile_id.clone();
        active_id.and_then(move |id| self.data.profiles.iter_mut().find(|p| p.id == id))
    }

    pub fn set_active_profile(&mut self, id: &str) -> Result<(), String> {
        if !self.data.profiles.iter().any(|p| p.id == id) {
            return Err("Profile not found".to_string());
        }
        self.data.active_profile_id = Some(id.to_string());
        self.save()
    }

    pub fn create_profile(&mut self, profile: PresetProfile) -> Result<PresetProfile, String> {
        self.data.profiles.push(profile.clone());
        if self.data.active_profile_id.is_none() {
            self.data.active_profile_id = Some(profile.id.clone());
        }
        self.save()?;
        Ok(profile)
    }

    pub fn save_profile(&mut self, profile: PresetProfile) -> Result<PresetProfile, String> {
        if let Some(pos) = self.data.profiles.iter().position(|p| p.id == profile.id) {
            self.data.profiles[pos] = profile.clone();
        } else {
            self.data.profiles.push(profile.clone());
        }
        self.save()?;
        Ok(profile)
    }

    pub fn delete_profile(&mut self, id: &str) -> Result<(), String> {
        let pos = self
            .data
            .profiles
            .iter()
            .position(|p| p.id == id)
            .ok_or("Profile not found")?;
        self.data.profiles.remove(pos);
        if self.data.active_profile_id.as_deref() == Some(id) {
            self.data.active_profile_id = self.data.profiles.first().map(|p| p.id.clone());
        }
        self.save()
    }

    // --- Preset operations (on active profile) ---

    pub fn get_presets(&self) -> Vec<Preset> {
        self.get_active_profile()
            .map(|p| p.presets.clone())
            .unwrap_or_default()
    }

    pub fn create_preset(&mut self, preset: Preset) -> Result<Preset, String> {
        let profile = self
            .get_active_profile_mut()
            .ok_or("No active profile")?;
        profile.presets.push(preset.clone());
        self.save()?;
        Ok(preset)
    }

    pub fn update_preset(&mut self, preset: Preset) -> Result<Preset, String> {
        let profile = self
            .get_active_profile_mut()
            .ok_or("No active profile")?;
        let pos = profile
            .presets
            .iter()
            .position(|p| p.id == preset.id)
            .ok_or("Preset not found")?;
        profile.presets[pos] = preset.clone();
        self.save()?;
        Ok(preset)
    }

    pub fn delete_preset(&mut self, preset_id: &str) -> Result<(), String> {
        let profile = self
            .get_active_profile_mut()
            .ok_or("No active profile")?;
        let pos = profile
            .presets
            .iter()
            .position(|p| p.id == preset_id)
            .ok_or("Preset not found")?;
        profile.presets.remove(pos);
        self.save()
    }

    pub fn find_preset(&self, preset_id: &str) -> Option<Preset> {
        self.get_active_profile()
            .and_then(|p| p.presets.iter().find(|pr| pr.id == preset_id).cloned())
    }

    /// Ensure there is at least one profile. Creates a default if empty.
    pub fn ensure_default_profile(&mut self) -> Result<(), String> {
        if self.data.profiles.is_empty() {
            let profile = PresetProfile {
                id: uuid::Uuid::new_v4().to_string(),
                name: "Default".to_string(),
                camera_fov_degrees: 60.0,
                endpoint_id: None,
                presets: Vec::new(),
            };
            self.create_profile(profile)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("ptzcam-test-profiles-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn make_profile(id: &str, name: &str) -> PresetProfile {
        PresetProfile {
            id: id.to_string(),
            name: name.to_string(),
            camera_fov_degrees: 60.0,
            endpoint_id: None,
            presets: Vec::new(),
        }
    }

    fn make_preset(id: &str, name: &str) -> Preset {
        Preset {
            id: id.to_string(),
            name: name.to_string(),
            pan: 0.0,
            tilt: 0.0,
            zoom: 0.5,
            color: "#3b82f6".to_string(),
        }
    }

    #[test]
    fn starts_with_no_profiles() {
        let dir = temp_dir();
        let store = ProfileStore::load_or_default(&dir);
        assert!(store.get_profiles().is_empty());
        assert!(store.get_active_profile().is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn create_profile_sets_it_as_active_when_first() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        let profile = make_profile("p1", "First");
        store.create_profile(profile).unwrap();

        assert_eq!(store.get_profiles().len(), 1);
        assert_eq!(store.get_active_profile().unwrap().id, "p1");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn create_second_profile_does_not_change_active() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        store.create_profile(make_profile("p1", "First")).unwrap();
        store.create_profile(make_profile("p2", "Second")).unwrap();

        assert_eq!(store.get_profiles().len(), 2);
        assert_eq!(store.get_active_profile().unwrap().id, "p1");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn set_active_profile_switches_active() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        store.create_profile(make_profile("p1", "First")).unwrap();
        store.create_profile(make_profile("p2", "Second")).unwrap();

        store.set_active_profile("p2").unwrap();
        assert_eq!(store.get_active_profile().unwrap().id, "p2");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn set_active_profile_rejects_unknown_id() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        store.create_profile(make_profile("p1", "First")).unwrap();

        let result = store.set_active_profile("nonexistent");
        assert!(result.is_err());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn delete_active_profile_switches_to_next() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        store.create_profile(make_profile("p1", "First")).unwrap();
        store.create_profile(make_profile("p2", "Second")).unwrap();

        store.delete_profile("p1").unwrap();
        assert_eq!(store.get_profiles().len(), 1);
        assert_eq!(store.get_active_profile().unwrap().id, "p2");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn delete_nonexistent_profile_returns_error() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        let result = store.delete_profile("nope");
        assert!(result.is_err());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn ensure_default_profile_creates_one_when_empty() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        assert!(store.get_profiles().is_empty());

        store.ensure_default_profile().unwrap();
        assert_eq!(store.get_profiles().len(), 1);
        assert_eq!(store.get_active_profile().unwrap().name, "Default");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn ensure_default_profile_is_idempotent() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        store.ensure_default_profile().unwrap();
        store.ensure_default_profile().unwrap();
        assert_eq!(store.get_profiles().len(), 1);
        fs::remove_dir_all(&dir).ok();
    }

    // --- Preset CRUD on active profile ---

    #[test]
    fn create_preset_on_active_profile() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        store.ensure_default_profile().unwrap();

        let preset = make_preset("pr1", "Front Row");
        store.create_preset(preset).unwrap();
        assert_eq!(store.get_presets().len(), 1);
        assert_eq!(store.get_presets()[0].name, "Front Row");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn create_preset_fails_without_active_profile() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        let result = store.create_preset(make_preset("pr1", "Test"));
        assert!(result.is_err());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn update_preset_modifies_existing() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        store.ensure_default_profile().unwrap();
        store.create_preset(make_preset("pr1", "Original")).unwrap();

        let mut updated = make_preset("pr1", "Renamed");
        updated.pan = 0.5;
        store.update_preset(updated).unwrap();

        let presets = store.get_presets();
        assert_eq!(presets.len(), 1);
        assert_eq!(presets[0].name, "Renamed");
        assert_eq!(presets[0].pan, 0.5);
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn update_nonexistent_preset_returns_error() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        store.ensure_default_profile().unwrap();
        let result = store.update_preset(make_preset("nope", "Ghost"));
        assert!(result.is_err());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn delete_preset_removes_it() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        store.ensure_default_profile().unwrap();
        store.create_preset(make_preset("pr1", "ToDelete")).unwrap();

        store.delete_preset("pr1").unwrap();
        assert!(store.get_presets().is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_preset_returns_matching() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        store.ensure_default_profile().unwrap();
        store.create_preset(make_preset("pr1", "Target")).unwrap();

        let found = store.find_preset("pr1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Target");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn find_preset_returns_none_for_missing() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        store.ensure_default_profile().unwrap();
        assert!(store.find_preset("nope").is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_and_reload_preserves_data() {
        let dir = temp_dir();
        {
            let mut store = ProfileStore::load_or_default(&dir);
            store.create_profile(make_profile("p1", "Persisted")).unwrap();
            store.create_preset(make_preset("pr1", "Saved Preset")).unwrap();
        }

        let store = ProfileStore::load_or_default(&dir);
        assert_eq!(store.get_profiles().len(), 1);
        assert_eq!(store.get_profiles()[0].name, "Persisted");
        assert_eq!(store.get_presets().len(), 1);
        assert_eq!(store.get_presets()[0].name, "Saved Preset");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_profile_updates_existing_or_creates_new() {
        let dir = temp_dir();
        let mut store = ProfileStore::load_or_default(&dir);
        store.create_profile(make_profile("p1", "Original")).unwrap();

        // Update existing
        let mut updated = make_profile("p1", "Updated");
        updated.camera_fov_degrees = 90.0;
        store.save_profile(updated).unwrap();
        assert_eq!(store.get_profiles().len(), 1);
        assert_eq!(store.get_profiles()[0].name, "Updated");

        // Create new via save_profile
        store.save_profile(make_profile("p2", "New")).unwrap();
        assert_eq!(store.get_profiles().len(), 2);
        fs::remove_dir_all(&dir).ok();
    }
}
