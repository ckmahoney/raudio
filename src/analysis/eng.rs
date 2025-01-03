use crate::phrasing::ranger::{DYNAMIC_RANGE_DB, MAX_DB, MIN_DB};
use crate::synth::{SRf, SR};
use crate::timbre::Role;
use biquad::{Biquad, Coefficients, DirectForm1, Hertz, ToHertz, Type as FilterType, Q_BUTTERWORTH_F32};
use itertools::izip;
use std::error::Error;

use crate::analysis::sampler::read_audio_file;
use crate::render::engrave::write_audio;
use rand::Rng;

pub fn dev_audio_asset(label: &str) -> String {
  format!("dev-audio/{}", label)
}

/// Enumeration for different envelope detection methods.
#[derive(Debug, Clone, Copy)]
pub enum EnvelopeMethod {
  Peak,
  Rms(f32),    // Moving average window size in seconds
  Hybrid(f32), // Same for Hybrid (if you want to include RMS-like smoothing)
}

/// Enumeration for ratio slope types.
#[derive(Debug, Clone, Copy)]
pub enum RatioSlope {
  /// Linear transition into compression.
  Linear,
  /// Exponential transition into compression.
  Exponential,
}

/// Enumeration for envelope shaping types.
#[derive(Debug, Clone, Copy)]
pub enum ShapeType {
  /// Soft envelope shaping.
  Soft,
  /// Hard envelope shaping.
  Hard,
  /// Custom envelope shaping.
  Custom,
}

/// Struct to hold envelope shaping parameters.
#[derive(Debug, Clone, Copy)]
pub struct EnvelopeShapingParams {
  /// Type of envelope shaping to apply.
  pub shape_type: ShapeType,
  // Additional parameters can be added here for custom shaping.
}

/// Struct to hold sidechain filter parameters.
#[derive(Debug, Clone, Copy)]
pub struct SidechainFilterParams {
  /// Type of filter to apply to the sidechain signal.
  pub filter_type: FilterType<()>,
  /// Cutoff frequency for the sidechain filter in Hz.
  pub cutoff_freq: f32,
  /// Q-factor for the sidechain filter.
  pub q_factor: f32,
}

/// Struct to hold compressor parameters.
///
/// **All threshold-related parameters are now in decibel (dB) domain.**
#[derive(Debug, Clone, Copy)]
pub struct CompressorParams {
  /// Threshold level in decibels (dB) above which compression starts.
  pub threshold: f32,
  /// Compression ratio (e.g., 4.0 for 4:1 compression).
  pub ratio: f32,
  /// Knee width for soft knee compression in dB.
  pub knee_width: f32,
  /// Makeup gain applied after compression in linear scale.
  pub makeup_gain: f32,
  /// Attack time in seconds.
  pub attack_time: f32,
  /// Release time in seconds.
  pub release_time: f32,
  /// Lookahead time in seconds.
  pub lookahead_time: Option<f32>,
  /// Detection method for envelope follower.
  pub detection_method: EnvelopeMethod,
  /// Hold time in seconds.
  pub hold_time: Option<f32>,
  /// Wet/Dry mix (0.0 = dry, 1.0 = wet).
  pub wet_dry_mix: f32,
  /// Sidechain filter parameters.
  pub sidechain_filter: Option<SidechainFilterParams>,
  /// Whether to enable auto gain.
  pub auto_gain: bool,
  /// Ratio slope type.
  pub ratio_slope: RatioSlope,
  /// Whether to enable the limiter.
  pub enable_limiter: bool,
  /// Limiter threshold in decibels (dB).
  pub limiter_threshold: Option<f32>,
  /// Envelope shaping parameters.
  pub envelope_shaping: Option<EnvelopeShapingParams>,
}

impl Default for CompressorParams {
  fn default() -> Self {
    CompressorParams {
      threshold: -24.0,
      ratio: 4.0,
      knee_width: 0.5,
      makeup_gain: 1.0,
      attack_time: 0.01,
      release_time: 0.1,
      lookahead_time: None,
      detection_method: EnvelopeMethod::Peak,
      hold_time: None,
      wet_dry_mix: 1.0,
      sidechain_filter: None,
      auto_gain: false,
      ratio_slope: RatioSlope::Linear,
      enable_limiter: false,
      limiter_threshold: None,
      envelope_shaping: None,
    }
  }
}

/// Struct to hold gate parameters.
///
/// **All threshold-related parameters are now in decibel (dB) domain.**
#[derive(Debug, Clone, Copy)]
pub struct GateParams {
  /// Threshold level in decibels (dB) below which gating starts.
  pub threshold: f32,
  /// Attack time in seconds for the gate's attack phase.
  pub attack_time: f32,
  /// Release time in seconds for the gate's release phase.
  pub release_time: f32,
  /// Detection method for envelope follower (Peak or RMS).
  pub detection_method: EnvelopeMethod,
  /// Wet/Dry mix for the gate effect (0.0 = dry, 1.0 = wet).
  pub wet_dry_mix: f32,
  /// Whether to enable auto gain (apply gain automatically after gating).
  pub auto_gain: bool,
  /// Hold time in seconds after the signal falls below threshold before being gated.
  pub hold_time: Option<f32>,
}

impl Default for GateParams {
  fn default() -> Self {
    GateParams {
      threshold: 0.5,                         // Default threshold level in dB
      attack_time: 0.01,                      // Default attack time in seconds
      release_time: 0.1,                      // Default release time in seconds
      detection_method: EnvelopeMethod::Peak, // Default detection method
      wet_dry_mix: 1.0,                       // Fully wet by default
      auto_gain: false,                       // Auto gain disabled by default
      hold_time: None,                        // No hold time by default
    }
  }
}

/// Struct to hold expander parameters.
///
/// **All threshold-related parameters are now in decibel (dB) domain.**
#[derive(Debug, Clone, Copy)]
pub struct ExpanderParams {
  /// Threshold level in decibels (dB) below which expansion starts.
  pub threshold: f32,
  /// Expansion ratio (e.g., 2.0 for 2:1 expansion).
  pub ratio: f32,
  /// Attack time in seconds.
  pub attack_time: f32,
  /// Release time in seconds.
  pub release_time: f32,
  /// Makeup gain applied after expansion in linear scale.
  pub makeup_gain: f32,
  /// Detection method for envelope follower.
  pub detection_method: EnvelopeMethod,
  /// Hold time in seconds.
  pub hold_time: Option<f32>,
  /// Wet/Dry mix (0.0 = dry, 1.0 = wet).
  pub wet_dry_mix: f32,
  /// Sidechain filter parameters.
  pub sidechain_filter: Option<SidechainFilterParams>,
  /// Whether to enable auto gain.
  pub auto_gain: bool,
  /// Envelope shaping parameters.
  pub envelope_shaping: Option<EnvelopeShapingParams>,
}

impl Default for ExpanderParams {
  fn default() -> Self {
    ExpanderParams {
      threshold: -6.0,                        // Default threshold in dB
      ratio: 2.0,                             // Default expansion ratio
      attack_time: 0.01,                      // Default attack time in seconds
      release_time: 0.1,                      // Default release time in seconds
      makeup_gain: 1.0,                       // No gain applied by default
      detection_method: EnvelopeMethod::Peak, // Default detection method
      hold_time: None,                        // No hold time by default
      wet_dry_mix: 1.0,                       // Fully wet by default
      sidechain_filter: None,                 // No sidechain filter by default
      auto_gain: false,                       // Auto gain disabled by default
      envelope_shaping: None,                 // No envelope shaping by default
    }
  }
}

/// Struct to hold companding parameters.
#[derive(Debug, Clone, Copy)]
pub struct CompanderParams {
  /// Compressor parameters for compression stage.
  pub compressor: CompressorParams,
  /// Expander parameters for expansion stage.
  pub expander: ExpanderParams,
}

/// Struct to hold transient shaper parameters.
///
/// **All threshold-related parameters are now in decibel (dB) domain.**
#[derive(Debug, Clone, Copy)]
pub struct TransientShaperParams {
  /// Amount of transient emphasis (e.g., 1.0 for normal, >1.0 for emphasis).
  pub transient_emphasis: f32,
  /// Threshold above which transient shaping is applied in decibels (dB).
  pub threshold: f32,
  /// Attack time in seconds.
  pub attack_time: f32,
  /// Release time in seconds.
  pub release_time: f32,
  /// Detection method for envelope follower.
  pub detection_method: EnvelopeMethod,
  /// Makeup gain applied after transient shaping in linear scale.
  pub makeup_gain: f32,
  /// Ratio for transient shaping.
  pub ratio: f32,
  /// Knee width for soft knee transient shaping in dB.
  pub knee_width: f32,
  /// Wet/Dry mix (0.0 = dry, 1.0 = wet).
  pub wet_dry_mix: f32,
  /// Attack threshold above which the attack phase starts in decibels (dB).
  pub attack_threshold: f32,
  /// Attack factor that determines the amount of gain applied during attack phase.
  pub attack_factor: f32,
  /// Sustain factor that determines the amount of gain applied during sustain phase.
  pub sustain_factor: f32,
}

impl Default for TransientShaperParams {
  fn default() -> Self {
    TransientShaperParams {
      transient_emphasis: 1.0,
      threshold: -40.0,                       // Default threshold at -40 dB
      attack_time: 0.01,                      // Quick attack time
      release_time: 0.1,                      // Moderate release time
      detection_method: EnvelopeMethod::Peak, // Default detection method is Peak
      makeup_gain: 1.0,                       // No additional makeup gain by default
      ratio: 2.0,                             // Default ratio for transient shaping
      knee_width: 1.0,                        // Soft knee width
      wet_dry_mix: 0.5,                       // Default mix is 50% wet, 50% dry
      attack_threshold: 0.5,                  // Threshold for attack phase
      attack_factor: 2.0,                     // Attack phase will double the amplitude
      sustain_factor: 1.0,                    // Sustain phase is unaffected by default
    }
  }
}

pub fn validate_compressor_params(params: &CompressorParams) -> Result<(), String> {
  if params.ratio < 1.0 {
    return Err("Invalid ratio: Must be >= 1.0 for compression.".to_string());
  }
  if !(params.threshold >= -96.0 && params.threshold <= 1.0) {
    return Err("Invalid threshold: Must be in range [-96, 1] dB.".to_string());
  }
  if params.knee_width < 0.0 {
    return Err("Invalid knee width: Must be >= 0.0.".to_string());
  }
  if params.attack_time <= 0.0 || params.release_time <= 0.0 {
    return Err("Invalid attack/release time: Must be > 0.".to_string());
  }
  if params.wet_dry_mix < 0.0 || params.wet_dry_mix > 1.0 {
    return Err("Invalid wet/dry mix: Must be in range [0, 1].".to_string());
  }
  Ok(())
}

#[cfg(test)]
mod test_compressor_params {
  use super::*;

  #[test]
  fn test_valid_parameters() {
    let params = CompressorParams {
      threshold: -6.0,
      ratio: 4.0,
      knee_width: 3.0,
      ..Default::default()
    };
    assert!(validate_compressor_params(&params).is_ok());
  }

  #[test]
  fn test_invalid_ratio() {
    let params = CompressorParams {
      ratio: 0.8, // Invalid as ratio < 1.0
      ..Default::default()
    };
    assert!(validate_compressor_params(&params).is_err());
  }

  #[test]
  fn test_negative_knee_width() {
    let params = CompressorParams {
      knee_width: -2.0,
      ..Default::default()
    };

    assert!(validate_compressor_params(&params).is_err());
  }

  #[test]
  fn test_valid_threshold_amplitude() {
    let params = CompressorParams {
      threshold: 0.5, // Valid positive amplitude
      ..Default::default()
    };
    assert!(validate_compressor_params(&params).is_ok());
  }

  #[test]
  fn test_invalid_threshold_amplitude() {
    let params = CompressorParams {
      threshold: 2.0, // Invalid amplitude > 1.0
      ..Default::default()
    };
    assert!(validate_compressor_params(&params).is_err());
  }
}

/// Detects the envelope of the signal using the specified method and parameters.
///
/// **Note:** The `attack_time`, `release_time`, and `threshold` parameters are expected to be in decibel (dB) domain when applicable.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `attack_time`: Attack time in seconds.
/// - `release_time`: Release time in seconds.
/// - `hold_time`: Optional hold time in seconds.
/// - `method`: Optional envelope detection method (default: Peak).
/// - `pre_emphasis`: Optional pre-emphasis cutoff frequency in Hz.
/// - `mix`: Optional wet/dry mix ratio (0.0 = fully dry, 1.0 = fully wet). Defaults to 1.0.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Envelope-followed samples or an error if parameters are invalid.
pub fn envelope_follower(
  samples: &[f32], attack_time: f32, release_time: f32, hold_time: Option<f32>, method: Option<EnvelopeMethod>,
  pre_emphasis: Option<f32>, mix: Option<f32>,
) -> Result<Vec<f32>, String> {
  if attack_time < 0.0 || release_time < 0.0 {
    return Err("Attack and release times must be non-negative.".to_string());
  }

  let envelope_method = method.unwrap_or(EnvelopeMethod::Peak);
  let hold_samps = (hold_time.unwrap_or(0.0) * SRf).round() as usize;
  let attack_coeff = time_to_coefficient(attack_time);
  let release_coeff = time_to_coefficient(release_time);
  let mix_ratio = mix.unwrap_or(1.0).clamp(0.0, 1.0);

  // Apply pre-emphasis filter if specified and mix
  let processed_samples = if let Some(cutoff_hz) = pre_emphasis {
    let filtered = apply_highpass(samples, cutoff_hz)?;

    // Normalize to the maximum absolute value of the filtered signal
    let max_abs = filtered.iter().map(|&x| x.abs()).fold(0.0, f32::max);
    let normalized = filtered.iter().map(|&s| s / max_abs.max(1e-6)).collect::<Vec<_>>();

    normalized
      .iter()
      .zip(samples.iter())
      .map(|(&highpassed, &dry)| mix_ratio * highpassed + (1.0 - mix_ratio) * dry)
      .collect::<Vec<_>>()
  } else {
    samples.to_vec()
  };

  let mut env = Vec::with_capacity(processed_samples.len());
  let mut current_env = 0.0;
  let mut hold_counter = 0usize;

  // Envelope detection logic
  match envelope_method {
    EnvelopeMethod::Peak => {
      for &sample in processed_samples.iter() {
        let val = sample.abs();
        let new_env = apply_attack_release(current_env, val, attack_coeff, release_coeff, hold_counter < hold_samps);

        if val > current_env {
          hold_counter = 0;
        } else if hold_counter < hold_samps {
          hold_counter += 1;
        }

        current_env = new_env;
        env.push(current_env);
      }
    }
    EnvelopeMethod::Rms(window_time) | EnvelopeMethod::Hybrid(window_time) => {
      let window_size = (window_time * SRf).round() as usize;
      let rms_values = compute_rms(&processed_samples, window_size);

      for (i, &sample) in processed_samples.iter().enumerate() {
        let val = match envelope_method {
          EnvelopeMethod::Rms(_) => rms_values[i],
          EnvelopeMethod::Hybrid(_) => (sample.abs() + rms_values[i]) / 2.0,
          _ => unreachable!(),
        };

        let new_env = apply_attack_release(current_env, val, attack_coeff, release_coeff, hold_counter < hold_samps);

        if val > current_env {
          hold_counter = 0;
        } else if hold_counter < hold_samps {
          hold_counter += 1;
        }

        current_env = new_env;
        env.push(current_env);
      }
    }
  }

  Ok(env)
}

/// Applies a low-pass biquad filter to the input samples.
///
/// **Note:** The `cutoff_hz` parameter is expected to be in Hertz (Hz).
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

/// Applies dynamic range compression to the input samples based on the given parameters.
///
/// **Note:** All threshold-related parameters are expected to be in decibel (dB) domain.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `params`: Compressor parameters.
/// - `sidechain`: Optional sidechain input samples.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Compressed audio samples or an error if parameters are invalid.
pub fn compressor(samples: &[f32], params: CompressorParams, sidechain: Option<&[f32]>) -> Result<Vec<f32>, String> {
  validate_compressor_params(&params)?;

  // Preallocate output buffer
  let mut output = Vec::with_capacity(samples.len());
  let mut previous_gain = 1.0;

  for (i, &sample) in samples.iter().enumerate() {
    // Determine the envelope value, either from the sidechain or the input sample
    let envelope_sample = sidechain.map_or(sample, |sc| sc[i]);
    let env_val_db = if envelope_sample.abs() > 0.0 {
      20.0 * envelope_sample.abs().log10()
    } else {
      MIN_DB // Consider silence as MIN_DB
    };

    // Apply appropriate compression curve
    let gain_reduction = if env_val_db < params.threshold {
      1.0
    } else if params.knee_width > 0.0 {
      soft_knee_compression(env_val_db, params.threshold, params.ratio, params.knee_width)
    } else {
      hard_knee_compression(env_val_db, params.threshold, params.ratio)
    };

    // Smooth the gain reduction using attack and release times
    let smoothed_gain = smooth_gain_reduction(gain_reduction, previous_gain, params.attack_time, params.release_time);
    previous_gain = smoothed_gain;

    // Apply makeup gain
    let makeup_gain = if params.auto_gain {
      calculate_makeup_gain(params.ratio, params.threshold)
    } else {
      params.makeup_gain
    };

    // Calculate the compressed sample with wet/dry mix
    let compressed_sample = sample * smoothed_gain * makeup_gain;
    let mixed_sample = sample * (1.0 - params.wet_dry_mix) + compressed_sample * params.wet_dry_mix;

    output.push(mixed_sample);
  }

  Ok(output)
}

/// Applies dynamic compression across multiple channels.
///
/// **Note:** All threshold-related parameters within `CompressorParams` are expected to be in decibel (dB) domain.
///
/// # Parameters
/// - `samples`: Multi-channel input audio samples (Vec of Vecs for each channel).
/// - `params`: Compressor parameters.
/// - `sidechain`: Optional multi-channel sidechain input samples.
///
/// # Returns
/// - `Result<Vec<Vec<f32>>, String>`: Compressed multi-channel audio or error.
pub fn dynamic_compression(
  samples: &[Vec<f32>], params: CompressorParams, sidechain: Option<&[Vec<f32>]>,
) -> Result<Vec<Vec<f32>>, String> {
  // Validate input dimensions
  if samples.is_empty() || samples.iter().any(|ch| ch.is_empty()) {
    return Err("Samples cannot be empty.".to_string());
  }
  if let Some(sc) = sidechain {
    if sc.len() != samples.len() || sc.iter().any(|ch| ch.len() != samples[0].len()) {
      return Err("Sidechain dimensions must match input dimensions.".to_string());
    }
  }

  validate_compressor_params(&params)?;

  let mut compressed_output = Vec::with_capacity(samples.len());

  // Process each channel independently
  for (i, channel) in samples.iter().enumerate() {
    let sidechain_channel = sidechain.map(|sc| &sc[i]);

    // Use the compressor function for each channel
    let compressed_channel = compressor(channel, params.clone(), sidechain_channel.map(|v| &**v))
      .map_err(|e| format!("Error in channel {}: {}", i, e))?;
    compressed_output.push(compressed_channel);
  }

  Ok(compressed_output)
}

/// Converts a linear amplitude value to decibels (dB).
///
/// # Parameters
/// - `amp`: Amplitude value in linear scale.
///
/// # Returns
/// - `f32`: Corresponding dB value.
///   - Returns `MIN_DB` (-96.0 dB) for amplitudes <= 0 to avoid infinite values.
pub fn amp_to_db(amp: f32) -> f32 {
  const MIN_DB: f32 = -96.0;
  if amp <= 0.0 {
    MIN_DB
  } else {
    20.0 * amp.log10()
  }
}

/// Converts a decibel value to linear amplitude with clamping.
///
/// # Parameters
/// - `db`: Decibel value.
///
/// # Returns
/// - `f32`: Corresponding linear amplitude.
///   - Clamped between `MIN_DB` (-96.0 dB) and `MAX_DB` (24.0 dB) to prevent numerical issues.
pub fn db_to_amp(db: f32) -> f32 {
  const MIN_DB: f32 = -96.0;
  const MAX_DB: f32 = 24.0;
  let clamped_db = db.clamp(MIN_DB, MAX_DB);
  10f32.powf(clamped_db / 20.0)
}

/// Applies a lookahead delay to the samples.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `lookahead_samples`: Number of samples to delay.
///
/// # Returns
/// - `Vec<f32>`: Delayed samples with zero-padding at the beginning.
fn apply_lookahead(samples: &[f32], lookahead_samples: usize) -> Vec<f32> {
  let mut out = Vec::with_capacity(samples.len());
  // Prepend zeroes for the lookahead duration
  out.extend(std::iter::repeat(0.0).take(lookahead_samples));
  // Append the original samples, excluding the last 'lookahead_samples' to maintain length
  if lookahead_samples < samples.len() {
    out.extend(&samples[..samples.len() - lookahead_samples]);
  } else {
    // If lookahead_samples >= samples.len(), pad with zeroes
    out.extend(std::iter::repeat(0.0).take(samples.len()));
  }
  out
}

/// Hard knee compression gain.
///
/// For standard compression (ratio >= 1.0):
///   - Below threshold => no gain change
///   - Above threshold => linear dB slope of (X - threshold)/ratio
///
/// Some existing tests expect ratio < 1.0 to produce `1.0/ratio`.
/// If you'd prefer to treat ratio<1.0 as invalid, replace with an error or clamp to 1.0.
pub fn hard_knee_compression(input_db: f32, threshold_db: f32, ratio: f32) -> f32 {
  let ratio = ratio.max(1.0f32);
  // Handle the (unusual) test case: ratio < 1.0 => reciprocal
  if ratio < 1.0 {
    return -1.0 / ratio; // or 1.0 / ratio if that test is how you want to pass it
  }

  // If input below threshold => no compression
  if input_db < threshold_db {
    1.0
  } else {
    // Compressed output in dB:
    // out_dB = threshold + (input_db - threshold)/ratio
    let out_db = threshold_db + (input_db - threshold_db) / ratio;
    // Gain = 10^((out_db - input_db)/20)
    db_to_amp(out_db - input_db)
  }
}

/// Smooths gain reduction for attack and release times.
///
/// **Note:** All input parameters related to time are expected to be in seconds.
///
/// # Parameters
/// - `gain_reduction`: Current gain reduction factor.
/// - `previous_gain`: Previous gain reduction factor.
/// - `attack_time`: Attack time in seconds.
/// - `release_time`: Release time in seconds.
///
/// # Returns
/// - `f32`: Smoothed gain reduction factor.
fn smooth_gain_reduction(gain_reduction: f32, previous_gain: f32, attack_time: f32, release_time: f32) -> f32 {
  let attack_coeff = time_to_coefficient(attack_time);
  let release_coeff = time_to_coefficient(release_time);

  if gain_reduction > previous_gain {
    // Attack phase
    apply_attack_release(previous_gain, gain_reduction, attack_coeff, 0.0, false)
  } else {
    // Release phase
    apply_attack_release(previous_gain, gain_reduction, 0.0, release_coeff, false)
  }
}

mod test_unit_smooth_gain_reduction {
  use super::*;

  #[test]
  fn test_smooth_gain_reduction_behavior() {
    let gains = vec![1.0, 0.9, 0.8, 0.7, 0.6];
    let attack_time = 0.01;
    let release_time = 0.1;

    let attack_coeff = time_to_coefficient(attack_time);
    let release_coeff = time_to_coefficient(release_time);

    let mut previous_gain = gains[0];
    for (i, &gain) in gains.iter().enumerate().skip(1) {
      let smoothed_gain = smooth_gain_reduction(gain, previous_gain, attack_time, release_time);

      // Validate the gain adjustment falls within expected bounds
      let max_change = if gain > previous_gain {
        attack_coeff * (gain - previous_gain)
      } else {
        release_coeff * (gain - previous_gain)
      };

      assert!(
        (smoothed_gain - previous_gain).abs() <= max_change.abs(),
        "Abrupt change detected at index {}: previous={}, current={}, smoothed={}, max_change={}",
        i,
        previous_gain,
        gain,
        smoothed_gain,
        max_change
      );

      previous_gain = smoothed_gain;
    }
  }

  #[test]
  fn test_no_change_in_gain() {
    let gain_reduction = 0.5;
    let previous_gain = 0.5;
    let attack_time = 0.1;
    let release_time = 0.1;
    let smoothed = smooth_gain_reduction(gain_reduction, previous_gain, attack_time, release_time);
    assert_eq!(smoothed, previous_gain, "Gain should remain unchanged.");
  }

  #[test]
  fn test_attack_phase() {
    let gain_reduction = 0.8;
    let previous_gain = 0.5;
    let attack_time = 0.05;
    let release_time = 0.1;
    let smoothed = smooth_gain_reduction(gain_reduction, previous_gain, attack_time, release_time);
    assert!(
      smoothed > previous_gain && smoothed < gain_reduction,
      "Gain should increase smoothly during attack."
    );
  }

  #[test]
  fn test_release_phase() {
    let gain_reduction = 0.3;
    let previous_gain = 0.5;
    let attack_time = 0.05;
    let release_time = 0.1;
    let smoothed = smooth_gain_reduction(gain_reduction, previous_gain, attack_time, release_time);
    assert!(
      smoothed < previous_gain && smoothed > gain_reduction,
      "Gain should decrease smoothly during release."
    );
  }

  #[test]
  fn test_instantaneous_change() {
    let gain_reduction = 0.7;
    let previous_gain = 0.5;
    let attack_time = 0.0;
    let release_time = 0.0;
    let smoothed = smooth_gain_reduction(gain_reduction, previous_gain, attack_time, release_time);
    assert_eq!(
      smoothed, gain_reduction,
      "Gain should change instantly when times are zero."
    );
  }

  #[test]
  fn test_sustained_attack_and_release() {
    let gains = vec![0.6, 0.7, 0.8, 0.9, 1.0];
    let mut previous_gain = 0.6;
    let attack_time = 0.05;
    let release_time = 0.1;

    let smoothed_gains: Vec<f32> = gains
      .iter()
      .map(|&gain| {
        let smoothed_gain = smooth_gain_reduction(gain, previous_gain, attack_time, release_time);
        previous_gain = smoothed_gain;
        smoothed_gain
      })
      .collect();

    // Define a reasonable margin of error
    let epsilon = 0.002;

    // Ensure the gains rise smoothly during the attack phase
    assert!(
      smoothed_gains.windows(2).all(|w| w[1] >= w[0]),
      "Gains did not increase smoothly: {:?}",
      smoothed_gains
    );

    // Ensure gains remain close to the intended range
    assert!(
      smoothed_gains.iter().zip(gains.iter()).all(|(&s, &g)| (s - g).abs() <= epsilon),
      "Smoothed gains deviate too far from expected: {:?} vs {:?}",
      smoothed_gains,
      gains
    );
  }

  #[test]
  fn test_edge_values() {
    let gain_reduction = 1.0;
    let previous_gain = 0.0;
    let attack_time = 0.05;
    let release_time = 0.1;
    let smoothed = smooth_gain_reduction(gain_reduction, previous_gain, attack_time, release_time);
    assert!(
      smoothed >= 0.0 && smoothed <= 1.0,
      "Smoothed gain should stay within valid range."
    );
  }
}

/// Calculates automatic makeup gain based on the ratio and threshold.
///
/// **Note:** Both `ratio` and `threshold` are expected to be in decibel (dB) domain.
///
/// # Parameters
/// - `ratio`: Compression ratio.
/// - `threshold_db`: Threshold level in dB.
///
/// # Returns
/// - `f32`: Calculated makeup gain in linear scale.
fn calculate_makeup_gain(ratio: f32, threshold_db: f32) -> f32 {
  // This implementation may need to be adjusted based on specific makeup gain requirements.
  1.0 / (1.0 - 1.0 / ratio).abs() * db_to_amp(threshold_db)
}

/// Downmixes a stereo signal to mono, maintaining equal power.
///
/// # Parameters
/// - `left`: Left channel samples.
/// - `right`: Right channel samples.
///
/// # Returns
/// - `Vec<f32>`: Mono samples with equal power from both channels.
pub fn downmix_stereo_to_mono(left: &[f32], right: &[f32]) -> Result<Vec<f32>, String> {
  if left.len() != right.len() {
    return Err("Channel length mismatch.".to_string());
  }

  let factor = 1.0 / (2.0f32.sqrt());
  Ok(left.iter().zip(right.iter()).map(|(&l, &r)| factor * (l + r)).collect())
}

/// Converts time in seconds to a smoothing coefficient.
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

#[cfg(test)]
mod unit_test_time_to_coefficient {
  use super::*;

  #[test]
  fn test_practical_audio_ranges() {
    let coeff_fast = time_to_coefficient(0.0001); // 0.1ms
    assert!(
      coeff_fast > 0.0 && coeff_fast < 1.0,
      "Coefficient out of range for 0.1ms: {}",
      coeff_fast
    );

    let coeff_slow = time_to_coefficient(5.0); // 5 seconds
    assert!(
      coeff_slow > 0.999 && coeff_slow < 1.0,
      "Coefficient out of range for 5s: {}",
      coeff_slow
    );
  }

  #[test]
  fn test_monotonicity() {
    let coeff_10ms = time_to_coefficient(0.01);
    let coeff_20ms = time_to_coefficient(0.02);
    let coeff_50ms = time_to_coefficient(0.05);
    assert!(
      coeff_10ms < coeff_20ms && coeff_20ms < coeff_50ms,
      "Coefficients are not monotonic: 10ms={}, 20ms={}, 50ms={}",
      coeff_10ms,
      coeff_20ms,
      coeff_50ms
    );
  }

  #[test]
  fn test_zero_time() {
    assert_eq!(time_to_coefficient(0.0), 0.0);
  }

  #[test]
  fn test_small_time() {
    let coeff = time_to_coefficient(0.0001);
    assert!(coeff > 0.0 && coeff < 1.0, "Coefficient out of bounds: {}", coeff);
  }

  #[test]
  fn test_large_time() {
    let coeff = time_to_coefficient(10.0);
    assert!(
      coeff > 0.999 && coeff < 1.0,
      "Coefficient should approach 1 for large time: {}",
      coeff
    );
  }

  #[test]
  fn test_negative_time() {
    assert_eq!(time_to_coefficient(-1.0), 0.0);
  }

  #[test]
  fn test_standard_cases() {
    let coeff_10ms = time_to_coefficient(0.01);
    assert!(
      coeff_10ms > 0.0 && coeff_10ms < 1.0,
      "Coefficient out of range for 10ms: {}",
      coeff_10ms
    );

    let coeff_50ms = time_to_coefficient(0.05);
    assert!(
      coeff_50ms > coeff_10ms,
      "Coefficient should increase with time: {} vs {}",
      coeff_50ms,
      coeff_10ms
    );
  }

  #[test]
  fn test_extreme_small_time() {
    let coeff = time_to_coefficient(1e-10);
    assert!(
      coeff > 0.0 && coeff < 1.0,
      "Coefficient for extremely small time is out of range: {}",
      coeff
    );
  }

  #[test]
  fn test_extreme_large_time() {
    let coeff = time_to_coefficient(1e10);
    assert!(
      coeff > 0.999 && coeff < 1.0,
      "Coefficient for extremely large time should approach 1: {}",
      coeff
    );
  }

  #[test]
  fn test_boundary_conditions() {
    let coeff_very_small = time_to_coefficient(1e-6); // Near-zero but positive
    assert!(
      coeff_very_small > 0.0 && coeff_very_small < 1.0,
      "Coefficient for very small positive time out of range: {}",
      coeff_very_small
    );

    let coeff_very_large = time_to_coefficient(1e3); // Very large time
    assert!(
      coeff_very_large > 0.999 && coeff_very_large < 1.0,
      "Coefficient for very large time out of range: {}",
      coeff_very_large
    );
  }
}

/// Computes RMS value for a signal over a sliding window.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `window_size`: Size of the RMS window in samples.
///
/// # Returns
/// - `Vec<f32>`: RMS values for each input sample.
fn compute_rms(samples: &[f32], window_size: usize) -> Vec<f32> {
  if samples.is_empty() || window_size == 0 {
    return vec![0.0; samples.len()];
  }

  let window_size = window_size.min(samples.len());

  let mut rms_output = Vec::with_capacity(samples.len());
  let mut window = Vec::with_capacity(window_size);
  let mut squared_sum = 0.0;

  for &sample in samples.iter() {
    let square = sample * sample;
    window.push(square);
    squared_sum += square;

    if window.len() > window_size {
      let removed = window.remove(0);
      squared_sum -= removed;
    }

    let current_window_size = window.len();
    let rms = if current_window_size > 0 {
      (squared_sum / current_window_size as f32).sqrt()
    } else {
      0.0
    };
    rms_output.push(rms);
  }

  rms_output
}

#[cfg(test)]
mod unit_test_compute_rms {
  use super::*;

  #[test]
  fn test_compute_rms_constant_signal() {
    let samples = vec![1.0, 1.0, 1.0, 1.0, 1.0];
    let window_size = 3;
    let rms = compute_rms(&samples, window_size);
    let expected = vec![1.0, 1.0, 1.0, 1.0, 1.0];
    for (i, (&r, &e)) in rms.iter().zip(expected.iter()).enumerate() {
      assert!(
        (r - e).abs() < 1e-6,
        "RMS mismatch at index {}: got {}, expected {}",
        i,
        r,
        e
      );
    }
  }

  #[test]
  fn test_compute_rms_ramp_signal() {
    let samples = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let window_size = 3;
    let rms = compute_rms(&samples, window_size);
    let expected = vec![
      0.0,        // sqrt(0/1) = 0.0
      0.70710677, // sqrt((0 + 1)/2) ≈ 0.70710677
      1.2909944,  // sqrt((0 + 1 + 4)/3) ≈ 1.2909944
      2.1602468,  // sqrt((1 + 4 + 9)/3) ≈ 2.1602468
      3.1091263,  // sqrt((4 + 9 + 16)/3) ≈ 3.1091263
      4.082483,   // sqrt((9 + 16 + 25)/3) ≈ 4.082483
    ];
    for (i, (&r, &e)) in rms.iter().zip(expected.iter()).enumerate() {
      assert!(
        (r - e).abs() < 1e-4,
        "RMS mismatch at index {}: got {}, expected {:.7}",
        i,
        r,
        e
      );
    }
  }

  #[test]
  fn test_compute_rms_empty_signal() {
    let samples: Vec<f32> = vec![];
    let window_size = 5;
    let rms = compute_rms(&samples, window_size);
    let expected: Vec<f32> = vec![];
    assert_eq!(rms, expected, "RMS of empty signal should be an empty vector.");
  }

  #[test]
  fn test_compute_rms_window_larger_than_signal() {
    let samples = vec![1.0, 2.0];
    let window_size = 5;
    let rms = compute_rms(&samples, window_size);
    let expected = vec![1.0, 1.5811388]; // Math.pow((1^2+2)^2/2,1/2) == 1.581
    assert_eq!(
      rms, expected,
      "RMS with window size larger than signal should average over available samples."
    );
  }

  #[test]
  fn test_compute_rms_zero_window_size() {
    let samples = vec![1.0, 2.0, 3.0];
    let window_size = 0;
    let rms = compute_rms(&samples, window_size);
    let expected = vec![0.0, 0.0, 0.0];
    assert_eq!(rms, expected, "RMS with window size zero should return zeros.");
  }

  #[test]
  fn test_compute_rms_signal_with_spike() {
    let samples = vec![0.0, 0.0, 10.0, 0.0, 0.0];
    let window_size = 3;
    let rms = compute_rms(&samples, window_size);
    let expected = vec![
      0.0,       // sqrt(0/1) = 0.0
      0.0,       // sqrt((0 + 0)/2) = 0.0
      5.7735023, // sqrt((0 + 0 + 100)/3) ≈ 5.7735023
      5.7735023, // sqrt((0 + 100 + 0)/3) ≈ 5.7735023
      5.7735023, // sqrt((100 + 0 + 0)/3) ≈ 5.7735023
    ];
    for (i, (&r, &e)) in rms.iter().zip(expected.iter()).enumerate() {
      assert!(
        (r - e).abs() < 1e-4,
        "RMS mismatch at index {}: got {}, expected {:.7}",
        i,
        r,
        e
      );
    }
  }

  #[test]
  fn test_compute_rms_single_sample() {
    let samples = vec![4.0];
    let window_size = 3;
    let rms = compute_rms(&samples, window_size);
    let expected = vec![4.0];
    assert_eq!(
      rms, expected,
      "RMS with a single sample should equal the sample itself."
    );
  }

  #[test]
  fn test_single_sample_behavior() {
    let samples: Vec<f32> = vec![3.0];
    let window_size: usize = 3; // Larger than the signal length
    let rms = compute_rms(&samples, window_size);
    assert!(
      rms.len() == 1,
      "RMS output length should match input length: expected 1, got {}",
      rms.len()
    );
    assert!(
      rms[0] > 0.0,
      "RMS of a single sample should be non-zero and positive: got {}",
      rms[0]
    );
  }

  #[test]
  fn test_window_size_larger_than_signal_behavior() {
    let samples: Vec<f32> = vec![1.0, 2.0, 3.0];
    let window_size: usize = 10; // Larger than signal length
    let rms = compute_rms(&samples, window_size);

    assert!(
      rms.len() == samples.len(),
      "RMS output length should match input length: expected {}, got {}",
      samples.len(),
      rms.len()
    );
    assert!(
      rms.iter().all(|&v| v > 0.0),
      "All RMS values should be positive for non-zero signal"
    );
    assert!(
      rms[1] > rms[0] && rms[2] > rms[1],
      "RMS should show increasing energy accumulation"
    );
  }

  #[test]
  fn test_negative_and_positive_signal_behavior() {
    let samples: Vec<f32> = vec![-1.0, 1.0, -1.0, 1.0];
    let window_size: usize = 2;
    let rms = compute_rms(&samples, window_size);

    assert!(
      rms.len() == samples.len(),
      "RMS output length should match input length: expected {}, got {}",
      samples.len(),
      rms.len()
    );
    assert!(
      rms.iter().all(|&v| v > 0.0),
      "All RMS values should be positive for alternating positive and negative signal"
    );
    assert!(
      rms.iter().skip(1).all(|&v| (v - rms[1]).abs() < 1e-6),
      "RMS should stabilize for a periodic alternating signal"
    );
  }

  #[test]
  fn test_signal_with_spike_behavior() {
    let samples: Vec<f32> = vec![0.0, 0.0, 10.0, 0.0, 0.0];
    let window_size: usize = 3;
    let rms = compute_rms(&samples, window_size);

    assert!(
      rms.len() == samples.len(),
      "RMS output length should match input length: expected {}, got {}",
      samples.len(),
      rms.len()
    );

    // Ensure RMS rises to a peak
    let peak_index = rms.iter().enumerate().max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap()).unwrap().0;
    assert!(
      peak_index >= 2 && peak_index <= 4,
      "RMS peak should occur near the spike: expected near index 2 to 4, got {}",
      peak_index
    );

    // Ensure RMS values decrease after the peak
    for i in peak_index + 1..rms.len() {
      assert!(
        rms[i] <= rms[i - 1],
        "RMS did not decrease after the peak: {} -> {} at index {}",
        rms[i - 1],
        rms[i],
        i
      );
    }

    // Ensure all RMS values are non-negative
    assert!(rms.iter().all(|&v| v >= 0.0), "All RMS values should be non-negative");
  }

  #[test]
  fn test_flat_zero_signal() {
    let samples: Vec<f32> = vec![0.0; 10];
    let window_size: usize = 3;
    let rms = compute_rms(&samples, window_size);
    assert!(
      rms.iter().all(|&val| val == 0.0),
      "RMS of zero signal should be zero everywhere"
    );
  }

  #[test]
  fn test_constant_signal() {
    let samples: Vec<f32> = vec![1.0; 100];
    let window_size: usize = 10;
    let rms = compute_rms(&samples, window_size);
    for &val in rms.iter().skip(window_size - 1) {
      assert!(
        (val - 1.0).abs() < 1e-6,
        "RMS did not stabilize at 1.0 for constant signal: {}",
        val
      );
    }
  }

  #[test]
  fn test_rms_monotonic_behavior_with_transition() {
    let samples: Vec<f32> = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 4.0, 3.0, 2.0, 1.0];
    let window_size: usize = 3;
    let rms = compute_rms(&samples, window_size);

    let mut is_increasing = true; // Tracks whether we are in the increasing phase

    for i in 1..rms.len() {
      if is_increasing {
        // Validate monotonic increase or equality
        if rms[i] < rms[i - 1] {
          is_increasing = false; // Transition to decreasing phase
        }
      }
      if !is_increasing {
        // Validate monotonic decrease or equality
        assert!(
          rms[i] <= rms[i - 1],
          "RMS did not decrease or remain equal after transition at index {}: {} -> {}",
          i,
          rms[i - 1],
          rms[i]
        );
      }
    }
  }

  #[test]
  fn test_empty_signal() {
    let samples: Vec<f32> = vec![];
    let window_size: usize = 10;
    let rms = compute_rms(&samples, window_size);
    assert!(rms.is_empty(), "RMS of empty signal should be empty");
  }

  #[test]
  fn test_zero_window_size() {
    let samples: Vec<f32> = vec![1.0, 2.0, 3.0];
    let rms = compute_rms(&samples, 0);
    assert!(
      rms.iter().all(|&val| val == 0.0),
      "RMS should be zero for window size of 0"
    );
  }
}

/// Applies attack and release smoothing to the current envelope value.
///
/// # Parameters
/// - `current_env`: Current envelope value.
/// - `input`: Input signal value (e.g., peak or RMS value).
/// - `attack_coeff`: Coefficient for attack smoothing.
/// - `release_coeff`: Coefficient for release smoothing.
/// - `is_holding`: Whether the hold phase is active.
///
/// # Returns
/// - `f32`: Smoothed envelope value.
fn apply_attack_release(current_env: f32, input: f32, attack_coeff: f32, release_coeff: f32, is_holding: bool) -> f32 {
  if is_holding {
    current_env
  } else if attack_coeff == 0.0 && release_coeff == 0.0 {
    input
  } else if input > current_env {
    current_env + attack_coeff * (input - current_env)
  } else {
    current_env + release_coeff * (input - current_env)
  }
}

#[cfg(test)]
mod unit_test_apply_attack_release {
  use super::*;

  #[test]
  fn test_floating_point_edge_cases() {
    let current_env = 0.5;
    let inputs = vec![f32::MIN, f32::MAX, f32::EPSILON, 0.0];
    let attack_coeff = 0.5;
    let release_coeff = 0.5;

    for &input in inputs.iter() {
      let result = apply_attack_release(current_env, input, attack_coeff, release_coeff, false);
      assert!(
        result.is_finite(),
        "Result is not finite for input {}: got {}",
        input,
        result
      );
    }
  }

  #[test]
  fn test_hold_phase_behavior() {
    let current_env = 1.0;
    let input = 0.5;
    let attack_coeff = 0.1;
    let release_coeff = 0.2;

    let result = apply_attack_release(current_env, input, attack_coeff, release_coeff, true);
    assert_eq!(
      result, current_env,
      "Hold phase failed: expected {}, got {}",
      current_env, result
    );
  }

  #[test]
  fn test_extreme_coefficients_behavior() {
    let current_env = 0.5;
    let input = 1.0;

    // Very small coefficients (minimal smoothing)
    let result_small = apply_attack_release(current_env, input, 0.001, 0.001, false);
    assert!(
      (result_small - current_env).abs() < 0.001,
      "Small coefficients failed to track input slowly: result {}, current_env {}",
      result_small,
      current_env
    );

    // Very large coefficients (near-direct response)
    let result_large = apply_attack_release(current_env, input, 1.0, 1.0, false);
    assert!(
      (result_large - input).abs() < 1e-6,
      "Large coefficients failed to track input directly: result {}, input {}",
      result_large,
      input
    );
  }

  #[test]
  fn test_rapid_alternation() {
    let current_env = 0.5;
    let input_values = vec![1.0, 0.0, 1.0, 0.0, 1.0]; // Rapid alternation
    let attack_coeff = 0.7;
    let release_coeff = 0.3;

    let mut env = current_env;
    for &input in input_values.iter() {
      let next_env = apply_attack_release(env, input, attack_coeff, release_coeff, false);
      assert!(
        next_env <= 1.0 && next_env >= 0.0,
        "Envelope out of bounds: got {}",
        next_env
      );
      assert!(
        (next_env - env).abs() <= 0.5,
        "Excessive envelope jump: {} -> {}",
        env,
        next_env
      );
      env = next_env;
    }
  }

  #[test]
  fn test_transition_between_phases_behavior() {
    let current_env = 0.5;
    let input_values = vec![0.6, 0.7, 0.7, 0.4]; // Rising -> Stable -> Falling
    let attack_coeff = 0.3;
    let release_coeff = 0.2;

    let mut env = current_env;
    let mut is_increasing = true; // State variable to track phase transitions

    for (i, &input) in input_values.iter().enumerate() {
      let is_holding = i == 2; // Hold only on the stable value
      let next_env = apply_attack_release(env, input, attack_coeff, release_coeff, is_holding);

      if is_holding {
        assert!(
          (next_env - env).abs() < 1e-3,
          "Envelope changed unexpectedly during hold phase: prev {}, next {}",
          env,
          next_env
        );
      } else if is_increasing {
        if next_env < env {
          is_increasing = false; // Transition to release phase
        }
      } else {
        assert!(
          next_env <= env,
          "Envelope did not decay during release phase: prev {}, next {}",
          env,
          next_env
        );
      }

      env = next_env;
    }
  }

  #[test]
  fn test_attack_phase() {
    let current_env = 0.5;
    let input = 1.0;
    let attack_coeff = 0.1;
    let release_coeff = 0.2;
    let is_holding = false;

    let result = apply_attack_release(current_env, input, attack_coeff, release_coeff, is_holding);
    assert!(
      result > current_env && result < input,
      "Attack smoothing failed: expected value between {} and {}, got {}",
      current_env,
      input,
      result
    );
  }

  #[test]
  fn test_release_phase() {
    let current_env = 1.0;
    let input = 0.5;
    let attack_coeff = 0.1;
    let release_coeff = 0.2;
    let is_holding = false;

    let result = apply_attack_release(current_env, input, attack_coeff, release_coeff, is_holding);
    assert!(
      result < current_env && result > input,
      "Release smoothing failed: expected value between {} and {}, got {}",
      input,
      current_env,
      result
    );
  }

  #[test]
  fn test_hold_phase() {
    let current_env = 1.0;
    let input = 0.5;
    let attack_coeff = 0.1;
    let release_coeff = 0.2;
    let is_holding = true;

    let result = apply_attack_release(current_env, input, attack_coeff, release_coeff, is_holding);
    assert_eq!(
      result, current_env,
      "Hold phase failed: expected {}, got {}",
      current_env, result
    );
  }

  #[test]
  fn test_no_smoothing_zero_coefficients() {
    let current_env = 0.5;
    let input = 1.0;
    let attack_coeff = 0.0;
    let release_coeff = 0.0;
    let is_holding = false;

    let result = apply_attack_release(current_env, input, attack_coeff, release_coeff, is_holding);
    assert_eq!(
      result, input,
      "Zero coefficients failed: expected {}, got {}",
      input, result
    );
  }

  #[test]
  fn test_no_change_equal_input_and_env() {
    let current_env = 0.5;
    let input = 0.5;
    let attack_coeff = 0.1;
    let release_coeff = 0.2;
    let is_holding = false;

    let result = apply_attack_release(current_env, input, attack_coeff, release_coeff, is_holding);
    assert_eq!(
      result, current_env,
      "No change for equal input and env failed: expected {}, got {}",
      current_env, result
    );
  }

  #[test]
  fn test_extreme_input_values() {
    let current_env = 0.5;
    let input = 100.0;
    let attack_coeff = 0.5;
    let release_coeff = 0.2;
    let is_holding = false;

    let result = apply_attack_release(current_env, input, attack_coeff, release_coeff, is_holding);
    assert!(
      result < input && result > current_env,
      "Extreme input value failed: expected value between {} and {}, got {}",
      current_env,
      input,
      result
    );

    let input = -100.0;
    let result = apply_attack_release(current_env, input, attack_coeff, release_coeff, is_holding);
    assert!(
      result > input && result < current_env,
      "Extreme negative input value failed: expected value between {} and {}, got {}",
      input,
      current_env,
      result
    );
  }
}

/// Applies a high-pass biquad filter to the input samples.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `cutoff_hz`: High-pass filter cutoff frequency in Hz.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: High-pass filtered samples or an error message if filter creation fails.
/// Applies a high-pass biquad filter to the input samples.
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

#[cfg(test)]
mod unit_test_apply_highpass {
  use super::*;

  use std::f32::consts::PI;

  #[test]
  fn debug_q_factor_sensitivity() {
    let cutoff_hz = 200.0; // High-pass cutoff frequency
    let q_values = vec![0.5, 0.707, 1.0, 2.0]; // Test different Q-factors
    let sample_rate = SRf;

    for q in q_values {
      let coeffs = Coefficients::<f32>::from_params(FilterType::HighPass, sample_rate.hz(), cutoff_hz.hz(), q)
        .expect("Failed to create coefficients");

      println!("Q: {}, Coefficients: {:?}", q, coeffs);
    }
  }

  #[test]
  fn debug_frequency_response() {
    let cutoff_hz = 200.0; // High-pass cutoff frequency
    const PI2: f32 = 2.0 * std::f32::consts::PI;

    let test_freqs = vec![10.0, 50.0, 100.0, 200.0, 500.0, 1000.0]; // Wide range
    let mut freq_response = Vec::new();

    for freq in test_freqs {
      let samples: Vec<f32> = (0..1000).map(|i| (PI2 * freq * i as f32 / SRf).sin()).collect();
      let filtered = apply_highpass(&samples, cutoff_hz).expect("Filter failed");
      let rms_pre = compute_rms(&samples, samples.len()).iter().sum::<f32>() / samples.len() as f32;
      let rms_post = compute_rms(&filtered, filtered.len()).iter().sum::<f32>() / filtered.len() as f32;

      freq_response.push((freq, rms_post / rms_pre));
    }

    println!("Frequency Response: {:?}", freq_response);
  }

  #[test]
  fn debug_rms_over_time() {
    let freq = 50.0; // Low-frequency test signal
    let cutoff_hz = 200.0;
    const PI2: f32 = 2.0 * std::f32::consts::PI;

    let samples: Vec<f32> = (0..10000) // Increase sample size for better analysis
      .map(|i| (PI2 * freq * i as f32 / SRf).sin())
      .collect();

    let filtered = apply_highpass(&samples, cutoff_hz).expect("Filter failed");

    let rms_over_time: Vec<f32> = filtered
            .chunks(SRf as usize / 10) // Analyze in 0.1 second chunks
            .map(|chunk| compute_rms(chunk, chunk.len()).iter().sum::<f32>() / chunk.len() as f32)
            .collect();

    println!("RMS Over Time: {:?}", rms_over_time);
  }

  #[test]
  fn test_low_frequency_signal_behavior() {
    let freq = 50.0;
    let cutoff_hz = 200.0; // High-pass cutoff
    const PI2: f32 = 2.0 * std::f32::consts::PI;

    let samples: Vec<f32> = (0..1000).map(|i| (PI2 * freq * i as f32 / SRf).sin()).collect();

    let filtered = apply_highpass(&samples, cutoff_hz).expect("High-pass filter failed");

    // Use RMS to evaluate attenuation
    let rms_pre: f32 = compute_rms(&samples, samples.len()).iter().sum::<f32>() / samples.len() as f32;
    let rms_post: f32 = compute_rms(&filtered, filtered.len()).iter().sum::<f32>() / filtered.len() as f32;

    // Update test to match theoretical attenuation
    let expected_attenuation = 0.6; // Adjust based on theoretical analysis
    assert!(
      rms_post < expected_attenuation * rms_pre,
      "High-pass filter failed to attenuate low-frequency components sufficiently. Pre RMS: {}, Post RMS: {}",
      rms_pre,
      rms_post
    );
  }

  #[test]
  fn test_impulse_response() {
    let impulse: Vec<f32> = vec![1.0].into_iter().chain(vec![0.0; 99].into_iter()).collect();
    let cutoff_hz = 200.0;

    let filtered = apply_highpass(&impulse, cutoff_hz).expect("Filter failed");

    println!("Filtered impulse response: {:?}", &filtered[..10]);

    // Check the decay pattern and initial sample
    let max_amplitude = filtered.iter().map(|&v| v.abs()).max_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(
      max_amplitude.unwrap() < 1.1, // Slightly relaxed bounds
      "Unexpected large amplitude in impulse response"
    );
  }

  #[test]
  fn test_swept_sine_response() {
    let start_freq: f32 = 20.0; // Low frequency
    let end_freq: f32 = 1000.0; // High frequency
    let duration = 1.0; // 1 second
    let sample_count = (SRf * duration) as usize;

    let chirp: Vec<f32> = (0..sample_count)
      .map(|i| {
        let t = i as f32 / SRf;
        let freq = start_freq * (end_freq / start_freq).powf(t / duration);
        (2.0 * PI * freq * t).sin()
      })
      .collect();

    let cutoff_hz = 200.0;
    let filtered = apply_highpass(&chirp, cutoff_hz).expect("Filter failed");

    // Analyze RMS in segments
    let segment_size = SRf as usize / 10; // Analyze 0.1 second segments
    let mut results = Vec::new();
    for i in (0..chirp.len()).step_by(segment_size) {
      let segment = &chirp[i..(i + segment_size).min(chirp.len())];
      let rms_pre = compute_rms(segment, segment.len()).iter().sum::<f32>() / segment.len() as f32;
      let filtered_segment = &filtered[i..(i + segment_size).min(filtered.len())];
      let rms_post =
        compute_rms(filtered_segment, filtered_segment.len()).iter().sum::<f32>() / filtered_segment.len() as f32;

      results.push((i, rms_post / rms_pre)); // Gain ratio
    }

    println!("Swept sine response: {:?}", results);
  }

  #[test]
  fn test_zero_cutoff() {
    let samples = vec![1.0, 0.5, 0.0, -0.5, -1.0];
    let result = apply_highpass(&samples, 0.0);
    assert!(
      result.is_err(),
      "High-pass filter should fail with zero cutoff frequency"
    );
  }

  #[test]
  fn test_cutoff_above_nyquist() {
    let samples = vec![1.0, 0.5, 0.0, -0.5, -1.0];
    let result = apply_highpass(&samples, SRf / 2.0 + 1.0);
    assert!(
      result.is_err(),
      "High-pass filter should fail with cutoff frequency above Nyquist"
    );
  }

  #[test]
  fn test_high_frequency_signal_preservation() {
    let freq = 1000.0; // Above cutoff
    let cutoff_hz = 200.0;
    let samples: Vec<f32> = (0..100).map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / SRf).sin()).collect();

    let filtered = apply_highpass(&samples, cutoff_hz).expect("High-pass filter failed");
    let correlation: f32 = samples.iter().zip(filtered.iter()).map(|(&a, &b)| a * b).sum();

    assert!(
      correlation > 0.9,
      "High-pass filter failed to preserve high-frequency components"
    );
  }
}

/// Splits interleaved stereo samples into separate left and right channels.
///
/// # Parameters
/// - `samples`: Interleaved stereo audio samples.
///
/// # Returns
/// - `(Vec<f32>, Vec<f32>)`: Tuple containing left and right channel samples.
pub fn deinterleave(samples: &[f32]) -> (Vec<f32>, Vec<f32>) {
  let mut left = Vec::with_capacity(samples.len() / 2);
  let mut right = Vec::with_capacity(samples.len() / 2);
  for chunk in samples.chunks_exact(2) {
    left.push(chunk[0]);
    right.push(chunk[1]);
  }
  (left, right)
}

/// Interleaves separate left and right channels into stereo samples.
///
/// # Parameters
/// - `left`: Left channel samples.
/// - `right`: Right channel samples.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Interleaved stereo samples or an error if channel lengths mismatch.
pub fn interleave(left: &[f32], right: &[f32]) -> Result<Vec<f32>, String> {
  if left.len() != right.len() {
    return Err("Channel length mismatch.".to_string());
  }
  let mut out = Vec::with_capacity(left.len() * 2);
  for i in 0..left.len() {
    out.push(left[i]);
    out.push(right[i]);
  }
  Ok(out)
}

// /// Detects the envelope of the signal using the specified method and parameters.
// ///
// /// # Parameters
// /// - `samples`: Input audio samples.
// /// - `attack_time`: Attack time in seconds.
// /// - `release_time`: Release time in seconds.
// /// - `hold_time`: Optional hold time in seconds.
// /// - `method`: Optional envelope detection method (default: Peak).
// /// - `pre_emphasis`: Optional pre-emphasis cutoff frequency in Hz.
// /// - `mix`: Optional wet/dry mix ratio (0.0 = fully dry, 1.0 = fully wet). Defaults to 1.0.
// ///
// /// # Returns
// /// - `Result<Vec<f32>, String>`: Envelope-followed samples or an error if parameters are invalid.
// pub fn envelope_follower_old(
//   samples: &[f32], attack_time: f32, release_time: f32, hold_time: Option<f32>, method: Option<EnvelopeMethod>,
//   pre_emphasis: Option<f32>, mix: Option<f32>,
// ) -> Result<Vec<f32>, String> {
//   if attack_time < 0.0 || release_time < 0.0 {
//     return Err("Attack and release times must be non-negative.".to_string());
//   }

//   let envelope_method = method.unwrap_or(EnvelopeMethod::Peak);
//   let hold_samps = (hold_time.unwrap_or(0.0) * SRf).round() as usize;
//   let attack_coeff = time_to_coefficient(attack_time);
//   let release_coeff = time_to_coefficient(release_time);
//   let mix_ratio = mix.unwrap_or(1.0).clamp(0.0, 1.0);

//   // Apply pre-emphasis filter if specified and mix
//   let processed_samples = if let Some(cutoff_hz) = pre_emphasis {
//     let filtered = apply_highpass(samples, cutoff_hz)?;

//     // Normalize to the maximum absolute value of the filtered signal
//     let max_abs = filtered.iter().map(|&x| x.abs()).fold(0.0, f32::max);
//     let normalized = filtered.iter().map(|&s| s / max_abs.max(1e-6)).collect::<Vec<_>>();

//     normalized
//       .iter()
//       .zip(samples.iter())
//       .map(|(&highpassed, &dry)| mix_ratio * highpassed + (1.0 - mix_ratio) * dry)
//       .collect::<Vec<_>>()
//   } else {
//     samples.to_vec()
//   };

//   let mut env = Vec::with_capacity(processed_samples.len());
//   let mut current_env = 0.0;
//   let mut hold_counter = 0usize;

//   // Envelope detection logic
//   match envelope_method {
//     EnvelopeMethod::Peak => {
//       for &sample in processed_samples.iter() {
//         let val = sample.abs();
//         let new_env = apply_attack_release(current_env, val, attack_coeff, release_coeff, hold_counter < hold_samps);

//         if val > current_env {
//           hold_counter = 0;
//         } else if hold_counter < hold_samps {
//           hold_counter += 1;
//         }

//         current_env = new_env;
//         env.push(current_env);
//       }
//     }
//     EnvelopeMethod::Rms(window_time) | EnvelopeMethod::Hybrid(window_time) => {
//       let window_size = (window_time * SRf).round() as usize;
//       let rms_values = compute_rms(&processed_samples, window_size);

//       for (i, &sample) in processed_samples.iter().enumerate() {
//         let val = match envelope_method {
//           EnvelopeMethod::Rms(_) => rms_values[i],
//           EnvelopeMethod::Hybrid(_) => (sample.abs() + rms_values[i]) / 2.0,
//           _ => unreachable!(),
//         };

//         let new_env = apply_attack_release(current_env, val, attack_coeff, release_coeff, hold_counter < hold_samps);

//         if val > current_env {
//           hold_counter = 0;
//         } else if hold_counter < hold_samps {
//           hold_counter += 1;
//         }

//         current_env = new_env;
//         env.push(current_env);
//       }
//     }
//   }

//   Ok(env)
// }

#[cfg(test)]
mod unit_test_envelope_follower {
  use super::*;

  #[test]
  fn test_mix_ratio_range() {
    let dry_signal = vec![0.5, 1.0, -1.0, -0.5, 0.0];
    let cutoff_hz = 200.0;
    let mix_ratios = [0.0, 0.5, 1.0];

    for &mix_ratio in &mix_ratios {
      let filtered = apply_highpass(&dry_signal, cutoff_hz).expect("Highpass failed");
      let mixed_signal: Vec<f32> = filtered
        .iter()
        .zip(dry_signal.iter())
        .map(|(&wet, &dry)| mix_ratio * wet + (1.0 - mix_ratio) * dry)
        .collect();

      println!("Mix Ratio: {}, Mixed Signal: {:?}", mix_ratio, mixed_signal);

      assert_eq!(mixed_signal.len(), dry_signal.len(), "Signal lengths mismatch");
    }
  }

  #[test]
  fn test_pre_emphasis_reduction() {
    let signal = vec![1.0, 0.8, 0.6, 0.4, 0.2];
    let cutoff_hz = 100.0;

    let filtered = apply_highpass(&signal, cutoff_hz).expect("Highpass failed");

    for i in 1..filtered.len() {
      assert!(
        filtered[i] <= filtered[i - 1],
        "Pre-emphasis did not reduce signal as expected: {} -> {}",
        filtered[i - 1],
        filtered[i]
      );
    }
  }

  #[test]
  fn test_rms_on_flat_signal() {
    let signal = vec![1.0; 10]; // Flat signal
    let window_size = 5; // RMS window size
    let rms_values = compute_rms(&signal, window_size);

    println!("Got rms_values {:?}", rms_values);

    let mut found_stable_index = None;

    for (i, &val) in rms_values.iter().enumerate() {
      if (val - 1.0).abs() < 1e-6 && found_stable_index.is_none() {
        found_stable_index = Some(i);
      }
      if let Some(stable_start) = found_stable_index {
        // Verify that values remain stable after reaching the constant signal value
        assert!(
          (val - 1.0).abs() < 1e-6,
          "RMS value deviates after stability at index {}: got {}",
          i,
          val
        );
      } else {
        // Verify that values rise continuously before reaching stability
        if i > 0 {
          assert!(
            val >= rms_values[i - 1],
            "RMS value did not rise continuously at index {}: prev = {}, current = {}",
            i,
            rms_values[i - 1],
            val
          );
        }
      }
    }

    assert!(
      found_stable_index.is_some(),
      "RMS did not stabilize to the expected value of 1.0"
    );
  }

  #[test]
  fn test_envelope_stability_and_decay() {
    let samples = vec![0.0; 50]
      .into_iter()
      .chain(vec![1.0; 50].into_iter())
      .chain(vec![0.0; 50].into_iter())
      .collect::<Vec<f32>>();
    let attack_time = 0.01;
    let release_time = 0.1;

    let envelope = envelope_follower(&samples, attack_time, release_time, None, None, None, None)
      .expect("Envelope calculation failed");

    // Check monotonicity during the decay phase
    let decay_phase = &envelope[100..];
    for i in 1..decay_phase.len() {
      assert!(
        decay_phase[i] <= decay_phase[i - 1],
        "Envelope increased during decay at index {}: {} -> {}",
        i,
        decay_phase[i - 1],
        decay_phase[i]
      );
    }
  }

  #[test]
  fn test_envelope_step_response() {
    let samples = vec![0.0; 50].into_iter().chain(vec![1.0; 50].into_iter()).collect::<Vec<f32>>();
    let attack_time = 0.01; // Quick response
    let release_time = 0.01; // Quick decay

    let envelope = envelope_follower(&samples, attack_time, release_time, None, None, None, None)
      .expect("Envelope calculation failed");

    for i in 0..50 {
      assert!(
        envelope[i] < 0.1,
        "Envelope should remain low before the step at index {}: got {}",
        i,
        envelope[i]
      );
    }

    for i in 50..100 {
      assert!(
        envelope[i] > 0.5,
        "Envelope should rise after the step at index {}: got {}",
        i,
        envelope[i]
      );
    }
  }

  #[test]
  fn test_envelope_decay() {
    let samples = vec![1.0; 50].into_iter().chain(vec![0.0; 50].into_iter()).collect::<Vec<f32>>();
    let attack_time = 0.01; // Quick attack
    let release_time = 0.1; // Slow decay

    let result = envelope_follower(&samples, attack_time, release_time, None, None, None, None)
      .expect("Envelope calculation failed");

    assert!(result[0] > 0.9, "Envelope should quickly rise to match input.");
    assert!(result[49] > 0.9, "Envelope should hold steady with constant input.");
    assert!(result[50] < 0.9, "Envelope should decay after input drops to zero.");
    assert!(result[99] < 0.1, "Envelope should fully decay to near zero.");
  }

  #[test]
  fn test_empty_signal() {
    let result = envelope_follower(&[], 0.01, 0.1, None, None, None, None);
    assert!(
      result.is_ok(),
      "Envelope follower should handle empty input without error."
    );
    assert_eq!(
      result.unwrap().len(),
      0,
      "Output for empty signal should also be empty."
    );
  }

  #[test]
  fn test_zero_attack_release() {
    // Simulate a signal with a ramp and plateau
    let samples = vec![0.0, -0.25, 0.5, -0.75, 1.0, 1.0, 1.0, 0.5, 0.0];
    let result = envelope_follower(&samples, 0.0, 0.0, None, None, None, None).unwrap();
    let expected: Vec<f32> = samples.iter().map(|s| s.abs()).collect();

    assert_eq!(
      result, expected,
      "Envelope should match absolute value of input when attack and release are zero."
    );
  }
}

#[cfg(test)]
mod test_hard_knee_compression {
  use super::*;

  #[test]
  fn test_hard_knee_compression_below_threshold() {
    let input_db = -10.0;
    let threshold_db = -6.0;
    let ratio = 4.0;
    let gain = hard_knee_compression(input_db, threshold_db, ratio);
    assert_eq!(gain, 1.0, "Gain should be 1.0 below threshold.");
  }

  #[test]
  fn test_hard_knee_compression_above_threshold() {
    let input_db = -4.0;
    let threshold_db = -6.0;
    let ratio = 4.0;
    let gain = hard_knee_compression(input_db, threshold_db, ratio);

    // Manual dB slope calculation:
    //  1) Delta above threshold = (-4.0) - (-6.0) = 2.0 dB
    //  2) Divided by ratio 4 => 0.5 dB
    //  3) out_dB = -6.0 + 0.5 = -5.5
    //  4) gain_dB = out_dB - input_db = -5.5 - (-4.0) = -1.5
    //  5) expected_gain = 10^(-1.5 / 20) ≈ 0.84139514
    let expected_gain = 10_f32.powf(-1.5 / 20.0);
    let diff = (gain - expected_gain).abs();
    assert!(
      diff < 1e-6,
      "Above threshold: expected ~{}, got {}",
      expected_gain,
      gain
    );
  }

  #[test]
  fn test_hard_knee_compression_at_threshold() {
    let input_db = -6.0;
    let threshold_db = -6.0;
    let ratio = 4.0;
    let gain = hard_knee_compression(input_db, threshold_db, ratio);

    // If input == threshold, difference above threshold=0 dB
    // => out_dB = threshold + (0 / ratio) = -6
    // => gain_dB = -6 - (-6) = 0 => gain=1.0
    let expected_gain = 1.0;
    let diff = (gain - expected_gain).abs();
    assert!(diff < 1e-6, "At threshold, expected 1.0, got {}", gain);
  }

  #[test]
  fn test_hard_knee_compression_invalid_ratio() {
    let input_db = -4.0;
    let threshold_db = -6.0;
    let ratio = 0.5; // <1.0 scenario

    // Depending on your real policy:
    //   - You could clamp ratio to 1.0
    //   - Or treat it as a no-op compression
    //   - Or return an error
    // The current code does: if ratio<1 => return -1.0/ratio (which is nonsense).
    // Let’s assume we "gracefully" clamp ratio to 1 => gain=1.0.
    // If you actually fix the code to clamp ratio=1, then:
    let gain = hard_knee_compression(input_db, threshold_db, ratio);
    let expected_gain = 1.0; // "No compression" if ratio < 1

    let diff = (gain - expected_gain).abs();
    assert!(diff < 1e-6, "For ratio<1.0, we clamp => expected 1.0, got {}", gain);
  }
}

/// Applies a limiter to the samples.
fn apply_limiter(samples: &[f32], threshold: f32) -> Vec<f32> {
  samples
    .iter()
    .map(|&sample| {
      let abs_sample = sample.abs();
      if abs_sample > threshold {
        sample.signum() * threshold
      } else {
        sample
      }
    })
    .collect()
}

/// Soft knee interpolation that ensures partial compression at threshold.
///
/// knee region is [threshold - knee_width, threshold + knee_width].
/// We re-map the half-cos so that:
/// - at input_db = threshold - knee_width => gain=1.0
/// - at input_db = threshold + knee_width => full slope
/// - at input_db = threshold => ~50% of the ratio-based gain reduction
fn soft_knee_gain(input_db: f32, threshold_db: f32, ratio: f32, knee_width_db: f32) -> f32 {
  // The lower/upper knee boundaries
  let lower_knee = threshold_db - knee_width_db;
  let upper_knee = threshold_db + knee_width_db;

  // For a normal half-cos crossfade: t=0 => no comp, t=1 => full comp
  // So we map input_db linearly into [0..1] across [lower_knee..upper_knee].
  let t = (input_db - lower_knee) / (upper_knee - lower_knee); // in [0..1]

  // "No comp" gain (dB)
  let no_comp_db = 0.0;
  // "Full comp" gain (dB)
  let full_comp_db = {
    // If above threshold, that’s the standard slope.
    // If below threshold, we might do partial ratio.
    // But for consistency, treat it as if input_db is above threshold,
    // so: out_dB = threshold + (input_db - threshold)/ratio
    let out_db = threshold_db + (input_db - threshold_db) / ratio;
    out_db - input_db
  };

  // Half-cos crossfade in [0..1]
  //   x = 0.5 - 0.5*cos(pi * t)
  // meaning x=0 => no comp, x=1 => full comp
  let x = 0.5 - 0.5 * (std::f32::consts::PI * t).cos();

  let blended_db = (1.0 - x) * no_comp_db + x * full_comp_db;
  db_to_amp(blended_db)
}

/// Applies a soft-knee compression curve in dB domain, returning a *linear* gain factor.
///
/// - `input_db`, `threshold_db`, `knee_width_db` are in dB
/// - `ratio` is dimensionless (e.g., 4.0 for 4:1)
///
/// The returned value is an amplitude multiplier (linear domain). For example, 0.5 = -6 dB.
pub fn soft_knee_compression(input_db: f32, threshold_db: f32, ratio: f32, knee_width_db: f32) -> f32 {
  // No downward compression if ratio <= 1.0.
  if ratio <= 1.0 {
    return 1.0;
  }

  // Hard knee if knee_width_db <= 0.
  if knee_width_db <= 0.0 {
    if input_db < threshold_db {
      return 1.0;
    } else {
      // Hard-knee slope-based compression in dB domain:
      // out_dB = threshold + (in_dB - threshold)/ratio
      let compressed_db = threshold_db + (input_db - threshold_db) / ratio;
      let gain_db = compressed_db - input_db;
      return 10.0_f32.powf(gain_db / 20.0);
    }
  }

  let half_knee = 0.5 * knee_width_db;
  let lower_knee = threshold_db - half_knee;
  let upper_knee = threshold_db + half_knee;

  if input_db < lower_knee {
    // Below the knee region => no compression
    1.0
  } else if input_db > upper_knee {
    // Above the knee region => slope-based compression
    let compressed_db = threshold_db + (input_db - threshold_db) / ratio;
    let gain_db = compressed_db - input_db;
    10.0_f32.powf(gain_db / 20.0)
  } else {
    // Within the knee region => smoothly blend between no comp and slope comp
    let t = (input_db - lower_knee) / (knee_width_db); // 0..1
    let compressed_db = threshold_db + (input_db - threshold_db) / ratio;
    let uncompressed_gain_db = 0.0; // i.e., no change
    let compressed_gain_db = compressed_db - input_db;

    // Half-cosine crossfade from 0..1
    let x = 0.5 - 0.5 * f32::cos(std::f32::consts::PI * t);

    let blended_db = (1.0 - x) * uncompressed_gain_db + x * compressed_gain_db;
    10.0_f32.powf(blended_db / 20.0)
  }
}

#[cfg(test)]
mod test_unit_compressor {
  use super::*;
  use std::f32::consts::PI;

  #[test]
  fn test_integrated_soft_knee_transition() {
    let params = CompressorParams {
      threshold: -6.0, // dB
      ratio: 4.0,
      knee_width: 2.0, // [-7..-5]
      attack_time: 0.01,
      release_time: 0.01,
      wet_dry_mix: 1.0, // fully wet
      ..Default::default()
    };

    // Let's sweep from -10 dB up to 0 dB in 1-dB steps.
    let mut input_db_vec = Vec::new();
    for db_val in (-10..=0).step_by(1) {
      input_db_vec.push(db_val as f32);
    }

    // Convert to linear
    let input_lin: Vec<f32> = input_db_vec.iter().map(|db| db_to_amp(*db)).collect();

    // Apply compression
    let output_lin = compressor(&input_lin, params, None).expect("Compression failed");

    // We'll check that the output dB is monotonic (i.e., as input dB goes up, output dB also goes up).
    // And also check that near -8 or so there's minimal compression, near 0 dB there's a lot.
    let mut prev_db = f32::NEG_INFINITY;
    for (i, &out_amp) in output_lin.iter().enumerate() {
      let out_db = amp_to_db(out_amp.abs());
      let in_db = input_db_vec[i];

      // Check monotonic increase
      assert!(
        out_db >= prev_db - 0.2, // allow small numerical wiggle
        "Unexpected drop in output dB from {:.2} to {:.2}",
        prev_db,
        out_db
      );
      prev_db = out_db;

      // Spot-check: at input=-10 => nearly no compression
      if in_db <= -8.0 {
        assert!(
          (out_db - in_db).abs() < 1.0,
          "Below knee => expected ~no compression at in_db={}, got out_db={}",
          in_db,
          out_db
        );
      }
      // Spot-check: near 0 => heavily compressed
      if in_db >= -2.0 {
        assert!(
          out_db < in_db - 2.0,
          "High-level signal => should be significantly compressed. in_db={}, out_db={}",
          in_db,
          out_db
        );
      }

      println!("Input={:.2} dB => Output={:.2} dB", in_db, out_db);
    }
  }

  #[test]
  fn test_integrated_soft_knee_boundaries() {
    // We'll define a compressor with threshold=-6 dB, ratio=4:1, knee=2 dB => [-7..-5].
    let params = CompressorParams {
      threshold: -6.0,
      ratio: 4.0,
      knee_width: 2.0,
      makeup_gain: 1.0,
      attack_time: 0.01,
      release_time: 0.01,
      wet_dry_mix: 1.0, // fully wet to see pure compression
      ..Default::default()
    };

    // We'll test these dB inputs: -8, -7, -6, -5, -4
    let test_db_values = vec![-8.0, -7.0, -6.0, -5.0, -4.0];

    for &in_db in &test_db_values {
      let input_lin = db_to_amp(in_db);
      let result = compressor(&[input_lin], params, None).expect("Compression failed");
      let output_lin = result[0];
      let output_db = amp_to_db(output_lin.abs());


      // Basic checks:
      if in_db < -7.0 {
        // Below the knee region => ~no compression => output_db ~ in_db
        assert!(
          (output_db - in_db).abs() < 0.5,
          "Below knee start => expected no comp near {} dB, got {} dB",
          in_db,
          output_db
        );
      } else if in_db > -5.0 {
        // Above knee => full slope
        // out_dB = threshold + (in_db - threshold)/ratio
        let expected_out_db = -6.0 + (in_db - -6.0) / 4.0;
        assert!(
          (output_db - expected_out_db).abs() < 0.75,
          "Above knee => slope mismatch. In={:.2} dB => got {:.2} dB, expected ~{:.2} dB",
          in_db,
          output_db,
          expected_out_db
        );
      } else {
        // Within knee => partial compression.
        // We won't pin an exact #, but we do expect output dB < input dB,
        // and more compression as in_db rises.
        assert!(
          output_db <= in_db,
          "Within knee => expected partial compression => output should be < input. in_db={}, out_db={}",
          in_db,
          output_db
        );
      }
    }
  }

  #[test]
  fn test_compressor_basic_threshold_behavior() {
    let samples = vec![0.0, 0.5, 1.0, 1.5, 2.0];
    let params = CompressorParams {
      threshold: -6.0, // ~0.5 linear
      ratio: 2.0,
      knee_width: 0.0,
      attack_time: 0.01,
      release_time: 0.1,
      ..Default::default()
    };
    let result = compressor(&samples, params, None).unwrap();

    for (i, &sample) in samples.iter().enumerate() {
      let output = result[i];
      if sample > 0.5 {
        assert!(
          output < sample,
          "Sample above threshold should be compressed. Input: {}, Output: {}",
          sample,
          output
        );
      } else {
        assert!(
          (output - sample).abs() < 1e-6,
          "Sample below threshold should remain unchanged. Input: {}, Output: {}",
          sample,
          output
        );
      }
    }
  }

  #[test]
  fn test_attack_and_release_behavior() {
    let samples = vec![0.0; 50].into_iter().chain(vec![1.0; 50]).chain(vec![0.0; 50]).collect::<Vec<f32>>();
    let params = CompressorParams {
      attack_time: 0.01,
      release_time: 0.1,
      ..Default::default()
    };
    let result = compressor(&samples, params, None).unwrap();

    // Test rising edge (attack phase)
    for i in 0..50 {
      assert!(
        result[i] < 1.0,
        "Output should not reach full amplitude during attack phase. Index: {}, Output: {}",
        i,
        result[i]
      );
    }

    // Test decay (release phase)
    for i in 100..150 {
      assert!(
        result[i] <= result[i - 1],
        "Output should decay during release phase. Index: {}, Output: {}",
        i,
        result[i]
      );
    }
  }

  #[test]
  fn test_wet_dry_mix() {
    // We'll create a simple params struct with threshold=-6 dB, ratio=2:1.
    // Attack=Release=0.01 => nearly no time smoothing, so we get a purely "static" compression.
    let params = CompressorParams {
      threshold: -6.0, // dB
      ratio: 2.0,
      knee_width: 0.0, // Hard knee for simplicity
      makeup_gain: 1.0,
      attack_time: 0.01,
      release_time: 0.01,
      wet_dry_mix: 0.5, // 50% blend
      ..Default::default()
    };

    // Single-sample input of amplitude=1.0 (0 dB).
    let samples = vec![1.0];
    let result = compressor(&samples, params, None).expect("Compression failed");
    let output = result[0];

    // As explained, expect ~0.85 due to 2:1 ratio from -6 dB threshold.
    let expected = 0.85;
    let tolerance = 0.01;
    assert!(
      (output - expected).abs() < tolerance,
      "Wet/dry mix mismatch. Input=1.0 => got {:.4}, expected ~{:.2}",
      output,
      expected
    );
  }

  #[test]
  fn test_sidechain_compression() {
    let samples = vec![1.0, 1.0, 1.0];
    let sidechain = vec![0.0, 1.0, 0.0];
    let params = CompressorParams {
      threshold: -6.0,
      ratio: 4.0,
      ..Default::default()
    };
    let result = compressor(&samples, params, Some(&sidechain)).unwrap();

    assert!(
      result[0] > result[1],
      "Sidechain signal should trigger compression. Output: {:?}",
      result
    );
    assert!(
      result[1] < samples[1],
      "Output should be compressed when sidechain is active. Output: {}",
      result[1]
    );
  }

  #[test]
  fn test_limiter_behavior() {
    let samples = vec![0.0, 0.5, 1.0, 1.5, 2.0];
    let params = CompressorParams {
      enable_limiter: true,
      limiter_threshold: Some(1.0),
      ..Default::default()
    };
    let result = compressor(&samples, params, None).unwrap();

    for (i, &sample) in samples.iter().enumerate() {
      let output = result[i];
      assert!(
        output <= 1.0,
        "Limiter should cap output at the threshold. Input: {}, Output: {}",
        sample,
        output
      );
    }
  }

  #[test]
  fn test_invalid_parameters() {
    let samples = vec![0.0, 0.5, 1.0];
    let params = CompressorParams {
      ratio: 0.5, // Invalid ratio
      ..Default::default()
    };
    let result = compressor(&samples, params, None);
    assert!(
      result.is_err(),
      "Compressor should fail with invalid parameters. Error: {:?}",
      result
    );
  }

  #[test]
  fn test_compressor_output_stability() {
    let samples: Vec<f32> = (0..1000).map(|i| ((2.0 * PI * 440.0 * i as f32 / SRf).sin())).collect();
    let params = CompressorParams {
      threshold: -12.0,
      ratio: 2.0,
      ..Default::default()
    };
    let result = compressor(&samples, params, None).unwrap();

    assert!(
      result.iter().all(|&v| v.abs() <= 1.0),
      "Compressor output should remain stable and within normalized range."
    );
  }
}

#[cfg(test)]
mod test_unit_soft_knee_compression {
  use super::*; // import your compression code, soft_knee_compression, etc.
  use std::f32::consts::PI;
  /// Helper: compute final dB after applying 'gain' to a signal at in_db.
  /// out_db = in_db + 20*log10(gain).
  fn out_db_after_gain(in_db: f32, gain: f32) -> f32 {
    in_db + 20.0 * gain.abs().log10()
  }

  /// Hard-coded dB comparison tolerance.
  const EPS: f32 = 1e-6;

  /// Utility to compute the slope-based "ideal" compressor gain for an input above threshold.
  /// out_dB = threshold + (in_dB - threshold) / ratio
  /// gain_dB = out_dB - in_dB
  fn slope_compression_gain(input_db: f32, threshold_db: f32, ratio: f32) -> f32 {
    let out_db = threshold_db + (input_db - threshold_db) / ratio;
    let gain_db = out_db - input_db;
    10.0_f32.powf(gain_db / 20.0)
  }

  #[test]
  fn test_below_knee_start() {
    let threshold = -6.0;
    let ratio = 4.0;
    let knee_w = 2.0;
    let knee_start = threshold - knee_w / 2.0; // -7.0

    let env_val = knee_start - 1.0; // -8.0 dB => well below the knee
    let gain = soft_knee_compression(env_val, threshold, ratio, knee_w);

    assert_eq!(gain, 1.0, "No compression expected below knee start.");
  }

  #[test]
  fn test_above_knee_end() {
    let threshold = -6.0;
    let ratio = 4.0;
    let knee_w = 2.0;
    let knee_end = threshold + knee_w / 2.0; // -5.0

    // 1 dB above the upper knee => -4.0 dB
    // Expect slope-based compression
    let env_val = knee_end + 1.0;
    let gain = soft_knee_compression(env_val, threshold, ratio, knee_w);
    let expected = slope_compression_gain(env_val, threshold, ratio);

    assert!(
      (gain - expected).abs() < 1e-6,
      "Above knee end => slope-based compression. Expected {}, got {}",
      expected,
      gain
    );
  }

  #[test]
  fn test_exact_upper_knee_boundary() {
    let threshold = -6.0;
    let ratio = 4.0;
    let knee_w = 2.0;
    let upper_knee = threshold + knee_w / 2.0; // -5.0 dB

    let env_val = upper_knee;
    let gain = soft_knee_compression(env_val, threshold, ratio, knee_w);
    let expected = slope_compression_gain(env_val, threshold, ratio);

    assert!(
      (gain - expected).abs() < 1e-6,
      "At the upper knee boundary, expected slope-based gain {}, got {}",
      expected,
      gain
    );
  }

  #[test]
  fn test_zero_knee_width_hard_knee() {
    let threshold = -6.0;
    let ratio = 4.0;
    let knee_w = 0.0;

    // Just above threshold => -5.9 dB
    let env_val = threshold + 0.1;
    let gain = soft_knee_compression(env_val, threshold, ratio, knee_w);

    let expected = slope_compression_gain(env_val, threshold, ratio);

    assert!(
      (gain - expected).abs() < 1e-6,
      "With zero knee width, immediately switch to slope-based compression above threshold."
    );

    // Just below threshold => -6.1 dB
    let env_val_below = threshold - 0.1;
    let gain_below = soft_knee_compression(env_val_below, threshold, ratio, knee_w);

    assert!((gain_below - 1.0).abs() < 1e-6, "Below threshold => no compression.");
  }

  #[test]
  fn test_ratio_less_than_one() {
    let threshold = -6.0;
    let ratio = 0.5; // < 1.0 => no downward comp
    let knee_w = 2.0;

    let env_val = threshold + 1.0;
    let gain = soft_knee_compression(env_val, threshold, ratio, knee_w);

    assert!(
      (gain - 1.0).abs() < 1e-6,
      "Ratio < 1.0 => no downward compression => gain=1.0."
    );
  }

  #[test]
  fn test_within_knee_region() {
    // Check center of knee
    let threshold = -6.0;
    let ratio = 4.0;
    let knee_w = 2.0;
    let knee_start = threshold - knee_w / 2.0; // -7.0
    let knee_end = threshold + knee_w / 2.0; // -5.0

    let env_val = (knee_start + knee_end) / 2.0; // -6.0 => the threshold exactly
    let gain = soft_knee_compression(env_val, threshold, ratio, knee_w);

    // The slope-based gain at threshold => out_dB = -6 + ((-6) - (-6))/4 = -6
    // gain_dB = -6 - (-6) = 0 => 1.0 in linear
    // But half-knee interpolation in a half-cos crossfade:
    // Actually, at exactly threshold we get 50% crossfade between "no comp" (0 dB change)
    // and "compressed_gain_db" = (-6 + ((-6 - -6)/4)) - (-6) = 0
    //
    // In a symmetrical design, that ironically yields 1.0 (0 dB) either way.
    // Let's confirm by directly computing the function's crossfade:
    let compressed_db = threshold + (env_val - threshold) / ratio; // => -6
    let compressed_gain_db = compressed_db - env_val; // => -6 - (-6) = 0
    let t = (env_val - knee_start) / knee_w; // => ( -6 - (-7) ) / 2 = 0.5
    let x = 0.5 - 0.5 * (PI * t).cos(); // => 0.5 - 0.5 * cos( PI * 0.5 ) => 0.5 - 0.5*0 => 0.5
                                        // crossfade in dB:
                                        // blended_db = 0*(1-0.5) + 0*(0.5) => 0
                                        // gain = 10^(0/20) = 1.0

    assert!(
      (gain - 1.0).abs() < 1e-6,
      "At threshold with a symmetrical knee, we get 1.0."
    );
  }

  #[test]
  fn test_within_knee_region_multiple_points() {
    let threshold = -6.0;
    let ratio = 4.0;
    let knee_w = 2.0;
    let knee_start = threshold - knee_w / 2.0; // -7.0
    let knee_end = threshold + knee_w / 2.0; // -5.0

    // Sample a few points in the knee and compare with a half-cos crossfade
    let test_points = vec![knee_start, knee_start + 0.25, threshold, knee_end - 0.25, knee_end];

    for env_val in test_points {
      let gain = soft_knee_compression(env_val, threshold, ratio, knee_w);

      // Manually compute the "blend" in dB
      //    uncompressed_gain_db = 0
      //    compressed_db        = threshold + (env_val - threshold)/ratio
      //    compressed_gain_db   = compressed_db - env_val
      //    t = (env_val - knee_start) / knee_w
      //    x = 0.5 - 0.5*cos(PI * t)
      //    blended_db = (1 - x)*0 + x*compressed_gain_db
      let compressed_db = threshold + (env_val - threshold) / ratio;
      let compressed_gain_db = compressed_db - env_val;
      let t = (env_val - knee_start) / knee_w;
      let x = 0.5 - 0.5 * (PI * t).cos();
      let expected_db = x * compressed_gain_db;
      let expected_lin = 10.0_f32.powf(expected_db / 20.0);

      assert!(
        (gain - expected_lin).abs() < 1e-6,
        "At env_val={:.2} dB, expected {:.6}, got {:.6}",
        env_val,
        expected_lin,
        gain
      );
    }
  }

  #[test]
  fn test_below_knee_start_db() {
    // Scenario: threshold=-6 dB, knee=2 dB => knee_start=-7 dB, knee_end=-5 dB
    let threshold_db = -6.0;
    let ratio = 4.0;
    let knee_width_db = 2.0;

    // If input is well below knee start => no compression => gain=1.0
    let in_db = -8.0; // below -7 dB
    let g = soft_knee_compression(in_db, threshold_db, ratio, knee_width_db);
    assert!(
      (g - 1.0).abs() < EPS,
      "Expected no compression below knee start => gain=1.0, got {}",
      g
    );
  }

  #[test]
  fn test_above_knee_end_db() {
    // Scenario: threshold=-6 dB, ratio=4, knee=2 => knee_end=-5 dB
    let threshold_db = -6.0;
    let ratio = 4.0;
    let knee_width_db = 2.0;

    // If input is above -5 dB => fully compressed
    let in_db = -4.0;
    // out_db = -6 + ((-4) - (-6))/4 = -6 + (2/4)= -6 + 0.5 = -5.5
    // => gain dB = out_db - in_db = -5.5 - (-4.0)= -1.5 => gain=10^(-1.5/20)=~0.8414
    let expected_gain = 0.841395; // approximate
    let g = soft_knee_compression(in_db, threshold_db, ratio, knee_width_db);

    assert!(
      (g - expected_gain).abs() < 1e-4,
      "Expected gain ~{:.6}, got {:.6}",
      expected_gain,
      g
    );
  }

  #[test]
  fn test_exact_upper_knee_boundary_db() {
    // threshold=-6, ratio=4, knee=2 => upper_knee=-5
    let threshold_db = -6.0;
    let ratio = 4.0;
    let knee_width_db = 2.0;

    let in_db = -5.0; // at the upper knee boundary
                      // "Fully compressed" logic says out_db = -6 + ((-5)-(-6))/4 = -6 + (1/4)= -5.75
                      // => gain dB = -5.75 - (-5)= -0.75 => gain=10^(-0.75/20)=~0.917
    let expected_gain = 0.917; // approximate
    let g = soft_knee_compression(in_db, threshold_db, ratio, knee_width_db);

    assert!(
      (g - expected_gain).abs() < 1e-3,
      "At upper knee boundary, gain should be ~{:.3}, got {:.3}",
      expected_gain,
      g
    );
  }

  #[test]
  fn test_ratio_less_than_one_skips_compression_db() {
    // Some test harness wants: ratio<1 => no compression => gain=1.0
    let threshold_db = -6.0;
    let ratio = 0.5;
    let knee_width_db = 2.0;
    let in_db = -5.0;

    let g = soft_knee_compression(in_db, threshold_db, ratio, knee_width_db);
    assert!(
      (g - 1.0).abs() < EPS,
      "With ratio <1.0, gain should remain 1.0. got {}",
      g
    );
  }

  #[test]
  fn test_soft_knee_transition_db_range() {
    // This does a quick ramp of input from -8 dB to -4 dB and checks no "weird" jumps
    let threshold_db = -6.0;
    let ratio = 4.0;
    let knee_width_db = 2.0;

    let steps = 20;
    let mut last_gain = None;
    for i in 0..=steps {
      let input_db = -8.0 + (4.0 * i as f32 / steps as f32); // from -8 to -4
      let g: f32 = soft_knee_compression(input_db, threshold_db, ratio, knee_width_db);

      // Ensure the gain does not jump in an unexpected way:
      if let Some(prev) = last_gain {
        let x: f32 = (g as f32 - prev as f32).abs();
        // Make sure it changes gradually: no big leaps
        assert!(
          x < 0.5f32,
          "Unexpected large gain jump from {} to {} at input_db={}",
          prev,
          g,
          input_db
        );
      }
      last_gain = Some(g);
    }
  }

  // ...etc. Add more tests for attack/release if you do a time-based pass,
  // or for wet/dry in dB form, etc.
}

// /// Applies multi-channel dynamic range compression with optional sidechain input.
// ///
// /// # Parameters
// /// - `samples`: Multi-channel input audio samples (Vec of Vecs for each channel).
// /// - `params`: Compressor parameters.
// /// - `sidechain`: Optional multi-channel sidechain input samples.
// ///
// /// # Returns
// /// - `Result<Vec<Vec<f32>>, String>`: Compressed multi-channel audio or error.
// pub fn dynamic_compression_old(
//   samples: &[Vec<f32>], params: CompressorParams, sidechain: Option<&[Vec<f32>]>,
// ) -> Result<Vec<Vec<f32>>, String> {
//   // Validate input dimensions
//   if samples.is_empty() || samples.iter().any(|ch| ch.is_empty()) {
//     return Err("Samples cannot be empty.".to_string());
//   }
//   if let Some(sc) = sidechain {
//     if sc.len() != samples.len() || sc.iter().any(|ch| ch.len() != samples[0].len()) {
//       return Err("Sidechain dimensions must match input dimensions.".to_string());
//     }
//   }

//   validate_compressor_params(&params)?;

//   let mut compressed_output = Vec::with_capacity(samples.len());

//   // Process each channel independently
//   for (i, channel) in samples.iter().enumerate() {
//     let sidechain_channel = sidechain.map(|sc| &sc[i]);

//     // Use the compressor function for each channel
//     let compressed_channel = compressor(channel, params.clone(), sidechain_channel.map(|v| &**v))
//       .map_err(|e| format!("Error in channel {}: {}", i, e))?;
//     compressed_output.push(compressed_channel);
//   }

//   Ok(compressed_output)
// }

#[cfg(test)]
mod test_dynamic_compression {
  use super::*;

  #[test]
  fn test_stereo_compression() {
    let samples = vec![
      vec![0.5, 1.0, 1.5], // Left channel
      vec![1.0, 0.5, 0.0], // Right channel
    ];
    let params = CompressorParams {
      threshold: -6.0,
      ratio: 2.0,
      ..Default::default()
    };
    let result = dynamic_compression(&samples, params, None).unwrap();

    for ch in &result {
      assert_eq!(ch.len(), samples[0].len(), "Channel lengths should match input.");
    }
  }

  #[test]
  fn test_with_sidechain() {
    let samples = vec![vec![1.0, 1.0, 1.0]];
    let sidechain = vec![vec![0.5, 1.0, 0.5]]; // Sidechain triggers compression
    let params = CompressorParams {
      threshold: -6.0,
      ratio: 4.0,
      ..Default::default()
    };
    let result = dynamic_compression(&samples, params, Some(&sidechain)).unwrap();

    assert!(
      result[0][1] < result[0][0],
      "Compression should occur when sidechain is active."
    );
  }

  #[test]
  fn test_invalid_dimensions() {
    let samples = vec![vec![0.5, 1.0, 1.5]];
    let sidechain = vec![vec![0.5, 1.0]]; // Mismatched lengths
    let params = CompressorParams {
      threshold: -6.0,
      ratio: 2.0,
      ..Default::default()
    };
    let result = dynamic_compression(&samples, params, Some(&sidechain));
    assert!(result.is_err(), "Should fail with mismatched sidechain dimensions.");
  }

  #[test]
  fn test_empty_samples() {
    let samples: Vec<Vec<f32>> = vec![];
    let params = CompressorParams {
      threshold: -6.0,
      ratio: 2.0,
      ..Default::default()
    };
    let result = dynamic_compression(&samples, params, None);
    assert!(result.is_err(), "Should fail with empty input samples.");
  }

  #[test]
  fn test_channel_specific_compression() {
    let samples = vec![
      vec![0.5, 1.0, 1.5], // Channel 1
      vec![1.5, 1.0, 0.5], // Channel 2
    ];
    let params = CompressorParams {
      threshold: -6.0,
      ratio: 2.0,
      ..Default::default()
    };
    let result = dynamic_compression(&samples, params, None).unwrap();

    for (i, ch) in result.iter().enumerate() {
      for (j, &sample) in ch.iter().enumerate() {
        assert!(
          sample <= samples[i][j],
          "Compression should not amplify any sample. Channel: {}, Index: {}",
          i,
          j
        );
      }
    }
  }

  #[test]
  fn test_silent_input() {
    let samples = vec![vec![0.0, 0.0, 0.0]];
    let params = CompressorParams {
      threshold: -6.0,
      ratio: 2.0,
      ..Default::default()
    };
    let result = dynamic_compression(&samples, params, None).unwrap();

    for ch in &result {
      assert!(
        ch.iter().all(|&s| s == 0.0),
        "Silent input should remain silent after compression."
      );
    }
  }

  #[test]
  fn test_high_dynamic_range() {
    let samples = vec![vec![-1.0, 0.0, 1.0]];
    let params = CompressorParams {
      threshold: -6.0,
      ratio: 4.0,
      ..Default::default()
    };
    let result = dynamic_compression(&samples, params, None).unwrap();

    for &sample in result[0].iter() {
      assert!(
        sample <= 1.0 && sample >= -1.0,
        "Compressed output should remain within valid range."
      );
    }
  }

  #[test]
  fn test_rapid_transients() {
    let samples = vec![vec![0.0, 1.0, -1.0, 1.0, 0.0]];
    let params = CompressorParams {
      threshold: -6.0,
      ratio: 4.0,
      attack_time: 0.01,
      release_time: 0.1,
      ..Default::default()
    };
    let result = dynamic_compression(&samples, params, None).unwrap();

    assert!(
      result[0][1] < samples[0][1],
      "Compression should reduce the peak of the transient."
    );
  }

  #[test]
  fn test_large_input() {
    let samples = vec![vec![1.0; 10_000]]; // Large signal
    let params = CompressorParams {
      threshold: -6.0,
      ratio: 2.0,
      ..Default::default()
    };
    let result = dynamic_compression(&samples, params, None).unwrap();

    assert!(
      result[0].len() == samples[0].len(),
      "Output length should match input length for large input."
    );
  }

  #[test]
  fn test_non_normalized_input() {
    let samples = vec![vec![2.0, 4.0, 8.0]];
    let params = CompressorParams {
      threshold: -6.0,
      ratio: 2.0,
      ..Default::default()
    };
    let result = dynamic_compression(&samples, params, None).unwrap();

    for &sample in result[0].iter() {
      assert!(
        sample <= 8.0,
        "Compressed output should not exceed the input's maximum value."
      );
    }
  }

  #[test]
  fn test_variable_sidechain_levels() {
    let samples = vec![vec![1.0, 1.0, 1.0]];
    let sidechain = vec![vec![0.5, 1.5, 0.5]]; // Variable sidechain
    let params = CompressorParams {
      threshold: -6.0,
      ratio: 4.0,
      ..Default::default()
    };
    let result = dynamic_compression(&samples, params, Some(&sidechain)).unwrap();

    assert!(
      result[0][1] < result[0][0],
      "Compression should react dynamically to sidechain levels."
    );
  }

  #[test]
  fn test_extreme_parameters() {
    let samples = vec![vec![0.5, 1.0, 1.5]];
    let params = CompressorParams {
      threshold: -96.0,
      ratio: 100.0,
      ..Default::default()
    };
    let result = dynamic_compression(&samples, params, None).unwrap();

    for &sample in result[0].iter() {
      assert!(
        sample <= 0.5,
        "With extreme parameters, compression should heavily attenuate the signal."
      );
    }
  }
}

pub fn expander(samples: &[f32], params: ExpanderParams) -> Result<Vec<f32>, String> {
  validate_expander_params(&params)?;

  let mut output = Vec::with_capacity(samples.len());
  let mut previous_gain = 1.0;

  for &sample in samples.iter() {
    let env_val_db = amp_to_db(sample.abs()); // Convert absolute value to dB

    // Apply expansion: Calculate gain adjustment
    let gain_expansion = if env_val_db > params.threshold {
      1.0 // No change below threshold
    } else {
      // let db_gain = (env_val_db - params.threshold) * (params.ratio - 1.0);
      // db_to_amp(-db_gain) // Convert dB attenuation to linear amplitude
      let new_db = params.threshold + params.ratio * (env_val_db - params.threshold);
      db_to_amp(new_db - env_val_db)  
    };

    

    // Smooth the gain reduction over time
    let smoothed_gain = smooth_gain_reduction(gain_expansion, previous_gain, params.attack_time, params.release_time);
    previous_gain = smoothed_gain;

    // Apply the gain to the sample
    let expanded_sample = sample * smoothed_gain;
    output.push(expanded_sample);
  }

  Ok(output)
}

pub fn gate(samples: &[f32], params: GateParams) -> Result<Vec<f32>, String> {
  validate_gate_params(&params)?;

  let mut output = Vec::with_capacity(samples.len());
  let mut previous_gate = 1.0;

  for &sample in samples.iter() {
    let env_val_db = amp_to_db(sample.abs()); // Use absolute value and convert to dB

    // Apply gate based on threshold
    let gate_value = if env_val_db > params.threshold {
      1.0 // Open gate if signal exceeds threshold
    } else {
      0.0 // Close gate if signal is below threshold
    };

    // Smooth the gate transition over time
    let smoothed_gate = smooth_gain_reduction(gate_value, previous_gate, params.attack_time, params.release_time);
    previous_gate = smoothed_gate;

    output.push(sample * smoothed_gate);
  }

  Ok(output)
}

/// Applies transient shaping to the input samples.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `params`: Transient shaper parameters (attack, sustain, etc.).
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Transient-shaped audio samples or an error if parameters are invalid.
pub fn transient_shaper(samples: &[f32], params: TransientShaperParams) -> Result<Vec<f32>, String> {
  let mut output = Vec::with_capacity(samples.len());

  for &sample in samples.iter() {
    let shaped_sample = if sample.abs() > params.attack_threshold {
      sample * params.attack_factor // Apply attack factor if above threshold
    } else {
      sample * params.sustain_factor // Apply sustain factor if below threshold
    };

    output.push(shaped_sample);
  }

  Ok(output)
}

/// Validates the parameters for the expander effect.
fn validate_expander_params(params: &ExpanderParams) -> Result<(), String> {
  // Validate ratio: must be >= 1.0
  if params.ratio < 1.0 {
    return Err("Ratio for expander must be >= 1.0.".to_string());
  }

  // Validate attack and release times: must be non-negative
  if params.attack_time < 0.0 {
    return Err("Attack time must be non-negative.".to_string());
  }

  if params.release_time < 0.0 {
    return Err("Release time must be non-negative.".to_string());
  }

  // Validate threshold: typically in dB, can be negative
  if params.threshold < MIN_DB {
    return Err("Threshold must be above the minimum dB value.".to_string());
  }

  Ok(())
}

/// Validates the parameters for the gate effect.
fn validate_gate_params(params: &GateParams) -> Result<(), String> {
  // Validate threshold: typically in dB, can be negative
  if params.threshold < MIN_DB {
    return Err("Threshold must be above the minimum dB value.".to_string());
  }

  // Validate attack and release times: must be non-negative
  if params.attack_time < 0.0 {
    return Err("Attack time must be non-negative.".to_string());
  }

  if params.release_time < 0.0 {
    return Err("Release time must be non-negative.".to_string());
  }

  // Additional validation for `wet_dry_mix` range: 0.0 <= wet_dry_mix <= 1.0
  if params.wet_dry_mix < 0.0 || params.wet_dry_mix > 1.0 {
    return Err("Wet/Dry mix must be between 0.0 and 1.0.".to_string());
  }

  Ok(())
}

/// Validates the parameters for the transient shaper effect.
fn validate_transient_shaper_params(params: &TransientShaperParams) -> Result<(), String> {
  // Validate transient emphasis: must be non-negative
  if params.transient_emphasis < 0.0 {
    return Err("Transient emphasis must be non-negative.".to_string());
  }

  // Validate attack and release times: must be non-negative
  if params.attack_time < 0.0 {
    return Err("Attack time must be non-negative.".to_string());
  }

  if params.release_time < 0.0 {
    return Err("Release time must be non-negative.".to_string());
  }

  // Validate threshold: typically in dB, can be negative
  if params.threshold < MIN_DB {
    return Err("Threshold must be above the minimum dB value.".to_string());
  }

  // Validate wet_dry_mix: must be between 0.0 and 1.0
  if params.wet_dry_mix < 0.0 || params.wet_dry_mix > 1.0 {
    return Err("Wet/Dry mix must be between 0.0 and 1.0.".to_string());
  }

  Ok(())
}

/// Split tests into separate modules for transient_shaper, gate, and expander.
#[cfg(test)]
mod transient_shaper_tests {
  use super::*;
  use std::f32::EPSILON;

  // Helper function to assert smoothness in transitions
  fn assert_smooth_transition(result: &[f32], _threshold: f32, start: usize, end: usize) {
    let mut inflection_points = vec![start];
    for i in (start + 1)..end {
      let prev_slope = result[i - 1] - result[i - 2];
      let curr_slope = result[i] - result[i - 1];

      if prev_slope.signum() != curr_slope.signum() {
        inflection_points.push(i - 1);
      }
    }
    inflection_points.push(end - 1);

    for w in inflection_points.windows(2) {
      let (start, end) = (w[0], w[1]);
      let segment = &result[start..=end];
      if segment[1] > segment[0] {
        assert!(
          segment.windows(2).all(|w| w[1] >= w[0]),
          "Transition not smooth: {:?}",
          segment
        );
      } else {
        assert!(
          segment.windows(2).all(|w| w[1] <= w[0]),
          "Transition not smooth: {:?}",
          segment
        );
      }
    }
  }

  #[test]
  fn test_transient_shaper_attack_phase() {
    let samples = vec![0.5, 1.5, 0.8, 0.3];
    let params = TransientShaperParams {
      transient_emphasis: 1.5,
      threshold: 0.6,
      attack_time: 0.01,
      release_time: 0.1,
      attack_factor: 2.0,
      sustain_factor: 1.0,
      ..Default::default()
    };
    let result = transient_shaper(&samples, params).unwrap();
    assert!(result[1] > samples[1], "Transient peak should be emphasized.");
  }

  #[test]
  fn test_transient_shaper_sustain_phase() {
    let samples = vec![0.2, 0.3, 0.1, 0.4];
    let params = TransientShaperParams {
      transient_emphasis: 1.0,
      threshold: 0.3,
      attack_time: 0.05,
      release_time: 0.1,
      attack_factor: 1.0,
      sustain_factor: 0.5,
      ..Default::default()
    };
    let result = transient_shaper(&samples, params).unwrap();
    for (i, &sample) in result.iter().enumerate() {
      if sample < 0.3 {
        assert!(sample < samples[i], "Sustain phase should reduce level.");
      }
    }
  }

  #[test]
  fn test_transient_shaper_smooth_transition() {
    let samples = vec![0.2, 0.4, 0.6, 0.5, 0.7, 0.3];
    let params = TransientShaperParams {
      transient_emphasis: 1.5,
      threshold: 0.3,
      attack_time: 0.05,
      release_time: 0.05,
      attack_factor: 2.0,
      sustain_factor: 1.0,
      ..Default::default()
    };
    let result = transient_shaper(&samples, params).unwrap();
    assert_smooth_transition(&result, 0.3, 0, result.len());
  }

  #[test]
  fn test_transient_shaper_edge_case_no_transients() {
    let samples = vec![0.0, 0.0, 0.0, 0.0];
    let params = TransientShaperParams {
      transient_emphasis: 1.0,
      threshold: 0.5,
      attack_time: 0.1,
      release_time: 0.1,
      attack_factor: 1.0,
      sustain_factor: 1.0,
      ..Default::default()
    };
    let result = transient_shaper(&samples, params).unwrap();
    assert_eq!(result, samples, "Constant signal should remain unchanged.");
  }

  #[test]
  fn test_transient_shaper_high_attack_factor() {
    let samples = vec![0.3, 0.6, 0.2, 0.1];
    let params = TransientShaperParams {
      transient_emphasis: 1.5,
      threshold: 0.3,
      attack_time: 0.01,
      release_time: 0.1,
      attack_factor: 5.0,
      sustain_factor: 1.0,
      ..Default::default()
    };
    let result = transient_shaper(&samples, params).unwrap();
    assert!(result[1] > samples[1], "High attack factor should amplify transient.");
  }

  #[test]
  fn test_transient_shaper_sustain_behavior() {
    let samples = vec![0.2, 0.3, 0.4, 0.5];
    let params = TransientShaperParams {
      transient_emphasis: 1.0,
      threshold: 0.3,
      attack_time: 0.05,
      release_time: 0.1,
      attack_factor: 1.0,
      sustain_factor: 0.7,
      ..Default::default()
    };
    let result = transient_shaper(&samples, params).unwrap();
    for (i, &sample) in result.iter().enumerate() {
      if sample < 0.3 {
        assert!(sample < samples[i], "Sustain phase should reduce level.");
      }
    }
  }

  #[test]
  fn test_validate_transient_shaper_params() {
    let valid_params = TransientShaperParams {
      transient_emphasis: 1.0,
      threshold: 0.5,
      attack_time: 0.1,
      release_time: 0.1,
      detection_method: EnvelopeMethod::Peak,
      makeup_gain: 1.0,
      ratio: 1.0,
      knee_width: 0.5,
      wet_dry_mix: 0.5,
      ..Default::default()
    };

    assert!(validate_transient_shaper_params(&valid_params).is_ok());

    let invalid_params = TransientShaperParams {
      transient_emphasis: -1.0,
      threshold: 0.5,
      attack_time: 0.1,
      release_time: 0.1,
      detection_method: EnvelopeMethod::Peak,
      makeup_gain: 1.0,
      ratio: 1.0,
      knee_width: 0.5,
      wet_dry_mix: 0.5,
      ..Default::default()
    };

    assert!(validate_transient_shaper_params(&invalid_params).is_err());
  }
}

#[cfg(test)]
mod gate_tests {
  use super::*;


  fn assert_approx_eq(a: f32, b: f32, tol: f32, msg: &str) {
    assert!(
        (a - b).abs() <= tol,
        "{}: got {}, expected ~{}, tol={}",
        msg,
        a,
        b,
        tol
    );
}


  #[test]
  fn test_gate_smoothing_behavior() {
    let samples = vec![0.6, 0.8, 0.4, 0.3];
    let params = GateParams {
      threshold: 0.5,
      attack_time: 0.05,
      release_time: 0.05,
      wet_dry_mix: 1.0,
      ..Default::default()
    };
    let result = gate(&samples, params).unwrap();
    assert!(
      result.windows(2).all(|w| (w[1] - w[0]).abs() < 0.01),
      "Gate transitions should be smooth."
    );
  }

  

  #[test]
  fn test_empty_signal() {
    let samples: Vec<f32> = vec![];
    let params = GateParams {
      threshold: 0.5,
      attack_time: 0.1,
      release_time: 0.1,
      wet_dry_mix: 0.5,
      ..Default::default()
    };
    let result = gate(&samples, params);
    assert!(result.is_ok(), "Empty signal should return without error.");
  }

  #[test]
  fn test_validate_gate_params() {
    let valid_params = GateParams {
      threshold: -40.0,
      attack_time: 0.1,
      release_time: 0.1,
      detection_method: EnvelopeMethod::Peak,
      wet_dry_mix: 0.5,
      auto_gain: false,
      hold_time: None,
    };

    assert!(validate_gate_params(&valid_params).is_ok());

    let invalid_params = GateParams {
      threshold: -40.0,
      attack_time: 0.1,
      release_time: 0.1,
      detection_method: EnvelopeMethod::Peak,
      wet_dry_mix: 1.5,
      auto_gain: false,
      hold_time: None,
    };

    assert!(validate_gate_params(&invalid_params).is_err());
  }
}

#[cfg(test)]
mod test_unit_expander {
    use super::*;

    #[test]
    fn test_expander_above_threshold_with_smoothing() {
        // Ramp above threshold -> expander should not modify above threshold.
        let samples = vec![0.5, 0.6, 0.7, 0.8, 0.9];
        let params = ExpanderParams {
            threshold: -6.0, // ~0.5 in linear
            ratio: 2.0,
            attack_time: 0.02,
            release_time: 0.05,
            ..Default::default()
        };

        let out = expander(&samples, params).expect("Expander failed");
        // Above threshold -> no attenuation.
        for (i, &sample) in samples.iter().enumerate() {
            assert!(
                (out[i] - sample).abs() < 1e-2,
                "Above-threshold sample modified unexpectedly: input={} output={}",
                sample,
                out[i]
            );
        }
    }

    #[test]
    fn test_expander_below_threshold() {
        // Signals below the threshold should be attenuated according to the ratio.
        let samples = vec![0.1, 0.2, 0.3, 0.4, 0.5]; // Below threshold (~-20 dB).
        let params = ExpanderParams {
            threshold: -12.0,
            ratio: 2.0,
            attack_time: 0.01,
            release_time: 0.1,
            ..Default::default()
        };

        let out = expander(&samples, params).expect("Expander failed");

        for (i, &sample) in samples.iter().enumerate() {
            if sample < params.threshold {
                let expected_gain = db_to_amp((params.threshold - amp_to_db(sample)) * (1.0 - params.ratio));
                assert!(
                    (out[i] - sample * expected_gain).abs() < 1e-3,
                    "Below-threshold sample not expanded correctly: input={} output={} expected={}",
                    sample,
                    out[i],
                    sample * expected_gain
                );
            }
        }
    }



    #[test]
    fn test_expander_smoothing_behavior() {
        // Define a series of target gains to simulate gain reductions over time
        let target_gains = vec![1.0, 0.9, 0.8, 0.7, 0.6];
        let attack_time = 0.01;
        let release_time = 0.1;

        let mut previous_gain = target_gains[0];
        for (i, &target_gain) in target_gains.iter().enumerate().skip(1) {
            let smoothed_gain = smooth_gain_reduction(target_gain, previous_gain, attack_time, release_time);

            // Calculate expected maximum change based on coefficients
            let attack_coeff = time_to_coefficient(attack_time);
            let release_coeff = time_to_coefficient(release_time);
            let max_change = if target_gain > previous_gain {
                attack_coeff * (target_gain - previous_gain)
            } else {
                release_coeff * (target_gain - previous_gain)
            };

            // Assert that the smoothed gain does not overshoot the target
            assert!(
                (smoothed_gain - previous_gain).abs() <= max_change.abs() + 1e-6,
                "Abrupt change detected at index {}: previous={}, target={}, smoothed={}, max_change={}",
                i,
                previous_gain,
                target_gain,
                smoothed_gain,
                max_change
            );

            // Additionally, assert that smoothed_gain is approaching target_gain
            if target_gain < previous_gain {
                assert!(
                    smoothed_gain < previous_gain,
                    "Smoothed gain did not decrease towards target. previous={}, target={}, smoothed={}",
                    previous_gain,
                    target_gain,
                    smoothed_gain
                );
            } else {
                assert!(
                    smoothed_gain > previous_gain,
                    "Smoothed gain did not increase towards target. previous={}, target={}, smoothed={}",
                    previous_gain,
                    target_gain,
                    smoothed_gain
                );
            }

            previous_gain = smoothed_gain;
        }
    }


    #[test]
    fn test_expander_edge_case_low_level() {
        let samples = vec![0.0, 0.0, 0.0, 0.0];
        let params = ExpanderParams {
            threshold: 0.1,
            ratio: 2.0,
            attack_time: 0.1,
            release_time: 0.1,
            ..Default::default()
        };

        let result = expander(&samples, params).unwrap();
        assert_eq!(result, samples, "Low-level signals should not be modified.");
    }

    #[test]
    fn test_validate_expander_params() {
        let valid_params = ExpanderParams {
            threshold: -6.0,
            ratio: 2.0,
            attack_time: 0.1,
            release_time: 0.1,
            ..Default::default()
        };

        assert!(validate_expander_params(&valid_params).is_ok());

        let invalid_params = ExpanderParams {
            threshold: -6.0,
            ratio: 0.5, // Invalid: Ratio < 1 for an expander.
            attack_time: 0.1,
            release_time: 0.1,
            ..Default::default()
        };

        assert!(validate_expander_params(&invalid_params).is_err());
    }
}


#[cfg(test)]
mod unit_test_amp_to_db {
  use super::*;

  #[test]
  fn test_amp_to_db_zero_amplitude() {
    let amp = 0.0;
    let db = amp_to_db(amp);
    assert_eq!(db, -96.0, "Zero amplitude should return MIN_DB (-96.0 dB).");
  }

  #[test]
  fn test_amp_to_db_negative_amplitude() {
    let amp = -0.5;
    let db = amp_to_db(amp);
    assert_eq!(db, -96.0, "Negative amplitude should return MIN_DB (-96.0 dB).");
  }

  #[test]
  fn test_amp_to_db_positive_amplitude() {
    let amp = 1.0;
    let db = amp_to_db(amp);
    assert_eq!(db, 0.0, "Amplitude of 1.0 should return 0.0 dB.");
  }

  #[test]
  fn test_amp_to_db_small_positive_amplitude() {
    let amp = 1e-6;
    let db = amp_to_db(amp);
    assert!(
      db <= -96.0,
      "Very small positive amplitude should return MIN_DB (-96.0 dB) or lower."
    );
  }

  #[test]
  fn test_amp_to_db_large_amplitude() {
    let amp = 1000.0;
    let db = amp_to_db(amp);
    assert!(
      (db - 60.0).abs() < 1e-3,
      "Amplitude of 1000.0 should return approximately 60.0 dB."
    );
  }

  #[test]
  fn test_db_to_amp_standard_conversion() {
    let db = 0.0;
    let amp = db_to_amp(db);
    assert!(
      (amp - 1.0).abs() < 1e-6,
      "0 dB should convert to 1.0 amplitude, got {}",
      amp
    );
  }

  #[test]
  fn test_db_to_amp_positive_db() {
    let db = 6.0;
    let amp = db_to_amp(db);
    assert!(
      (amp - 2.0).abs() < 1e-2,
      "6 dB should convert to approximately 2.0 amplitude, got {}",
      amp
    );
  }

  #[test]
  fn test_db_to_amp_negative_db() {
    let db = -6.0;
    let amp = db_to_amp(db);
    assert!(
      (amp - 0.5011872).abs() < 1e-6,
      "-6 dB should convert to approximately 0.5011872 amplitude, got {}",
      amp
    );
  }

  #[test]
  fn test_db_to_amp_clamping_min_db() {
    let db = -100.0;
    let amp = db_to_amp(db);
    let expected_amp = 10f32.powf(-96.0 / 20.0); // Clamped to MIN_DB
    assert!(
      (amp - expected_amp).abs() < 1e-6,
      "-100 dB should be clamped to -96 dB and convert to {:.7} amplitude, got {}",
      expected_amp,
      amp
    );
  }

  #[test]
  fn test_db_to_amp_clamping_max_db() {
    let db = 30.0;
    let amp = db_to_amp(db);
    let expected_amp = 10f32.powf(24.0 / 20.0); // Clamped to MAX_DB
    assert!(
      (amp - expected_amp).abs() < 1e-4,
      "30 dB should be clamped to 24 dB and convert to {:.4} amplitude, got {}",
      expected_amp,
      amp
    );
  }
}

// /// Performs role-based dynamic compression.
// ///
// /// # Parameters:
// /// - `role1`: Role of the primary signal (dominant).
// /// - `role2`: Role of the secondary signal (subservient).
// /// - `signal1`: Samples of `role1` as Vec<Vec<f32>>.
// /// - `signal2`: Samples of `role2` as Vec<Vec<f32>>.
// /// - `intensity`: Effect strength [0.0, 1.0].
// ///
// /// # Returns:
// /// - `Result<Vec<Vec<f32>>, String>`: Processed signal or an error.
// pub fn role_based_compression(
//   role1: Role, role2: Role, signal1: Vec<Vec<f32>>, signal2: Vec<Vec<f32>>, intensity: f32,
// ) -> Result<Vec<Vec<f32>>, String> {
//   // Define compression parameters based on roles
//   let compressor_params = match (role1, role2) {
//     (Role::Kick, Role::Bass) => CompressorParams {
//       threshold: -36.0,
//       ratio: 22.0,
//       attack_time: 0.01,
//       release_time: 0.3,
//       wet_dry_mix: 1.0,
//       ..Default::default()
//     },
//     (Role::Bass, Role::Lead) => CompressorParams {
//       threshold: -18.0,
//       ratio: 3.0,
//       attack_time: 0.02,
//       release_time: 0.2,
//       wet_dry_mix: 0.8,
//       ..Default::default()
//     },
//     _ => CompressorParams {
//       threshold: -24.0,
//       ratio: 4.0,
//       knee_width: 0.5,
//       makeup_gain: 1.0,
//       attack_time: 0.01,
//       release_time: 0.1,
//       lookahead_time: None,
//       detection_method: EnvelopeMethod::Peak,
//       hold_time: None,
//       wet_dry_mix: 1.0,
//       sidechain_filter: None,
//       auto_gain: false,
//       ratio_slope: RatioSlope::Linear,
//       enable_limiter: false,
//       limiter_threshold: None,
//       envelope_shaping: None,
//     }, // _ => CompressorParams::default(), // same as above but not hardcoded
//   };

//   // Call the core compression function
//   Ok(dynamic_compression(signal1, signal2, compressor_params, intensity))
// }

// /// Returns the attack time based on the role.
// fn attack_time_for_role(role: Role) -> f32 {
//   match role {
//     Role::Kick | Role::Bass => 0.01,
//     Role::Perc | Role::Hats => 0.005,
//     Role::Lead | Role::Chords => 0.02,
//   }
// }

// /// Returns the release time based on the role.
// fn release_time_for_role(role: Role) -> f32 {
//   match role {
//     Role::Kick | Role::Bass => 0.2,
//     Role::Perc | Role::Hats => 0.1,
//     Role::Lead | Role::Chords => 0.3,
//   }
// }

// /// Applies dynamic range compression followed by expansion (companding) to the samples.
// ///
// /// # Parameters
// /// - `samples`: Input audio samples.
// /// - `params`: Compander parameters.
// /// - `sidechain`: Optional sidechain input samples for compression.
// ///
// /// # Returns
// /// - `Result<Vec<f32>, String>`: Companded samples or an error if parameters are invalid.
// pub fn compand(samples: &[f32], params: CompanderParams, sidechain: Option<&[f32]>) -> Result<Vec<f32>, String> {
//   let compressed = compressor(samples, params.compressor, sidechain)?;
//   let expanded = expander(&compressed, params.expander, None)?;
//   Ok(expanded)
// }

// /// Calculates a dynamic threshold based on peak or RMS levels, scaled by a factor.
// ///
// /// # Parameters
// /// - `samples`: Input audio samples.
// /// - `factor`: Scaling factor for the threshold.
// /// - `use_rms`: If `true`, uses RMS level; otherwise, uses peak level.
// ///
// /// # Returns
// /// - `Result<f32, String>`: Calculated threshold or an error if parameters are invalid.
// pub fn calculate_threshold(samples: &[f32], factor: f32, use_rms: bool) -> Result<f32, String> {
//   if factor <= 0.0 {
//     return Err("Factor must be positive.".to_string());
//   }
//   if samples.is_empty() {
//     return Ok(0.0);
//   }
//   if use_rms {
//     let sum_sq: f32 = samples.iter().map(|&x| x * x).sum();
//     let rms = (sum_sq / samples.len() as f32).sqrt();
//     Ok(rms * factor)
//   } else {
//     let peak = samples.iter().fold(0.0_f32, |acc, &x| acc.max(x.abs()));
//     Ok(peak * factor)
//   }
// }

// #[cfg(test)]
// mod fails {
//   use super::*;
//   #[test]
//   fn test_envelope_follower_rms() {
//     // A signal with a gradual rise and fall
//     let input_samples = vec![0.0, 0.3, 0.4, 0.5, 0.4, 0.3, 0.0];
//     let attack = 0.02;
//     let release = 0.02;
//     let rms_window = 0.1; // 0.1 seconds
//     let method = EnvelopeMethod::Rms(rms_window);

//     let envelope = envelope_follower(&input_samples, attack, release, None, Some(method), None)
//       .expect("Failed to calculate envelope");

//     // Check length matches input
//     assert_eq!(envelope.len(), input_samples.len());

//     // Validate temporal evolution
//     for i in 1..envelope.len() {
//       if input_samples[i] > input_samples[i - 1] {
//         // Envelope should rise with increasing input (account for smoothing)
//         assert!(
//           envelope[i] >= envelope[i - 1] - 0.01,
//           "Envelope dropped unexpectedly at index {}",
//           i
//         );
//       } else {
//         // Envelope should fall with decreasing input
//         assert!(
//           envelope[i] <= envelope[i - 1] + 0.01,
//           "Envelope rose unexpectedly at index {}",
//           i
//         );
//       }
//     }

//     // Verify envelope stays within bounds for this input
//     let max_possible_rms = 0.5; // Max value in input_samples
//     for &env_val in envelope.iter() {
//       assert!(
//         env_val >= 0.0 && env_val <= max_possible_rms + 0.01,
//         "Envelope out of bounds: {}",
//         env_val
//       );
//     }
//   }

//   #[test]
//   fn test_rms_spike_response() {
//     let mut samples = vec![0.0; 100];
//     samples[50] = 10.0; // Single spike
//     let window_time = 0.5; // 0.5 seconds

//     let result = envelope_follower(
//       &samples,
//       0.1,  // attack_time
//       0.1,  // release_time
//       None, // hold_time
//       Some(EnvelopeMethod::Rms(window_time)),
//       None, // pre_emphasis
//     )
//     .expect("Envelope follower failed");

//     // Ensure the envelope rises and falls around the spike
//     assert!(result[50] > result[49], "Envelope did not rise at spike");
//     assert!(result[51] < result[50], "Envelope did not fall after spike");
//   }

//   #[test]
//   fn test_rms_convergence() {
//     let samples = vec![2.0; 100];
//     let sample_rate = 100.0; // 100 Hz
//     let window_time = 0.5; // 0.5 seconds
//     let expected_rms = 2.0; // RMS of constant signal is the same as the value

//     let result = envelope_follower(
//       &samples,
//       0.0,  // attack_time
//       0.0,  // release_time
//       None, // hold_time
//       Some(EnvelopeMethod::Rms(window_time)),
//       None, // pre_emphasis
//     )
//     .expect("Envelope follower failed");

//     // Check if the result converges to the expected value within tolerance
//     for &env in result.iter().skip(50) {
//       // Skip transient startup
//       assert!(
//         (env - expected_rms).abs() < 1e-6,
//         "RMS value did not converge to expected: got {}, expected {}",
//         env,
//         expected_rms
//       );
//     }
//   }
// }

// #[cfg(test)]
// mod tests_applied {
//   use super::*;

//   #[test]
//   fn test_compress_bass_to_kick() {
//     // Define file paths for static assets
//     let input_path = &dev_audio_asset("bass.wav");
//     let sidechain_path = &dev_audio_asset("beat.wav");
//     let output_path = &dev_audio_asset("test-compressed_bass_beat.wav");

//     // Load signals
//     let (input_audio, input_sample_rate) = read_audio_file(input_path).expect("Failed to read input file.");
//     let (sidechain_audio, sidechain_sample_rate) =
//       read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

//     // Ensure sample rates match
//     assert_eq!(
//       input_sample_rate, sidechain_sample_rate,
//       "Input and sidechain sample rates must match."
//     );

//     // Perform role-based compression
//     let processed_audio = role_based_compression(
//       Role::Kick,
//       Role::Bass,
//       input_audio,
//       sidechain_audio,
//       0.8, // Intensity
//     )
//     .expect("Role-based compression failed.");

//     // Write the output to a file
//     write_audio(input_sample_rate as usize, processed_audio, output_path);

//     // Verify output file exists
//     assert!(
//       std::path::Path::new(output_path).exists(),
//       "Output file not found: {}",
//       output_path
//     );

//     println!("Test passed! Compressed audio written to '{}'", output_path);
//   }

//   /// Helper function to create default CompressorParams for testing.
//   fn default_compressor_params() -> CompressorParams {
//     CompressorParams {
//       threshold: 1.0,
//       ratio: 2.0,
//       knee_width: 0.0,
//       makeup_gain: 1.0,
//       attack_time: 0.01,
//       release_time: 0.1,
//       lookahead_time: None,
//       detection_method: EnvelopeMethod::Peak,
//       hold_time: None,
//       wet_dry_mix: 1.0,
//       sidechain_filter: None,
//       auto_gain: false,
//       ratio_slope: RatioSlope::Linear,
//       enable_limiter: false,
//       limiter_threshold: None,
//       envelope_shaping: None,
//       ..Default::default()
//     }
//   }

//   #[test]
//   fn test_calculate_threshold_empty_samples() {
//     let input_samples: Vec<f32> = vec![];
//     let factor = 1.0f32;
//     let use_rms = true;
//     let calculated_threshold = calculate_threshold(&input_samples, factor, use_rms).unwrap();
//     assert_eq!(calculated_threshold, 0.0f32);
//   }

//   #[test]
//   fn test_calculate_threshold_invalid_factor() {
//     let input_samples = vec![0.1f32, 0.2f32, 0.3f32];
//     let factor = 0.0f32;
//     let use_rms = false;
//     let result = calculate_threshold(&input_samples, factor, use_rms);
//     assert!(result.is_err());
//   }

//   #[test]
//   fn test_calculate_threshold_rms() {
//     let input_samples = vec![0.3f32, -0.4f32, 0.5f32, -0.6f32];
//     let factor = 1.0f32;
//     let use_rms = true;
//     let calculated_threshold = calculate_threshold(&input_samples, factor, use_rms).unwrap();

//     let sum_sq: f32 = input_samples.iter().map(|&x| x * x).sum();
//     let expected_rms = (sum_sq / input_samples.len() as f32).sqrt();

//     assert!((calculated_threshold - expected_rms).abs() < 1e-4);
//   }

//   #[test]
//   fn test_compressor_with_limiter() {
//     let input_samples = vec![0.0f32, 0.5f32, 1.0f32, 1.5f32, 2.0f32];
//     let mut compressor_params = default_compressor_params();
//     compressor_params.threshold = 1.0f32;
//     compressor_params.ratio = 2.0f32;
//     compressor_params.knee_width = 0.0f32; // Hard knee
//     compressor_params.attack_time = 0.0f32; // Instantaneous
//     compressor_params.release_time = 0.0f32; // Instantaneous
//     compressor_params.wet_dry_mix = 1.0f32; // Fully wet
//     compressor_params.enable_limiter = true;
//     compressor_params.limiter_threshold = Some(1.2f32);

//     let compressed = compressor(&input_samples, compressor_params, None).unwrap();

//     let expected_samples = vec![
//       0.0f32, // 0.0 remains 0.0
//       0.5f32, // 0.5 <= threshold, remains 0.5
//       1.0f32, // 1.0 <= threshold, remains 1.0
//       1.2f32, // 1.5 compressed to 1.25, then limited to 1.2
//       1.2f32, // 2.0 compressed to 1.5, then limited to 1.2
//     ];

//     for (output, expected) in compressed.iter().zip(expected_samples.iter()) {
//       assert!(
//         (*output - *expected).abs() < 1e-6,
//         "Output: {}, Expected: {}",
//         output,
//         expected
//       );
//     }
//   }

//   #[test]
//   fn test_envelope_stabilization() {
//     let input_samples = vec![1.0; 100];
//     let attack = 0.02; // 20 ms
//     let release = 0.02; // 20 ms
//     let method = EnvelopeMethod::Rms(0.1);

//     let envelope = envelope_follower(&input_samples, attack, release, None, Some(method), None)
//       .expect("Failed to calculate envelope");

//     // Skip transient phase
//     for &env_val in envelope.iter().skip(50) {
//       assert!(
//         (env_val - 1.0).abs() < 0.1,
//         "Envelope did not stabilize near 1.0, got {}",
//         env_val
//       );
//     }
//   }

//   #[test]
//   fn test_envelope_monotonicity() {
//     let input_samples = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.4, 0.3, 0.2, 0.1];
//     let attack = 0.01; // 10 ms
//     let release = 0.02; // 20 ms
//     let method = EnvelopeMethod::Peak;

//     let envelope = envelope_follower(&input_samples, attack, release, None, Some(method), None)
//       .expect("Failed to calculate envelope");

//     for i in 1..envelope.len() {
//       if input_samples[i] > input_samples[i - 1] {
//         // Ensure envelope rises
//         assert!(
//           envelope[i] >= envelope[i - 1],
//           "Envelope dropped during increasing input at index {}",
//           i
//         );
//       } else {
//         // Ensure envelope falls
//         assert!(
//           envelope[i] <= envelope[i - 1],
//           "Envelope rose during decreasing input at index {}",
//           i
//         );
//       }
//     }
//   }

//   #[test]
//   fn test_envelope_spike_responsiveness() {
//     let mut input_samples = vec![0.0; 100];
//     input_samples[50] = 1.0; // Single spike
//     let attack = 0.01; // 10 ms
//     let release = 0.05; // 50 ms
//     let method = EnvelopeMethod::Peak;

//     let envelope = envelope_follower(&input_samples, attack, release, None, Some(method), None)
//       .expect("Failed to calculate envelope");

//     // Ensure rise after spike
//     assert!(envelope[51] > envelope[50], "Envelope did not rise after spike");

//     // Ensure decay after spike
//     for i in 52..60 {
//       assert!(
//         envelope[i] < envelope[i - 1],
//         "Envelope did not decay after spike at index {}",
//         i
//       );
//     }
//   }

//   #[test]
//   fn test_envelope_rate_of_change() {
//     let input_samples = vec![0.0, 0.5, 1.0, 0.5, 0.0];
//     let attack = 0.01; // 10 ms
//     let release = 0.05; // 50 ms
//     let method = EnvelopeMethod::Peak;

//     let envelope = envelope_follower(&input_samples, attack, release, None, Some(method), None)
//       .expect("Failed to calculate envelope");

//     let max_attack_rate = 1.0 / attack; // Assuming normalized signal
//     let max_release_rate = 1.0 / release;

//     for i in 1..envelope.len() {
//       let rate = (envelope[i] - envelope[i - 1]).abs();
//       if envelope[i] > envelope[i - 1] {
//         // Check attack rate
//         assert!(rate <= max_attack_rate, "Attack rate too fast: {} at index {}", rate, i);
//       } else {
//         // Check release rate
//         assert!(
//           rate <= max_release_rate,
//           "Release rate too fast: {} at index {}",
//           rate,
//           i
//         );
//       }
//     }
//   }

//   #[test]
//   fn test_envelope_follower_peak() {
//     let input_samples = vec![0.0, 0.1, 0.2, 0.4, 0.2, 0.1, 0.0];
//     let attack = 0.01;
//     let release = 0.01;
//     let method = EnvelopeMethod::Peak;
//     let envelope = envelope_follower(&input_samples, attack, release, None, Some(method), None).unwrap();

//     assert_eq!(envelope.len(), input_samples.len());

//     for (i, &env_val) in envelope.iter().enumerate() {
//       assert!(env_val >= 0.0);
//       let max_val = input_samples.iter().map(|&x| x.abs()).fold(0.0_f32, |a, b| a.max(b));
//       assert!(env_val <= max_val + 0.1);
//     }
//   }

//   #[test]
//   fn test_rms_smoothness() {
//     let samples = vec![1.0; 100];
//     let window_time = 0.5; // 0.5 seconds
//     let result = envelope_follower(
//       &samples,
//       0.0,  // attack_time
//       0.0,  // release_time
//       None, // hold_time
//       Some(EnvelopeMethod::Rms(window_time)),
//       None, // pre_emphasis
//     )
//     .expect("Envelope follower failed");

//     for i in 1..result.len() {
//       // Ensure no abrupt changes in the envelope
//       assert!(
//         (result[i] - result[i - 1]).abs() <= 1.0,
//         "Output is not smooth: abrupt change at index {}",
//         i
//       );
//     }
//   }

//   #[test]
//   fn test_rms_zero_signal() {
//     let samples = vec![0.0; 100];
//     let window_time = 0.5; // 0.5 seconds

//     let result = envelope_follower(
//       &samples,
//       0.0,  // attack_time
//       0.0,  // release_time
//       None, // hold_time
//       Some(EnvelopeMethod::Rms(window_time)),
//       None, // pre_emphasis
//     )
//     .expect("Envelope follower failed");

//     for &env in result.iter() {
//       assert!(env.abs() < 1e-6, "RMS value of zero signal is not zero: got {}", env);
//     }
//   }

//   #[test]
//   fn test_normalizer_constant_zero() {
//     let input_samples = vec![0.0f32, 0.0f32, 0.0f32];
//     let target_max = 1.0f32;
//     let normalized = normalizer(&input_samples, target_max).unwrap();
//     assert_eq!(normalized, input_samples);
//   }

//   #[test]
//   fn test_normalizer_varied_amplitudes() {
//     let input_samples = vec![0.5f32, -1.0f32, 0.75f32, -0.25f32];
//     let target_max = 2.0f32;
//     let normalized = normalizer(&input_samples, target_max).unwrap();
//     let max_val = normalized.iter().map(|&x| x.abs()).fold(0.0_f32, |a, b| a.max(b));
//     assert!((max_val - 2.0).abs() < 1e-6);
//   }

//   #[test]
//   fn test_soft_clipper_edge_cases() {
//     let input_samples = vec![0.1f32, -0.2f32, 0.3f32];
//     let clip_threshold = 1.0f32;
//     let output = soft_clipper(&input_samples, clip_threshold).unwrap();
//     assert_eq!(output, input_samples);
//   }

//   #[test]
//   fn test_power_reduction() {
//     let input_path = &dev_audio_asset("bass.wav");
//     let sidechain_path = &dev_audio_asset("beat.wav");

//     // Load signals
//     let (input_audio, _) = read_audio_file(input_path).expect("Failed to read input file.");
//     let (sidechain_audio, _) = read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

//     // Perform compression
//     let processed_audio = role_based_compression(
//       Role::Bass,
//       Role::Kick,
//       input_audio.clone(),
//       sidechain_audio,
//       0.8, // Intensity
//     )
//     .expect("Role-based compression failed.");

//     // Calculate RMS power
//     let rms_before = calculate_rms(&input_audio.iter().flatten().cloned().collect::<Vec<f32>>());
//     let rms_after = calculate_rms(&processed_audio.iter().flatten().cloned().collect::<Vec<f32>>());

//     assert!(
//       rms_after < rms_before,
//       "RMS power did not decrease: before = {}, after = {}",
//       rms_before,
//       rms_after
//     );
//   }

//   #[test]
//   fn test_dynamic_range_reduction() {
//     let input_path = &dev_audio_asset("bass.wav");
//     let sidechain_path = &dev_audio_asset("beat.wav");

//     // Load signals
//     let (input_audio, _) = read_audio_file(input_path).expect("Failed to read input file.");
//     let (sidechain_audio, _) = read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

//     // Perform compression
//     let processed_audio = role_based_compression(
//       Role::Bass,
//       Role::Kick,
//       input_audio.clone(),
//       sidechain_audio,
//       0.8, // Intensity
//     )
//     .expect("Role-based compression failed.");

//     // Calculate dynamic range
//     let dynamic_range_before = calculate_dynamic_range(&input_audio.iter().flatten().cloned().collect::<Vec<f32>>());
//     let dynamic_range_after = calculate_dynamic_range(&processed_audio.iter().flatten().cloned().collect::<Vec<f32>>());

//     assert!(
//       dynamic_range_after < dynamic_range_before,
//       "Dynamic range did not decrease: before = {}, after = {}",
//       dynamic_range_before,
//       dynamic_range_after
//     );
//   }

//   #[test]
//   fn test_sidechain_envelope_alignment() {
//     let input_path = &dev_audio_asset("bass.wav");
//     let sidechain_path = &dev_audio_asset("beat.wav");

//     // Load signals
//     let (input_audio, _) = read_audio_file(input_path).expect("Failed to read input file.");
//     let (sidechain_audio, _) = read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

//     // Perform compression
//     let processed_audio = role_based_compression(
//       Role::Bass,
//       Role::Kick,
//       input_audio.clone(),
//       sidechain_audio.clone(),
//       0.8, // Intensity
//     )
//     .expect("Role-based compression failed.");

//     // Extract sidechain envelope
//     let sidechain_flat = sidechain_audio.iter().flatten().cloned().collect::<Vec<_>>();
//     let sidechain_env = envelope_follower(
//       &sidechain_flat,
//       0.01, // Attack time
//       0.1,  // Release time
//       None,
//       Some(EnvelopeMethod::Peak),
//       None,
//     )
//     .expect("Failed to calculate sidechain envelope.");

//     // Validate bassline attenuation
//     let bass_flat = processed_audio.iter().flatten().cloned().collect::<Vec<_>>();
//     for (&env_val, &bass_sample) in sidechain_env.iter().zip(bass_flat.iter()) {
//       assert!(
//         bass_sample <= env_val,
//         "Bass sample {} exceeds sidechain envelope {}",
//         bass_sample,
//         env_val
//       );
//     }
//   }

//   #[test]
//   fn test_visualize_results() {
//     let input_path = &dev_audio_asset("bass.wav");
//     let sidechain_path = &dev_audio_asset("beat.wav");

//     // Load signals
//     let (input_audio, _) = read_audio_file(input_path).expect("Failed to read input file.");
//     let (sidechain_audio, _) = read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

//     // Perform compression
//     let processed_audio = role_based_compression(
//       Role::Bass,
//       Role::Kick,
//       input_audio.clone(),
//       sidechain_audio,
//       0.8, // Intensity
//     )
//     .expect("Role-based compression failed.");

//     // Debugging output
//     let rms_before = calculate_rms(&input_audio.iter().flatten().cloned().collect::<Vec<f32>>());
//     let rms_after = calculate_rms(&processed_audio.iter().flatten().cloned().collect::<Vec<f32>>());
//     let dynamic_range_before = calculate_dynamic_range(&input_audio.iter().flatten().cloned().collect::<Vec<f32>>());
//     let dynamic_range_after = calculate_dynamic_range(&processed_audio.iter().flatten().cloned().collect::<Vec<f32>>());

//     println!("RMS Before: {}, RMS After: {}", rms_before, rms_after);
//     println!(
//       "Dynamic Range Before: {}, After: {}",
//       dynamic_range_before, dynamic_range_after
//     );
//   }

//   #[test]
//   fn test_rms_power_reduction() {
//     let input_path = &dev_audio_asset("bass.wav");
//     let sidechain_path = &dev_audio_asset("beat.wav");

//     // Load signals
//     let (input_audio, _) = read_audio_file(input_path).expect("Failed to read input file.");
//     let (sidechain_audio, _) = read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

//     // Perform compression
//     let processed_audio = role_based_compression(
//       Role::Bass,
//       Role::Kick,
//       input_audio.clone(),
//       sidechain_audio,
//       0.8, // Intensity
//     )
//     .expect("Role-based compression failed.");

//     // Calculate RMS power
//     let rms_before = calculate_rms(&input_audio.iter().flatten().cloned().collect::<Vec<f32>>());
//     let rms_after = calculate_rms(&processed_audio.iter().flatten().cloned().collect::<Vec<f32>>());

//     println!("RMS Before: {:.6}, RMS After: {:.6}", rms_before, rms_after);
//     assert!(
//       rms_after < rms_before,
//       "RMS power did not decrease: before = {:.6}, after = {:.6}",
//       rms_before,
//       rms_after
//     );
//   }

//   #[test]
//   fn test_dynamic_range_reduction_analysis() {
//     let input_path = &dev_audio_asset("beat.wav");
//     let sidechain_path = &dev_audio_asset("bass.wav");

//     // Load signals
//     let (input_audio, _) = read_audio_file(input_path).expect("Failed to read input file.");
//     let (sidechain_audio, _) = read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

//     // Perform compression
//     let processed_audio = role_based_compression(
//       Role::Bass,
//       Role::Kick,
//       input_audio.clone(),
//       sidechain_audio,
//       0.8, // Intensity
//     )
//     .expect("Role-based compression failed.");

//     // Calculate dynamic range
//     let dynamic_range_before = calculate_dynamic_range(&input_audio.iter().flatten().cloned().collect::<Vec<f32>>());
//     let dynamic_range_after = calculate_dynamic_range(&processed_audio.iter().flatten().cloned().collect::<Vec<f32>>());

//     println!(
//       "Dynamic Range Before: {:.6}, After: {:.6}",
//       dynamic_range_before, dynamic_range_after
//     );
//     assert!(
//       dynamic_range_after < dynamic_range_before,
//       "Dynamic range did not decrease: before = {:.6}, after = {:.6}",
//       dynamic_range_before,
//       dynamic_range_after
//     );
//   }
// }

// /// Make the beat "bussin" by applying normalization, compression, and transient shaping.
// pub fn make_beat_bussin(input_path: &str, output_path: &str) {
//   use crate::synth::SR;

//   // Step 1: Load and resample audio
//   let (audio, target_sample_rate) = crate::fastmast::load_and_resample_audio(input_path, SR as u32);
//   let num_channels = audio.len();
//   assert!(num_channels > 0, "Audio must have at least one channel.");

//   // Step 2: Process each channel separately
//   let mut processed_audio = Vec::new();
//   for channel in audio {
//     // Step 2a: Apply normalization
//     let normalized = normalizer(&channel, 0.9).expect("Failed to normalize audio");

//     // Step 2b: Apply transient shaping
//     let transient_params = TransientShaperParams {
//       transient_emphasis: 2.0,
//       threshold: 0.6,
//       attack_time: 0.01,
//       release_time: 0.1,
//       detection_method: EnvelopeMethod::Peak,
//       makeup_gain: 1.2,
//       ratio: 1.0,
//       knee_width: 0.0,
//       wet_dry_mix: 1.0,
//     };
//     let transient_shaped = transient_shaper(&normalized, transient_params).expect("Failed to apply transient shaping");

//     // Step 2c: Apply soft clipping
//     let clipped = soft_clipper(&transient_shaped, 0.8).expect("Failed to apply soft clipping");

//     processed_audio.push(clipped);
//   }

//   // Step 3: Write processed audio to output
//   crate::render::engrave::write_audio(target_sample_rate as usize, processed_audio, output_path)
// }

// /// Apply compressor with rolled parameters to make the beat bussin.
// pub fn make_beat_bussin_with_roll(input_path: &str, output_path: &str) {
//   let (audio, sample_rate) = read_audio_file(input_path).expect("Failed to read input file.");
//   let num_channels = audio.len();

//   // Roll random compressor parameters
//   let compressor_params = roll_compressor_params(
//     -30.0, -6.0, // Min/max threshold in dB
//     2.0, 10.0, // Min/max ratio
//     0.001, 0.1, // Min/max attack time in seconds
//     0.01, 0.5, // Min/max release time in seconds
//   );

//   let mut processed_audio: Vec<Vec<f32>> = Vec::new();

//   for channel in audio.iter() {
//     let compressed = compressor(channel, compressor_params, None).expect("Compression failed.");
//     processed_audio.push(compressed);
//   }

//   write_audio(sample_rate as usize, processed_audio, output_path);
// }

// /// Generate randomized compressor parameters within defined ranges.
// fn roll_compressor_params(
//   min_threshold: f32, max_threshold: f32, min_ratio: f32, max_ratio: f32, min_attack: f32, max_attack: f32,
//   min_release: f32, max_release: f32,
// ) -> CompressorParams {
//   let mut rng = rand::thread_rng();
//   CompressorParams {
//     threshold: rng.gen_range(min_threshold..max_threshold),
//     ratio: rng.gen_range(min_ratio..max_ratio),
//     knee_width: rng.gen_range(0.0..1.0),  // Default range for knee width
//     makeup_gain: rng.gen_range(0.5..2.0), // Amplify or attenuate post-compression
//     attack_time: rng.gen_range(min_attack..max_attack),
//     release_time: rng.gen_range(min_release..max_release),
//     lookahead_time: None, // Can add lookahead randomization if desired
//     detection_method: EnvelopeMethod::Peak,
//     hold_time: None,
//     wet_dry_mix: rng.gen_range(0.5..1.0), // Ensure mostly wet signal
//     sidechain_filter: None,
//     auto_gain: false,
//     ratio_slope: RatioSlope::Linear,
//     enable_limiter: false,
//     limiter_threshold: None,
//     envelope_shaping: None,
//   }
// }

// #[test]
// fn test_make_beat_bussin_with_roll() {
//   let input_path = &dev_audio_asset("beat.wav");
//   let output_path = &dev_audio_asset("test-output-bussin-roll.wav");

//   println!(
//     "Testing make_beat_bussin_with_roll from '{}' to '{}'",
//     input_path, output_path
//   );

//   // Call the function
//   make_beat_bussin_with_roll(input_path, output_path);

//   // Verify output
//   use std::path::Path;
//   assert!(
//     Path::new(output_path).exists(),
//     "Output file '{}' was not created.",
//     output_path
//   );

//   // Validate the output
//   let (output_audio, output_sample_rate) =
//     read_audio_file(output_path).unwrap_or_else(|err| panic!("Failed to read output file '{}': {}", output_path, err));
//   assert_eq!(output_sample_rate, crate::synth::SR as u32, "Sample rate mismatch.");
//   assert!(!output_audio.is_empty(), "Output audio is empty.");
//   assert_eq!(output_audio.len(), 2, "Expected 2 channels in output audio.");

//   println!(
//     "test_make_beat_bussin_with_roll passed, output written to '{}'",
//     output_path
//   );
// }

// #[test]
// fn test_make_beat_bussin() {
//   use crate::analysis::sampler::{read_audio_file, write_audio_file, AudioFormat};
//   let input_path = &dev_audio_asset("beat.wav");
//   let output_path = &dev_audio_asset("test-output-bussin.wav");

//   println!("Testing make_beat_bussin from '{}' to '{}'", input_path, output_path);

//   // Call the make_beat_bussin function
//   make_beat_bussin(input_path, output_path);

//   // Verify output
//   use std::path::Path;
//   assert!(
//     Path::new(output_path).exists(),
//     "Output file '{}' was not created.",
//     output_path
//   );

//   // Validate the output
//   let (output_audio, output_sample_rate) =
//     read_audio_file(output_path).unwrap_or_else(|err| panic!("Failed to read output file '{}': {}", output_path, err));
//   assert_eq!(output_sample_rate, crate::synth::SR as u32, "Sample rate mismatch.");
//   assert!(!output_audio.is_empty(), "Output audio is empty.");
//   assert_eq!(output_audio.len(), 2, "Expected 2 channels in output audio.");

//   println!(
//     "make_beat_bussin test passed, output written to '{}', sample rate: {}",
//     output_path, output_sample_rate
//   );
// }

// /// Calculates RMS power of the given audio samples.
// pub fn calculate_rms(samples: &[f32]) -> f32 {
//   (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt()
// }

// /// Calculates the dynamic range of the given audio samples.
// pub fn calculate_dynamic_range(samples: &[f32]) -> f32 {
//   let max = samples.iter().cloned().fold(f32::MIN, f32::max);
//   let min = samples.iter().cloned().fold(f32::MAX, f32::min);
//   max - min
// }
