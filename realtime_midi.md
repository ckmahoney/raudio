# Epic: Real-Time MIDI Synthesizer Support

## Objective
Enable real-time MIDI input and audio output for the existing non-realtime synthesizer, including support for any MIDI device, mapping MIDI controls to application parameters, saving presets, and playback through the system audio device or other selected devices.

## Requirements
1. **MIDI Device Support**: Integrate with any MIDI device, allowing mapping of controller knobs to application parameters.
2. **Preset Management**: Save and load parameter presets, including constrained ranges.
3. **Audio Output**: Play audio through the system audio device or selected output devices.

## Tasks

### Task 1: MIDI Device Integration
- **Subtask 1.1**: Research and select a Rust crate for MIDI input (e.g., `midir`).
- **Subtask 1.2**: Implement a MIDI handler that can detect and connect to any MIDI device.
- **Subtask 1.3**: Create a mapping system to link MIDI controller inputs to application parameters.
- **Subtask 1.4**: Develop a user interface for mapping MIDI controls to parameters, allowing for user customization.

### Task 2: Preset Management
- **Subtask 2.1**: Design a data structure to represent presets, including parameter values and constraints.
- **Subtask 2.2**: Implement functionality to save current parameter settings as a preset.
- **Subtask 2.3**: Implement functionality to load presets, applying the saved settings to the application.
- **Subtask 2.4**: Develop a user interface for managing presets, including saving, loading, and deleting presets.

### Task 3: Audio Output
- **Subtask 3.1**: Research and select a Rust crate for audio output (e.g., `portaudio`).
- **Subtask 3.2**: Implement audio output functionality, defaulting to the system audio device.
- **Subtask 3.3**: Add support for selecting different audio output devices.
- **Subtask 3.4**: Ensure low latency and high performance in the audio output implementation.

### Task 4: Real-Time Processing Integration
- **Subtask 4.1**: Refactor existing synthesizer code to support real-time audio processing.
- **Subtask 4.2**: Integrate MIDI input handling with real-time audio processing, ensuring seamless interaction.
- **Subtask 4.3**: Optimize the audio processing pipeline for real-time performance.

### Task 5: Testing and Validation
- **Subtask 5.1**: Develop unit tests for MIDI input handling and mapping functionality.
- **Subtask 5.2**: Develop unit tests for preset management functionality.
- **Subtask 5.3**: Conduct performance testing for real-time audio processing.
- **Subtask 5.4**: Conduct user testing to validate the overall functionality and usability of the system.

## Deliverables
- **Crates**: New and updated crates for MIDI input, preset management, and audio output.
- **Documentation**: Comprehensive documentation for setup, usage, and customization of the real-time MIDI synthesizer.
- **User Interface**: User-friendly interfaces for mapping MIDI controls, managing presets, and selecting audio output devices.
