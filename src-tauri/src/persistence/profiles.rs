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
