# Modulators

### Frequency Modulation (fmod)
- **Purpose**: Modulates the frequency of a signal, affecting the pitch.
- **Context**: Can benefit from access to frequency context (current, previous, and next frequencies).
- **Scenarios**:
  - Dynamic pitch shifting
  - Harmonic and inharmonic sounds
  - Complex waveform creation

### Amplitude Modulation (amod)
- **Purpose**: Modulates the amplitude of a signal, affecting loudness.
- **Context**: Can use additional contexts such as envelope position, velocity, harmonic index, and timbre.
- **Scenarios**:
  - Dynamic envelope shaping
  - Velocity-sensitive modulation
  - Harmonic-specific modulation
  - Timbre-based modulation

### Phase Modulation (pmod)
- **Purpose**: Modulates the phase of a signal, affecting the phase angle and timbre.
- **Context**: Can use additional contexts such as relative phase, time, modulation depth, intensity, and frequency range.
- **Scenarios**:
  - Relative phase modulation
  - Time-evolving phase changes
  - Depth-dependent modulation
  - Intensity-sensitive phase shifts
  - Frequency range-specific modulation
