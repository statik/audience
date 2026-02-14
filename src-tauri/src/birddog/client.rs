use crate::ptz::controller::{PtzController, PtzError};
use crate::ptz::types::PtzPosition;
use async_trait::async_trait;

/// BirdDog REST API client for BirdDog PTZ cameras.
/// Uses HTTP POST/GET requests to the BirdDog API (default port 8080).
pub struct BirdDogClient {
    base_url: String,
    client: reqwest::Client,
}

impl BirdDogClient {
    pub fn new(host: &str, port: u16) -> Result<Self, PtzError> {
        crate::ptz::types::validate_host(host)
            .map_err(PtzError::ConnectionFailed)?;
        Ok(Self {
            base_url: format!("http://{}:{}", host, port),
            client: reqwest::Client::new(),
        })
    }

    async fn post_json(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, PtzError> {
        let url = format!("{}/{}", self.base_url, endpoint);
        let response = self
            .client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| PtzError::ConnectionFailed(e.to_string()))?;

        let json = response
            .json()
            .await
            .map_err(|e| PtzError::CommandFailed(e.to_string()))?;

        Ok(json)
    }

    async fn get_json(&self, endpoint: &str) -> Result<serde_json::Value, PtzError> {
        let url = format!("{}/{}", self.base_url, endpoint);
        let response = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| PtzError::ConnectionFailed(e.to_string()))?;

        let json = response
            .json()
            .await
            .map_err(|e| PtzError::CommandFailed(e.to_string()))?;

        Ok(json)
    }
}

#[async_trait]
impl PtzController for BirdDogClient {
    async fn move_absolute(&self, pan: f64, tilt: f64, zoom: f64) -> Result<(), PtzError> {
        self.post_json(
            "ptz",
            serde_json::json!({
                "pan": pan,
                "tilt": tilt,
                "zoom": zoom,
                "mode": "absolute"
            }),
        )
        .await?;
        Ok(())
    }

    async fn move_relative(&self, pan_delta: f64, tilt_delta: f64) -> Result<(), PtzError> {
        self.post_json(
            "ptz",
            serde_json::json!({
                "pan": pan_delta,
                "tilt": tilt_delta,
                "mode": "relative"
            }),
        )
        .await?;
        Ok(())
    }

    async fn zoom_to(&self, zoom: f64) -> Result<(), PtzError> {
        self.post_json(
            "ptz",
            serde_json::json!({
                "zoom": zoom,
                "mode": "absolute"
            }),
        )
        .await?;
        Ok(())
    }

    async fn recall_preset(&self, preset_index: u8) -> Result<(), PtzError> {
        self.post_json(
            "recall",
            serde_json::json!({
                "preset": preset_index
            }),
        )
        .await?;
        Ok(())
    }

    async fn store_preset(&self, preset_index: u8) -> Result<(), PtzError> {
        self.post_json(
            "store",
            serde_json::json!({
                "preset": preset_index
            }),
        )
        .await?;
        Ok(())
    }

    async fn get_position(&self) -> Result<PtzPosition, PtzError> {
        let response = self.get_json("ptz/position").await?;
        Ok(PtzPosition {
            pan: response["pan"].as_f64().unwrap_or(0.0),
            tilt: response["tilt"].as_f64().unwrap_or(0.0),
            zoom: response["zoom"].as_f64().unwrap_or(0.0),
        })
    }

    async fn test_connection(&self) -> Result<(), PtzError> {
        self.get_json("about").await?;
        Ok(())
    }
}
