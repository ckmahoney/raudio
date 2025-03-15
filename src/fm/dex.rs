use super::*;

/// Adapted from "FM Theory and Applications: By Musicians for Musicians" by John Chowning and David Bristow
/// Page 166
const TL_VALUES: [u8; 100] = [
  127, 122, 118, 114, 110, 107, 104, 102, 100, 98, 96, 94, 92, 90, 88, 86, 85, 84, 82, 81, 79, 78, 77, 76, 75, 74, 73,
  72, 71, 70, 69, 68, 67, 66, 65, 64, 63, 62, 61, 60, 59, 58, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48, 47, 46, 45, 44,
  43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15,
  14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
];

/// Calculates the modulation index for an FM synthesis system,
/// inspired by the Yamaha DX7. This function combines a normalized input (0–1),
/// and a Total Level (TL) mapping to emulate the DX7 behavior.
///
/// # Parameters
/// - `normalized_input`: A value in the range [0, 1] representing the user-adjusted input.
///   Typically sourced from a slider or similar UI element.
///
/// # Returns
/// - A floating-point value representing the modulation index, bounded in [0, 13).
fn calculate_modulation_index(normalized_input: f32) -> f32 {
  // Clamp the normalized input to the range [0, 1]
  let input = normalized_input.clamp(0.0, 1.0);

  // Map normalized input (0.0–1.0) to the DX7 Output range (0–99)
  let output_level = (input * 99.0).round() as usize;

  // Look up Total Level (TL) from the array
  let total_level = TL_VALUES[output_level] as f32;

  // Calculate x = (33/16) - (TL / 8)
  let x = (33.0 / 16.0) - (total_level / 8.0);

  // Calculate modulation index I = PI * 2^x, as specified on page 166
  pi * 2f32.powf(x)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_modulation_index_monotonic_increase() {
    // Ensure the modulation index increases monotonically with input
    let step = 0.05;
    let mut previous_value = 0.0;
    for i in 0..=20 {
      let input = i as f32 * step; // Input in the range [0, 1] at intervals of 0.05
      let result = calculate_modulation_index(input);
      println!("For input {:.2}, modulation index is {:.4}", input, result);
      assert!(
        result >= previous_value,
        "Modulation index did not increase for input {:.2}: got {:.4}, previous {:.4}",
        input,
        result,
        previous_value
      );
      previous_value = result;
    }
  }

  #[test]
  fn test_modulation_index_bounds() {
    // Test bounds of modulation index
    assert!(
      (calculate_modulation_index(0.0) - 0.0).abs() < 1e-4,
      "Failed at input 0.0"
    );
    assert!(
      calculate_modulation_index(1.0) < 13.0,
      "Modulation index exceeds upper bound at input 1.0"
    );
  }
}

/// Render and mix signals for multiple operators into a single output signal.
///
/// # Parameters
/// - `operators`: A vector of `Operator` structs to be rendered and mixed.
/// - `n_cycles`: Number of cycles to render.
/// - `cps`: Cycles per second (frequency scaling).
/// - `sample_rate`: The sample rate for rendering.
///
/// # Returns
/// - A single vector of mixed samples.
pub fn render_operators(operators: Vec<Operator>, n_cycles: f32, cps: f32, sample_rate: usize) -> Vec<f32> {
  let n_samples = crate::time::samples_of_cycles(cps, n_cycles); // Total number of samples
  let operator_signals: Vec<Vec<f32>> =
    operators.iter().map(|operator| operator.render(n_cycles, cps, sample_rate)).collect();
  let max_len = operator_signals.iter().map(|x| x.len()).max().unwrap();
  let mut mixed_signal = vec![0.0; max_len];

  for signal in operator_signals {
    for (i, sample) in signal.iter().enumerate() {
      mixed_signal[i] += sample;
    }
  }

  mixed_signal
}

/// Render and mix signals for multiple operators into a single output signal.
///
/// # Parameters
/// - `operators`: A vector of `Operator` structs to be rendered and mixed.
/// - `n_cycles`: Number of cycles to render.
/// - `cps`: Cycles per second (frequency scaling).
/// - `sample_rate`: The sample rate for rendering.
///
/// # Returns
/// - A single vector of mixed samples.
pub fn render_operators_gain(
  operators: Vec<Operator>, gain: f32, n_cycles: f32, cps: f32, sample_rate: usize,
) -> Vec<f32> {
  let n_samples = crate::time::samples_of_cycles(cps, n_cycles); // Total number of samples
  let operator_signals: Vec<Vec<f32>> =
    operators.iter().map(|operator| operator.render(n_cycles, cps, sample_rate)).collect();
  let max_len = operator_signals.iter().map(|x| x.len()).max().unwrap();
  let mut mixed_signal = vec![0.0; max_len];

  for signal in operator_signals {
    for (i, sample) in signal.iter().enumerate() {
      mixed_signal[i] += sample * gain;
    }
  }

  mixed_signal
}

pub fn dx_to_mod_index(dx_level: f32) -> f32 {
  calculate_modulation_index(dx_level / 99.0) // Normalize DX level to [0, 1]
}

pub fn single_modulator(op: Operator) -> Vec<ModulationSource> {
  vec![ModulationSource::Operator(op)]
}

/// Computes the effective center frequency and total resulting bandwidth of an operator.
///
/// # Parameters
/// - `operator`: The `Operator` whose bandwidth is being calculated.
/// - `offset_frequency`: An additional frequency offset applied to the operator's base frequency.
/// - `t`: The current time or phase in the envelope's evolution, used for evaluating modulation envelopes.
///
/// # Returns
/// A tuple containing:
/// - `(f32)`: The effective center frequency of the operator, calculated as the sum of the operator's
///   `frequency` and `offset_frequency`.
/// - `(f32)`: The total bandwidth of the operator, including its own contribution and the bandwidth
///   of all its modulators (calculated recursively).
///
/// # Details
/// - The operator's bandwidth is computed as:
///   \[ \text{Bandwidth} = 2 \times \text{base\_mod\_index} \times \text{frequency} \]
///   where `base_mod_index` is influenced by the operator's modulation envelopes.
/// - If the operator has modulators, their bandwidth contributions are recursively computed
///   and added to the operator's own bandwidth.
///
/// # Example
/// ```rust
/// let operator = Operator {
///     frequency: 440.0,
///     modulation_index: 1.0,
///     modulators: vec![],
///     mod_index_env_mul: Envelope::Constant(1.0),
///     mod_index_env_sum: Envelope::Constant(0.0),
///     mod_freq_env_mul: Envelope::Constant(1.0),
///     mod_freq_env_sum: Envelope::Constant(0.0),
///     mod_gain_env_mul: Envelope::Constant(1.0),
///     mod_gain_env_sum: Envelope::Constant(1.0),
///     termination: TerminationParams::instant_death(),
/// };
///
/// let (center_freq, bandwidth) = compute_bandwidth(&operator, 0.0, 0.0);
/// println!("Center Frequency: {}, Bandwidth: {}", center_freq, bandwidth);
/// ```
///
/// # Notes
/// - The function assumes a time-domain evaluation context for modulation envelopes.
/// - If there are no modulators, the bandwidth is derived solely from the operator's properties.
pub fn compute_bandwidth(operator: &Operator, offset_frequency: f32, t: f32) -> (f32, f32) {
  // Calculate the effective frequency of the operator
  let f = operator.frequency + offset_frequency;
  // Calculate the base modulation index
  let mut base_mod_index = operator.modulation_index;

  // Apply envelopes for modulation index
  base_mod_index += operator.mod_index_env_sum.get_at(t, SR);

  base_mod_index *= operator.mod_index_env_mul.get_at(t, SR);

  // Handle the case where there are no modulators
  if operator.modulators.is_empty() {
    if base_mod_index > 0.0 {
      // Positive modulation index
      return (f, 2.0 * base_mod_index * f);
    } else {
      // Zero modulation index
      return (f, 1.0);
    }
  }

  // Recursive computation for modulators
  let mut total_bandwidth = 0.0;
  for modulator in &operator.modulators {
    if let ModulationSource::Operator(mod_op) = modulator {
      // Compute the bandwidth contribution of this modulator
      let (_mod_freq, mod_bandwidth) = compute_bandwidth(&mod_op, 0.0, t);
      total_bandwidth += mod_bandwidth;
    }
  }

  // Combine the operator's own bandwidth with the total modulator bandwidth
  let operator_bandwidth = 2.0 * base_mod_index * f;

  // println!("Checking bandwidth operator freq {} offset {}", operator.frequency, offset_frequency);
  // Return the effective center frequency and total bandwidth
  (f, total_bandwidth + operator_bandwidth)
}

/// Determines how much frequency range remains for an operator, given a minimum and maximum frequency.
/// This version is strict: if **any** part of the operator's band is out of [min_freq, max_freq],
/// it returns `None`. Otherwise, returns `Some((dist_below, dist_above))`.
///
/// # Parameters
/// - `operator`: The `Operator` whose frequency/bandwidth we’re checking.
/// - `offset_frequency`: The frequency offset passed into `compute_bandwidth`.
/// - `t`: The current time for envelope evaluation in `compute_bandwidth`.
/// - `min_freq`: The minimum allowed frequency.
/// - `max_freq`: The maximum allowed frequency.
///
/// # Returns
/// - `None` if any portion of the operator’s band is out of [min_freq, max_freq].
/// - `Some((dist_below, dist_above))` otherwise, where:
///   - `dist_below` = lower_edge - min_freq
///   - `dist_above` = max_freq - upper_edge
///
/// # Logic
/// 1. We call `compute_bandwidth` to get `(center_freq, total_bw)`.
/// 2. Define the band edges as:
///    - `lower_edge = center_freq - total_bw / 2.0`
///    - `upper_edge = center_freq + total_bw / 2.0`
/// 3. If `lower_edge` is **at or below** `min_freq` or `upper_edge` is **at or above** `max_freq`,
///    we return `None`.
/// 4. Otherwise, we return the distances to the bounding frequencies.
///
/// # Example
/// ```rust
/// let operator = Operator::carrier(440.0);
/// let (min_freq, max_freq) = (20.0, 20_000.0);
///
/// if let Some((db, da)) = compute_strict_range_in_bounds(&operator, 0.0, 0.0, min_freq, max_freq) {
///     println!("Entire band is in range. Dist below: {}, Dist above: {}", db, da);
/// } else {
///     println!("Operator band is out of range!");
/// }
/// ```
pub fn get_remaining_range(
  operator: &Operator, offset_frequency: f32, t: f32, min_freq: f32, max_freq: f32,
) -> Option<(f32, f32)> {
  // 1. Compute center frequency and total bandwidth using the existing function.
  let (center_freq, total_bw) = compute_bandwidth(operator, offset_frequency, t);

  // 2. Calculate the lower and upper edges of the bandwidth.
  let half_bw = total_bw / 2.0;
  let lower_edge = center_freq - half_bw;
  let upper_edge = center_freq + half_bw;

  // Debug output for edges and operator information.
  println!(
    "[DEBUG] get_remaining_range: operator={:?}, lower_edge={:.2}, upper_edge={:.2}",
    operator, lower_edge, upper_edge
  );

  // 3. Check if the band is strictly within bounds.
  if lower_edge <= min_freq || upper_edge >= max_freq {
    println!(
      "[DEBUG] get_remaining_range: Out of bounds! min_freq={:.2}, max_freq={:.2}",
      min_freq, max_freq
    );
    return None;
  }

  // 4. Compute distances from edges to bounds.
  let dist_below = lower_edge - min_freq; // strictly > 0 if in range
  let dist_above = max_freq - upper_edge; // strictly > 0 if in range

  // Debug output for distances.
  println!(
    "[DEBUG] get_remaining_range: distances below={:.2}, above={:.2}",
    dist_below, dist_above
  );

  Some((dist_below, dist_above))
}

fn scale_volume(operator: &Operator, gain: f32) -> Operator {
  // Clone the operator to create a new one with the updated values
  let mut updated_operator = operator.clone();

  // If the operator is a carrier (modulation_index == 0), scale the gain directly
  if updated_operator.modulation_index == 0.0 {
    updated_operator.mod_gain_env_sum = scale_envelope(&updated_operator.mod_gain_env_sum, gain);
  } else {
    // Otherwise, scale the modulation index sum envelope
    updated_operator.mod_index_env_mul = scale_envelope(&updated_operator.mod_index_env_mul, gain);
  }

  updated_operator
}

// Helper function to scale an envelope by a given gain factor
fn scale_envelope(envelope: &Envelope, gain: f32) -> Envelope {
  match envelope {
    Envelope::Constant(prev) => Envelope::Constant(prev * gain),
    Envelope::Parametric {
      attack,
      decay,
      sustain,
      release,
      min,
      max,
      mean,
    } => Envelope::Parametric {
      attack: *attack,
      decay: *decay,
      sustain: sustain * gain,
      release: *release,
      min: min * gain,
      max: max * gain,
      mean: mean * gain,
    },
    Envelope::SampleBased { samples } => Envelope::SampleBased {
      samples: samples.iter().map(|&value| value * gain).collect(),
    },
  }
}

/// Computes the remaining bandwidth available for modulation.
///
/// # Parameters
/// - `operator`: The operator whose bandwidth is being evaluated.
/// - `max_bandwidth`: The maximum allowable bandwidth, typically the Nyquist frequency.
/// - `t`: The current time parameter, used for dynamic envelope evaluation.
///
/// # Returns
/// The remaining bandwidth (in Hz) available for modulation.
pub fn get_remaining_bandwidth(operator: &Operator, max_bandwidth: f32, t: f32) -> f32 {
  let constrained_bandwidth = max_bandwidth.min(NFf); // Ensure bandwidth does not exceed NFf
  fn compute_total_bandwidth(operator: &Operator, t: f32) -> f32 {
    let f = operator.frequency;

    let mut base_mod_index = operator.modulation_index;
    base_mod_index += operator.mod_index_env_sum.get_at(t, SR);

    base_mod_index *= operator.mod_index_env_mul.get_at(t, SR);

    let mut total_bandwidth = 2.0 * base_mod_index * f;

    for modulator in &operator.modulators {
      if let ModulationSource::Operator(mod_op) = modulator {
        total_bandwidth += compute_total_bandwidth(mod_op, t);
      }
    }

    total_bandwidth
  }

  let consumed_bandwidth = compute_total_bandwidth(operator, t);
  (constrained_bandwidth - consumed_bandwidth).max(0.0) // Ensure no negative bandwidth
}

/// Determines the maximum modulation frequency for a given remaining bandwidth and modulation index.
///
/// # Parameters
/// - `remaining_bandwidth`: The bandwidth left available for modulation (in Hz).
/// - `mod_index`: The modulation index to be used for the calculation.
///
/// # Returns
/// The maximum allowable modulation frequency (in Hz).
///
/// # Panics
/// Panics if `mod_index` is less than or equal to 0.
fn determine_mod_freq(remaining_bandwidth: f32, mod_index: f32) -> f32 {
  // Ensure modulation index is positive and reasonable
  if mod_index <= 0.0 {
    panic!("Modulation index must be greater than zero");
  }

  // Calculate the maximum allowable modulation frequency
  let max_mod_freq = remaining_bandwidth / (2.0 * (mod_index + 1.0));

  // Clamp the frequency to an audible range (optional)
  max_mod_freq.clamp(20.0, 20000.0)
}

/// Determines the maximum modulation index for a given remaining bandwidth and modulation frequency.
///
/// # Parameters
/// - `remaining_bandwidth`: The bandwidth left available for modulation (in Hz).
/// - `mod_freq`: The modulation frequency to be used for the calculation.
///
/// # Returns
/// The maximum allowable modulation index.
///
/// # Panics
/// Panics if `mod_freq` is less than or equal to 0.
fn determine_mod_index(remaining_bandwidth: f32, mod_freq: f32) -> f32 {
  if mod_freq <= 0.0 {
    panic!("Modulation frequency must be greater than zero");
  }

  // Calculate the maximum allowable modulation index
  let max_mod_index = (remaining_bandwidth / (2.0 * mod_freq)) - 1.0;

  // Ensure index is non-negative
  max_mod_index.max(0.0)
}

pub fn generate_serial_modulation_chain(operator: &Operator, lowpass_filter: f32) -> Option<Operator> {
  let mut current_operator = operator.clone();
  let max_bandwidth = NFf.min(lowpass_filter);

  loop {
    let bandwidth_remaining = get_remaining_bandwidth(&current_operator, max_bandwidth, 0.0).max(0.0);
    println!("v bandwidth_remaining {}", bandwidth_remaining);
    if bandwidth_remaining <= 0.0 {
      break; // No remaining bandwidth
    }

    // Generate candidate modulators
    let candidates: Vec<Operator> = vec![
      extend_harmonic_range(&current_operator, 1.5),
      thicken_harmonic_density(&current_operator, 2),
      subtle_enhancement(&current_operator),
    ]
    .into_iter()
    .filter_map(|candidate| {
      if let Some(operator) = candidate {
        // Check if the candidate's bandwidth exceeds the remaining bandwidth
        let (_, candidate_bandwidth) = compute_bandwidth(&operator, 0.0, 0.0);
        if candidate_bandwidth <= bandwidth_remaining {
          Some(operator) // Include valid candidate
        } else {
          None // Discard candidate exceeding the bandwidth
        }
      } else {
        None // Discard invalid candidates
      }
    })
    .collect();

    // If no valid candidates remain, stop
    if candidates.is_empty() {
      break;
    }

    // Randomly select a valid candidate and add it as a modulator
    use rand::seq::SliceRandom;
    let mut new_operator = current_operator.clone();
    let new_modulator = candidates.choose(&mut rand::thread_rng()).unwrap().clone();
    new_operator.modulators.push(ModulationSource::Operator(new_modulator));
    current_operator = new_operator;
  }

  // Return the operator if modulators were added, or None otherwise
  if current_operator.modulators.is_empty() {
    None
  } else {
    Some(current_operator)
  }
}

fn extend_harmonic_range(operator: &Operator, amount: f32) -> Option<Operator> {
  let remaining_bandwidth = get_remaining_bandwidth(operator, NFf, 0.0);

  // Determine the maximum modulation frequency for the desired extension
  let new_mod_freq = determine_mod_freq(remaining_bandwidth, 1.0) * amount;
  if new_mod_freq > 20.0 && new_mod_freq < NFf {
    let mut new_operator = operator.clone();
    let new_modulator = Operator::modulator(new_mod_freq, 1.0);
    new_operator.modulators.push(ModulationSource::Operator(new_modulator));
    return Some(new_operator);
  }
  None
}

fn thicken_harmonic_density(operator: &Operator, density_factor: usize) -> Option<Operator> {
  let remaining_bandwidth = get_remaining_bandwidth(operator, NFf, 0.0);
  let base_frequency = operator.frequency;

  let mut new_operator = operator.clone();
  let mut added = false;

  for i in 1..=density_factor {
    let new_mod_freq = determine_mod_freq(remaining_bandwidth / density_factor as f32, 0.5);
    if new_mod_freq > 20.0 && new_mod_freq < NFf {
      let new_modulator = Operator::modulator(base_frequency + new_mod_freq / i as f32, 0.5);
      new_operator.modulators.push(ModulationSource::Operator(new_modulator));
      added = true;
    }
  }

  if added {
    Some(new_operator)
  } else {
    None
  }
}

fn subtle_enhancement(operator: &Operator) -> Option<Operator> {
  let remaining_bandwidth = get_remaining_bandwidth(operator, NFf, 0.0);

  // Choose a moderate frequency and modulation index
  let mod_freq = determine_mod_freq(remaining_bandwidth, 0.8);
  if mod_freq > 20.0 && mod_freq < NFf {
    let mut new_operator = operator.clone();
    let new_modulator = Operator::modulator(mod_freq, 0.8);
    new_operator.modulators.push(ModulationSource::Operator(new_modulator));
    return Some(new_operator);
  }
  None
}

#[cfg(test)]
mod test_bandwidth {
  use super::*;

  #[test]
  fn test_compute_bandwidth_sine() {
    let operator: Operator = Operator::carrier(440.0);
    let (result_center, result_bandwidth) = compute_bandwidth(&operator, 0f32, 0f32);

    assert_eq!(
      result_center, operator.frequency,
      "Must have the same center frequency as fundamental operator"
    );
    assert_eq!(
      result_bandwidth, 1f32,
      "For simple carriers (no modulation) must return 1 for its bandwidth"
    );
  }

  #[test]
  fn test_compute_bandwidth_simple_modulation() {
    let modulator = Operator::modulator(10f32, 1f32);
    let operator: Operator = Operator {
      modulators: vec![ModulationSource::Operator(modulator.clone())],
      ..Operator::carrier(440.0)
    };
    let (result_center, result_bandwidth) = compute_bandwidth(&operator, 0f32, 0f32);

    assert_eq!(
      result_center, operator.frequency,
      "Must have the same center frequency as fundamental operator"
    );
    assert_eq!(
      result_bandwidth,
      modulator.frequency * 2f32,
      "For simple carriers (single modulator), the bandwidth must be twice the modulation frequency"
    );
  }

  #[test]
  fn test_compute_bandwidth_no_modulation_index() {
    let operator: Operator = Operator {
      modulation_index: 0.0,
      ..Operator::carrier(440.0)
    };
    let (result_center, result_bandwidth) = compute_bandwidth(&operator, 0f32, 0f32);

    assert_eq!(
      result_center, operator.frequency,
      "Must have the same center frequency as fundamental operator"
    );
    assert_eq!(
      result_bandwidth, 1.0,
      "For carriers with zero modulation index, bandwidth must be 1"
    );
  }

  #[test]
  fn test_compute_bandwidth_nested_modulators() {
    let inner_modulator = Operator::modulator(20.0, 1.0);
    let modulator = Operator {
      modulators: vec![ModulationSource::Operator(inner_modulator.clone())],
      ..Operator::modulator(10.0, 1.0)
    };
    let operator: Operator = Operator {
      modulators: vec![ModulationSource::Operator(modulator.clone())],
      ..Operator::carrier(440.0)
    };

    let (result_center, result_bandwidth) = compute_bandwidth(&operator, 0f32, 0f32);

    assert_eq!(
      result_center, operator.frequency,
      "Must have the same center frequency as fundamental operator"
    );
    let expected_bandwidth = modulator.frequency * 2.0 + inner_modulator.frequency * 2.0;
    assert_eq!(
      result_bandwidth, expected_bandwidth,
      "Bandwidth must account for nested modulation chains"
    );
  }

  #[test]
  fn test_compute_bandwidth_multiple_modulators() {
    let modulator1 = Operator::modulator(10.0, 1.0);
    let modulator2 = Operator::modulator(15.0, 1.0);
    let operator: Operator = Operator {
      modulators: vec![
        ModulationSource::Operator(modulator1.clone()),
        ModulationSource::Operator(modulator2.clone()),
      ],
      ..Operator::carrier(440.0)
    };

    let (result_center, result_bandwidth) = compute_bandwidth(&operator, 0f32, 0f32);

    assert_eq!(
      result_center, operator.frequency,
      "Must have the same center frequency as fundamental operator"
    );
    let expected_bandwidth = modulator1.frequency * 2.0 + modulator2.frequency * 2.0;
    assert_eq!(
      result_bandwidth, expected_bandwidth,
      "Bandwidth must account for multiple modulators"
    );
  }

  #[test]
  fn test_compute_bandwidth_dynamic_modulation_index() {
    let operator: Operator = Operator {
      modulation_index: 1.0,
      mod_index_env_mul: Envelope::unit_mul(),
      mod_index_env_sum: Envelope::unit_sum(),
      ..Operator::carrier(440.0)
    };
    let (result_center, result_bandwidth) = compute_bandwidth(&operator, 0f32, 0f32);

    let expected_bandwidth = 2.0 * (1.0 * 1.0 + 1.0) * operator.frequency; // Includes unit envelopes
    assert_eq!(
      result_center, operator.frequency,
      "Must have the same center frequency as fundamental operator"
    );
    assert_eq!(
      result_bandwidth, expected_bandwidth,
      "Bandwidth must consider dynamic envelopes for modulation index"
    );
  }

  #[test]
  fn test_compute_bandwidth_high_modulation_index() {
    let operator: Operator = Operator {
      modulation_index: 5.0,
      ..Operator::carrier(440.0)
    };
    let (result_center, result_bandwidth) = compute_bandwidth(&operator, 0f32, 0f32);

    let expected_bandwidth = 2.0 * 5.0 * operator.frequency;
    assert_eq!(
      result_center, operator.frequency,
      "Must have the same center frequency as fundamental operator"
    );
    assert_eq!(
      result_bandwidth, expected_bandwidth,
      "Bandwidth must scale with high modulation index"
    );
  }

  #[test]
  fn test_generate_modulator_with_maximum_frequency() {
    let carrier = Operator {
      modulation_index: 1.0,
      frequency: 440.0,
      ..Operator::default()
    };

    let max_bandwidth = 20000.0; // Example Nyquist frequency
    let remaining_bandwidth = get_remaining_bandwidth(&carrier, max_bandwidth, 0.0);
    let mod_index = 1.0;

    let max_mod_freq = determine_mod_freq(remaining_bandwidth, mod_index);
    assert!(
      max_mod_freq <= 20000.0,
      "Modulation frequency must not exceed Nyquist frequency"
    );

    let new_modulator = Operator::modulator(max_mod_freq, mod_index);
    let (new_center, new_bandwidth) = compute_bandwidth(&new_modulator, 0.0, 0.0);

    assert!(
      new_bandwidth <= remaining_bandwidth,
      "Generated modulator must fit within remaining bandwidth"
    );
    assert_eq!(
      new_center, new_modulator.frequency,
      "New modulator center frequency must match its defined frequency"
    );
  }

  #[test]
  fn test_generate_modulator_with_maximum_modulation_index() {
    let carrier = Operator {
      modulation_index: 1.0,
      frequency: 440.0,
      ..Operator::default()
    };

    let max_bandwidth = 20000.0; // Example Nyquist frequency
    let remaining_bandwidth = get_remaining_bandwidth(&carrier, max_bandwidth, 0.0);
    let mod_freq = 1000.0;

    let max_mod_index = determine_mod_index(remaining_bandwidth, mod_freq);
    assert!(max_mod_index > 0.0, "Modulation index must be positive");

    let new_modulator = Operator::modulator(mod_freq, max_mod_index);
    let (new_center, new_bandwidth) = compute_bandwidth(&new_modulator, 0.0, 0.0);

    assert!(
      new_bandwidth <= remaining_bandwidth,
      "Generated modulator must fit within remaining bandwidth"
    );
    assert_eq!(
      new_center, new_modulator.frequency,
      "New modulator center frequency must match its defined frequency"
    );
  }

  #[test]
  fn test_generate_nested_modulators() {
    let inner_modulator = Operator::modulator(500.0, 1.0);
    let middle_modulator = Operator {
      modulators: vec![ModulationSource::Operator(inner_modulator.clone())],
      frequency: 200.0,
      modulation_index: 2.0,
      ..Operator::default()
    };

    let carrier = Operator {
      modulators: vec![ModulationSource::Operator(middle_modulator.clone())],
      frequency: 440.0,
      modulation_index: 1.0,
      ..Operator::default()
    };

    let max_bandwidth = 20000.0; // Example Nyquist frequency
    let remaining_bandwidth = get_remaining_bandwidth(&carrier, max_bandwidth, 0.0);
    let mod_index = 1.5;

    let max_mod_freq = determine_mod_freq(remaining_bandwidth, mod_index);
    assert!(
      max_mod_freq > 0.0 && max_mod_freq <= 20000.0,
      "Nested modulator frequency must fit within available range"
    );

    let new_nested_modulator = Operator::modulator(max_mod_freq, mod_index);
    let (new_center, new_bandwidth) = compute_bandwidth(&new_nested_modulator, 0.0, 0.0);

    assert!(
      new_bandwidth <= remaining_bandwidth,
      "Generated nested modulator must fit within remaining bandwidth"
    );
    assert_eq!(
      new_center, new_nested_modulator.frequency,
      "Nested modulator center frequency must match its defined frequency"
    );
  }

  #[test]
  fn test_generate_modulator_with_scaling() {
    let carrier = Operator {
      modulation_index: 1.0,
      frequency: 440.0,
      ..Operator::default()
    };

    let max_bandwidth = 20000.0; // Example Nyquist frequency
    let remaining_bandwidth = get_remaining_bandwidth(&carrier, max_bandwidth, 0.0);
    let mod_index = 2.0;
    let mod_freq = determine_mod_freq(remaining_bandwidth, mod_index);

    let new_modulator = Operator::modulator(mod_freq, mod_index);
    let scaled_modulator = scale_volume(&new_modulator, 0.8); // Scale the modulator volume down

    let (original_center, original_bandwidth) = compute_bandwidth(&new_modulator, 0.0, 0.0);
    let (scaled_center, scaled_bandwidth) = compute_bandwidth(&scaled_modulator, 0.0, 0.0);

    assert_eq!(
      original_center, scaled_center,
      "Scaling volume must not affect center frequency"
    );
    assert!(
      scaled_bandwidth <= original_bandwidth,
      "Scaled modulator must have reduced bandwidth"
    );
  }

  #[test]
  fn test_generate_serial_modulation_chain_with_filter() {
    let operator = Operator::carrier(440.0);

    if let Some(chain_operator) = generate_serial_modulation_chain(&operator, NFf) {
      let (_center, total_bandwidth) = compute_bandwidth(&chain_operator, 0.0, 0.0);

      // Ensure the bandwidth is within limits (10,000 Hz)
      assert!(
        total_bandwidth <= NFf,
        "Total bandwidth ({}) exceeds lowpass filter limit",
        total_bandwidth
      );

      // Ensure at least one modulator was added
      assert!(
        !chain_operator.modulators.is_empty(),
        "No modulators were added to the chain"
      );
    } else {
      panic!("Failed to generate serial modulation chain with lowpass filter");
    }
  }
}

/// Represents Zalgorithm 1 in SYX-9 operator configuration for a synth lead
/// Z1 { { 3 > 2 + 6 > 5 > 4 } > 1, { 3 > 2 } > [7, 8, 9]  }
fn dexed_lead(p: f32, n_cycles: f32, cps: f32, freq: f32, mod_gain: f32) -> Vec<Operator> {
  let mut rng = thread_rng();

  let op7_detune_cents = get_dexed_detune(freq, 5);
  let op8_detune_cents = get_dexed_detune(freq, -4);
  let op9_detune_cents = get_dexed_detune(freq, 7);

  let op_frequency = freq / 2f32;

  let fadein = ranger::eval_knob_mod(
    ranger::amod_cycle_fadein_4_16,
    &Knob {
      a: 0.95f32,
      b: 0.5f32,
      c: 0.66f32,
    },
    cps,
    freq,
    1f32,
  );

  let op2 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: fadein.clone(),
    },
    mod_index_env_mul: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.85f32,
          b: 0.77f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    modulators: vec![ModulationSource::Feedback(0.95)],
    ..Operator::modulator(op_frequency * 2f32.powi(-1), mod_gain * dx_to_mod_index(77.0))
  };

  let mut op1 = Operator {
    mod_gain_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.99f32,
          b: 0.95f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    mod_gain_env_mul: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.95f32,
          b: 0.15f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles / 2f32,
      ),
    },
    modulators: single_modulator(op2),
    ..Operator::carrier(op_frequency * 2f32.powi(0))
  };

  let op6 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.85f32,
          b: 0.2f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    mod_index_env_mul: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.95f32,
          b: 0.5f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    ..Operator::modulator(op_frequency * 10f32, mod_gain * dx_to_mod_index(86.0))
  };

  let op5 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.88f32,
          b: 0.2f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    mod_index_env_mul: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_cycle_fadein_1_4,
        &Knob {
          a: 1f32,
          b: 0.5f32,
          c: 1f32,
        },
        cps,
        1f32,
        n_cycles,
      ),
    },
    modulators: single_modulator(op6),
    ..Operator::modulator(op_frequency * 3f32, mod_gain * dx_to_mod_index(69.0))
  };

  // burst of harmonics on note entry
  let op4 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.85f32,
          b: 0.05f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    mod_index_env_mul: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_cycle_fadein_1_4,
        &Knob {
          a: 0.25f32,
          b: 0.5f32,
          c: 1f32,
        },
        cps,
        1f32,
        n_cycles,
      ),
    },
    ..Operator::modulator(op_frequency * 17.38, mod_gain * dx_to_mod_index(75.0))
  };

  let op3 = Operator {
    mod_gain_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.75f32,
          b: 0.2f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    modulators: vec![ModulationSource::Operator(op4), ModulationSource::Operator(op5)],
    ..Operator::carrier(op_frequency * 2f32.powi(0))
  };

  vec![op1, op3]
}

/// Creates a carrier and attaches multiple random modulators with envelopes.
fn random_carrier_with_modulators(
  base_freq: f32, cps: f32, n_cycles: f32, num_modulators: usize, depth: usize,
) -> Operator {
  let modulators: Vec<ModulationSource> = (0..num_modulators)
    .map(|_| ModulationSource::Operator(random_modulator_with_envelope(cps, n_cycles, base_freq, depth)))
    .collect();

  let mut carrier = Operator::carrier(base_freq);
  carrier.modulators.extend(modulators);

  carrier
}

/// Renders all carriers with dynamic modulation applied via their envelopes.
pub fn render_operators_with_envelopes(
  operators: Vec<Operator>, n_cycles: f32, cps: f32, sample_rate: usize,
) -> Vec<f32> {
  let mut mixed_signal = vec![];
  for operator in operators {
    let signal = operator.render(n_cycles, cps, sample_rate);
    mixed_signal.extend(signal);
  }
  mixed_signal
}

#[test]
fn animated_fm_synthesis_demo() {
  for cps in vec![1.0f32, 1.2f32, 1.4f32, 1.6f32] {
    let base_freqs: Vec<f32> = vec![110.0, 220.0, 330.0]; // Example base frequencies
    let n_cycles = 4.0 * cps;

    let mut final_signal = vec![];

    // Create carriers and render them with their modulators
    for base_freq in base_freqs {
      let carrier = random_carrier_with_modulators(base_freq, cps, n_cycles, 3, 2); // 3 modulators, depth 2
      let rendered_signal = carrier.render(n_cycles, cps, SR);
      final_signal.extend(rendered_signal);
    }

    // Save the rendered signal to a WAV file
    engrave::samples(SR, &final_signal, &format!("animated_fm_synthesis_demo_{}_cps.wav", cps));
  }
}
