use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

use crate::config::Config;

/// A lightweight input monitor that reports whether audio input is present.
///
/// Holds only the shared signal state — the stream is returned from
/// `start_monitoring` so the caller can keep it alive without making
/// `InputMonitor` non-Send.
pub struct InputMonitor {
    has_signal: Arc<Mutex<bool>>,
    config: Config,
}

impl InputMonitor {
    pub fn new(config: &Config) -> Self {
        InputMonitor {
            has_signal: Arc::new(Mutex::new(false)),
            config: config.clone(),
        }
    }

    /// Starts audio input monitoring. The returned stream must be kept alive
    /// for monitoring to continue.
    pub fn start_monitoring(&self) -> anyhow::Result<cpal::Stream> {
        let has_signal = self.has_signal.clone();

        let host = cpal::default_host();
        let available_mics = self.config.devices();
        let device = match self.config.microphone() {
            Some(name) => host
                .input_devices()?
                .find(|d| d.name().map(|n| n.contains(name)).unwrap_or(false))
                .ok_or_else(||anyhow::anyhow!("Input device '{name}' not found. Are you sure your device is connected to your Mac? Declared devices: {}", available_mics.join(", ")))?,
            None => host
                .default_input_device()
                .ok_or_else(|| anyhow::anyhow!("No default input device found"))?,
        };

        let stream_config = device.default_input_config()?;

        let stream = device.build_input_stream(
            &stream_config.into(),
            move |data: &[f32], _| {
                // guitar playing should fire around 0.01 - 0.02
                // we want to ignore low-level noise, so use a threshold
                let threshold: f32 = 0.01;
                let has_audio = data.iter().any(|&s| s.abs() > threshold);
                *has_signal.lock().unwrap() = has_audio;
            },
            |err| tracing::error!(error = %err, "audio stream error"),
            None,
        )?;

        stream.play()?;

        Ok(stream)
    }

    pub fn has_input(&self) -> bool {
        *self.has_signal.lock().unwrap()
    }
}
