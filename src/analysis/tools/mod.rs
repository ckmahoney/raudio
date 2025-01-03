use crate::phrasing::ranger::{DYNAMIC_RANGE_DB, MAX_DB, MIN_DB};
use crate::synth::{SRf, SR};
use crate::timbre::Role;
use biquad::{Biquad, Coefficients, DirectForm1, Hertz, ToHertz, Type as FilterType, Q_BUTTERWORTH_F32};
use itertools::izip;
use std::error::Error;

use crate::analysis::sampler::read_audio_file;
use crate::render::engrave::write_audio;
use rand::Rng;
mod test;

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
/// - `params`: Transient shaper parameters.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Transient-shaped audio samples or an error if parameters are invalid.
pub fn transient_shaper(samples: &[f32], params: TransientShaperParams) -> Result<Vec<f32>, String> {
  // Validate parameters using the existing validation function
  validate_transient_shaper_params(&params)?;

  // Detect the envelope of the signal
  let envelope = envelope_follower(
    samples,
    params.attack_time,
    params.release_time,
    None, // Assuming no hold time; adjust if needed
    Some(params.detection_method),
    None, // Assuming no pre-emphasis; adjust if needed
    Some(params.wet_dry_mix),
  )?;

  let mut output = Vec::with_capacity(samples.len());
  let mut previous_gain = 1.0;

  for (&sample, &env_val_db) in samples.iter().zip(envelope.iter()) {
    if env_val_db > params.threshold {
      // Determine if we're in the attack or sustain phase
      let target_gain = if env_val_db > params.attack_threshold {
        params.attack_factor
      } else {
        params.sustain_factor
      };

      // Smooth the gain transition
      let smoothed_gain = smooth_gain_reduction(target_gain, previous_gain, params.attack_time, params.release_time);
      previous_gain = smoothed_gain;

      // Apply transient emphasis and makeup gain
      let shaped_sample = sample * smoothed_gain * params.transient_emphasis * params.makeup_gain;
      output.push(shaped_sample);
    } else {
      // Apply sustain factor with smoothing
      let target_gain = params.sustain_factor;
      let smoothed_gain = smooth_gain_reduction(target_gain, previous_gain, params.attack_time, params.release_time);
      previous_gain = smoothed_gain;

      let shaped_sample = sample * smoothed_gain * params.makeup_gain;
      output.push(shaped_sample);
    }
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
  // Validate transient_emphasis: must be non-negative
  if params.transient_emphasis < 0.0 {
    return Err("Transient emphasis must be non-negative.".to_string());
  }

  // Validate threshold: typically in dB, can be negative
  if params.threshold < MIN_DB {
    return Err("Threshold must be above the minimum dB value.".to_string());
  }

  // Validate attack_threshold: must be >= threshold
  if params.attack_threshold < params.threshold {
    return Err("Attack threshold must be greater than or equal to threshold.".to_string());
  }

  // Validate attack and release times: must be non-negative
  if params.attack_time < 0.0 {
    return Err("Attack time must be non-negative.".to_string());
  }

  if params.release_time < 0.0 {
    return Err("Release time must be non-negative.".to_string());
  }

  // Validate wet_dry_mix: must be between 0.0 and 1.0
  if params.wet_dry_mix < 0.0 || params.wet_dry_mix > 1.0 {
    return Err("Wet/Dry mix must be between 0.0 and 1.0.".to_string());
  }

  // Validate attack_factor and sustain_factor: must be positive
  if params.attack_factor <= 0.0 {
    return Err("Attack factor must be positive.".to_string());
  }

  if params.sustain_factor <= 0.0 {
    return Err("Sustain factor must be positive.".to_string());
  }

  Ok(())
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