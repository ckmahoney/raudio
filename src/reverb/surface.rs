use std::f32::consts::PI;

/// Converts a distance to a time delay based on the speed of sound.
///
/// # Arguments
///
/// * `distance` - The distance to the reflecting surface in meters.
/// * `speed_of_sound` - The speed of sound in meters per second.
///
/// # Returns
///
/// The time delay in seconds.
pub fn distance_to_time_delay(distance: f32, speed_of_sound: f32) -> f32 {
    2.0 * distance / speed_of_sound
}

/// Converts a time delay to a phase offset for a given frequency.
///
/// # Arguments
///
/// * `time_delay` - The time delay in seconds.
/// * `frequency` - The frequency in Hz.
///
/// # Returns
///
/// The phase offset in radians.
pub fn time_to_phase_offset(time_delay: f32, frequency: f32) -> f32 {
    2.0 * PI * frequency * time_delay
}

/// Determines the phase shift on reflection based on the boundary conditions.
///
/// # Arguments
///
/// * `is_higher_density_reflection` - Whether the reflection is from a higher density medium.
/// * `is_rigid_boundary` - Whether the boundary is rigid.
///
/// # Returns
///
/// The phase shift in radians.
pub fn reflection_phase_shift(is_higher_density_reflection: bool, is_rigid_boundary: bool) -> f32 {
    if is_higher_density_reflection {
        PI // 180 degrees phase shift
    } else if is_rigid_boundary {
        PI // 180 degrees phase shift
    } else {
        0.0 // No phase shift
    }
}

/// Applies a phase shift and amplitude decay to a given magnitude and phase.
///
/// # Arguments
///
/// * `magnitude` - The magnitude of the frequency component.
/// * `current_phase` - The current phase of the frequency component.
/// * `phase_shift` - The phase shift to be applied, in radians.
/// * `reflection_coefficient` - The reflection coefficient (0 to 1) indicating amplitude decay.
///
/// # Returns
///
/// A tuple containing the new magnitude and new phase after applying the phase shift and amplitude decay.
pub fn apply_phase_shift_and_decay(magnitude: f32, current_phase: f32, phase_shift: f32, reflection_coefficient: f32) -> (f32, f32) {
    let new_phase = current_phase + phase_shift;
    let new_magnitude = magnitude * reflection_coefficient;
    (new_magnitude, new_phase)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_to_time_delay() {
        let distance = 10.0; // 10 meters
        let speed_of_sound_air = 343.0; // Speed of sound in air at 20°C
        let time_delay_air = distance_to_time_delay(distance, speed_of_sound_air);
        assert!((time_delay_air - (2.0 * 10.0 / 343.0)).abs() < 1e-6);

        let speed_of_sound_water = 1482.0; // Speed of sound in water at 25°C
        let time_delay_water = distance_to_time_delay(distance, speed_of_sound_water);
        assert!((time_delay_water - (2.0 * 10.0 / 1482.0)).abs() < 1e-6);
    }

    #[test]
    fn test_time_to_phase_offset() {
        let time_delay = 0.1; // 100 ms
        let frequency = 1000.0; // 1 kHz
        let phase_offset = time_to_phase_offset(time_delay, frequency);
        assert!((phase_offset - (2.0 * PI * 1000.0 * 0.1)).abs() < 1e-6);
    }

    #[test]
    fn test_reflection_phase_shift() {
        let phase_shift_high_density = reflection_phase_shift(true, false);
        assert!((phase_shift_high_density - PI).abs() < 1e-6);

        let phase_shift_rigid_boundary = reflection_phase_shift(false, true);
        assert!((phase_shift_rigid_boundary - PI).abs() < 1e-6);

        let phase_shift_no_change = reflection_phase_shift(false, false);
        assert!((phase_shift_no_change).abs() < 1e-6);
    }

    #[test]
    fn test_apply_phase_shift_and_decay() {
        let magnitude = 1.0;
        let current_phase = 0.0;
        let phase_shift = PI;
        let reflection_coefficient = 0.8; // 20% amplitude decay
        let (new_magnitude, new_phase) = apply_phase_shift_and_decay(magnitude, current_phase, phase_shift, reflection_coefficient);
        assert!((new_magnitude - (magnitude * reflection_coefficient)).abs() < 1e-6);
        assert!((new_phase - PI).abs() < 1e-6);
    }
}
