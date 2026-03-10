use std::sync::{Arc, Mutex};

use midir::{Ignore, MidiInput};
use tracing_subscriber::fmt;

mod config;
mod coreaudio;
use config::Config;
use coreaudio::InputMonitor;

fn main() {
    fmt().json().init();

    tracing::info!("staring midi-lights ...");
    tracing::info!("loading device config ...");
    let load = Config::load("config.yaml");

    let cfg = match load {
        Ok(cfg) => cfg,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load config");
            std::process::exit(1);
        }
    };
    tracing::info!("successfully loaded device config!");

    // Start audio input monitoring
    let input_monitor = InputMonitor::new(&cfg);
    let _audio_stream = match input_monitor.start_monitoring() {
        Ok(stream) => stream,
        Err(e) => {
            tracing::error!(error = %e, "Failed to start audio monitoring");
            std::process::exit(1);
        }
    };

    // Track transport state
    let is_running = Arc::new(Mutex::new(false));
    let is_running_clone = is_running.clone();

    // MIDI monitoring (from before)
    let mut midi_in = MidiInput::new("Studio One Monitor").unwrap();
    midi_in.ignore(Ignore::None);

    let ports = midi_in.ports();
    let iac_port = ports
        .iter()
        .find(|p| midi_in.port_name(p).unwrap().contains("IAC"))
        .expect("IAC Driver not found");

    let _conn = midi_in
        .connect(
            iac_port,
            "callback",
            move |_stamp, message, _| {
                match message[0] {
                    // started | continue
                    0xFB | 0xFA => {
                        *is_running_clone.lock().unwrap() = true;

                        if input_monitor.has_input() {
                            tracing::info!(
                                state = "recording",
                                "transport started with audio input"
                            );
                            // Trigger lights here
                        } else {
                            tracing::info!(
                                state = "playback",
                                "transport started without audio input"
                            );
                            // Don't trigger lights
                        }
                    }
                    // stopped
                    0xFC => {
                        *is_running_clone.lock().unwrap() = false;
                        tracing::info!(state = "stopped", "transport stopped");
                        // Turn off lights
                    }
                    _ => {}
                }
            },
            (),
        )
        .unwrap();

    tracing::info!("monitoring MIDI with audio input detection");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}
