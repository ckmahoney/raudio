/// These functions look good but are not in use anywhere.

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
