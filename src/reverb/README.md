# Phase Helpers Module

This Rust module provides helper functions for computing phase shifts and amplitude decay for sound reflections. It is designed to assist in simulating realistic reverb effects by handling the computational aspects of phase shifts and amplitude changes due to reflections.

## Overview

Sound reflections play a crucial role in creating realistic auditory experiences, especially in the context of reverb. When sound waves encounter boundaries (such as walls or other surfaces), they reflect back into the medium they originated from. The phase and amplitude of these reflected waves depend on several factors, including the nature of the boundary and the relative densities of the media involved.

This module encapsulates the physics principles behind these reflections and provides a set of functions to compute time delays, phase offsets, and amplitude decay based on user-defined parameters.

## Functions

### `distance_to_time_delay`

Converts a distance to a time delay based on the speed of sound.

#### Parameters:
- `distance`: The distance to the reflecting surface in meters.
- `speed_of_sound`: The speed of sound in meters per second.

#### Returns:
The time delay in seconds.

#### Explanation:
The time delay for a sound wave to travel to a reflecting surface and back is given by:
\[ \text{time\_delay} = \frac{2 \times \text{distance}}{\text{speed\_sound}} \]
This accounts for the round-trip travel of the sound wave.

### `time_to_phase_offset`

Converts a time delay to a phase offset for a given frequency.

#### Parameters:
- `time_delay`: The time delay in seconds.
- `frequency`: The frequency in Hz.

#### Returns:
The phase offset in radians.

#### Explanation:
The phase offset introduced by a time delay is calculated using:
\[ \Delta \phi = 2 \pi f \Delta t \]
where \( f \) is the frequency and \( \Delta t \) is the time delay. This reflects how the phase of a wave changes over time.

### `reflection_phase_shift`

Determines the phase shift on reflection based on the boundary conditions.

#### Parameters:
- `is_higher_density_reflection`: Whether the reflection is from a higher density medium.
- `is_rigid_boundary`: Whether the boundary is rigid.

#### Returns:
The phase shift in radians.

#### Explanation:
The phase shift upon reflection depends on the boundary conditions:
- A phase change of \( \pi \) (180 degrees) occurs when a wave reflects from a higher density medium.
- A phase change of \( \pi \) also occurs at a rigid boundary.
- No phase change occurs when reflecting from a lower density medium or at a free boundary.

### `apply_phase_shift_and_decay`

Applies a phase shift and amplitude decay to a given magnitude and phase.

#### Parameters:
- `magnitude`: The magnitude of the frequency component.
- `current_phase`: The current phase of the frequency component.
- `phase_shift`: The phase shift to be applied, in radians.
- `reflection_coefficient`: The reflection coefficient (0 to 1) indicating amplitude decay.

#### Returns:
A tuple containing the new magnitude and new phase after applying the phase shift and amplitude decay.

#### Explanation:
- The new phase is calculated by adding the phase shift to the current phase.
- The new magnitude is calculated by multiplying the original magnitude by the reflection coefficient, accounting for amplitude decay.

## Physics Background

### Speed of Sound

The speed of sound varies depending on the medium and environmental conditions. For instance:
- In air at 20°C (68°F), the speed of sound is approximately 343 meters per second.
- In water at 25°C (77°F), the speed of sound is approximately 1482 meters per second.
- The speed of sound decreases with altitude due to lower air density and temperature.

### Reflection and Phase Change

When sound waves reflect off surfaces, their phase may change depending on the boundary conditions:
- **Higher Density Reflection**: A phase change of \( \pi \) occurs when a wave reflects from a less dense to a more dense medium.
- **Rigid Boundary**: A phase change of \( \pi \) occurs when reflecting off a rigid boundary.
- **Free Boundary**: No phase change occurs when reflecting off a free boundary or from a more dense to a less dense medium.

### Amplitude Decay

Amplitude decay upon reflection is characterized by the reflection coefficient, which ranges from 0 to 1. This coefficient represents the proportion of the wave's amplitude that is retained after reflection.

## Example Usage

```rust
fn main() {
    let distance_to_wall = 10.0; // in meters
    let speed_of_sound_air = 343.0; // Speed of sound in air at 20°C
    let speed_of_sound_water = 1482.0; // Speed of sound in water at 25°C

    let time_delay_air = distance_to_time_delay(distance_to_wall, speed_of_sound_air);
    let time_delay_water = distance_to_time_delay(distance_to_wall, speed_of_sound_water);

    println!("Time delay in air: {} seconds", time_delay_air);
    println!("Time delay in water: {} seconds", time_delay_water);

    let frequency = 1000.0; // 1 kHz
    let phase_offset_air = time_to_phase_offset(time_delay_air, frequency);
    let phase_offset_water = time_to_phase_offset(time_delay_water, frequency);

    println!("Phase offset in air: {} radians", phase_offset_air);
    println!("Phase offset in water: {} radians", phase_offset_water);

    let reflection_phase = reflection_phase_shift(true, false); // Reflection from a higher density medium
    println!("Reflection phase shift: {} radians", reflection_phase);

    let (new_magnitude, new_phase) = apply_phase_shift_and_decay(1.0, 0.0, reflection_phase, 0.8);
    println!("New magnitude: {}, New phase: {} radians", new_magnitude, new_phase);
}
