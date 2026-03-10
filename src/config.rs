use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Config {
    pub audio: Option<AudioConfig>,
    pub devices: Option<Vec<String>>,
    pub iot: Option<IoTConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AudioConfig {
    pub microphone: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IoTConfig {
    pub homebridge: Option<HomebridgeConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HomebridgeConfig {
    pub api_url: String,
    pub device: HomebridgeDevice,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HomebridgeDevice {
    pub unique_id: String,
    pub characteristic_type: String,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path)?;
        Ok(serde_yaml::from_str(&contents)?)
    }

    pub fn microphone(&self) -> Option<&str> {
        self.audio.as_ref()?.microphone.as_deref()
    }
    pub fn devices(&self) -> Vec<String> {
        self.devices.clone().unwrap_or_default()
    }

    pub fn iot(&self) -> Option<&IoTConfig> {
        self.iot.as_ref()
    }

    pub fn homebridge(&self) -> Option<&HomebridgeConfig> {
        self.iot()?.homebridge.as_ref()
    }
}
