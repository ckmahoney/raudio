use crate::Visibility;

use super::*;

/// Enumeration for different envelope detection methods.
///
/// This enum allows selecting the method used to detect the envelope of an audio signal.
/// - `Peak`: Detects the peak amplitude.
/// - `Rms`: Detects the root mean square (RMS) amplitude over a specified window.
/// - `Hybrid`: Combines peak and RMS detection for smoother envelope tracking.
#[derive(Debug, Clone, Copy)]
pub enum EnvelopeMethod {
  Peak,
  Rms(f32),    // Moving average window size in seconds
  Hybrid(f32), // Same for Hybrid (if you want to include RMS-like smoothing)
}

/// Converts a linear amplitude value to decibels (dB).
///
/// This function ensures that amplitude values are safely converted to the dB scale,
/// clamping to `MIN_DB` to prevent infinite values when the amplitude is zero.
///
/// **Implementation Details:**
/// - Amplitude values less than or equal to zero are clamped to `MIN_DB`.
///
/// # Parameters
/// - `amp`: Amplitude value in linear scale.
///
/// # Returns
/// - `f32`: Corresponding dB value.
///   - Returns `MIN_DB` (-96.0 dB) for amplitudes <= 0 to avoid infinite values.
pub fn amp_to_db(amp: f32) -> f32 {
  let amp = amp.abs();
  const MIN_DB: f32 = -96.0;
  if amp == 0.0 {
    MIN_DB
  } else {
    20.0 * amp.log10()
  }
}

/// Converts a decibel value to linear amplitude with clamping.
///
/// This function ensures that dB values are safely converted back to the linear scale,
/// clamping to `MIN_DB` and `MAX_DB` to prevent numerical issues.
///
/// **Implementation Details:**
/// - Clamps the input `db` to the range `[MIN_DB, MAX_DB]` before conversion.
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

/// Computes RMS value for a signal over a sliding window.
///
/// This function calculates the root mean square (RMS) of the input audio samples
/// using a sliding window approach. It ensures that each sample's RMS value is
/// computed over the specified window size, providing a smoothed envelope of the signal.
///
/// **Implementation Details:**
/// - Utilizes a sliding window to accumulate squared samples.
/// - Avoids division by zero by ensuring window size is at least 1.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `window_size`: Size of the RMS window in samples.
///
/// # Returns
/// - `Vec<f32>`: RMS values for each input sample.
pub fn compute_rms(samples: &[f32], window_size: usize) -> Vec<f32> {
  if samples.is_empty() || window_size == 0 {
    eprintln!("Warning, skipping rms computation because signal or windw_size is empty");
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
    let rms = if current_window_size > 0 && squared_sum > 0.0 {
      (squared_sum / current_window_size as f32).sqrt()
    } else {
      0.0
    };
    rms_output.push(rms);
  }

  rms_output
}

/// For a signal in range [-1, 1],
/// provides the RMS of the entire signal.
pub fn count_energy(samples: &[f32]) -> f32 {
  if samples.is_empty() {
    return 0.0;
  }
  let sum_squares: f32 = samples.iter().map(|x| x.powi(2i32)).collect::<Vec<f32>>().iter().sum();
  (sum_squares / samples.len() as f32).sqrt()
}

/// Given a signal and the ideal RMS for it
/// Creates a new version of the stem with gain staging applied.
///
/// ## Arguments
///
/// target_rms: A value in [0, infinity] describing the target RMS value for the signal.
/// samples: The signal to rescale
pub fn rescale_amplitude(target_rms: f32, samples: &Vec<f32>) -> Vec<f32> {
  let current_rms = count_energy(samples);
  let scale_factor = target_rms / current_rms;

  samples.iter().map(|x| *x * scale_factor).collect()
}

/// Detects the envelope of the signal using the specified method and parameters.
///
/// This function analyzes the input audio samples to detect their amplitude envelope using
/// the chosen detection method (Peak, RMS, or Hybrid). It supports optional pre-emphasis filtering
/// and wet/dry mixing.
///
/// **Implementation Choices:**
/// - Supports sidechain processing through envelope shaping.
/// - Allows optional hold time to maintain envelope levels temporarily after signal drops.
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

/// Measures the integrated loudness of the input samples in LUFS.
///
/// This function calculates the Loudness Units relative to Full Scale (LUFS) of the audio signal,
/// providing a standardized measure of perceived loudness.
///
/// **Implementation Details:**
/// - Applies the K-weighting filter to the samples.
/// - Squares the K-weighted samples to obtain power.
/// - Integrates the power over a 400 ms window using an exponential integrator.
/// - Converts the integrated power to LUFS.
///
/// # Parameters
/// - `samples`: Input audio samples.
/// - `window_time`: Window size in seconds for loudness integration (typically 0.4 seconds).
///
/// # Returns
/// - `Result<f32, String>`: Calculated LUFS value or an error if processing fails.
pub fn compute_lufs(samples: &[f32], window_time: f32) -> Result<f32, String> {
  // Apply K-weighting to the samples
  let k_weighted_samples = apply_k_weighting(samples)?;

  // Square-law integration (power)
  let squared_samples: Vec<f32> = k_weighted_samples.iter().map(|&x| x * x).collect();

  // Time integration using an exponential moving average with a 400 ms window
  let integration_time = 0.4; // 400 ms
  let alpha = time_to_coefficient(integration_time);
  let mut integrated_power = 0.0;
  let mut sum = 0.0;
  let mut count = 0usize;

  for &power in squared_samples.iter() {
    integrated_power = (1.0 - alpha) * integrated_power + alpha * power;
    sum += integrated_power;
    count += 1;
  }

  if count == 0 {
    return Err("No samples to process for LUFS.".to_string());
  }

  let mean_power = sum / count as f32;

  // Convert to LUFS
  let lufs = 10.0 * mean_power.log10();

  Ok(lufs)
}

/// Applies the K-weighting filter to the input samples.
///
/// The K-weighting filter is composed of a high-pass filter at 60 Hz and a high-shelf filter at 2 kHz with +4 dB gain.
/// This filter approximates the human ear's sensitivity to different frequency ranges.
///
/// **Implementation Details:**
/// - Cascades the high-pass and high-shelf filters.
/// - Ensures that both filters are applied in sequence.
///
/// # Parameters
/// - `samples`: Input audio samples.
///
/// # Returns
/// - `Result<Vec<f32>, String>`: K-weighted audio samples or an error message if filtering fails.
fn apply_k_weighting(samples: &[f32]) -> Result<Vec<f32>, String> {
  // Apply high-pass filter at 60 Hz
  let hp_filtered = apply_highpass(samples, 60.0)?;

  // Apply high-shelf filter at 2 kHz with +4 dB gain
  let hs_filtered = apply_highshelf(&hp_filtered, 2000.0, 4.0)?;

  Ok(hs_filtered)
}

#[cfg(test)]
mod test_unit_keith_factor {
  use super::*;
  use rand::thread_rng;

  #[test]
  fn test_punchy_signal() {
    let mut rng = thread_rng();

    let thresh_fall_db = 18f32;

    let dur_onset_seconds = 0.1f32;
    let dur_punch_seconds = 0.1f32;

    // construct a decay duration, tail duration, and window size such that we know
    // the filter will trigger and return an filtered signal
    let dur_decay_seconds = 0.05f32;
    let dur_tail_seconds_short = 0.1f32;
    let dur_tail_seconds_long = 1.1f32;

    let window_size_sec = dur_decay_seconds / 2f32;

    // use 'featured signal' values above the fall threshold
    let onset_db_range = (-45.0, -12.0);
    let punch_db_range = (-12.0, -6.0);
    let decay_db_range = (-24.0, -12.0);
    // use a db range that is guaranteed to be under the fall threshold in the tail
    let tail_db_range = (
      decay_db_range.0 - thresh_fall_db - 1.0,
      decay_db_range.1 - thresh_fall_db - 1.0,
    );

    let total_signal_length_short = dur_onset_seconds + dur_punch_seconds + dur_decay_seconds + dur_tail_seconds_short;
    let total_signal_length_long = dur_onset_seconds + dur_punch_seconds + dur_decay_seconds + dur_tail_seconds_long;
    let n_samples_short = time::samples_of_seconds(1f32, total_signal_length_short);
    let n_samples_long = time::samples_of_seconds(1f32, total_signal_length_long);
    let mut signal_short: Vec<f32> = Vec::with_capacity(n_samples_short);
    let mut signal_long: Vec<f32> = Vec::with_capacity(n_samples_long);

    for i in 0..n_samples_short {
      let curr_pos = time::samples_to_seconds(i);
      let (min_db, max_db) = if curr_pos > dur_onset_seconds + dur_punch_seconds + dur_decay_seconds {
        // in the tail
        tail_db_range
      } else if curr_pos > dur_onset_seconds + dur_punch_seconds {
        // in the decay
        decay_db_range
      } else if curr_pos > dur_onset_seconds {
        // in the punch
        punch_db_range
      } else {
        // in the onset
        onset_db_range
      };
      let sign = if rng.gen::<f32>() < 0.5f32 { 1f32 } else { -1f32 };
      let sample_val = sign * in_range(&mut rng, db_to_amp(min_db), db_to_amp(max_db));
      signal_short.push(sample_val);
      signal_long.push(sample_val);
    }

    for i in 0..(n_samples_long - n_samples_short) {
      let (min_db, max_db) = tail_db_range;
      let sign = if rng.gen::<f32>() < 0.5f32 { 1f32 } else { -1f32 };
      let sample_val = sign * in_range(&mut rng, db_to_amp(min_db), db_to_amp(max_db));

      signal_long.push(sample_val)
    }

    let label_short = format!("unit-keith-factor_short.wav");
    let label_long = format!("unit-keith-factor_long.wav");
    crate::render::engrave::samples(SR, &signal_short, &dev_audio_asset(&label_short));
    crate::render::engrave::samples(SR, &signal_long, &dev_audio_asset(&label_long));
    let keith_factor_short = keith_factor(&signal_short, thresh_fall_db, window_size_sec);
    let keith_factor_long = keith_factor(&signal_long, thresh_fall_db, window_size_sec);
    assert!(
      keith_factor_short > keith_factor_long,
      "Shorter punchy signals must have a lower Keith factor than longer smoother signals"
    )
  }
}

#[cfg(test)]
mod test_unit_rescale_signal {
  use super::*;
  use rand::thread_rng;

  #[test]
  fn test_rms_rescaling() {
    let mut rng = thread_rng();

    let thresh_fall_db = 18f32;

    let dur_onset_seconds = 0.1f32;
    let dur_punch_seconds = 0.1f32;

    // construct a decay duration, tail duration, and window size such that we know
    // the filter will trigger and return an filtered signal
    let dur_decay_seconds = 0.05f32;
    let dur_tail_seconds_short = 0.1f32;
    let dur_tail_seconds_long = 1.1f32;

    let window_size_sec = dur_decay_seconds / 2f32;

    // use 'featured signal' values above the fall threshold
    let onset_db_range = (-45.0, -12.0);
    let punch_db_range = (-12.0, -6.0);
    let decay_db_range = (-24.0, -12.0);
    // use a db range that is guaranteed to be under the fall threshold in the tail
    let tail_db_range = (
      decay_db_range.0 - thresh_fall_db - 1.0,
      decay_db_range.1 - thresh_fall_db - 1.0,
    );

    let total_signal_length_short = dur_onset_seconds + dur_punch_seconds + dur_decay_seconds + dur_tail_seconds_short;
    let total_signal_length_long = dur_onset_seconds + dur_punch_seconds + dur_decay_seconds + dur_tail_seconds_long;
    let n_samples_short = time::samples_of_seconds(1f32, total_signal_length_short);
    let n_samples_long = time::samples_of_seconds(1f32, total_signal_length_long);
    let mut signal_short: Vec<f32> = Vec::with_capacity(n_samples_short);
    let mut signal_long: Vec<f32> = Vec::with_capacity(n_samples_long);

    for i in 0..n_samples_short {
      let curr_pos = time::samples_to_seconds(i);
      let (min_db, max_db) = if curr_pos > dur_onset_seconds + dur_punch_seconds + dur_decay_seconds {
        // in the tail
        tail_db_range
      } else if curr_pos > dur_onset_seconds + dur_punch_seconds {
        // in the decay
        decay_db_range
      } else if curr_pos > dur_onset_seconds {
        // in the punch
        punch_db_range
      } else {
        // in the onset
        onset_db_range
      };
      let sign = if rng.gen::<f32>() < 0.5f32 { 1f32 } else { -1f32 };
      let sample_val = sign * in_range(&mut rng, db_to_amp(min_db), db_to_amp(max_db));
      signal_short.push(sample_val);
      signal_long.push(sample_val);
    }

    for i in 0..(n_samples_long - n_samples_short) {
      let (min_db, max_db) = tail_db_range;
      let sign = if rng.gen::<f32>() < 0.5f32 { 1f32 } else { -1f32 };
      let sample_val = sign * in_range(&mut rng, db_to_amp(min_db), db_to_amp(max_db));

      signal_long.push(sample_val)
    }

    let label_short = format!("unit-rescale_short.wav");
    let label_long = format!("unit-rescale_long.wav");
    let label_short_rescaled = format!("unit-rescale_short_rescaled.wav");
    let label_short_rescaled_softer = format!("unit-rescale_short_rescaled_softer.wav");
    let label_long_rescaled = format!("unit-rescale_long_rescaled.wav");
    crate::render::engrave::samples(SR, &signal_short, &dev_audio_asset(&label_short));

    let curr_energy_short = count_energy(&signal_short);
    let curr_energy_long = count_energy(&signal_long);

    let test_scale_factor_louder = 1.25;
    let expected_short_rms = curr_energy_short * test_scale_factor_louder;

    let short_rescaled_louder = rescale_amplitude(expected_short_rms, &signal_short);
    crate::render::engrave::samples(SR, &short_rescaled_louder, &dev_audio_asset(&label_short_rescaled));

    assert!(
      count_energy(&signal_short) < count_energy(&short_rescaled_louder),
      "Rescaling to be louder must make the signal louder."
    );

    let test_scale_factor_louder = 0.5;
    let expected_short_rms_softer = curr_energy_short * test_scale_factor_louder;

    let short_rescaled_softer = rescale_amplitude(expected_short_rms_softer, &signal_short);
    crate::render::engrave::samples(
      SR,
      &short_rescaled_softer,
      &dev_audio_asset(&label_short_rescaled_softer),
    );
    let softer_rms = count_energy(&short_rescaled_softer);

    assert!(
      count_energy(&signal_short) > count_energy(&short_rescaled_softer),
      "Rescaling to be quieter must make the signal quieter."
    );
  }
}

/// Computes the Keith Factor of an audio signal, which measures the relative spread of energy.
/// A value of 1 indicates a very punchy signal, while a value of 0 indicates a constant signal.
///
/// # Parameters
/// - `signal`: Slice of audio samples (f32).
/// - `thresh_fallen`: Threshold in decibels (dB) below the peak to trigger the kill index.
/// - `fallen_window_sec`: Duration in seconds that the signal must remain below the threshold to confirm the kill index.
///
/// # Returns
/// - A `f32` value representing the ratio of energy up to the kill index over the total energy.
///   Always returns a value between 0 and 1. Returns `1.0` if no kill index is found.
///
/// # Behavior
/// 1. Identifies the peak value of the signal in dB.
/// 2. Determines the kill threshold based on `thresh_fallen`.
/// 3. Searches for the first index after the peak where the signal falls below the threshold
///    and remains below for `fallen_window_sec`.
/// 4. Calculates the RMS energy up to the kill index and the total RMS energy.
/// 5. Returns the ratio of these energies.
///
/// # Example
/// ```
/// let signal = vec![0.0, 0.1, 0.3, 0.5, 0.2, 0.1, 0.0];
/// let keith = keith_factor(&signal, 18.0, 0.5, 0.1);
/// println!("Keith Factor: {}", keith);
/// ```
fn keith_factor(
  signal: &[f32],
  thresh_fallen: f32,     // e.g., 18 dB
  fallen_window_sec: f32, // Hold time in seconds
) -> f32 {
  // Determine sample count for fallen_window_sec
  let fallen_window_samples = crate::time::samples_of_seconds(1f32, fallen_window_sec);

  // Handle empty input signal edge case
  if signal.is_empty() {
    return 1.0; // Constant signal-like behavior
  }

  // Precompute dB values for all samples
  let db_signal: Vec<f32> = signal.iter().map(|&val| amp_to_db(val.abs())).collect();

  // Find peak amplitude and corresponding dB
  let (peak_index, peak_db) =
    db_signal.iter().enumerate().fold((0, f32::NEG_INFINITY), |(max_idx, max_db), (i, &db_val)| {
      if db_val > max_db {
        (i, db_val)
      } else {
        (max_idx, max_db)
      }
    });

  // Threshold for kill trigger in dB
  let kill_threshold_db = peak_db - thresh_fallen;

  // Search for kill_index
  let mut kill_index = None;
  for i in (peak_index + 1)..signal.len() {
    if db_signal[i] < kill_threshold_db {
      // Check if the signal remains below the threshold for the required window
      let end_index = i + fallen_window_samples;
      let below_threshold = if end_index > signal.len() {
        db_signal[i..].iter().all(|&db| db < kill_threshold_db)
      } else {
        db_signal[i..end_index].iter().all(|&db| db < kill_threshold_db)
      };

      if below_threshold {
        kill_index = Some(i);
        break;
      }
    }
  }

  // If no kill_index is found, assume constant signal
  let kill_index = kill_index.unwrap_or(signal.len());

  // Compute energy ratios
  let rms_featured = count_energy(&signal[..kill_index]);
  let rms_lifetime = count_energy(signal);

  // Avoid division by zero
  if rms_lifetime == 0.0 {
    return 1.0;
  }

  println!("rms_featured {} rms_lifetime {}", rms_featured, rms_lifetime);

  rms_lifetime / rms_featured
}
