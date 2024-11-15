use crate::synth::SampleBuffer;

fn detect_transient(samples: &SampleBuffer, window_length: usize, threshold: f32) -> Vec<usize> {
  let mut transients = Vec::new();
  let mut previous_rms = 0.0;

  for window_start in (0..samples.len()).step_by(window_length) {
    let window_end = usize::min(window_start + window_length, samples.len());
    let window = &samples[window_start..window_end];

    // Calculate the RMS of the current window
    let current_rms: f32 =
      (window.iter().map(|&sample| sample.powi(2) as f32).sum::<f32>() / window.len() as f32).sqrt();

    // Look for sudden increase in RMS volume
    if (current_rms - previous_rms) > threshold {
      transients.push(window_start);
    }

    // Update previous_rms for the next iteration
    previous_rms = current_rms;
  }

  transients
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_transient_in_center() {
    // Create a buffer with transients in the center
    let mut samples = vec![0.0; 100]; // buffer of 100 samples
    for i in 45..55 {
      samples[i] = 1.0; // set the middle 10 samples to 1.0
    }

    // Expect the transient to be detected at the start of the middle segment
    let window_length = 10;
    let threshold = 0.5;
    let detected_transients = detect_transient(&samples, window_length, threshold);

    assert_eq!(
      detected_transients,
      vec![40],
      "Transient should be detected at sample index 40"
    );
  }

  #[test]
  fn test_transient_at_two_thirds() {
    // Create a buffer with a transient about two-thirds of the way through
    let mut samples = vec![0.0; 150]; // buffer of 150 samples
    for i in 90..100 {
      samples[i] = 1.0; // set ten samples at two-thirds to 1.0
    }

    // Expect the transient to be detected at the start of this segment
    let window_length = 10;
    let threshold = 0.5;
    let detected_transients = detect_transient(&samples, window_length, threshold);

    assert_eq!(
      detected_transients,
      vec![90],
      "Transient should be detected at sample index 90"
    );
  }
}
