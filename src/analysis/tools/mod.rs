use crate::phrasing::ranger::{DYNAMIC_RANGE_DB, MAX_DB, MIN_DB};
use crate::synth::{SRf, SR};
use crate::timbre::Role;
use biquad::{Biquad, Coefficients, DirectForm1, Hertz, ToHertz, Type as FilterType, Q_BUTTERWORTH_F32};
use std::error::Error;

use rand::Rng;
mod dynamics;
mod volume;
mod test;
mod more;
pub use dynamics::*;
pub use more::*;
pub use volume::*;


pub fn dev_audio_asset(label: &str) -> String {
    format!("dev-audio/{}", label)
  }
/// Applies a low-pass biquad filter to the input samples.
///
/// This function designs and applies a low-pass filter to the provided audio samples using
/// the biquad filter structure. The filter coefficients are calculated based on the
/// specified cutoff frequency.
///
/// **Implementation Details:**
/// - Utilizes a Butterworth filter with a Q-factor of 0.707.
/// - Processes each sample through the `DirectForm1` filter.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `cutoff_hz`: Low-pass filter cutoff frequency in Hz.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Low-pass filtered samples or an error message if filter creation fails.
fn apply_lowpass(samples: &[f32], cutoff_hz: f32) -> Result<Vec<f32>, String> {
    // Define filter coefficients for a low-pass filter
    let coeffs = Coefficients::<f32>::from_params(
        FilterType::LowPass,
        Hertz::from_hz(SRf).unwrap(),
        Hertz::from_hz(cutoff_hz).unwrap(),
        0.707, // Q-factor (1/sqrt(2) for Butterworth)
    )
    .map_err(|e| format!("Failed to create low-pass filter coefficients: {:?}", e))?;

    // Initialize the filter
    let mut filter = DirectForm1::<f32>::new(coeffs);

    // Process each sample through the filter
    let mut out = Vec::with_capacity(samples.len());
    for &sample in samples.iter() {
        let filtered = filter.run(sample);
        out.push(filtered);
    }
    Ok(out)
}

/// Converts time in seconds to a smoothing coefficient.
///
/// This function translates a time constant into a coefficient used for smoothing
/// envelope followers or gain changes. It ensures that the coefficient stays within
/// a valid range to prevent numerical instability.
///
/// **Implementation Details:**
/// - Uses an exponential decay formula based on the sample rate.
///
/// # Parameters
/// - `time_sec`: Time in seconds.
///
/// # Returns
/// - `f32`: Calculated smoothing coefficient.
fn time_to_coefficient(time_sec: f32) -> f32 {
    if time_sec <= 0.0 {
        0.0
    } else {
        let coeff = (-1.0 / (time_sec * SRf)).exp();
        coeff.min(1.0 - f32::EPSILON).max(f32::EPSILON) // Prevent exactly 1.0 and underflow to 0
    }
}

/// Applies a high-pass biquad filter to the input samples.
///
/// This function designs and applies a high-pass filter to the provided audio samples using
/// the biquad filter structure. The filter coefficients are calculated based on the
/// specified cutoff frequency.
///
/// **Implementation Details:**
/// - Utilizes a Butterworth filter with a Q-factor defined by `Q_BUTTERWORTH_F32`.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `cutoff_hz`: High-pass filter cutoff frequency in Hz.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: High-pass filtered samples or an error message if filter creation fails.
fn apply_highpass(samples: &[f32], cutoff_hz: f32) -> Result<Vec<f32>, String> {
    let sample_rate = SRf;
    if cutoff_hz <= 0.0 || cutoff_hz >= sample_rate / 2.0 {
        return Err(format!(
            "Invalid cutoff frequency: {} Hz. Must be between 0 and Nyquist ({} Hz).",
            cutoff_hz,
            sample_rate / 2.0
        ));
    }

    // Define filter coefficients for a high-pass filter
    let coeffs = Coefficients::<f32>::from_params(
        FilterType::HighPass,
        sample_rate.hz(),
        cutoff_hz.hz(),
        Q_BUTTERWORTH_F32,
    )
    .map_err(|e| format!("Failed to create high-pass filter coefficients: {:?}", e))?;

    // Initialize the filter
    let mut filter = DirectForm1::<f32>::new(coeffs);

    // Process each sample through the filter
    let out: Vec<f32> = samples.iter().map(|&sample| filter.run(sample)).collect();

    Ok(out)
}


/// Applies a high-shelf biquad filter to the input samples.
///
/// This function designs and applies a high-shelf filter to the provided audio samples using
/// the biquad filter structure. The filter coefficients are calculated based on the
/// specified cutoff frequency and gain.
///
/// **Implementation Details:**
/// - Utilizes a shelving filter with the specified gain.
/// - Processes each sample through the `DirectForm1` filter.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `cutoff_hz`: High-shelf filter cutoff frequency in Hz.
/// - `gain_db`: Gain in decibels (positive for boost, negative for cut).
///
/// # Returns
/// - `Result<Vec<f32>, String>`: High-shelf filtered samples or an error message if filter creation fails.
fn apply_highshelf(samples: &[f32], cutoff_hz: f32, gain_db: f32) -> Result<Vec<f32>, String> {
    let sample_rate = SRf;
    if cutoff_hz <= 0.0 || cutoff_hz >= sample_rate / 2.0 {
        return Err(format!(
            "Invalid cutoff frequency for high-shelf: {} Hz. Must be between 0 and Nyquist ({} Hz).",
            cutoff_hz,
            sample_rate / 2.0
        ));
    }

    // Define filter coefficients for a high-shelf filter
    let coeffs = Coefficients::<f32>::from_params(
        FilterType::HighShelf(gain_db),
        sample_rate.hz(),
        cutoff_hz.hz(),
        Q_BUTTERWORTH_F32,
    )
    .map_err(|e| format!("Failed to create high-shelf filter coefficients: {:?}", e))?;

    // Initialize the filter
    let mut filter = DirectForm1::<f32>::new(coeffs);

    // Process each sample through the filter and apply gain
    let mut out = Vec::with_capacity(samples.len());
    for &sample in samples.iter() {
        let filtered = filter.run(sample);
        out.push(filtered);
    }
    Ok(out)
}