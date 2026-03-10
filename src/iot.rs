use std::collections::HashMap;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct Accessory {
    // `type` is a reserved ident in Rust; rename it during deserialization.
    #[serde(rename = "type")]
    pub _kind: Option<String>,
    pub _human_type: Option<String>,
    pub _service_name: Option<String>,
    #[serde(rename = "serviceCharacteristics")]
    pub service_characteristics: Option<Vec<ServiceCharacteristic>>,
    pub _values: Option<HashMap<String, Value>>,
    #[serde(rename = "uniqueId")]
    pub unique_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceCharacteristic {
    #[serde(rename = "type")]
    pub kind: String,
}

#[async_trait]
pub trait IoTCommand {
    async fn execute(&self);
}

pub struct HomebridgeCommand {
    pub api_url: String,
    pub device_id: String,
    pub characteristic: String,
    pub value: String,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    access_token: String,
}

impl HomebridgeCommand {
    async fn auth_token(&self, client: &reqwest::Client) -> Option<String> {
        let username = std::env::var("HOMEBRIDGE_USER").unwrap_or_default();
        let password = std::env::var("HOMEBRIDGE_PASS").unwrap_or_default();

        let base = self.api_url.trim_end_matches('/');
        let response = client
            .post(format!("{}/auth/login", base))
            .json(&serde_json::json!({ "username": username, "password": password }))
            .send()
            .await
            .map_err(|e| tracing::error!(error = %e, "Auth request failed"))
            .ok()?
            .error_for_status()
            .map_err(|e| tracing::error!(error = %e, "Auth returned error status"))
            .ok()?
            .json::<AuthResponse>()
            .await
            .map_err(|e| tracing::error!(error = %e, "Failed to parse auth response"))
            .ok()?;

        Some(response.access_token)
    }
}

#[async_trait]
impl IoTCommand for HomebridgeCommand {
    async fn execute(&self) {
        let client = reqwest::Client::new();
        let base = self.api_url.trim_end_matches('/');

        let token = match self.auth_token(&client).await {
            Some(t) => t,
            None => {
                tracing::error!(
                    "Failed to authenticate with homebridge — check if .env is correctly set"
                );
                return;
            }
        };

        let url = format!("{}/accessories", base);
        let resp = match client.get(&url).bearer_auth(&token).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(error = %e, "Failed to contact homebridge");
                return;
            }
        };

        let resp = match resp.error_for_status() {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(error = %e, "Homebridge returned error status");
                return;
            }
        };

        let accessories = match resp.json::<Vec<Accessory>>().await {
            Ok(a) => a,
            Err(e) => {
                tracing::error!(error = %e, "Failed to parse accessories response");
                return;
            }
        };

        let ac = match accessories.into_iter().find(|a| {
            a.unique_id.as_deref() == Some(&self.device_id)
                && a.service_characteristics
                    .as_ref()
                    .is_some_and(|chars| chars.iter().any(|c| c.kind == self.characteristic))
        }) {
            Some(ac) => ac,
            None => {
                tracing::warn!(device_id = %self.device_id, "No matching accessory found");
                return;
            }
        };

        let unique_id = match &ac.unique_id {
            Some(id) => id.clone(),
            None => {
                tracing::error!("Accessory has no uniqueId");
                return;
            }
        };

        let value: bool = self.value.eq_ignore_ascii_case("true");
        let put_url = format!("{}/accessories/{}", base, unique_id);
        let payload = serde_json::json!({
            "characteristicType": self.characteristic,
            "value": value,
        });

        match client
            .put(&put_url)
            .bearer_auth(&token)
            .json(&payload)
            .send()
            .await
        {
            Ok(r) => {
                if let Err(e) = r.error_for_status() {
                    tracing::error!(error = %e, "PUT /accessories failed");
                } else {
                    tracing::info!(device_id = %unique_id, characteristic = %self.characteristic, value = %value, "Accessory updated");
                }
            }
            Err(e) => tracing::error!(error = %e, "Failed to send PUT /accessories request"),
        }
    }
}
