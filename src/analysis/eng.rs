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

use crate::synth::{SR, SRf};
use std::error::Error;
use biquad::{Biquad, Coefficients, DirectForm1, Hertz, Type as FilterType};
use itertools::izip;
use crate::phrasing::ranger::{MIN_DB, MAX_DB, DYNAMIC_RANGE_DB};
use crate::timbre::Role;

use crate::analysis::sampler::read_audio_file;
use crate::render::engrave::write_audio;
use rand::Rng;


pub fn dev_audio_asset(label:&str) -> String {
    format!("dev-audio/{}", label)
}


/// Enumeration for different envelope detection methods.
#[derive(Debug, Clone, Copy)]
pub enum EnvelopeMethod {
    /// Peak envelope detection.
    Peak,
    /// Root Mean Square (RMS) envelope detection.
    Rms,
    /// Hybrid envelope detection combining Peak and RMS.
    Hybrid,
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
    role1: Role, 
    role2: Role, 
    signal1: Vec<Vec<f32>>, 
    signal2: Vec<Vec<f32>>, 
    intensity: f32,
) -> Result<Vec<Vec<f32>>, String> {
    // Define compression parameters based on roles
    let compressor_params = match (role1, role2) {
        (Role::Kick, Role::Bass) => CompressorParams {
            threshold: -24.0,
            ratio: 1f32/4.0,
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
        _ => CompressorParams::default(), // Generic settings
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
    input: Vec<Vec<f32>>,
    sidechain: Vec<Vec<f32>>,
    compressor_params: CompressorParams,
    intensity: f32,
) -> Vec<Vec<f32>> {
    let n_input = input.len();
    let n_sidechain = sidechain.len();

    // Ensure intensity is within bounds
    let intensity = intensity.clamp(0.0, 1.0);

    // Helper function to process and scale a single channel
    let compress_and_scale = |input_channel: &[f32], sidechain_channel: &[f32]| -> Vec<f32> {
        let compressed = compressor(input_channel, compressor_params, Some(sidechain_channel))
            .expect("Compression failed.");
        compressed
            .iter()
            .zip(input_channel.iter())
            .map(|(&compressed_sample, &original_sample)| {
                compressed_sample * intensity + original_sample * (1.0 - intensity)
            })
            .collect()
    };

    match (n_input, n_sidechain) {
        // Mono input and mono sidechain
        (1, 1) => vec![compress_and_scale(&input[0], &sidechain[0])],

        // Mono input and stereo sidechain
        (1, 2) => {
            let downmixed_sidechain = downmix_stereo_to_mono(&sidechain[0], &sidechain[1])
                .expect("Failed to downmix sidechain.");
            vec![compress_and_scale(&input[0], &downmixed_sidechain)]
        }

        // Stereo input and mono sidechain
        (2, 1) => vec![
            compress_and_scale(&input[0], &sidechain[0]),
            compress_and_scale(&input[1], &sidechain[0]),
        ],

        // Stereo input and stereo sidechain
        (2, 2) => {
            let downmixed_sidechain = downmix_stereo_to_mono(&sidechain[0], &sidechain[1])
                .expect("Failed to downmix sidechain.");
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
    samples: &[f32],
    attack_time: f32,
    release_time: f32,
    hold_time: Option<f32>,
    method: Option<EnvelopeMethod>,
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

    for &sample in processed_samples.iter() {
        let val = match envelope_method {
            EnvelopeMethod::Peak => sample.abs(),
            EnvelopeMethod::Rms => (sample * sample).sqrt(),
            EnvelopeMethod::Hybrid => (sample.abs() + (sample * sample).sqrt()) / 2.0,
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

/// Applies dynamic range compression to the samples based on the provided parameters.
/// Supports sidechain input, lookahead, and separate attack/release times.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `params`: Compressor parameters.
/// - `sidechain`: Optional sidechain input samples.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Compressed samples or an error if parameters are invalid.
pub fn compressor(
    samples: &[f32],
    params: CompressorParams,
    sidechain: Option<&[f32]>,
) -> Result<Vec<f32>, String> {
    if params.ratio < 1.0 {
        return Err("Compression ratio must be >= 1.0.".to_string());
    }
    if let Some(t) = params.lookahead_time {
        if t < 0.0 {
            return Err("Lookahead time must be non-negative.".to_string());
        }
    }

    let lookahead_samples = params
        .lookahead_time
        .map(|t| (t * SRf).round() as usize)
        .unwrap_or(0);
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

    let mut output = Vec::with_capacity(samples.len());
    for (i, &sample) in samples.iter().enumerate() {
        let env_val = envelope[i];
        let gain_reduction = if env_val > params.threshold {
            let compressed_level = params.threshold + (env_val - params.threshold) / params.ratio;
            if env_val != 0.0 {
                compressed_level / env_val
            } else {
                1.0
            }
        } else {
            1.0
        };

        // Apply envelope shaping if specified
        let final_gain = if let Some(shaping) = params.envelope_shaping {
            match shaping.shape_type {
                ShapeType::Soft => 1.0 - (1.0 - gain_reduction).powf(2.0), // Example soft shaping
                ShapeType::Hard => gain_reduction.powf(3.0), // Example hard shaping
                ShapeType::Custom => gain_reduction,         // Placeholder for custom shaping
            }
        } else {
            gain_reduction
        };

        // Apply gain reduction
        let compressed_sample = sample * final_gain;

        // Apply makeup gain
        let makeup = if params.auto_gain {
            params.makeup_gain
        } else {
            params.makeup_gain
        };

        let final_sample = compressed_sample * makeup;

        // Apply wet/dry mix
        let mixed_sample = params.wet_dry_mix * final_sample + (1.0 - params.wet_dry_mix) * sample;

        output.push(mixed_sample);
    }

    // Apply limiter if enabled
    if params.enable_limiter {
        let limiter_threshold = params
            .limiter_threshold
            .unwrap_or(params.threshold);
        let limited_output = limiter(&output, limiter_threshold);
        output = limited_output;
    }

    Ok(output)
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
pub fn expander(
    samples: &[f32],
    params: ExpanderParams,
    sidechain: Option<&[f32]>,
) -> Result<Vec<f32>, String> {
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
                ShapeType::Hard => gain_increase.powf(3.0), // Example hard shaping
                ShapeType::Custom => gain_increase,         // Placeholder for custom shaping
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
pub fn compand(
    samples: &[f32],
    params: CompanderParams,
    sidechain: Option<&[f32]>,
) -> Result<Vec<f32>, String> {
    let compressed = compressor(samples, params.compressor, sidechain)?;
    let expanded = expander(&compressed, params.expander, None)?;
    Ok(expanded)
}

/// Applies transient shaping by enhancing or attenuating transients based on the envelope.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `params`: Transient shaper parameters.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Transient-shaped samples or an error if parameters are invalid.
pub fn transient_shaper(
    samples: &[f32],
    params: TransientShaperParams,
) -> Result<Vec<f32>, String> {
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

/// Splits the signal into multiple frequency bands using biquad filters.
///
/// Note: For real-world applications, more sophisticated filtering (e.g., multi-order biquad filters) is recommended.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `crossover_freqs`: Vector of crossover frequencies in Hz.
/// - `filter_types`: Vector of filter types for each crossover frequency.
/// - `filter_slopes`: Vector of filter slopes for each crossover frequency.
///
/// # Returns
/// - `Result<Vec<Vec<f32>>, String>`: Vector of frequency band samples or an error if parameters are invalid.
fn split_bands(
    samples: &[f32],
    crossover_freqs: &[f32],
    filter_types: &[FilterType<()>],
    filter_slopes: &[FilterSlope],
) -> Result<Vec<Vec<f32>>, String> {
    if crossover_freqs.len() != filter_types.len() || crossover_freqs.len() != filter_slopes.len() {
        return Err("Crossover frequencies, filter types, and filter slopes must have the same length.".to_string());
    }

    let mut bands = Vec::new();
    let mut previous_freq = 0.0;
    for ((&freq, &filter_type), &filter_slope) in crossover_freqs.iter().zip(filter_types.iter()).zip(filter_slopes.iter()) {
        let band = band_split(samples, previous_freq, freq, filter_type, filter_slope)?;
        bands.push(band);
        previous_freq = freq;
    }
    // Last band up to Nyquist frequency using SRf / 2.0
    let last_band = band_split(samples, previous_freq, SRf / 2.0, FilterType::LowPass::<()>, FilterSlope::TwoPole)?;
    bands.push(last_band);
    Ok(bands)
}

/// Splits the signal between two frequencies using biquad high-pass and low-pass filters.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `low_freq`: Lower cutoff frequency in Hz.
/// - `high_freq`: Upper cutoff frequency in Hz.
/// - `filter_type`: Type of filter to apply.
/// - `filter_slope`: Slope of the filter.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Band-split samples or an error if frequencies are invalid.
fn band_split(
    samples: &[f32],
    low_freq: f32,
    high_freq: f32,
    filter_type: FilterType<()>,
    filter_slope: FilterSlope,
) -> Result<Vec<f32>, String> {
    if low_freq >= high_freq {
        return Err("Low frequency must be less than high frequency.".to_string());
    }

    // Apply the specified filter type
    let filtered = match filter_type {
        FilterType::HighPass::<()> => apply_highpass(samples, low_freq)?,
        FilterType::LowPass::<()> => apply_lowpass(samples, high_freq)?,
        FilterType::BandPass::<()> => {
            // Example: Apply high-pass then low-pass for band-pass
            let hp_filtered = apply_highpass(samples, low_freq)?;
            apply_lowpass(&hp_filtered, high_freq)?
        },
        _ => return Err("Unsupported filter type for band splitting.".to_string()),
    };

    Ok(filtered)
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
pub fn expander_with_sidechain(
    samples: &[f32],
    params: ExpanderParams,
    sidechain: &[f32],
) -> Result<Vec<f32>, String> {
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
pub fn gate(
    samples: &[f32],
    threshold: f32,
    attack_time: f32,
    release_time: f32,
) -> Result<Vec<f32>, String> {
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
pub fn calculate_threshold(
    samples: &[f32],
    factor: f32,
    use_rms: bool,
) -> Result<f32, String> {
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
    Ok(samples
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
        .collect())
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
    samples: &[f32],
    sidechain: &[f32],
    threshold: f32,
    attack_time: f32,
    release_time: f32,
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
mod tests {
    use super::*;

    #[test]
    fn test_compress_bass_to_kick() {
        // Define file paths for static assets
        let input_path = &dev_audio_asset("bass.wav");  
        let sidechain_path = &dev_audio_asset("beat.wav"); 
        let output_path = &dev_audio_asset("test-compressed_bass_beat.wav");

        // Load signals
        let (input_audio, input_sample_rate) =
            read_audio_file(input_path).expect("Failed to read input file.");
        let (sidechain_audio, sidechain_sample_rate) =
            read_audio_file(sidechain_path).expect("Failed to read sidechain file.");

        // Ensure sample rates match
        assert_eq!(
            input_sample_rate, sidechain_sample_rate,
            "Input and sidechain sample rates must match."
        );

        // Perform role-based compression
        let processed_audio = role_based_compression(
            Role::Bass,
            Role::Kick,
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
            0.0f32,          // 0.0 remains 0.0
            0.5f32,          // 0.5 <= threshold, remains 0.5
            1.0f32,          // 1.0 <= threshold, remains 1.0
            1.2f32,          // 1.5 compressed to 1.25, then limited to 1.2
            1.2f32,          // 2.0 compressed to 1.5, then limited to 1.2
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
    fn test_envelope_follower_peak() {
        let input_samples = vec![0.0, 0.1, 0.2, 0.4, 0.2, 0.1, 0.0];
        let attack = 0.01;
        let release = 0.01;
        let method = EnvelopeMethod::Peak;
        let envelope = envelope_follower(
            &input_samples,
            attack,
            release,
            None,
            Some(method),
            None,
        ).unwrap();

        assert_eq!(envelope.len(), input_samples.len());

        for (i, &env_val) in envelope.iter().enumerate() {
            assert!(env_val >= 0.0);
            let max_val = input_samples.iter().map(|&x| x.abs()).fold(0.0_f32, |a, b| a.max(b));
            assert!(env_val <= max_val + 0.1);
        }
    }

    #[test]
    fn test_envelope_follower_rms() {
        let input_samples = vec![0.0, 0.3, 0.4, 0.5, 0.4, 0.3, 0.0];
        let attack = 0.02;
        let release = 0.02;
        let method = EnvelopeMethod::Rms;
        let envelope = envelope_follower(
            &input_samples,
            attack,
            release,
            None,
            Some(method),
            None,
        ).unwrap();

        assert_eq!(envelope.len(), input_samples.len());

        for &env_val in envelope.iter() {
            assert!(env_val >= 0.0 && env_val <= 1.0 + 0.1);
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
        let transient_shaped = transient_shaper(&normalized, transient_params)
            .expect("Failed to apply transient shaping");

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
        -30.0, -6.0,  // Min/max threshold in dB
        2.0, 10.0,    // Min/max ratio
        0.001, 0.1,   // Min/max attack time in seconds
        0.01, 0.5     // Min/max release time in seconds
    );

    let mut processed_audio: Vec<Vec<f32>> = Vec::new();

    for channel in audio.iter() {
        let compressed = compressor(channel, compressor_params, None)
            .expect("Compression failed.");
        processed_audio.push(compressed);
    }

    write_audio(sample_rate as usize, processed_audio, output_path);
}

/// Generate randomized compressor parameters within defined ranges.
fn roll_compressor_params(min_threshold: f32, max_threshold: f32, 
                          min_ratio: f32, max_ratio: f32,
                          min_attack: f32, max_attack: f32,
                          min_release: f32, max_release: f32) -> CompressorParams {
    let mut rng = rand::thread_rng();
    CompressorParams {
        threshold: rng.gen_range(min_threshold..max_threshold),
        ratio: rng.gen_range(min_ratio..max_ratio),
        knee_width: rng.gen_range(0.0..1.0), // Default range for knee width
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

    println!("Testing make_beat_bussin_with_roll from '{}' to '{}'", input_path, output_path);

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
    let (output_audio, output_sample_rate) = read_audio_file(output_path)
        .unwrap_or_else(|err| panic!("Failed to read output file '{}': {}", output_path, err));
    assert_eq!(output_sample_rate, crate::synth::SR as u32, "Sample rate mismatch.");
    assert!(!output_audio.is_empty(), "Output audio is empty.");
    assert_eq!(output_audio.len(), 2, "Expected 2 channels in output audio.");

    println!("test_make_beat_bussin_with_roll passed, output written to '{}'", output_path);
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
    let (output_audio, output_sample_rate) = read_audio_file(output_path)
        .unwrap_or_else(|err| panic!("Failed to read output file '{}': {}", output_path, err));
    assert_eq!(output_sample_rate, crate::synth::SR as u32, "Sample rate mismatch.");
    assert!(!output_audio.is_empty(), "Output audio is empty.");
    assert_eq!(output_audio.len(), 2, "Expected 2 channels in output audio.");

    println!(
        "make_beat_bussin test passed, output written to '{}', sample rate: {}",
        output_path, output_sample_rate
    );
}
