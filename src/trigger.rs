use crate::config::Config;
use crate::iot::{HomebridgeCommand, IoTCommand};
pub fn record(config: Config, rt: &tokio::runtime::Handle) {
    tracing::info!("triggered redlight!");

    if let Some(hb) = &config.homebridge() {
        let cmd = HomebridgeCommand {
            api_url: hb.api_url.clone(),
            device_id: hb.device.unique_id.clone(),
            characteristic: hb.device.characteristic_type.clone(),
            // TODO: depending on the characteristic type, we may want to set different values
            value: "true".to_string(),
        };
        rt.block_on(cmd.execute());
    }
}

pub fn playback(_config: Config) {
    tracing::info!("triggered redlight playback!");
}

pub fn stop(config: Config, rt: &tokio::runtime::Handle) {
    tracing::info!("triggered redlight stop!");

    if let Some(hb) = &config.homebridge() {
        let cmd = HomebridgeCommand {
            api_url: hb.api_url.clone(),
            device_id: hb.device.unique_id.clone(),
            characteristic: hb.device.characteristic_type.clone(),
            // TODO: depending on the characteristic type, we may want to set different values
            value: "false".to_string(),
        };
        rt.block_on(cmd.execute());
    }
}
