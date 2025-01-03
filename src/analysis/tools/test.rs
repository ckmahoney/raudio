use super::*;

use crate::analysis::sampler::read_audio_file;
use crate::render::engrave::write_audio;

pub fn dev_audio_asset(label: &str) -> String {
  format!("dev-audio/{}", label)
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

#[cfg(test)]
mod test_unit_transient_shaper {
  use super::*;
  use std::f32::EPSILON;

  // Helper function to assert smoothness in transitions
  fn assert_smooth_transition(result: &[f32], _threshold: f32, start: usize, end: usize) {
    if end < 2 {
      // Not enough samples to check transitions
      return;
    }
    let mut inflection_points = vec![start];
    for i in (start + 2)..end {
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
  fn test_transient_shaper_smooth_transition() {
    let samples = vec![0.2, 0.4, 0.6, 0.5, 0.7, 0.3];
    let params = TransientShaperParams {
      transient_emphasis: 1.5,
      threshold: 0.3,
      attack_threshold: 0.5, // Ensure attack_threshold >= threshold
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
  fn test_transient_shaper_attack_phase() {
    let samples = vec![0.5, 1.5, 0.8, 0.3];
    let params = TransientShaperParams {
      transient_emphasis: 1.5,
      threshold: 0.6,
      attack_threshold: 0.7, // Ensure attack_threshold >= threshold
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
      attack_threshold: 0.35, // Ensure attack_threshold >= threshold
      attack_time: 0.05,
      release_time: 0.1,
      attack_factor: 1.0,
      sustain_factor: 0.5, // Set to attenuate sustain phase
      ..Default::default()
    };
    let result = transient_shaper(&samples, params).unwrap();
    let tolerance = 1e-4;
    for (i, &sample) in result.iter().enumerate() {
      if samples[i].abs() <= params.threshold {
        let expected = samples[i].abs() * params.sustain_factor;
        assert!(
          (sample.abs() - expected).abs() < tolerance,
          "Sustain phase should reduce level for sample {}: expected {}, got {}",
          i,
          expected,
          sample.abs()
        );
      }
    }
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
mod test_unit_gate {
  use super::*;

  fn assert_approx_eq(a: f32, b: f32, tol: f32, msg: &str) {
    assert!((a - b).abs() <= tol, "{}: got {}, expected ~{}, tol={}", msg, a, b, tol);
  }

  #[test]
  fn test_empty_signal() {
    let samples: Vec<f32> = vec![];
    let params = GateParams {
      threshold: -10f32,
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
      makeup_gain: 1f32
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
      makeup_gain: 1f32
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

    let out = expander(&samples, params, None).expect("Expander failed");
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

    let out = expander(&samples, params, None).expect("Expander failed");

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

    let result = expander(&samples, params, None).unwrap();
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
    assert_eq!(db, amp_to_db(amp.abs()), "Negative amplitude should return its symmetric value.");
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

#[cfg(test)]
mod test_edm_compression_patterns {
  use super::*;
  use crate::analysis::sampler::read_audio_file;
  use crate::render::engrave::write_audio;
  use std::path::Path;

  /// Helper: calculate overall RMS using the existing compute_rms function.
  fn overall_rms(samples: &[f32]) -> f32 {
    // Set window_size to the total number of samples to get a single RMS value
    let rms_values = compute_rms(samples, samples.len());
    // Assuming compute_rms returns at least one value
    rms_values.first().copied().unwrap_or(0.0)
  }

  /// Helper: compute dynamic range in dB (20*log10(max/min))
  fn calc_dynamic_range_db(samples: &[f32]) -> f32 {
    let max_val = samples.iter().copied().fold(f32::MIN, f32::max).abs();
    let min_val = samples.iter().copied().fold(f32::MAX, f32::min).abs();
    if min_val == 0.0 {
      amp_to_db(max_val)
    } else {
      amp_to_db(max_val / min_val)
    }
  }

  /// Loads the dev-audio/lead.wav file as single-channel or merges if stereo.
  fn load_lead_mono() -> (Vec<f32>, u32) {
    let (channels, sample_rate) = read_audio_file("dev-audio/lead.wav").expect("Missing dev-audio/lead.wav for tests");
    // If stereo or multi-channel, downmix here:
    if channels.len() > 1 {
      let mut mono = Vec::with_capacity(channels[0].len());
      for frame_idx in 0..channels[0].len() {
        let mut sum = 0.0;
        for channel in &channels {
          sum += channel[frame_idx];
        }
        mono.push(sum / channels.len() as f32);
      }
      (mono, sample_rate)
    } else {
      (channels[0].clone(), sample_rate)
    }
  }

  /// Generates the output file path based on the test name and effect type.
  fn generate_output_path(effect: &str, intensity: &str) -> String {
    dev_audio_asset(&format!("lead_{}_{}.wav", effect, intensity))
  }

  #[test]
  fn test_light_compression() {
    let (samples, sample_rate) = load_lead_mono();
    let rms_before = overall_rms(&samples);
    let dr_before = calc_dynamic_range_db(&samples);

    let params = CompressorParams {
      threshold: -20.0, // Fairly gentle
      ratio: 2.0,       // Light ratio
      attack_time: 0.01,
      release_time: 0.1,
      makeup_gain: 1.0, // No makeup gain
      ..Default::default()
    };
    let processed = compressor(&samples, params, None).unwrap();

    let rms_after = overall_rms(&processed);
    let dr_after = calc_dynamic_range_db(&processed);

    assert!(
      rms_after <= rms_before * 1.05,
      "Light compression shouldn't drastically raise RMS. Before={:.3}, After={:.3}",
      rms_before,
      rms_after
    );
    assert!(
      dr_after < dr_before,
      "Dynamic range should decrease under compression. Before={:.3} dB, After={:.3} dB",
      dr_before,
      dr_after
    );

    // Write processed audio to file after successful assertions
    let output_path = generate_output_path("compression_light_test", "test");
    write_audio(sample_rate as usize, vec![processed.clone()], &output_path);

    // Verify the output file was created
    assert!(
      Path::new(&output_path).exists(),
      "Processed audio file was not created at {}",
      output_path
    );
  }

  #[test]
  fn test_moderate_compression() {
    let (samples, sample_rate) = load_lead_mono();
    let (rms_before, dr_before) = (overall_rms(&samples), calc_dynamic_range_db(&samples));

    let params = CompressorParams {
      threshold: -15.0,
      ratio: 4.0, // Moderate ratio
      attack_time: 0.01,
      release_time: 0.2,
      makeup_gain: 1.0, // No makeup gain
      ..Default::default()
    };
    let processed = compressor(&samples, params, None).unwrap();

    let (rms_after, dr_after) = (overall_rms(&processed), calc_dynamic_range_db(&processed));
    assert!(
      dr_after < dr_before,
      "Moderate compression should decrease DR. Before={:.3} dB, After={:.3} dB",
      dr_before,
      dr_after
    );
    assert!(
      rms_after <= rms_before,
      "RMS should attenuate or remain the same under compression without makeup gain."
    );

    // Write processed audio to file after successful assertions
    let output_path = generate_output_path("compression_moderate_test", "test");
    write_audio(sample_rate as usize, vec![processed.clone()], &output_path);

    // Verify the output file was created
    assert!(
      Path::new(&output_path).exists(),
      "Processed audio file was not created at {}",
      output_path
    );
  }

  #[test]
  fn test_heavy_compression() {
    let (samples, sample_rate) = load_lead_mono();
    let (rms_before, dr_before) = (overall_rms(&samples), calc_dynamic_range_db(&samples));

    let params = CompressorParams {
      threshold: -25.0,
      ratio: 10.0, // Heavy ratio
      attack_time: 0.005,
      release_time: 0.2,
      makeup_gain: 1.0, // No makeup gain
      ..Default::default()
    };
    let processed = compressor(&samples, params, None).unwrap();

    let (rms_after, dr_after) = (overall_rms(&processed), calc_dynamic_range_db(&processed));
    assert!(
      dr_after < dr_before * 0.7,
      "Heavy compression should greatly decrease DR. Before={:.3} dB, After={:.3} dB",
      dr_before,
      dr_after
    );
    assert!(
      rms_after <= rms_before,
      "Heavy compression typically lowers (or flattens) RMS unless makeup_gain is large."
    );

    // Write processed audio to file after successful assertions
    let output_path = generate_output_path("compression_heavy_test", "test");
    write_audio(sample_rate as usize, vec![processed.clone()], &output_path);

    // Verify the output file was created
    assert!(
      Path::new(&output_path).exists(),
      "Processed audio file was not created at {}",
      output_path
    );
  }

  #[test]
  fn test_light_expansion() {
    let (samples, sample_rate) = load_lead_mono();
    let (rms_before, dr_before) = (overall_rms(&samples), calc_dynamic_range_db(&samples));

    let params = ExpanderParams {
      threshold: -10.0,
      ratio: 1.5,
      attack_time: 0.01,
      release_time: 0.1,
      makeup_gain: 1.0, // No makeup gain
      ..Default::default()
    };
    let processed = expander(&samples, params, None).unwrap();

    let (rms_after, dr_after) = (overall_rms(&processed), calc_dynamic_range_db(&processed));
    // Light expansion slightly raises dynamic range
    assert!(
      dr_after > dr_before,
      "Light expansion should increase DR. Before={:.3} dB, After={:.3} dB",
      dr_before,
      dr_after
    );

    // Write processed audio to file after successful assertions
    let output_path = generate_output_path("expansion_light_test", "test");
    write_audio(sample_rate as usize, vec![processed.clone()], &output_path);

    // Verify the output file was created
    assert!(
      Path::new(&output_path).exists(),
      "Processed audio file was not created at {}",
      output_path
    );
  }

  #[test]
  fn test_moderate_expansion() {
    let (samples, sample_rate) = load_lead_mono();
    let (rms_before, dr_before) = (overall_rms(&samples), calc_dynamic_range_db(&samples));

    let params = ExpanderParams {
      threshold: -10.0,
      ratio: 4.0, // Moderate expansion ratio
      attack_time: 0.01,
      release_time: 0.2,
      makeup_gain: 1.0, // No makeup gain
      ..Default::default()
    };
    let processed = expander(&samples, params, None).unwrap();

    let (rms_after, dr_after) = (overall_rms(&processed), calc_dynamic_range_db(&processed));
    assert!(
      dr_after > dr_before,
      "Moderate expansion should increase DR. Before={:.3} dB, After={:.3} dB",
      dr_before,
      dr_after
    );
    assert!(
      rms_after <= rms_before * 1.2,
      "Expansion shouldn't excessively boost RMS."
    );

    // Write processed audio to file after successful assertions
    let output_path = generate_output_path("expansion_moderate_test", "test");
    write_audio(sample_rate as usize, vec![processed.clone()], &output_path);

    // Verify the output file was created
    assert!(
      Path::new(&output_path).exists(),
      "Processed audio file was not created at {}",
      output_path
    );
  }

  #[test]
  fn test_heavy_expansion() {
    let (samples, sample_rate) = load_lead_mono();
    let (rms_before, dr_before) = (overall_rms(&samples), calc_dynamic_range_db(&samples));

    let params = ExpanderParams {
      threshold: -10.0,
      ratio: 8.0, // Large expansion ratio
      attack_time: 0.005,
      release_time: 0.2,
      makeup_gain: 1.0, // No makeup gain
      ..Default::default()
    };
    let processed = expander(&samples, params, None).unwrap();

    let (rms_after, dr_after) = (overall_rms(&processed), calc_dynamic_range_db(&processed));
    assert!(
      dr_after > dr_before * 1.2,
      "Heavy expansion significantly increases dynamic range. Before={:.3} dB, After={:.3} dB",
      dr_before,
      dr_after
    );
    // Heavy expansion can reduce overall RMS if quiet parts are pushed quieter
    assert!(
      rms_after <= rms_before,
      "Heavy expansion typically lowers the average level if no makeup_gain is used."
    );

    // Write processed audio to file after successful assertions
    let output_path = generate_output_path("expansion_heavy_test", "test");
    write_audio(sample_rate as usize, vec![processed.clone()], &output_path);

    // Verify the output file was created
    assert!(
      Path::new(&output_path).exists(),
      "Processed audio file was not created at {}",
      output_path
    );
  }
}

