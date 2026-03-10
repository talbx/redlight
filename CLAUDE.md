# midi-lights

midi-lights is a small rust program that listens to certain MIDI events/messages
and triggers certain APIs upon receiving them. It uses the the devices
microphone input to detect whether the user is actively tracking or not. This
is needed since DAWs usually do not distinguish between playback and recording
MIDI-wise, so the microhphone input is used to determine the action.

## Usecases
- Playback detection (start / stop)
- Recording detection using core audio (record start / stop)
  - Microphone Input detection using CPAL
  - Microphone selection via config file
- Triggering IoT devices upon recording in Logic Pro X / Fender Studio Pro
