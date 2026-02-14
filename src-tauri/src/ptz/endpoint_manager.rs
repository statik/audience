use super::types::CameraEndpoint;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct EndpointStore {
    endpoints: Vec<CameraEndpoint>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ptz::types::{ProtocolConfig, PtzProtocol};
    use std::fs;

    fn temp_dir() -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ptzcam-test-endpoints-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn make_endpoint(id: &str, name: &str) -> CameraEndpoint {
        CameraEndpoint {
            id: id.to_string(),
            name: name.to_string(),
            protocol: PtzProtocol::Visca,
            config: ProtocolConfig::Visca {
                host: "192.168.1.100".to_string(),
                port: 1259,
            },
        }
    }

    #[test]
    fn starts_empty() {
        let dir = temp_dir();
        let mgr = EndpointManager::load_or_default(&dir);
        assert!(mgr.get_all().is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn create_and_get_endpoint() {
        let dir = temp_dir();
        let mut mgr = EndpointManager::load_or_default(&dir);
        mgr.create(make_endpoint("e1", "Camera 1")).unwrap();

        assert_eq!(mgr.get_all().len(), 1);
        let ep = mgr.get("e1").unwrap();
        assert_eq!(ep.name, "Camera 1");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn get_returns_none_for_missing() {
        let dir = temp_dir();
        let mgr = EndpointManager::load_or_default(&dir);
        assert!(mgr.get("nope").is_none());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn update_modifies_existing() {
        let dir = temp_dir();
        let mut mgr = EndpointManager::load_or_default(&dir);
        mgr.create(make_endpoint("e1", "Old Name")).unwrap();

        let updated = make_endpoint("e1", "New Name");
        mgr.update(updated).unwrap();
        assert_eq!(mgr.get("e1").unwrap().name, "New Name");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn update_nonexistent_returns_error() {
        let dir = temp_dir();
        let mut mgr = EndpointManager::load_or_default(&dir);
        let result = mgr.update(make_endpoint("nope", "Ghost"));
        assert!(result.is_err());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn delete_removes_endpoint() {
        let dir = temp_dir();
        let mut mgr = EndpointManager::load_or_default(&dir);
        mgr.create(make_endpoint("e1", "ToDelete")).unwrap();
        mgr.delete("e1").unwrap();
        assert!(mgr.get_all().is_empty());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn delete_nonexistent_returns_error() {
        let dir = temp_dir();
        let mut mgr = EndpointManager::load_or_default(&dir);
        let result = mgr.delete("nope");
        assert!(result.is_err());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_and_reload_persists_data() {
        let dir = temp_dir();
        {
            let mut mgr = EndpointManager::load_or_default(&dir);
            mgr.create(make_endpoint("e1", "Persisted")).unwrap();
        }
        let mgr = EndpointManager::load_or_default(&dir);
        assert_eq!(mgr.get_all().len(), 1);
        assert_eq!(mgr.get("e1").unwrap().name, "Persisted");
        fs::remove_dir_all(&dir).ok();
    }
}
