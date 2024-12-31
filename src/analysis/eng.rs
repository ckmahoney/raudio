// eng.rs

// audio.rs
//
// This module provides refined implementations of typical audio processing stages,
// including envelope following, compression, gating, transient shaping, soft clipping,
// normalization, and interleaving/deinterleaving. Enhancements support advanced
// waveshaping effects like  dynamic range expansion, and companding.
//
// Dependencies:
// - crate::synth::{SR, SRf} for sample-rate constants.
// - biquad = "0.5" for biquad filter implementations.
// - itertools = "0.10" for iterator utilities.

use crate::phrasing::ranger::{DYNAMIC_RANGE_DB, MAX_DB, MIN_DB};
use crate::synth::{SRf, SR};
use crate::timbre::Role;
use biquad::{Biquad, Coefficients, DirectForm1, Hertz, Type as FilterType};
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
#[derive(Debug, Clone, Copy)]
pub struct CompressorParams {
  /// Threshold level in linear scale above which compression starts.
  pub threshold: f32,
  /// Compression ratio (e.g., 4.0 for 4:1 compression).
  pub ratio: f32,
  /// Knee width for soft knee compression.
  pub knee_width: f32,
  /// Makeup gain applied after compression.
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
  /// Limiter threshold in linear scale.
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

/// Struct to hold companding parameters.
#[derive(Debug, Clone, Copy)]
pub struct CompanderParams {
  /// Compressor parameters for compression stage.
  pub compressor: CompressorParams,
  /// Expander parameters for expansion stage.
  pub expander: ExpanderParams,
}

/// Struct to hold expander parameters.
#[derive(Debug, Clone, Copy)]
pub struct ExpanderParams {
  /// Threshold level in linear scale below which expansion starts.
  pub threshold: f32,
  /// Expansion ratio (e.g., 2.0 for 2:1 expansion).
  pub ratio: f32,
  /// Attack time in seconds.
  pub attack_time: f32,
  /// Release time in seconds.
  pub release_time: f32,
  /// Makeup gain applied after expansion.
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

/// Struct to hold transient shaper parameters.
#[derive(Debug, Clone, Copy)]
pub struct TransientShaperParams {
  /// Amount of transient emphasis (e.g., 1.0 for normal, >1.0 for emphasis).
  pub transient_emphasis: f32,
  /// Threshold above which transient shaping is applied.
  pub threshold: f32,
  /// Attack time in seconds.
  pub attack_time: f32,
  /// Release time in seconds.
  pub release_time: f32,
  /// Detection method for envelope follower.
  pub detection_method: EnvelopeMethod,
  /// Makeup gain applied after transient shaping.
  pub makeup_gain: f32,
  /// Ratio for transient shaping.
  pub ratio: f32,
  /// Knee width for soft knee transient shaping.
  pub knee_width: f32,
  /// Wet/Dry mix (0.0 = dry, 1.0 = wet).
  pub wet_dry_mix: f32,
}

/// Enumeration for filter slope types.
#[derive(Debug, Clone, Copy)]
pub enum FilterSlope {
  OnePole,
  TwoPole,
  // Extend with more complex slopes if needed
}

/// Performs role-based dynamic compression.
///
/// # Parameters:
/// - `role1`: Role of the primary signal (dominant).
/// - `role2`: Role of the secondary signal (subservient).
/// - `signal1`: Samples of `role1` as Vec<Vec<f32>>.
/// - `signal2`: Samples of `role2` as Vec<Vec<f32>>.
/// - `intensity`: Effect strength [0.0, 1.0].
///
/// # Returns:
/// - `Result<Vec<Vec<f32>>, String>`: Processed signal or an error.
pub fn role_based_compression(
  role1: Role, role2: Role, signal1: Vec<Vec<f32>>, signal2: Vec<Vec<f32>>, intensity: f32,
) -> Result<Vec<Vec<f32>>, String> {
  // Define compression parameters based on roles
  let compressor_params = match (role1, role2) {
    (Role::Kick, Role::Bass) => CompressorParams {
      threshold: -36.0,
      ratio: 22.0,
      attack_time: 0.01,
      release_time: 0.3,
      wet_dry_mix: 1.0,
      ..Default::default()
    },
    (Role::Bass, Role::Lead) => CompressorParams {
      threshold: -18.0,
      ratio: 3.0,
      attack_time: 0.02,
      release_time: 0.2,
      wet_dry_mix: 0.8,
      ..Default::default()
    },
    _ => CompressorParams {
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
    }, // _ => CompressorParams::default(), // same as above but not hardcoded
  };

  // Call the core compression function
  Ok(dynamic_compression(signal1, signal2, compressor_params, intensity))
}

/// Applies dynamic range compression with sidechain support, adapting to channel configurations.
///
/// # Parameters:
/// - `input`: Input audio samples (e.g., bass).
/// - `sidechain`: Sidechain audio samples (e.g., kick).
/// - `compressor_params`: Compressor parameters.
/// - `intensity`: Range [0.0, 1.0], scaling the effect strength.
///
/// # Returns:
/// - `Vec<Vec<f32>>`: Processed audio channels.
pub fn dynamic_compression(
  input: Vec<Vec<f32>>, sidechain: Vec<Vec<f32>>, compressor_params: CompressorParams, intensity: f32,
) -> Vec<Vec<f32>> {
  let n_input = input.len();
  let n_sidechain = sidechain.len();

  // Ensure intensity is within bounds
  let intensity = intensity.clamp(0.0, 1.0);

  // Helper function to process and scale a single channel
  let compress_and_scale = |input_channel: &[f32], sidechain_channel: &[f32]| -> Vec<f32> {
    let compressed =
      compressor(input_channel, compressor_params, Some(sidechain_channel)).expect("Compression failed.");
    compressed
      .iter()
      .zip(input_channel.iter())
      .map(|(&compressed_sample, &original_sample)| compressed_sample * intensity + original_sample * (1.0 - intensity))
      .collect()
  };

  match (n_input, n_sidechain) {
    // Mono input and mono sidechain
    (1, 1) => vec![compress_and_scale(&input[0], &sidechain[0])],

    // Mono input and stereo sidechain
    (1, 2) => {
      let downmixed_sidechain =
        downmix_stereo_to_mono(&sidechain[0], &sidechain[1]).expect("Failed to downmix sidechain.");
      vec![compress_and_scale(&input[0], &downmixed_sidechain)]
    }

    // Stereo input and mono sidechain
    (2, 1) => vec![
      compress_and_scale(&input[0], &sidechain[0]),
      compress_and_scale(&input[1], &sidechain[0]),
    ],

    // Stereo input and stereo sidechain
    (2, 2) => {
      let downmixed_sidechain =
        downmix_stereo_to_mono(&sidechain[0], &sidechain[1]).expect("Failed to downmix sidechain.");
      vec![
        compress_and_scale(&input[0], &downmixed_sidechain),
        compress_and_scale(&input[1], &downmixed_sidechain),
      ]
    }

    // Mono or stereo input with no sidechain
    (_, 0) => input, // Pass-through

    // Unsupported configurations
    _ => panic!(
      "Unsupported channel configuration: input = {}, sidechain = {}",
      n_input, n_sidechain
    ),
  }
}

/// Returns the attack time based on the role.
fn attack_time_for_role(role: Role) -> f32 {
  match role {
    Role::Kick | Role::Bass => 0.01,
    Role::Perc | Role::Hats => 0.005,
    Role::Lead | Role::Chords => 0.02,
  }
}

/// Returns the release time based on the role.
fn release_time_for_role(role: Role) -> f32 {
  match role {
    Role::Kick | Role::Bass => 0.2,
    Role::Perc | Role::Hats => 0.1,
    Role::Lead | Role::Chords => 0.3,
  }
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
    (-1.0 / (time_sec * SRf)).exp()
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

  let mut rms_buffer = vec![0.0; window_size];
  let mut rms_accumulator = 0.0;
  let mut buffer_index = 0;
  let mut rms_output = Vec::with_capacity(samples.len());

  for &sample in samples.iter() {
    let square = sample * sample;
    rms_accumulator += square - rms_buffer[buffer_index];
    rms_buffer[buffer_index] = square;
    buffer_index = (buffer_index + 1) % window_size;
    rms_output.push((rms_accumulator / window_size as f32).sqrt());
  }

  rms_output
}

#[cfg(test)]
mod unit_test_compute_rms {
  use super::*;

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
  } else if input > current_env {
    attack_coeff * (current_env - input) + input
  } else {
    release_coeff * (current_env - input) + input
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
  fn test_extreme_coefficients_behavior() {
    let current_env = 0.5;
    let input = 1.0;

    // Very small attack and release coefficients (minimal smoothing)
    let result_small = apply_attack_release(current_env, input, 0.001, 0.001, false);
    assert!(
      (result_small - input).abs() < 1e-3,
      "Small coefficients failed to track input closely: result {}, input {}",
      result_small,
      input
    );

    // Very large attack and release coefficients (heavy smoothing)
    let result_large = apply_attack_release(current_env, input, 0.999, 0.999, false);
    assert!(
      (result_large - current_env).abs() < 1e-3,
      "Large coefficients failed to maintain smoothing: result {}, current_env {}",
      result_large,
      current_env
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
fn apply_highpass(samples: &[f32], cutoff_hz: f32) -> Result<Vec<f32>, String> {
  // Define filter coefficients for a high-pass filter
  let coeffs = Coefficients::<f32>::from_params(
    FilterType::HighPass,
    Hertz::from_hz(SRf).unwrap(),
    Hertz::from_hz(cutoff_hz).unwrap(),
    0.707, // Q-factor (1/sqrt(2) for Butterworth)
  )
  .map_err(|e| format!("Failed to create high-pass filter coefficients: {:?}", e))?;

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

/// Applies a low-pass biquad filter to the input samples.
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

/// Detects the envelope of the signal using the specified method and parameters.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `attack_time`: Attack time in seconds.
/// - `release_time`: Release time in seconds.
/// - `hold_time`: Optional hold time in seconds.
/// - `method`: Optional envelope detection method (default: Peak).
/// - `pre_emphasis`: Optional pre-emphasis cutoff frequency in Hz.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Envelope-followed samples or an error if parameters are invalid.
pub fn envelope_follower(
  samples: &[f32], attack_time: f32, release_time: f32, hold_time: Option<f32>, method: Option<EnvelopeMethod>,
  pre_emphasis: Option<f32>,
) -> Result<Vec<f32>, String> {
  if attack_time < 0.0 || release_time < 0.0 {
    return Err("Attack and release times must be non-negative.".to_string());
  }

  let envelope_method = method.unwrap_or(EnvelopeMethod::Peak);
  let hold_samps = (hold_time.unwrap_or(0.0) * SRf).round() as usize;
  let attack_coeff = if attack_time == 0.0 {
    1.0
  } else {
    time_to_coefficient(attack_time)
  };
  let release_coeff = if release_time == 0.0 {
    1.0
  } else {
    time_to_coefficient(release_time)
  };

  let processed_samples = if let Some(cutoff_hz) = pre_emphasis {
    apply_highpass(samples, cutoff_hz)?
  } else {
    samples.to_vec()
  };

  let mut env = Vec::with_capacity(processed_samples.len());
  let mut current_env = 0.0;
  let mut hold_counter = 0usize;

  // Setup for RMS or Hybrid if necessary
  let mut rms_buffer = vec![0.0; 1]; // Default buffer size
  let mut rms_accumulator = 0.0;
  let mut buffer_index = 0;

  if let EnvelopeMethod::Rms(window_time) | EnvelopeMethod::Hybrid(window_time) = envelope_method {
    let rms_window_size = (window_time * SRf).round() as usize;
    rms_buffer = vec![0.0; rms_window_size];
  }

  for &sample in processed_samples.iter() {
    let val = match envelope_method {
      EnvelopeMethod::Peak => sample.abs(),
      EnvelopeMethod::Rms(_) => {
        let square = sample * sample;
        rms_accumulator += square - rms_buffer[buffer_index];
        rms_buffer[buffer_index] = square;
        buffer_index = (buffer_index + 1) % rms_buffer.len();
        (rms_accumulator / rms_buffer.len() as f32).sqrt()
      }
      EnvelopeMethod::Hybrid(_) => {
        let square = sample * sample;
        rms_accumulator += square - rms_buffer[buffer_index];
        rms_buffer[buffer_index] = square;
        buffer_index = (buffer_index + 1) % rms_buffer.len();
        let rms = (rms_accumulator / rms_buffer.len() as f32).sqrt();
        (sample.abs() + rms) / 2.0
      }
    };

    if val > current_env {
      current_env = attack_coeff * (current_env - val) + val;
      hold_counter = 0;
    } else {
      if hold_counter < hold_samps {
        hold_counter += 1;
      } else {
        current_env = release_coeff * (current_env - val) + val;
      }
    }
    env.push(current_env);
  }
  Ok(env)
}

/// Applies dynamic range expansion to the samples based on the provided parameters.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `params`: Expander parameters.
/// - `sidechain`: Optional sidechain input samples.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Expanded samples or an error if parameters are invalid.
pub fn expander(samples: &[f32], params: ExpanderParams, sidechain: Option<&[f32]>) -> Result<Vec<f32>, String> {
  if params.ratio < 1.0 {
    return Err("Expansion ratio must be >= 1.0.".to_string());
  }

  let envelope = if let Some(sc) = sidechain {
    envelope_follower(
      sc,
      params.attack_time,
      params.release_time,
      params.hold_time,
      Some(params.detection_method),
      params.sidechain_filter.map(|f| f.cutoff_freq),
    )?
  } else {
    envelope_follower(
      samples,
      params.attack_time,
      params.release_time,
      params.hold_time,
      Some(params.detection_method),
      None,
    )?
  };

  let mut output = Vec::with_capacity(samples.len());
  for (i, &sample) in samples.iter().enumerate() {
    let env_val = envelope[i];
    let gain_increase = if env_val < params.threshold && env_val != 0.0 {
      let expanded_level = params.threshold - (params.threshold - env_val) / params.ratio;
      expanded_level / env_val
    } else {
      1.0
    };

    // Apply envelope shaping if specified
    let final_gain = if let Some(shaping) = params.envelope_shaping {
      match shaping.shape_type {
        ShapeType::Soft => 1.0 - (1.0 - gain_increase).powf(2.0), // Example soft shaping
        ShapeType::Hard => gain_increase.powf(3.0),               // Example hard shaping
        ShapeType::Custom => gain_increase,                       // Placeholder for custom shaping
      }
    } else {
      gain_increase
    };

    // Apply gain increase
    let expanded_sample = sample * final_gain;

    // Apply makeup gain
    let makeup = if params.auto_gain {
      params.makeup_gain
    } else {
      params.makeup_gain
    };

    let final_sample = expanded_sample * makeup;

    // Apply wet/dry mix
    let mixed_sample = params.wet_dry_mix * final_sample + (1.0 - params.wet_dry_mix) * sample;

    output.push(mixed_sample);
  }

  Ok(output)
}

/// Applies a limiter to the samples to prevent clipping.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `threshold`: Limiter threshold in linear scale.
///
/// # Returns
/// - `Vec<f32>`: Limited samples.
fn limiter(samples: &[f32], threshold: f32) -> Vec<f32> {
  samples
    .iter()
    .map(|&s| {
      let sign = s.signum();
      let abs_s = s.abs();
      if abs_s > threshold {
        sign * threshold
      } else {
        s
      }
    })
    .collect()
}

/// Applies dynamic range compression followed by expansion (companding) to the samples.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `params`: Compander parameters.
/// - `sidechain`: Optional sidechain input samples for compression.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Companded samples or an error if parameters are invalid.
pub fn compand(samples: &[f32], params: CompanderParams, sidechain: Option<&[f32]>) -> Result<Vec<f32>, String> {
  let compressed = compressor(samples, params.compressor, sidechain)?;
  let expanded = expander(&compressed, params.expander, None)?;
  Ok(expanded)
}

pub fn compressor(samples: &[f32], params: CompressorParams, sidechain: Option<&[f32]>) -> Result<Vec<f32>, String> {
  if params.ratio < 1.0 {
    return Err("Compression ratio must be >= 1.0.".to_string());
  }
  if let Some(t) = params.lookahead_time {
    if t < 0.0 {
      return Err("Lookahead time must be non-negative.".to_string());
    }
  }

  let lookahead_samples = params.lookahead_time.map(|t| (t * SRf).round() as usize).unwrap_or(0);
  let delayed_samples = if lookahead_samples > 0 {
    apply_lookahead(samples, lookahead_samples)
  } else {
    samples.to_vec()
  };

  let envelope = if let Some(sc) = sidechain {
    envelope_follower(
      sc,
      params.attack_time,
      params.release_time,
      params.hold_time,
      Some(params.detection_method),
      params.sidechain_filter.map(|f| f.cutoff_freq),
    )?
  } else {
    envelope_follower(
      &delayed_samples,
      params.attack_time,
      params.release_time,
      params.hold_time,
      Some(params.detection_method),
      None,
    )?
  };

  println!(
    "Envelope (sampled): {:?}",
    &envelope[..std::cmp::min(envelope.len(), 10)]
  );

  // Calculate threshold_linear once
  let threshold_linear = if params.threshold < 0.0 {
    10f32.powf(params.threshold / 20.0).clamp(0.0, 1.0)
  } else {
    params.threshold.clamp(0.0, 1.0)
  };
  if threshold_linear > 1.0 {
    eprintln!("Warning: Threshold value exceeds normalized range.");
  }

  let mut output = Vec::with_capacity(samples.len());
  let mut previous_gain = 1.0; // Smoothing state
  for (i, &sample) in samples.iter().enumerate() {
    let env_val = envelope[i];

    let mut gain_reduction = if env_val > threshold_linear {
      let compressed_level = threshold_linear + (env_val - threshold_linear) / params.ratio;
      compressed_level / env_val
    } else {
      1.0
    };

    // Apply smoothing for musicality
    let smooth_gain = time_to_coefficient(params.release_time);
    gain_reduction = smooth_gain * gain_reduction + (1.0 - smooth_gain) * previous_gain;
    previous_gain = gain_reduction;

    // Debugging: Check anomalies
    if gain_reduction < 0.0 || gain_reduction > 1.0 {
      println!(
        "Gain anomaly: env_val = {}, threshold_linear = {}, ratio = {}, gain_reduction = {}",
        env_val, threshold_linear, params.ratio, gain_reduction
      );
    }

    let compressed_sample = sample * gain_reduction;

    let makeup = params.makeup_gain;

    let final_sample = compressed_sample * makeup;

    let mixed_sample = params.wet_dry_mix * final_sample + (1.0 - params.wet_dry_mix) * sample;

    output.push(mixed_sample);
  }

  if params.enable_limiter {
    let limiter_threshold = params.limiter_threshold.unwrap_or(threshold_linear);
    let limited_output = limiter(&output, limiter_threshold);
    output = limited_output;
  }

  println!("Output (sampled): {:?}", &output[..std::cmp::min(output.len(), 10)]);

  Ok(output)
}

/// Applies transient shaping by enhancing or attenuating transients based on the envelope.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `params`: Transient shaper parameters.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Transient-shaped samples or an error if parameters are invalid.
pub fn transient_shaper(samples: &[f32], params: TransientShaperParams) -> Result<Vec<f32>, String> {
  if params.transient_emphasis < 0.0 {
    return Err("Transient emphasis must be non-negative.".to_string());
  }

  let envelope = envelope_follower(
    samples,
    params.attack_time,
    params.release_time,
    None,
    Some(params.detection_method),
    None,
  )?;

  let mut output = Vec::with_capacity(samples.len());

  for (&sample, &env_val) in samples.iter().zip(envelope.iter()) {
    let factor = if env_val > params.threshold {
      1.0 + params.transient_emphasis * ((env_val / params.threshold).powf(params.ratio) - 1.0)
    } else {
      1.0
    };
    let shaped_sample = sample * factor * params.makeup_gain;
    // Apply wet/dry mix
    let mixed_sample = params.wet_dry_mix * shaped_sample + (1.0 - params.wet_dry_mix) * sample;
    output.push(mixed_sample);
  }

  Ok(output)
}

/// Applies expansion to a single sample based on threshold and ratio.
///
/// # Parameters
/// - `sample`: Input audio sample.
/// - `threshold`: Expansion threshold in linear scale.
/// - `ratio`: Expansion ratio.
///
/// # Returns
/// - `f32`: Expanded audio sample.
fn apply_expansion(sample: f32, threshold: f32, ratio: f32) -> f32 {
  let sign = sample.signum();
  let abs_s = sample.abs();
  if abs_s >= threshold {
    sample
  } else {
    let deficit = threshold - abs_s;
    let expanded = threshold - deficit * ratio;
    sign * expanded
  }
}

/// Applies compression to a single sample based on threshold and ratio.
///
/// # Parameters
/// - `sample`: Input audio sample.
/// - `threshold`: Compression threshold in linear scale.
/// - `ratio`: Compression ratio.
///
/// # Returns
/// - `f32`: Compressed audio sample.
fn apply_compression(sample: f32, threshold: f32, ratio: f32) -> f32 {
  let sign = sample.signum();
  let abs_s = sample.abs();
  if abs_s <= threshold {
    sample
  } else {
    let excess = abs_s - threshold;
    let compressed = threshold + excess / ratio;
    sign * compressed
  }
}

/// Applies dynamic range expansion with sidechain support.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `params`: Expander parameters.
/// - `sidechain`: Sidechain input samples to control expansion.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Expanded samples or an error if parameters are invalid.
pub fn expander_with_sidechain(samples: &[f32], params: ExpanderParams, sidechain: &[f32]) -> Result<Vec<f32>, String> {
  expander(samples, params, Some(sidechain))
}

/// Applies a noise gate to the samples, zeroing those below the threshold.
/// Includes attack and release smoothing to prevent abrupt transitions.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `threshold`: Threshold level in linear scale.
/// - `attack_time`: Attack time in seconds.
/// - `release_time`: Release time in seconds.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Gated samples or an error if parameters are invalid.
pub fn gate(samples: &[f32], threshold: f32, attack_time: f32, release_time: f32) -> Result<Vec<f32>, String> {
  if attack_time < 0.0 || release_time < 0.0 {
    return Err("Attack and release times must be non-negative.".to_string());
  }

  let envelope = envelope_follower(
    samples,
    attack_time,
    release_time,
    None,
    Some(EnvelopeMethod::Peak),
    None,
  )?;

  let mut output = Vec::with_capacity(samples.len());
  for (&sample, &env_val) in samples.iter().zip(envelope.iter()) {
    if env_val <= threshold {
      output.push(0.0);
    } else {
      output.push(sample);
    }
  }
  Ok(output)
}

/// Calculates a dynamic threshold based on peak or RMS levels, scaled by a factor.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `factor`: Scaling factor for the threshold.
/// - `use_rms`: If `true`, uses RMS level; otherwise, uses peak level.
///
/// # Returns
/// - `Result<f32, String>`: Calculated threshold or an error if parameters are invalid.
pub fn calculate_threshold(samples: &[f32], factor: f32, use_rms: bool) -> Result<f32, String> {
  if factor <= 0.0 {
    return Err("Factor must be positive.".to_string());
  }
  if samples.is_empty() {
    return Ok(0.0);
  }
  if use_rms {
    let sum_sq: f32 = samples.iter().map(|&x| x * x).sum();
    let rms = (sum_sq / samples.len() as f32).sqrt();
    Ok(rms * factor)
  } else {
    let peak = samples.iter().fold(0.0_f32, |acc, &x| acc.max(x.abs()));
    Ok(peak * factor)
  }
}

/// Applies a combination of soft clipping and normalization to achieve gentle distortion and consistent levels.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `clip_threshold`: Threshold above which clipping starts.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Soft-clipped and normalized samples or an error if parameters are invalid.
pub fn soft_clipper(samples: &[f32], clip_threshold: f32) -> Result<Vec<f32>, String> {
  if clip_threshold <= 0.0 {
    return Err("Clip threshold must be positive.".to_string());
  }
  Ok(
    samples
      .iter()
      .map(|&s| {
        if s.abs() <= clip_threshold {
          s
        } else {
          // Standard soft clipping using a polynomial for smoothness
          let s_abs = s.abs();
          let clipped = clip_threshold * (s_abs - clip_threshold) / (s_abs + clip_threshold);
          clipped * s.signum()
        }
      })
      .collect(),
  )
}

/// Normalizes the samples to a target maximum amplitude.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `target_max`: Target maximum amplitude after normalization.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Normalized samples or an error if parameters are invalid.
pub fn normalizer(samples: &[f32], target_max: f32) -> Result<Vec<f32>, String> {
  if target_max <= 0.0 {
    return Err("Target maximum must be positive.".to_string());
  }
  let current_max = samples.iter().fold(0.0_f32, |acc, &x| acc.max(x.abs()));
  if current_max == 0.0 {
    return Ok(samples.to_vec()); // Avoid division by zero
  }
  let gain = target_max / current_max;
  Ok(samples.iter().map(|&s| s * gain).collect())
}

/// Applies a noise gate with sidechain support.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `sidechain`: Sidechain input samples to control gating.
/// - `threshold`: Threshold level in linear scale.
/// - `attack_time`: Attack time in seconds.
/// - `release_time`: Release time in seconds.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Gated samples or an error if parameters are invalid.
pub fn gate_with_sidechain(
  samples: &[f32], sidechain: &[f32], threshold: f32, attack_time: f32, release_time: f32,
) -> Result<Vec<f32>, String> {
  let envelope = envelope_follower(
    sidechain,
    attack_time,
    release_time,
    None,
    Some(EnvelopeMethod::Peak),
    None,
  )?;

  let mut output = Vec::with_capacity(samples.len());
  for (&sample, &env_val) in samples.iter().zip(envelope.iter()) {
    if env_val <= threshold {
      output.push(0.0);
    } else {
      output.push(sample);
    }
  }
  Ok(output)
}

#[cfg(test)]
mod fails {
  use super::*;
  #[test]
  fn test_envelope_follower_rms() {
    // A signal with a gradual rise and fall
    let input_samples = vec![0.0, 0.3, 0.4, 0.5, 0.4, 0.3, 0.0];
    let attack = 0.02;
    let release = 0.02;
    let rms_window = 0.1; // 0.1 seconds
    let method = EnvelopeMethod::Rms(rms_window);

    let envelope = envelope_follower(&input_samples, attack, release, None, Some(method), None)
      .expect("Failed to calculate envelope");

    // Check length matches input
    assert_eq!(envelope.len(), input_samples.len());

    // Validate temporal evolution
    for i in 1..envelope.len() {
      if input_samples[i] > input_samples[i - 1] {
        // Envelope should rise with increasing input (account for smoothing)
        assert!(
          envelope[i] >= envelope[i - 1] - 0.01,
          "Envelope dropped unexpectedly at index {}",
          i
        );
      } else {
        // Envelope should fall with decreasing input
        assert!(
          envelope[i] <= envelope[i - 1] + 0.01,
          "Envelope rose unexpectedly at index {}",
          i
        );
      }
    }

    // Verify envelope stays within bounds for this input
    let max_possible_rms = 0.5; // Max value in input_samples
    for &env_val in envelope.iter() {
      assert!(
        env_val >= 0.0 && env_val <= max_possible_rms + 0.01,
        "Envelope out of bounds: {}",
        env_val
      );
    }
  }

  #[test]
  fn test_rms_spike_response() {
    let mut samples = vec![0.0; 100];
    samples[50] = 10.0; // Single spike
    let window_time = 0.5; // 0.5 seconds

    let result = envelope_follower(
      &samples,
      0.1,  // attack_time
      0.1,  // release_time
      None, // hold_time
      Some(EnvelopeMethod::Rms(window_time)),
      None, // pre_emphasis
    )
    .expect("Envelope follower failed");

    // Ensure the envelope rises and falls around the spike
    assert!(result[50] > result[49], "Envelope did not rise at spike");
    assert!(result[51] < result[50], "Envelope did not fall after spike");
  }

  #[test]
  fn test_rms_convergence() {
    let samples = vec![2.0; 100];
    let sample_rate = 100.0; // 100 Hz
    let window_time = 0.5; // 0.5 seconds
    let expected_rms = 2.0; // RMS of constant signal is the same as the value

    let result = envelope_follower(
      &samples,
      0.0,  // attack_time
      0.0,  // release_time
      None, // hold_time
      Some(EnvelopeMethod::Rms(window_time)),
      None, // pre_emphasis
    )
    .expect("Envelope follower failed");

    // Check if the result converges to the expected value within tolerance
    for &env in result.iter().skip(50) {
      // Skip transient startup
      assert!(
        (env - expected_rms).abs() < 1e-6,
        "RMS value did not converge to expected: got {}, expected {}",
        env,
        expected_rms
      );
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_compress_bass_to_kick() {
    // Define file paths for static assets
    let input_path = &dev_audio_asset("bass.wav");
    let sidechain_path = &dev_audio_asset("beat.wav");
    let output_path = &dev_audio_asset("test-compressed_bass_beat.wav");

    // Load signals
    let (input_audio, input_sample_rate) = read_audio_file(input_path).expect("Failed to read input file.");
    let (sidechain_audio, sidechain_sample_rate) =
      read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

    // Ensure sample rates match
    assert_eq!(
      input_sample_rate, sidechain_sample_rate,
      "Input and sidechain sample rates must match."
    );

    // Perform role-based compression
    let processed_audio = role_based_compression(
      Role::Kick,
      Role::Bass,
      input_audio,
      sidechain_audio,
      0.8, // Intensity
    )
    .expect("Role-based compression failed.");

    // Write the output to a file
    write_audio(input_sample_rate as usize, processed_audio, output_path);

    // Verify output file exists
    assert!(
      std::path::Path::new(output_path).exists(),
      "Output file not found: {}",
      output_path
    );

    println!("Test passed! Compressed audio written to '{}'", output_path);
  }

  /// Helper function to create default CompressorParams for testing.
  fn default_compressor_params() -> CompressorParams {
    CompressorParams {
      threshold: 1.0,
      ratio: 2.0,
      knee_width: 0.0,
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
      ..Default::default()
    }
  }

  #[test]
  fn test_calculate_threshold_empty_samples() {
    let input_samples: Vec<f32> = vec![];
    let factor = 1.0f32;
    let use_rms = true;
    let calculated_threshold = calculate_threshold(&input_samples, factor, use_rms).unwrap();
    assert_eq!(calculated_threshold, 0.0f32);
  }

  #[test]
  fn test_calculate_threshold_invalid_factor() {
    let input_samples = vec![0.1f32, 0.2f32, 0.3f32];
    let factor = 0.0f32;
    let use_rms = false;
    let result = calculate_threshold(&input_samples, factor, use_rms);
    assert!(result.is_err());
  }

  #[test]
  fn test_calculate_threshold_rms() {
    let input_samples = vec![0.3f32, -0.4f32, 0.5f32, -0.6f32];
    let factor = 1.0f32;
    let use_rms = true;
    let calculated_threshold = calculate_threshold(&input_samples, factor, use_rms).unwrap();

    let sum_sq: f32 = input_samples.iter().map(|&x| x * x).sum();
    let expected_rms = (sum_sq / input_samples.len() as f32).sqrt();

    assert!((calculated_threshold - expected_rms).abs() < 1e-4);
  }

  #[test]
  fn test_compressor_with_limiter() {
    let input_samples = vec![0.0f32, 0.5f32, 1.0f32, 1.5f32, 2.0f32];
    let mut compressor_params = default_compressor_params();
    compressor_params.threshold = 1.0f32;
    compressor_params.ratio = 2.0f32;
    compressor_params.knee_width = 0.0f32; // Hard knee
    compressor_params.attack_time = 0.0f32; // Instantaneous
    compressor_params.release_time = 0.0f32; // Instantaneous
    compressor_params.wet_dry_mix = 1.0f32; // Fully wet
    compressor_params.enable_limiter = true;
    compressor_params.limiter_threshold = Some(1.2f32);

    let compressed = compressor(&input_samples, compressor_params, None).unwrap();

    let expected_samples = vec![
      0.0f32, // 0.0 remains 0.0
      0.5f32, // 0.5 <= threshold, remains 0.5
      1.0f32, // 1.0 <= threshold, remains 1.0
      1.2f32, // 1.5 compressed to 1.25, then limited to 1.2
      1.2f32, // 2.0 compressed to 1.5, then limited to 1.2
    ];

    for (output, expected) in compressed.iter().zip(expected_samples.iter()) {
      assert!(
        (*output - *expected).abs() < 1e-6,
        "Output: {}, Expected: {}",
        output,
        expected
      );
    }
  }

  #[test]
  fn test_envelope_stabilization() {
    let input_samples = vec![1.0; 100];
    let attack = 0.02; // 20 ms
    let release = 0.02; // 20 ms
    let method = EnvelopeMethod::Rms(0.1);

    let envelope = envelope_follower(&input_samples, attack, release, None, Some(method), None)
      .expect("Failed to calculate envelope");

    // Skip transient phase
    for &env_val in envelope.iter().skip(50) {
      assert!(
        (env_val - 1.0).abs() < 0.1,
        "Envelope did not stabilize near 1.0, got {}",
        env_val
      );
    }
  }

  #[test]
  fn test_envelope_monotonicity() {
    let input_samples = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.4, 0.3, 0.2, 0.1];
    let attack = 0.01; // 10 ms
    let release = 0.02; // 20 ms
    let method = EnvelopeMethod::Peak;

    let envelope = envelope_follower(&input_samples, attack, release, None, Some(method), None)
      .expect("Failed to calculate envelope");

    for i in 1..envelope.len() {
      if input_samples[i] > input_samples[i - 1] {
        // Ensure envelope rises
        assert!(
          envelope[i] >= envelope[i - 1],
          "Envelope dropped during increasing input at index {}",
          i
        );
      } else {
        // Ensure envelope falls
        assert!(
          envelope[i] <= envelope[i - 1],
          "Envelope rose during decreasing input at index {}",
          i
        );
      }
    }
  }

  #[test]
  fn test_envelope_spike_responsiveness() {
    let mut input_samples = vec![0.0; 100];
    input_samples[50] = 1.0; // Single spike
    let attack = 0.01; // 10 ms
    let release = 0.05; // 50 ms
    let method = EnvelopeMethod::Peak;

    let envelope = envelope_follower(&input_samples, attack, release, None, Some(method), None)
      .expect("Failed to calculate envelope");

    // Ensure rise after spike
    assert!(envelope[51] > envelope[50], "Envelope did not rise after spike");

    // Ensure decay after spike
    for i in 52..60 {
      assert!(
        envelope[i] < envelope[i - 1],
        "Envelope did not decay after spike at index {}",
        i
      );
    }
  }

  #[test]
  fn test_envelope_rate_of_change() {
    let input_samples = vec![0.0, 0.5, 1.0, 0.5, 0.0];
    let attack = 0.01; // 10 ms
    let release = 0.05; // 50 ms
    let method = EnvelopeMethod::Peak;

    let envelope = envelope_follower(&input_samples, attack, release, None, Some(method), None)
      .expect("Failed to calculate envelope");

    let max_attack_rate = 1.0 / attack; // Assuming normalized signal
    let max_release_rate = 1.0 / release;

    for i in 1..envelope.len() {
      let rate = (envelope[i] - envelope[i - 1]).abs();
      if envelope[i] > envelope[i - 1] {
        // Check attack rate
        assert!(rate <= max_attack_rate, "Attack rate too fast: {} at index {}", rate, i);
      } else {
        // Check release rate
        assert!(
          rate <= max_release_rate,
          "Release rate too fast: {} at index {}",
          rate,
          i
        );
      }
    }
  }

  #[test]
  fn test_envelope_follower_peak() {
    let input_samples = vec![0.0, 0.1, 0.2, 0.4, 0.2, 0.1, 0.0];
    let attack = 0.01;
    let release = 0.01;
    let method = EnvelopeMethod::Peak;
    let envelope = envelope_follower(&input_samples, attack, release, None, Some(method), None).unwrap();

    assert_eq!(envelope.len(), input_samples.len());

    for (i, &env_val) in envelope.iter().enumerate() {
      assert!(env_val >= 0.0);
      let max_val = input_samples.iter().map(|&x| x.abs()).fold(0.0_f32, |a, b| a.max(b));
      assert!(env_val <= max_val + 0.1);
    }
  }

  #[test]
  fn test_rms_smoothness() {
    let samples = vec![1.0; 100];
    let window_time = 0.5; // 0.5 seconds
    let result = envelope_follower(
      &samples,
      0.0,  // attack_time
      0.0,  // release_time
      None, // hold_time
      Some(EnvelopeMethod::Rms(window_time)),
      None, // pre_emphasis
    )
    .expect("Envelope follower failed");

    for i in 1..result.len() {
      // Ensure no abrupt changes in the envelope
      assert!(
        (result[i] - result[i - 1]).abs() <= 1.0,
        "Output is not smooth: abrupt change at index {}",
        i
      );
    }
  }

  #[test]
  fn test_rms_zero_signal() {
    let samples = vec![0.0; 100];
    let window_time = 0.5; // 0.5 seconds

    let result = envelope_follower(
      &samples,
      0.0,  // attack_time
      0.0,  // release_time
      None, // hold_time
      Some(EnvelopeMethod::Rms(window_time)),
      None, // pre_emphasis
    )
    .expect("Envelope follower failed");

    for &env in result.iter() {
      assert!(env.abs() < 1e-6, "RMS value of zero signal is not zero: got {}", env);
    }
  }

  #[test]
  fn test_normalizer_constant_zero() {
    let input_samples = vec![0.0f32, 0.0f32, 0.0f32];
    let target_max = 1.0f32;
    let normalized = normalizer(&input_samples, target_max).unwrap();
    assert_eq!(normalized, input_samples);
  }

  #[test]
  fn test_normalizer_varied_amplitudes() {
    let input_samples = vec![0.5f32, -1.0f32, 0.75f32, -0.25f32];
    let target_max = 2.0f32;
    let normalized = normalizer(&input_samples, target_max).unwrap();
    let max_val = normalized.iter().map(|&x| x.abs()).fold(0.0_f32, |a, b| a.max(b));
    assert!((max_val - 2.0).abs() < 1e-6);
  }

  #[test]
  fn test_soft_clipper_edge_cases() {
    let input_samples = vec![0.1f32, -0.2f32, 0.3f32];
    let clip_threshold = 1.0f32;
    let output = soft_clipper(&input_samples, clip_threshold).unwrap();
    assert_eq!(output, input_samples);
  }

  #[test]
  fn test_power_reduction() {
    let input_path = &dev_audio_asset("bass.wav");
    let sidechain_path = &dev_audio_asset("beat.wav");

    // Load signals
    let (input_audio, _) = read_audio_file(input_path).expect("Failed to read input file.");
    let (sidechain_audio, _) = read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

    // Perform compression
    let processed_audio = role_based_compression(
      Role::Bass,
      Role::Kick,
      input_audio.clone(),
      sidechain_audio,
      0.8, // Intensity
    )
    .expect("Role-based compression failed.");

    // Calculate RMS power
    let rms_before = calculate_rms(&input_audio.iter().flatten().cloned().collect::<Vec<f32>>());
    let rms_after = calculate_rms(&processed_audio.iter().flatten().cloned().collect::<Vec<f32>>());

    assert!(
      rms_after < rms_before,
      "RMS power did not decrease: before = {}, after = {}",
      rms_before,
      rms_after
    );
  }

  #[test]
  fn test_dynamic_range_reduction() {
    let input_path = &dev_audio_asset("bass.wav");
    let sidechain_path = &dev_audio_asset("beat.wav");

    // Load signals
    let (input_audio, _) = read_audio_file(input_path).expect("Failed to read input file.");
    let (sidechain_audio, _) = read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

    // Perform compression
    let processed_audio = role_based_compression(
      Role::Bass,
      Role::Kick,
      input_audio.clone(),
      sidechain_audio,
      0.8, // Intensity
    )
    .expect("Role-based compression failed.");

    // Calculate dynamic range
    let dynamic_range_before = calculate_dynamic_range(&input_audio.iter().flatten().cloned().collect::<Vec<f32>>());
    let dynamic_range_after = calculate_dynamic_range(&processed_audio.iter().flatten().cloned().collect::<Vec<f32>>());

    assert!(
      dynamic_range_after < dynamic_range_before,
      "Dynamic range did not decrease: before = {}, after = {}",
      dynamic_range_before,
      dynamic_range_after
    );
  }

  #[test]
  fn test_sidechain_envelope_alignment() {
    let input_path = &dev_audio_asset("bass.wav");
    let sidechain_path = &dev_audio_asset("beat.wav");

    // Load signals
    let (input_audio, _) = read_audio_file(input_path).expect("Failed to read input file.");
    let (sidechain_audio, _) = read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

    // Perform compression
    let processed_audio = role_based_compression(
      Role::Bass,
      Role::Kick,
      input_audio.clone(),
      sidechain_audio.clone(),
      0.8, // Intensity
    )
    .expect("Role-based compression failed.");

    // Extract sidechain envelope
    let sidechain_flat = sidechain_audio.iter().flatten().cloned().collect::<Vec<_>>();
    let sidechain_env = envelope_follower(
      &sidechain_flat,
      0.01, // Attack time
      0.1,  // Release time
      None,
      Some(EnvelopeMethod::Peak),
      None,
    )
    .expect("Failed to calculate sidechain envelope.");

    // Validate bassline attenuation
    let bass_flat = processed_audio.iter().flatten().cloned().collect::<Vec<_>>();
    for (&env_val, &bass_sample) in sidechain_env.iter().zip(bass_flat.iter()) {
      assert!(
        bass_sample <= env_val,
        "Bass sample {} exceeds sidechain envelope {}",
        bass_sample,
        env_val
      );
    }
  }

  #[test]
  fn test_visualize_results() {
    let input_path = &dev_audio_asset("bass.wav");
    let sidechain_path = &dev_audio_asset("beat.wav");

    // Load signals
    let (input_audio, _) = read_audio_file(input_path).expect("Failed to read input file.");
    let (sidechain_audio, _) = read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

    // Perform compression
    let processed_audio = role_based_compression(
      Role::Bass,
      Role::Kick,
      input_audio.clone(),
      sidechain_audio,
      0.8, // Intensity
    )
    .expect("Role-based compression failed.");

    // Debugging output
    let rms_before = calculate_rms(&input_audio.iter().flatten().cloned().collect::<Vec<f32>>());
    let rms_after = calculate_rms(&processed_audio.iter().flatten().cloned().collect::<Vec<f32>>());
    let dynamic_range_before = calculate_dynamic_range(&input_audio.iter().flatten().cloned().collect::<Vec<f32>>());
    let dynamic_range_after = calculate_dynamic_range(&processed_audio.iter().flatten().cloned().collect::<Vec<f32>>());

    println!("RMS Before: {}, RMS After: {}", rms_before, rms_after);
    println!(
      "Dynamic Range Before: {}, After: {}",
      dynamic_range_before, dynamic_range_after
    );
  }

  #[test]
  fn test_rms_power_reduction() {
    let input_path = &dev_audio_asset("bass.wav");
    let sidechain_path = &dev_audio_asset("beat.wav");

    // Load signals
    let (input_audio, _) = read_audio_file(input_path).expect("Failed to read input file.");
    let (sidechain_audio, _) = read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

    // Perform compression
    let processed_audio = role_based_compression(
      Role::Bass,
      Role::Kick,
      input_audio.clone(),
      sidechain_audio,
      0.8, // Intensity
    )
    .expect("Role-based compression failed.");

    // Calculate RMS power
    let rms_before = calculate_rms(&input_audio.iter().flatten().cloned().collect::<Vec<f32>>());
    let rms_after = calculate_rms(&processed_audio.iter().flatten().cloned().collect::<Vec<f32>>());

    println!("RMS Before: {:.6}, RMS After: {:.6}", rms_before, rms_after);
    assert!(
      rms_after < rms_before,
      "RMS power did not decrease: before = {:.6}, after = {:.6}",
      rms_before,
      rms_after
    );
  }

  #[test]
  fn test_dynamic_range_reduction_analysis() {
    let input_path = &dev_audio_asset("beat.wav");
    let sidechain_path = &dev_audio_asset("bass.wav");

    // Load signals
    let (input_audio, _) = read_audio_file(input_path).expect("Failed to read input file.");
    let (sidechain_audio, _) = read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

    // Perform compression
    let processed_audio = role_based_compression(
      Role::Bass,
      Role::Kick,
      input_audio.clone(),
      sidechain_audio,
      0.8, // Intensity
    )
    .expect("Role-based compression failed.");

    // Calculate dynamic range
    let dynamic_range_before = calculate_dynamic_range(&input_audio.iter().flatten().cloned().collect::<Vec<f32>>());
    let dynamic_range_after = calculate_dynamic_range(&processed_audio.iter().flatten().cloned().collect::<Vec<f32>>());

    println!(
      "Dynamic Range Before: {:.6}, After: {:.6}",
      dynamic_range_before, dynamic_range_after
    );
    assert!(
      dynamic_range_after < dynamic_range_before,
      "Dynamic range did not decrease: before = {:.6}, after = {:.6}",
      dynamic_range_before,
      dynamic_range_after
    );
  }
}

/// Make the beat "bussin" by applying normalization, compression, and transient shaping.
pub fn make_beat_bussin(input_path: &str, output_path: &str) {
  use crate::synth::SR;

  // Step 1: Load and resample audio
  let (audio, target_sample_rate) = crate::fastmast::load_and_resample_audio(input_path, SR as u32);
  let num_channels = audio.len();
  assert!(num_channels > 0, "Audio must have at least one channel.");

  // Step 2: Process each channel separately
  let mut processed_audio = Vec::new();
  for channel in audio {
    // Step 2a: Apply normalization
    let normalized = normalizer(&channel, 0.9).expect("Failed to normalize audio");

    // Step 2b: Apply transient shaping
    let transient_params = TransientShaperParams {
      transient_emphasis: 2.0,
      threshold: 0.6,
      attack_time: 0.01,
      release_time: 0.1,
      detection_method: EnvelopeMethod::Peak,
      makeup_gain: 1.2,
      ratio: 1.0,
      knee_width: 0.0,
      wet_dry_mix: 1.0,
    };
    let transient_shaped = transient_shaper(&normalized, transient_params).expect("Failed to apply transient shaping");

    // Step 2c: Apply soft clipping
    let clipped = soft_clipper(&transient_shaped, 0.8).expect("Failed to apply soft clipping");

    processed_audio.push(clipped);
  }

  // Step 3: Write processed audio to output
  crate::render::engrave::write_audio(target_sample_rate as usize, processed_audio, output_path)
}

/// Apply compressor with rolled parameters to make the beat bussin.
pub fn make_beat_bussin_with_roll(input_path: &str, output_path: &str) {
  let (audio, sample_rate) = read_audio_file(input_path).expect("Failed to read input file.");
  let num_channels = audio.len();

  // Roll random compressor parameters
  let compressor_params = roll_compressor_params(
    -30.0, -6.0, // Min/max threshold in dB
    2.0, 10.0, // Min/max ratio
    0.001, 0.1, // Min/max attack time in seconds
    0.01, 0.5, // Min/max release time in seconds
  );

  let mut processed_audio: Vec<Vec<f32>> = Vec::new();

  for channel in audio.iter() {
    let compressed = compressor(channel, compressor_params, None).expect("Compression failed.");
    processed_audio.push(compressed);
  }

  write_audio(sample_rate as usize, processed_audio, output_path);
}

/// Generate randomized compressor parameters within defined ranges.
fn roll_compressor_params(
  min_threshold: f32, max_threshold: f32, min_ratio: f32, max_ratio: f32, min_attack: f32, max_attack: f32,
  min_release: f32, max_release: f32,
) -> CompressorParams {
  let mut rng = rand::thread_rng();
  CompressorParams {
    threshold: rng.gen_range(min_threshold..max_threshold),
    ratio: rng.gen_range(min_ratio..max_ratio),
    knee_width: rng.gen_range(0.0..1.0),  // Default range for knee width
    makeup_gain: rng.gen_range(0.5..2.0), // Amplify or attenuate post-compression
    attack_time: rng.gen_range(min_attack..max_attack),
    release_time: rng.gen_range(min_release..max_release),
    lookahead_time: None, // Can add lookahead randomization if desired
    detection_method: EnvelopeMethod::Peak,
    hold_time: None,
    wet_dry_mix: rng.gen_range(0.5..1.0), // Ensure mostly wet signal
    sidechain_filter: None,
    auto_gain: false,
    ratio_slope: RatioSlope::Linear,
    enable_limiter: false,
    limiter_threshold: None,
    envelope_shaping: None,
  }
}

#[test]
fn test_make_beat_bussin_with_roll() {
  let input_path = &dev_audio_asset("beat.wav");
  let output_path = &dev_audio_asset("test-output-bussin-roll.wav");

  println!(
    "Testing make_beat_bussin_with_roll from '{}' to '{}'",
    input_path, output_path
  );

  // Call the function
  make_beat_bussin_with_roll(input_path, output_path);

  // Verify output
  use std::path::Path;
  assert!(
    Path::new(output_path).exists(),
    "Output file '{}' was not created.",
    output_path
  );

  // Validate the output
  let (output_audio, output_sample_rate) =
    read_audio_file(output_path).unwrap_or_else(|err| panic!("Failed to read output file '{}': {}", output_path, err));
  assert_eq!(output_sample_rate, crate::synth::SR as u32, "Sample rate mismatch.");
  assert!(!output_audio.is_empty(), "Output audio is empty.");
  assert_eq!(output_audio.len(), 2, "Expected 2 channels in output audio.");

  println!(
    "test_make_beat_bussin_with_roll passed, output written to '{}'",
    output_path
  );
}

#[test]
fn test_make_beat_bussin() {
  use crate::analysis::sampler::{read_audio_file, write_audio_file, AudioFormat};
  let input_path = &dev_audio_asset("beat.wav");
  let output_path = &dev_audio_asset("test-output-bussin.wav");

  println!("Testing make_beat_bussin from '{}' to '{}'", input_path, output_path);

  // Call the make_beat_bussin function
  make_beat_bussin(input_path, output_path);

  // Verify output
  use std::path::Path;
  assert!(
    Path::new(output_path).exists(),
    "Output file '{}' was not created.",
    output_path
  );

  // Validate the output
  let (output_audio, output_sample_rate) =
    read_audio_file(output_path).unwrap_or_else(|err| panic!("Failed to read output file '{}': {}", output_path, err));
  assert_eq!(output_sample_rate, crate::synth::SR as u32, "Sample rate mismatch.");
  assert!(!output_audio.is_empty(), "Output audio is empty.");
  assert_eq!(output_audio.len(), 2, "Expected 2 channels in output audio.");

  println!(
    "make_beat_bussin test passed, output written to '{}', sample rate: {}",
    output_path, output_sample_rate
  );
}

/// Calculates RMS power of the given audio samples.
pub fn calculate_rms(samples: &[f32]) -> f32 {
  (samples.iter().map(|&x| x * x).sum::<f32>() / samples.len() as f32).sqrt()
}

/// Calculates the dynamic range of the given audio samples.
pub fn calculate_dynamic_range(samples: &[f32]) -> f32 {
  let max = samples.iter().cloned().fold(f32::MIN, f32::max);
  let min = samples.iter().cloned().fold(f32::MAX, f32::min);
  max - min
}
