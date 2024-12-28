// audio.rs
//
// This module provides refined implementations of typical audio processing stages,
// including envelope following, compression, gating, transient shaping, soft clipping,
// normalization, and interleaving/deinterleaving. Enhancements support advanced
// waveshaping effects like multi-band compression, dynamic range expansion, and companding.
//
// Dependencies:
// - crate::synth::{SR, SRf} for sample-rate constants.

use crate::synth::{SR, SRf};
use std::error::Error;

/// Enumeration for different envelope detection methods.
#[derive(Debug, Clone, Copy)]
pub enum EnvelopeMethod {
    /// Peak envelope detection.
    Peak,
    /// Root Mean Square (RMS) envelope detection.
    Rms,
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
}

/// Struct to hold compander parameters, combining compression and expansion.
#[derive(Debug, Clone, Copy)]
pub struct CompanderParams {
    /// Parameters for the compression stage.
    pub compressor: CompressorParams,
    /// Parameters for the expansion stage.
    pub expander: ExpanderParams,
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
}

/// Struct to hold multi-band compressor parameters.
#[derive(Debug, Clone)]
pub struct MultiBandCompressorParams {
    /// Vector of compressor parameters for each frequency band.
    pub bands: Vec<CompressorParams>,
    /// Vector of crossover frequencies defining the boundaries between bands.
    pub crossover_freqs: Vec<f32>,
}

/// Struct to hold parameters for individual bands in multi-band compression.
#[derive(Debug, Clone, Copy)]
pub struct MultiBandCompressorBandParams {
    /// Crossover frequency for the current band.
    pub crossover_freq: f32,
    /// Compressor parameters for the current band.
    pub compressor_params: CompressorParams,
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

/// Applies a high-pass filter to emphasize higher frequencies.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `cutoff_hz`: Cutoff frequency in Hz.
///
/// # Returns
/// - `Vec<f32>`: High-pass filtered samples.
fn apply_highpass(samples: &[f32], cutoff_hz: f32) -> Vec<f32> {
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
    let alpha = rc / (rc + 1.0 / SRf);
    let mut out = Vec::with_capacity(samples.len());
    let mut prev_in = 0.0;
    let mut prev_out = 0.0;
    for &input in samples {
        let filtered = alpha * (prev_out + input - prev_in);
        out.push(filtered);
        prev_out = filtered;
        prev_in = input;
    }
    out
}

/// Applies a low-pass filter to the samples.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `cutoff_hz`: Cutoff frequency in Hz.
///
/// # Returns
/// - `Vec<f32>`: Low-pass filtered samples.
fn apply_lowpass(samples: &[f32], cutoff_hz: f32) -> Vec<f32> {
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
    let alpha = 1.0 / (rc + 1.0 / SRf);
    let mut out = Vec::with_capacity(samples.len());
    let mut prev_out = 0.0;
    for &input in samples {
        let filtered = alpha * input + (1.0 - alpha) * prev_out;
        out.push(filtered);
        prev_out = filtered;
    }
    out
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
    // Prepend the first sample repeated for the lookahead duration
    out.extend(std::iter::repeat(samples.get(0).cloned().unwrap_or(0.0)).take(lookahead_samples));
    // Append the original samples, excluding the last 'lookahead_samples' to maintain length
    out.extend(&samples[..samples.len().saturating_sub(lookahead_samples)]);
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
    let attack_coeff = time_to_coefficient(attack_time);
    let release_coeff = time_to_coefficient(release_time);

    let processed_samples = if let Some(cutoff_hz) = pre_emphasis {
        apply_highpass(samples, cutoff_hz)
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
    if params.lookahead_time.unwrap_or(0.0) < 0.0 {
        return Err("Lookahead time must be non-negative.".to_string());
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
            None,
            Some(EnvelopeMethod::Peak),
            None,
        )?
    } else {
        envelope_follower(
            &delayed_samples,
            params.attack_time,
            params.release_time,
            None,
            Some(EnvelopeMethod::Peak),
            None,
        )?
    };

    let mut output = Vec::with_capacity(samples.len());
    for (i, &sample) in samples.iter().enumerate() {
        let env_val = envelope[i];
        let compressed = if env_val <= params.threshold {
            sample
        } else {
            if params.knee_width > 0.0 {
                let half_knee = params.knee_width * 0.5;
                let lower_bound = params.threshold - half_knee;
                let upper_bound = params.threshold + half_knee;
                if env_val < lower_bound {
                    sample
                } else if env_val > upper_bound {
                    apply_compression(sample, params.threshold, params.ratio)
                } else {
                    let normalized = (env_val - lower_bound) / params.knee_width;
                    let interp_ratio = params.ratio + (1.0 - params.ratio) * (1.0 - normalized);
                    apply_compression(sample, params.threshold, interp_ratio)
                }
            } else {
                apply_compression(sample, params.threshold, params.ratio)
            }
        };
        output.push(compressed * params.makeup_gain);
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

    let envelope = envelope_follower(
        samples,
        params.attack_time,
        params.release_time,
        None,
        Some(EnvelopeMethod::Peak),
        None,
    )?;

    let mut output = Vec::with_capacity(samples.len());
    for (i, &sample) in samples.iter().enumerate() {
        let env_val = envelope[i];
        let expanded = if env_val < params.threshold {
            apply_expansion(sample, params.threshold, params.ratio)
        } else {
            sample
        };
        output.push(expanded * params.makeup_gain);
    }
    Ok(output)
}

/// Combines compression and expansion to perform companding on the samples.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `params`: Compander parameters.
/// - `sidechain`: Optional sidechain input samples for compression.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Companded samples or an error if parameters are invalid.
pub fn compander(
    samples: &[f32],
    params: CompanderParams,
    sidechain: Option<&[f32]>,
) -> Result<Vec<f32>, String> {
    let compressed = compressor(samples, params.compressor, sidechain)?;
    let expanded = expander(&compressed, params.expander, None)?;
    Ok(expanded)
}

/// Applies dynamic range compression followed by expansion across multiple frequency bands.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `params`: Multi-band compressor parameters.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Multi-band compressed samples or an error if parameters are invalid.
pub fn multi_band_compressor(
    samples: &[f32],
    params: MultiBandCompressorParams,
) -> Result<Vec<f32>, String> {
    if params.crossover_freqs.len() + 1 != params.bands.len() {
        return Err("Number of crossover frequencies must be one less than the number of bands.".to_string());
    }

    // Split the signal into frequency bands
    let bands = split_bands(samples, &params.crossover_freqs)?;

    // Apply compression to each band
    let mut compressed_bands = Vec::with_capacity(bands.len());
    for (band, comp_params) in bands.iter().zip(params.bands.iter()) {
        let compressed = compressor(band, *comp_params, None)?;
        compressed_bands.push(compressed);
    }

    // Sum the bands back together
    let mut output = vec![0.0_f32; samples.len()];
    for band in compressed_bands.iter() {
        for (i, &sample) in band.iter().enumerate() {
            output[i] += sample;
        }
    }

    Ok(output)
}

/// Splits the signal into multiple frequency bands using simple crossover filters.
///
/// Note: For real-world applications, more sophisticated filtering (e.g., biquad filters) is recommended.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `crossover_freqs`: Vector of crossover frequencies in Hz.
///
/// # Returns
/// - `Result<Vec<Vec<f32>>, String>`: Vector of frequency band samples or an error if parameters are invalid.
fn split_bands(samples: &[f32], crossover_freqs: &[f32]) -> Result<Vec<Vec<f32>>, String> {
    let mut bands = Vec::new();
    let mut previous_freq = 0.0;
    for &freq in crossover_freqs.iter() {
        let band = band_split(samples, previous_freq, freq)?;
        bands.push(band);
        previous_freq = freq;
    }
    // Last band up to Nyquist frequency using SRf instead of SR as f32
    let last_band = band_split(samples, previous_freq, SRf / 2.0)?;
    bands.push(last_band);
    Ok(bands)
}

/// Splits the signal between two frequencies using a simple two-pole filter.
///
/// This is a placeholder for more accurate band-splitting filters.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `low_freq`: Lower cutoff frequency in Hz.
/// - `high_freq`: Upper cutoff frequency in Hz.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Band-split samples or an error if frequencies are invalid.
fn band_split(samples: &[f32], low_freq: f32, high_freq: f32) -> Result<Vec<f32>, String> {
    if low_freq >= high_freq {
        return Err("Low frequency must be less than high frequency.".to_string());
    }
    // Apply high-pass filter followed by low-pass filter to create a band-pass filter
    let hp_filtered = apply_highpass(samples, low_freq);
    let lp_filtered = apply_lowpass(&hp_filtered, high_freq);
    Ok(lp_filtered)
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
        let diff = abs_s - threshold;
        let compressed = threshold + diff / ratio;
        sign * compressed
    }
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
        let diff = threshold - abs_s;
        let expanded = threshold - diff * ratio;
        sign * expanded
    }
}

/// Performs transient shaping by enhancing or attenuating transients based on the envelope.
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
    let TransientShaperParams {
        transient_emphasis,
        threshold,
        attack_time,
        release_time,
    } = params;

    if transient_emphasis < 0.0 {
        return Err("Transient emphasis must be non-negative.".to_string());
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
    let mut local_env = 0.0;

    for (&sample, &env_val) in samples.iter().zip(envelope.iter()) {
        let diff = (env_val - local_env).max(0.0);
        local_env = env_val;

        let factor = if env_val > threshold {
            1.0 + transient_emphasis * diff
        } else {
            1.0
        };
        output.push(sample * factor);
    }

    Ok(output)
}

/// Applies soft clipping to the samples with adjustable clipping curves.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `clip_threshold`: Threshold above which clipping starts.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Soft-clipped samples or an error if parameters are invalid.
pub fn soft_clipper(samples: &[f32], clip_threshold: f32) -> Result<Vec<f32>, String> {
    if clip_threshold <= 0.0 {
        return Err("Clip threshold must be positive.".to_string());
    }
    let out = samples
        .iter()
        .map(|&s| {
            let abs_s = s.abs();
            let sign = s.signum();
            if abs_s <= clip_threshold {
                s
            } else if abs_s <= 2.0 * clip_threshold {
                // Smoothly compress between threshold and 2*threshold
                sign * ((3.0 - (2.0 - abs_s / clip_threshold).powi(2)) * clip_threshold / 3.0)
            } else {
                // Hard limit beyond 2*threshold
                sign * clip_threshold
            }
        })
        .collect();
    Ok(out)
}

/// Normalizes the samples to a specified target maximum amplitude.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `target_max`: Target maximum amplitude in linear scale.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Normalized samples or an error if parameters are invalid.
pub fn normalizer(samples: &[f32], target_max: f32) -> Result<Vec<f32>, String> {
    if target_max <= 0.0 {
        return Err("Target maximum must be positive.".to_string());
    }
    let mut max_val: f32 = 0.0;
    for &s in samples.iter() {
        max_val = max_val.max(s.abs());
    }
    if max_val <= 0.0 {
        return Ok(samples.to_vec());
    }
    let gain = target_max / max_val;
    Ok(samples.iter().map(|&s| s * gain).collect())
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

/// Performs transient shaping by enhancing or attenuating transients based on the envelope.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `params`: Transient shaper parameters.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Transient-shaped samples or an error if parameters are invalid.
pub fn apply_transient_shaping(
    samples: &[f32],
    params: TransientShaperParams,
) -> Result<Vec<f32>, String> {
    transient_shaper(samples, params)
}

/// Applies a combination of soft clipping and normalization to achieve gentle distortion and consistent levels.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `clip_threshold`: Threshold above which clipping starts.
/// - `target_max`: Target maximum amplitude after normalization.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Soft-clipped and normalized samples or an error if parameters are invalid.
pub fn soft_clipper_with_normalization(
    samples: &[f32],
    clip_threshold: f32,
    target_max: f32,
) -> Result<Vec<f32>, String> {
    let clipped = soft_clipper(samples, clip_threshold)?;
    normalizer(&clipped, target_max)
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
    compander(samples, params, sidechain)
}

/// Applies dynamic range compression with sidechain support.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `params`: Compressor parameters.
/// - `sidechain`: Sidechain input samples to control compression.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Compressed samples or an error if parameters are invalid.
pub fn compressor_with_sidechain(
    samples: &[f32],
    params: CompressorParams,
    sidechain: &[f32],
) -> Result<Vec<f32>, String> {
    compressor(samples, params, Some(sidechain))
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

/// Applies multi-band compression with detailed band parameters.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `band_params`: Slice of multi-band compressor band parameters.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Multi-band compressed samples or an error if parameters are invalid.
pub fn multi_band_compressor_detailed(
    samples: &[f32],
    band_params: &[MultiBandCompressorBandParams],
) -> Result<Vec<f32>, String> {
    if band_params.is_empty() {
        return Err("At least one band parameter must be provided.".to_string());
    }

    let mut bands = Vec::new();
    let mut previous_freq = 0.0;
    for params in band_params.iter() {
        let band = band_split(samples, previous_freq, params.crossover_freq)?;
        let compressed = compressor(&band, params.compressor_params, None)?;
        bands.push(compressed);
        previous_freq = params.crossover_freq;
    }
    // Last band up to Nyquist frequency
    let last_band = band_split(samples, previous_freq, SR as f32 / 2.0)?;
    let last_compressed = compressor(&last_band, band_params[0].compressor_params, None)?;
    bands.push(last_compressed);

    // Sum the bands back together
    let mut output = vec![0.0_f32; samples.len()];
    for band in bands.iter() {
        for (i, &sample) in band.iter().enumerate() {
            output[i] += sample;
        }
    }

    Ok(output)
}

/// Performs comprehensive dynamic range processing, including compression, expansion, and gating.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `compressor_params`: Compressor parameters.
/// - `expander_params`: Expander parameters.
/// - `gate_params`: Optional gating parameters as a tuple `(threshold, attack_time, release_time)`.
/// - `sidechain`: Optional sidechain input samples to control compression and expansion.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Processed samples or an error if parameters are invalid.
pub fn dynamics_processor(
    samples: &[f32],
    compressor_params: CompressorParams,
    expander_params: ExpanderParams,
    gate_params: Option<(f32, f32, f32)>, // (threshold, attack_time, release_time)
    sidechain: Option<&[f32]>,
) -> Result<Vec<f32>, String> {
    let compressed = compressor(samples, compressor_params, sidechain)?;
    let expanded = expander(&compressed, expander_params, sidechain)?;
    if let Some((threshold, attack, release)) = gate_params {
        gate(&expanded, threshold, attack, release)
    } else {
        Ok(expanded)
    }
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

/// Applies multi-band compression with specified parameters.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `params`: Multi-band compressor parameters.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: Multi-band compressed samples or an error if parameters are invalid.
pub fn apply_multi_band_compression(
    samples: &[f32],
    params: MultiBandCompressorParams,
) -> Result<Vec<f32>, String> {
    multi_band_compressor(samples, params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_follower_peak() {
        let input = vec![0.0, 0.1, 0.2, 0.4, 0.2, 0.1, 0.0];
        let result = envelope_follower(&input, 0.01, 0.01, None, Some(EnvelopeMethod::Peak), None).unwrap();
        // We expect a rising/falling envelope near the absolute values
        assert_eq!(result.len(), input.len());
        // Just check it doesn't overshoot drastically
        for (r, i) in result.iter().zip(input.iter()) {
            assert!(*r >= 0.0 && *r <= i.abs().max(0.4) + 0.1);
        }
    }

    #[test]
    fn test_compressor_hard_knee() {
        let input = vec![0.0, 0.5, 1.0, 1.5];
        let params = CompressorParams {
            threshold: 1.0,
            ratio: 2.0,
            knee_width: 0.0,
            makeup_gain: 1.0,
            attack_time: 0.0, // Set to 0.0 for instantaneous envelope tracking
            release_time: 0.0, // Set to 0.0 for instantaneous envelope tracking
            lookahead_time: None,
        };
        let output = compressor(&input, params, None).unwrap();
        // Values above threshold should be compressed
        // e.g. sample=1.5 above threshold => 1.0 + (0.5/2.0)=1.25
        assert_eq!(output[0], 0.0);
        assert_eq!(output[1], 0.5);
        assert_eq!(output[2], 1.0);
        assert!((output[3] - 1.25).abs() < 1e-6);
    }
    
    #[test]
    fn test_soft_clipper() {
        let input = vec![0.0, 0.2, 0.8, 1.2, -1.4, 1.0, 2.5];
        let clip_thresh = 1.0;
        let out = soft_clipper(&input, clip_thresh).unwrap();
        assert_eq!(out.len(), input.len());
        // Values <= threshold should be unchanged
        assert!((out[0] - 0.0).abs() < 1e-6);
        assert!((out[1] - 0.2).abs() < 1e-6);
        assert!((out[2] - 0.8).abs() < 1e-6);
        // Values > threshold should be softly clipped
        assert!((out[3] - 0.7866667).abs() < 1e-5); // 1.2 clipped to ~0.7867
        assert!((out[4] + 0.88).abs() < 1e-5);      // -1.4 clipped to ~-0.88
        assert!((out[5] - 1.0).abs() < 1e-6);
        // Values beyond 2*threshold should be hard clipped to threshold
        assert!((out[6] - 1.0).abs() < 1e-6);      // 2.5 clipped to 1.0
    }

    #[test]
    fn test_normalizer() {
        let input = vec![0.0, 0.2, -0.5, 0.8];
        let out = normalizer(&input, 1.0).unwrap();
        let max_val = out.iter().fold(0.0_f32, |acc, &x| acc.max(x.abs()));
        assert!((max_val - 1.0).abs() < 1e-6);
    }

        #[test]
    fn test_gate() {
        let input = vec![0.01, 0.2, 0.001, -0.009];
        let out = gate(&input, 0.01, 0.0, 0.0).unwrap(); // Set attack and release times to 0.0
        // Values <= 0.01 in absolute value => 0
        assert_eq!(out.len(), input.len());
        assert_eq!(out[0], 0.0); // 0.01 <= 0.01 => 0.0
        assert_eq!(out[1], 0.2); // 0.2 > 0.01 => 0.2
        assert_eq!(out[2], 0.0); // 0.001 <= 0.01 => 0.0
        assert_eq!(out[3], 0.0); // -0.009 <= 0.01 => 0.0
    }

    #[test]
    fn test_calculate_threshold_peak() {
        let input = vec![0.1, 0.2, 0.9, -1.2];
        let th = calculate_threshold(&input, 0.5, false).unwrap();
        // Peak is 1.2 => scaled by 0.5 => 0.6
        assert!((th - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_deinterleave_interleave() {
        let input_stereo = vec![0.1, 0.2, 0.3, 0.4, -0.5, -0.6];
        let (left, right) = deinterleave(&input_stereo);
        assert_eq!(left.len(), 3);
        assert_eq!(right.len(), 3);
        let reinterleaved = interleave(&left, &right).unwrap();
        assert_eq!(reinterleaved, input_stereo);
    }
}
