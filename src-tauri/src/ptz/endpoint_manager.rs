use super::types::CameraEndpoint;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EndpointStore {
    endpoints: Vec<CameraEndpoint>,
}

impl Default for EndpointStore {
    fn default() -> Self {
        Self {
            endpoints: Vec::new(),
        }
    }
}

/// Manages CRUD operations and persistence for camera endpoints.
pub struct EndpointManager {
    store: EndpointStore,
    file_path: PathBuf,
}

impl EndpointManager {
    pub fn load_or_default(data_dir: &Path) -> Self {
        let file_path = data_dir.join("endpoints.json");
        let store = if file_path.exists() {
            std::fs::read_to_string(&file_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            EndpointStore::default()
        };
        Self { store, file_path }
    }

    pub fn save(&self) -> Result<(), String> {
        let json = serde_json::to_string_pretty(&self.store).map_err(|e| e.to_string())?;
        std::fs::write(&self.file_path, json).map_err(|e| e.to_string())
    }

    pub fn get_all(&self) -> Vec<CameraEndpoint> {
        self.store.endpoints.clone()
    }

    pub fn get(&self, id: &str) -> Option<CameraEndpoint> {
        self.store.endpoints.iter().find(|e| e.id == id).cloned()
    }

    pub fn create(&mut self, endpoint: CameraEndpoint) -> Result<CameraEndpoint, String> {
        self.store.endpoints.push(endpoint.clone());
        self.save()?;
        Ok(endpoint)
    }

    pub fn update(&mut self, endpoint: CameraEndpoint) -> Result<CameraEndpoint, String> {
        let pos = self
            .store
            .endpoints
            .iter()
            .position(|e| e.id == endpoint.id)
            .ok_or("Endpoint not found")?;
        self.store.endpoints[pos] = endpoint.clone();
        self.save()?;
        Ok(endpoint)
    }

    pub fn delete(&mut self, id: &str) -> Result<(), String> {
        let pos = self
            .store
            .endpoints
            .iter()
            .position(|e| e.id == id)
            .ok_or("Endpoint not found")?;
        self.store.endpoints.remove(pos);
        self.save()
    }
}
