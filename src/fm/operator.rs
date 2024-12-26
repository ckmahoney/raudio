use crate::synth::{SR,SRf};
use crate::types::timbre::Visibility;
use std::sync::Arc;
use super::*;

#[derive(Clone)]
/// Represents an individual FM synthesis operator.
pub struct Operator {
  /// Base frequency of the operator in Hz.
  pub frequency: f32,
  /// Modulation index for this operator.
  pub modulation_index: f32,
  /// Defines modulators applied to this operator.
  pub modulators: Vec<ModulationSource>,
  /// Amplitude envelope for multiplicative modulation index.
  pub mod_index_env_mul: Envelope,
  /// Amplitude envelope for additive modulation index.
  pub mod_index_env_sum: Envelope,
  /// Callback for multiplicative modulation index.
  pub mod_index_mul: Option<Callback>,
  /// Callback for additive modulation index.
  pub mod_index_sum: Option<Callback>,
  /// Amplitude envelope for multiplicative modulation frequency.
  pub mod_freq_env_mul: Envelope,
  /// Amplitude envelope for additive modulation frequency.
  pub mod_freq_env_sum: Envelope,
  pub mod_gain_env_sum: Envelope,
  pub mod_gain_env_mul: Envelope,
  /// Callback for multiplicative modulation frequency.
  pub mod_freq_mul: Option<Callback>,
  /// Callback for additive modulation frequency.
  pub mod_freq_sum: Option<Callback>,
  /// Termination logic parameters.
  pub termination: TerminationParams,
}

impl Default for Operator {
  fn default() -> Self {
      Operator {
          frequency: 440.0, // Default frequency (A4)
          modulation_index: 0.0, // No modulation index by default
          modulators: Vec::new(), // No modulators by default
          mod_index_env_mul: Envelope::unit_mul(),
          mod_index_env_sum: Envelope::empty_sum(),
          mod_index_mul: None,
          mod_index_sum: None,
          mod_freq_env_mul: Envelope::unit_mul(),
          mod_freq_env_sum: Envelope::empty_sum(),
          mod_freq_mul: None,
          mod_freq_sum: None,
          mod_gain_env_mul: Envelope::unit_mul(),
          mod_gain_env_sum: Envelope::unit_sum(),
          termination: TerminationParams::instant_death(),
      }
  }
}

fn to_modulators(mods: &Vec<(f32, f32)>, modulator_playback_rate: f32) -> Vec<ModulationSource> {
  mods.iter().map(|(w, m)| ModulationSource::Operator(
    Operator::modulator(*m * modulator_playback_rate, *w)
  )).collect()
}


#[derive(Clone)]
pub enum ModulationSource {
  Operator(Operator),
  Feedback(f32), // Feedback gain
}

fn count_feedbacks(mod_sources: &[ModulationSource]) -> usize {
  mod_sources.iter().map(|m| match m {
      ModulationSource::Feedback(_) => 1,
      ModulationSource::Operator(op) => count_feedbacks(&op.modulators),
  }).sum()
}

fn initialize_feedback_states(mod_sources: &[ModulationSource], feedback_states: &mut Vec<f32>, offset: &mut usize) {
  for mod_source in mod_sources {
      match mod_source {
          ModulationSource::Feedback(_) => {
              // Initialize the feedback state for this modulator
              feedback_states[*offset] = 0.0;
              *offset += 1;
          }
          ModulationSource::Operator(op) => {
              // Recursively initialize feedback states for nested operators
              initialize_feedback_states(&op.modulators, feedback_states, offset);
          }
      }
  }
}


impl Operator {
  
pub fn render(&self, n_cycles: f32, cps: f32, sample_rate: usize) -> Vec<f32> {
  let n_samples = crate::time::samples_of_cycles(cps, n_cycles);
  let mut signal: Vec<f32> = Vec::with_capacity(n_samples);

  // Count all feedback modulators in the hierarchy
  let n_feedbacks = count_feedbacks(&self.modulators);

  // Allocate and initialize feedback states
  let mut feedback_states = vec![0.0; n_feedbacks];
  let mut offset = 0;
  initialize_feedback_states(&self.modulators, &mut feedback_states, &mut offset);

  // Generate the signal
  for i in 0..n_samples {
      let t = i as f32 / sample_rate as f32;

      // Evaluate the operator for the current sample, passing shared feedback states
      let output = self.eval(t, &mut feedback_states);

      // Append the result to the signal
      signal.push(output);
  }

  signal
}

pub fn eval(&self, t: f32, feedback_states: &mut [f32]) -> f32 {
  // Calculate the effective frequency considering modulation and envelopes
  let effective_frequency = self.frequency
      * (self.mod_freq_env_mul.get_at(t, SR)
          * self.mod_freq_mul.as_ref().map_or(1.0, |callback| callback.evaluate(t)))
      + (self.mod_freq_env_sum.get_at(t, SR)
          + self.mod_freq_sum.as_ref().map_or(0.0, |callback| callback.evaluate(t)));
  
  let angular_frequency = pi2 * effective_frequency;

  let mut feedback_offset = 0;
  let mut phase_offset = 0.0;

  // Iterate over modulators and apply feedback/modulation
  for mod_source in &self.modulators {
      phase_offset += match mod_source {
          ModulationSource::Operator(mod_op) => {
              let sub_feedback_states = &mut feedback_states[feedback_offset..];
              let feedback_count = count_feedbacks(&mod_op.modulators);
              feedback_offset += feedback_count;

              // Compute modulation index with envelopes and callbacks
              let modulation_index = mod_op.modulation_index;
              let env_mul = mod_op.mod_index_env_mul.get_at(t, SR);
              let callback_mul = mod_op.mod_index_mul.as_ref().map_or(1.0, |callback| callback.evaluate(t));
              let env_sum = mod_op.mod_index_env_sum.get_at(t, SR);
              let callback_sum = mod_op.mod_index_sum.as_ref().map_or(0.0, |callback| callback.evaluate(t));

              let mod_index = modulation_index * env_mul * callback_mul * (env_sum + callback_sum);

              if (3f32*t) % 1f32 < 0.001f32 {
                // println!("t: {}", t);
                // println!("modulation_index {} mod_index: {}", modulation_index, mod_index);
                // println!("env_mul component: {}", env_mul);
                // println!("callback_mul component: {}", callback_mul);
                // println!("env_sum component: {}", env_sum);
                // println!("callback_sum component: {}", callback_sum);
                // println!("final mod_index: {}", mod_index);
            }

              mod_op.eval(t, sub_feedback_states) * mod_index
          }
          ModulationSource::Feedback(gain) => {
            // Retrieve the previous feedback state
            let prev_feedback_value = feedback_states[feedback_offset];

            // Compute the current feedback signal
            let feedback_mod_index = self.modulation_index 
                * self.mod_index_env_mul.get_at(t, SR)
                * self.mod_index_mul.as_ref().map_or(1.0, |callback| callback.evaluate(t))
                + self.mod_index_env_sum.get_at(t, SR)
                + self.mod_index_sum.as_ref().map_or(0.0, |callback| callback.evaluate(t));

            let current_feedback_signal = (angular_frequency * t + prev_feedback_value).sin();

            // Apply feedback modulation
            let modulated_feedback = *gain * current_feedback_signal * feedback_mod_index;

            // Update feedback state for the next iteration
            feedback_states[feedback_offset] = current_feedback_signal;

            feedback_offset += 1; 
            modulated_feedback
        }
      };
  }


  // Compute the signal gain with envelopes
  let gain = self.mod_gain_env_mul.get_at(t, SR)
      * self.mod_gain_env_sum.get_at(t, SR);

      if (1f32*t) % 1f32 <  0.0004 && (t < 1f32 || t > 7f32 ){
        // println!("t: {} gain {} freq {}", t, gain, angular_frequency);
        // println!("modulation_index {} mod_index: {}", modulation_index, mod_index);
        // println!("env_mul component: {}", env_mul);
        // println!("callback_mul component: {}", callback_mul);
        // println!("env_sum component: {}", env_sum);
        // println!("callback_sum component: {}", callback_sum);
        // println!("final mod_index: {}", mod_index);
    }
  let y = (angular_frequency * t + phase_offset).sin();
  y * gain
}





  /// Constructs a standalone carrier operator (no modulators, no modulation index).
  pub fn carrier(frequency: f32) -> Self {
    Operator {
      frequency,
      modulation_index: 0.0, 
      modulators: Vec::new(),
      mod_index_env_mul: Envelope::unit_mul(),
      mod_index_env_sum: Envelope::empty_sum(),
      mod_index_mul: None,
      mod_index_sum: None,
      mod_freq_env_mul: Envelope::unit_mul(),
      mod_freq_env_sum: Envelope::unit_sum(),
      mod_freq_mul: None,
      mod_freq_sum: None,
      mod_gain_env_mul: Envelope::unit_mul(),
      mod_gain_env_sum: Envelope::unit_sum(),
      termination: TerminationParams::instant_death(),
    }
  }

  /// Constructs a modulator operator with a given frequency and modulation index.
  pub fn modulator(frequency: f32, modulation_index: f32) -> Self {
    Operator {
      frequency,
      modulation_index,
      modulators: Vec::new(),
      mod_index_env_mul: Envelope::unit_mul(),
      mod_index_env_sum: Envelope::empty_sum(),
      mod_index_mul: None,
      mod_index_sum: None,
      mod_freq_env_mul: Envelope::unit_mul(),
      mod_freq_env_sum: Envelope::empty_sum(),
      mod_freq_mul: None,
      mod_freq_sum: None,
      mod_gain_env_mul: Envelope::unit_mul(),
      mod_gain_env_sum: Envelope::unit_sum(),
      termination: TerminationParams::instant_death(),
    }
  }
}

/// Defines a callback for modulation, which can either be a function pointer or a cloneable closure.
#[derive(Clone)]
pub enum Callback {
    /// A function pointer taking time `t` as input and returning a modulation value.
    Pointer(fn(f32) -> f32),
    /// A cloneable closure taking time `t` as input and returning a modulation value.
    Closure(Arc<dyn Fn(f32) -> f32 + Send + Sync>),
}

impl Callback {
    /// Evaluates the callback at a given time `t`.
    pub fn evaluate(&self, t: f32) -> f32 {
      match self {
        Callback::Pointer(func) => func(t),
        Callback::Closure(closure) => closure(t),
      }
    }
  }
  
/// Represents an amplitude or modulation envelope for operators.
#[derive(Clone,Debug)]
pub enum Envelope {
  /// A parametric envelope defined by ADSR parameters.
  Parametric {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    min: f32,
    max: f32,
    mean: f32,
  },
  /// A sample-based envelope defined by a series of precomputed samples.
  SampleBased { samples: Vec<f32> },
}

impl Envelope {
  /// Creates an empty parametric envelope for additive modulation.
  pub fn empty_sum() -> Self {
    Envelope::Parametric {
      attack: 0.0,
      decay: 0.0,
      sustain: 0.0,
      release: 0.0,
      min: 0.0,
      max: 0.0,
      mean: 0.0,
    }
  }

  /// Creates an empty parametric envelope for multiplicative modulation.
  pub fn empty_mul() -> Self {
    Envelope::Parametric {
      attack: 0.0,
      decay: 0.0,
      sustain: 1.0,
      release: 0.0,
      min: 0.0,
      max: 1.0,
      mean: 1.0,
    }
  }

  /// Creates a unit parametric envelope for additive modulation.
  pub fn unit_sum() -> Self {
    Envelope::Parametric {
      attack: 0.0,
      decay: 0.0,
      sustain: 1.0,
      release: 0.0,
      min: 1.0,
      max: 1.0,
      mean: 1.0,
    }
  }

  /// Creates a unit parametric envelope for multiplicative modulation.
  pub fn unit_mul() -> Self {
    Envelope::Parametric {
      attack: 0.0,
      decay: 0.0,
      sustain: 1.0,
      release: 0.0,
      min: 1.0,
      max: 1.0,
      mean: 1.0,
    }
  }

  /// Creates a sample-based envelope from a precomputed set of samples.
  pub fn from_samples(samples: &[f32]) -> Self {
    Envelope::SampleBased {
      samples: samples.to_vec(),
    }
  }

  /// Evaluates the envelope at a given sample index.
  pub fn evaluate(&self, sample_index: usize) -> f32 {
    match self {
      Envelope::Parametric {
        attack,
        decay,
        sustain,
        release,
        min,
        max,
        mean,
      } => {
        // Placeholder parametric envelope evaluation logic
        // Replace with proper ADSR envelope logic
        let t = sample_index as f32;
        *mean + t * (*max - *min) // Simplified for demonstration
      }
      Envelope::SampleBased { samples } => {
        if sample_index < samples.len() {
          samples[sample_index]
        } else {
          0.0 // Default to zero if out of range
        }
      }
    }
  }

  /// Retrieves the envelope value at time `t`.
  pub fn get_at(&self, t: f32, sr: usize) -> f32 {
    let sample_rate = sr as f32;
    match self {
      Envelope::Parametric {
        attack,
        decay,
        sustain,
        release,
        min,
        max,
        ..
      } => {
        // ADSR-like behavior
        if t < *attack {
          // Attack phase
          let progress = t / *attack;
          min + progress * (max - min)
        } else if t < *attack + *decay {
          // Decay phase
          let progress = (t - *attack) / *decay;
          max - progress * (max - sustain)
        } else if t < *attack + *decay + *sustain {
          // Sustain phase
          *sustain
        } else if t < *attack + *decay + *sustain + *release {
          // Release phase
          let progress = (t - (*attack + *decay + *sustain)) / *release;
          sustain - progress * (sustain - min)
        } else {
          // After release, return minimum
          *min
        }
      }
      Envelope::SampleBased { samples } => {
        // Compute the sample index based on time and sample rate
        let sample_index = (t * sample_rate) as usize;
        // Retrieve the sample value if within range, else return zero
        samples.get(sample_index).cloned().unwrap_or(0.0)
      }
    }
  }
}

/// Represents the termination value for operators.
#[derive(Clone)]
pub enum TerminalValue {
  Negative,
  Positive,
  Null,
  C(f32),
}

#[derive(Clone)]
/// Handles termination behavior for operators in terms of signal energy and time.
pub struct TerminationParams {
  /// Minimum signal magnitude for termination.
  threshold: f32,
  /// Time below threshold before termination, in seconds.
  duration: f32,
  /// Final value upon termination.
  end_value: TerminalValue,
}

impl TerminationParams {
  /// Creates a termination parameter with instant termination.
  pub fn instant_death() -> Self {
    TerminationParams {
      threshold: 0.0,
      duration: 0.0,
      end_value: TerminalValue::Null,
    }
  }
}



  /// Counts the total number of unique sidebands for this operator and its modulators.
  /// Includes carrier and accounts for cascading modulators.
  // pub fn count_sidebands(&self) -> usize {
  //   // Recursive function to count sidebands for a given modulator chain
  //   fn count_modulator_sidebands(operator: &Operator, depth: usize) -> usize {
  //     // Number of sidebands introduced by this operator
  //     let sidebands = 1 << depth; // 2^depth sidebands
  //                                 // Recurse through each modulator
  //     operator.modulators.iter().fold(sidebands, |acc, modulator| {
  //       acc + count_modulator_sidebands(modulator, depth + 1)
  //     })
  //   }

  //   // Start with the carrier (depth 0)
  //   count_modulator_sidebands(self, 0)
  // }

  // pub fn identify_sideband_frequencies_with_order(&self, sideband_order: i32) -> Vec<f32> {
  //   fn recursive_sidebands(operator: &Operator, accumulated_carrier: f32, sideband_order: i32) -> Vec<f32> {
  //     // Start by enumerating sidebands for the *current* operator
  //     // relative to the accumulated carrier frequency.
  //     let mut freq_list = Vec::new();
  //     for n in -sideband_order..=sideband_order {
  //       // For each integer n, we shift by n × f_op × index
  //       let offset = n as f32 * operator.frequency * operator.modulation_index;
  //       freq_list.push(accumulated_carrier + offset);
  //     }

  //     // Recursively handle each child modulator
  //     for child in &operator.modulators {
  //       // Instead of calling `recursive_sidebands(child, child.frequency, ...)`
  //       // we pass the parent's *accumulated* frequency to preserve
  //       // the nested offset. We do it per existing frequency in freq_list.
  //       let mut new_freqs = Vec::new();
  //       for &freq in &freq_list {
  //         // Compute child sidebands starting from `freq` as the new base.
  //         let child_sidebands = recursive_sidebands(child, freq, sideband_order);

  //         // Optionally fold them into `new_freqs` more elaborately:
  //         new_freqs.extend(child_sidebands);
  //       }
  //       // Merge new child frequencies into our current operator's freq list
  //       freq_list.extend(new_freqs);
  //     }

  //     freq_list
  //   }

  //   let mut freqs = recursive_sidebands(self, self.frequency, sideband_order);
  //   freqs.sort_by(|a, b| a.partial_cmp(b).unwrap());
  //   freqs.dedup();
  //   freqs
  // }

  // /// Computes the bandwidth for a given modulator index.
  // pub fn lookup_bandwidth(&self, modulator_index: usize) -> Option<(f32, f32)> {
  //   fn compute_bandwidth(operator: &Operator, t: f32) -> (f32, f32) {
  //     if operator.modulators.is_empty() {
  //       return (operator.eval(t), 0.0);
  //     }

  //     let mut min_freq = operator.eval(t);
  //     let mut max_freq = operator.eval(t);

  //     for modulator in &operator.modulators {
  //       let (sub_center, sub_bandwidth) = compute_bandwidth(modulator, t);
  //       let modulation_depth = operator.modulation_index;

  //       let sub_min = sub_center - sub_bandwidth / 2.0;
  //       let sub_max = sub_center + sub_bandwidth / 2.0;

  //       min_freq -= modulation_depth * sub_max;
  //       max_freq += modulation_depth * sub_max;
  //     }

  //     let center = (min_freq + max_freq) / 2.0;
  //     let bandwidth = max_freq - min_freq;

  //     (center, bandwidth)
  //   }

  //   if modulator_index == 0 {
  //     Some(compute_bandwidth(self, 0.0))
  //   } else if modulator_index <= self.modulators.len() {
  //     Some(compute_bandwidth(&self.modulators[modulator_index - 1], 0.0))
  //   } else {
  //     None
  //   }
  // }
impl Operator {
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::render::engrave;
  use crate::synth::SR;

  #[test]
  fn test_single_carrier_with_no_modulator() {
    // Carrier at 330 Hz
    let carrier = Operator::carrier(330.0);

    let signal = carrier.render(12f32, 1.5, SR);

    // Ensure signal has values
    assert!(!signal.is_empty());
    let filename = format!("dev-audio/test-single-carrier-no-operator-freq-{}", carrier.frequency);
    engrave::samples(SR, &signal, &filename);
  }

  // #[test]
  // fn test_single_carrier_with_single_modulator() {
  //   // Carrier at 330 Hz
  //   let mut carrier = Operator::carrier(330.0);

  //   // Modulator at 110 Hz with modulation index of 1.0
  //   let modulator = Operator::modulator(110.0, 1.0);

  //   // Combine modulator into the carrier
  //   carrier.modulators.push(modulator);
  //   let sideband_n = carrier.count_sidebands();

  //   // Generate the signal
  //   let signal = render_operator(&carrier, 12f32, 1.5, SR);

  //   // Ensure signal has values
  //   assert!(!signal.is_empty());
  //   let filename = format!(
  //     "dev-audio/test-single-carrier-one-operator-freq-{}-modfreq-{}",
  //     carrier.frequency, carrier.modulators[0].frequency
  //   );
  //   engrave::samples(SR, &signal, &filename);
  // }

  // #[test]
  // fn test_sideband_identification() {
  //   let mut carrier = Operator::carrier(1330.0);
  //   let modulator_d = Operator::modulator(220.0, 1.0);
  //   let mut modulator_c = Operator::modulator(110.0, 0.5);
  //   modulator_c.modulators.push(modulator_d);
  //   let mut modulator_b = Operator::modulator(55.0, 0.3);
  //   modulator_b.modulators.push(modulator_c);
  //   carrier.modulators.push(modulator_b);

  //   let sidebands = carrier.identify_sideband_frequencies_with_order(12i32);
  //   println!("Sidebands: {:?}", sidebands);

  //   // Validate the number and range of sidebands
  //   assert!(sidebands.len() > 0);
  //   assert!(sidebands.contains(&carrier.frequency));
  //   let signal = render_operator(&carrier, 12f32, 1.5, SR);

  //   // Ensure signal has values
  //   assert!(!signal.is_empty());
  //   let ffs: Vec<f32> = (&carrier).modulators.iter().map(|m| m.frequency).collect();
  //   let filename = format!(
  //     "dev-audio/test-single-carrier-one-operator-freq-{}-modfreqs-{:?}",
  //     carrier.frequency, ffs
  //   );
  //   engrave::samples(SR, &signal, &filename);
  // }

  // #[test]
  // fn test_single_carrier_with_three_compound_modulators() {
  //   // Carrier at 330 Hz
  //   let mut carrier = Operator::carrier(330.0);

  //   // Modulator chain: D -> C -> B
  //   let modulator_d = Operator::modulator(220.0, 1.0);
  //   let mut modulator_c = Operator::modulator(110.0, 0.5);
  //   modulator_c.modulators.push(ModulationSource::Operator(modulator_d));

  //   let mut modulator_b = Operator::modulator(55.0, 0.3);
  //   modulator_c.modulators.push(ModulationSource::Operator(modulator_c));

  //   // expected analysis of the above
  //   // modulator_d has an effective output range of +/- 220 = |220|
  //   // modulator_c has an effective output range of 0.5 * (+/- 110 + 220) = |330/2| = |150|
  //   // modulator_b has an effective output range of 0.3 * (+/- 55 + 150) = |205*0.3| = |61.5|
  //   // we should see spectral components at carrier = 330 ->
  //   // layer 1: 330 +/ 61.5 -> [269.5, 391.5]
  //   // layer 2: [layer 1] +- 150 -> [ 419.5, 118.5,  541, 241]
  //   // layer 3: [layer 2] +- 220 -> [ 199.5, 639.5, -101.5, 338.5, 321, 461]
  //   // combine all laters [ carrier + .. layer1  + .. layer 2 + .. layer 3]
  //   // [ 330, 269.5, 391.5, 419.5, 118.5,  541, 241, 199.5, 639.5, -101.5, 338.5, 321, 461 ]

  //   // Add modulators to the carrier
  //   carrier.modulators.push(modulator_b);
  //   let sideband_n = carrier.count_sidebands();
  //   // Generate the signal
  //   let signal = render_operator(&carrier, 12f32, 1.5, SR);

  //   // Ensure signal has values
  //   assert!(!signal.is_empty());
  //   let ffs: Vec<f32> = (&carrier).modulators.iter().map(|m| m.frequency).collect();
  //   let filename = format!(
  //     "dev-audio/test-single-carrier-one-operator-freq-{}-modfreqs-{:?}",
  //     carrier.frequency, ffs
  //   );
  //   engrave::samples(SR, &signal, &filename);
  // }

  // #[test]
  // fn test_lookup_bandwidth() {
  //   let mut carrier = Operator::carrier(330.0);

  //   let modulator_d = Operator::modulator(220.0, 1.0);
  //   let mut modulator_c = Operator::modulator(110.0, 0.5);
  //   modulator_c.modulators.push(modulator_d);

  //   let mut modulator_b = Operator::modulator(55.0, 0.3);
  //   modulator_b.modulators.push(modulator_c);

  //   carrier.modulators.push(modulator_b);

  //   // Lookup bandwidth for the carrier
  //   let (center, bandwidth) = carrier.lookup_bandwidth(0).unwrap();
  //   println!("Carrier - Center: {}, Bandwidth: {}", center, bandwidth);

  //   // Lookup bandwidth for modulator B
  //   let (center, bandwidth) = carrier.lookup_bandwidth(1).unwrap();
  //   println!("Modulator B - Center: {}, Bandwidth: {}", center, bandwidth);

  //   // Lookup bandwidth for modulator C
  //   let (center, bandwidth) = carrier.lookup_bandwidth(2).unwrap();
  //   println!("Modulator C - Center: {}, Bandwidth: {}", center, bandwidth);

  //   // Lookup bandwidth for modulator D
  //   let (center, bandwidth) = carrier.lookup_bandwidth(3).unwrap();
  //   println!("Modulator D - Center: {}, Bandwidth: {}", center, bandwidth);
  // }
}



mod demo {
  use super::*;
  use crate::render::engrave;
  use crate::synth::SR;

  fn build_modulators(w: f32, modulator_playback_rate: f32) -> Vec<ModulationSource> {
    to_modulators(
      &vec![(w, 300.0), (w, 100.0), (w, 150.0), (w, 50.0), (w, 25.0)],
      modulator_playback_rate,
    )
  }

  // For rearranging lowest to highest order:
  fn build_modulators_low_to_high(w: f32, modulator_playback_rate: f32) -> Vec<Operator> {
    vec![(w, 25.0), (w, 50.0), (w, 100.0), (w, 150.0), (w, 300.0)]
      .iter()
      .map(|(w, m)| Operator::modulator(*m * modulator_playback_rate, *w))
      .collect()
  }

  // For random sort:
  fn build_modulators_random_order(w: f32, modulator_playback_rate: f32) -> Vec<Operator> {
    vec![(w, 150.0), (w, 25.0), (w, 50.0), (w, 300.0), (w, 100.0)]
      .iter()
      .map(|(w, m)| Operator::modulator(*m * modulator_playback_rate, *w))
      .collect()
  }

  // Example usage
  #[test]
  fn example_usage() {
    let carrier_frequency = 440.0;
    let modulation_index = 1.0;
    let modulator_playback_rate = 1.0;

    // Build modulator chains
    let modulators = build_modulators(modulation_index, modulator_playback_rate);

    // Create a carrier and attach the modulators
    let mut carrier = Operator::carrier(carrier_frequency);
    carrier.modulators.extend(modulators);
    let cps = 1.5f32;
    let signal = carrier.render(32f32 * cps, cps, SR);

    let filename = format!(
      "dev-audio/test-single-carrier-one-operator-freq-{}-fm-song",
      carrier.frequency
    );
    engrave::samples(SR, &signal, &filename);
    // Now `carrier` has a modulator chain you can use for synthesis
  }

  // Example usage
  #[test]
  fn example_usage2() {
    let freqs: Vec<f32> = (1..11).map(|i| i as f32 * 110f32).collect();
    let mut melody: Vec<f32> = vec![];
    for carrier_frequency in &freqs {
      let modulator_playback_rate = 1.0;

      // Build modulator chains
      let w = 8f32;
      let modulators = build_modulators(w, modulator_playback_rate);

      // Create a carrier and attach the modulators
      let mut carrier = Operator::carrier(*carrier_frequency);
      carrier.modulators.extend(modulators);
      let cps = 1.5f32;
      let signal = carrier.render(16f32 * cps, cps, SR);
      melody.extend(&signal);
    }

    let filename = format!("dev-audio/test-melodic-fm-song");
    engrave::samples(SR, &melody, &filename);
    // Now `carrier` has a modulator chain you can use for synthesis
  }
}


mod demo2 {

  use crate::render::engrave;
  use super::*;

  #[test]
  fn example_usage() {
    let freqs: Vec<f32> = (1..11).map(|i| i as f32 * 44f32).collect();
    for i in 0..5 {
    let mut melody: Vec<f32> = vec![];
        let cps = 1.0; // Playback rate in cycles per second
        let n_cycles = 2f32 * cps;
        let modulators :Vec<ModulationSource> = generate_modulators_with_envelopes(4, cps, n_cycles, 100f32)
          .iter().map(|o|ModulationSource::Operator((*o).clone())).collect();

        for fund in &freqs {
            let modulator_playback_rate = 1.0;

            // Create a carrier and attach the modulators
            let carrier = Operator {
              modulators: modulators.clone(),
              ..Operator::carrier(*fund)
            };
            let mut signal =  carrier.render(n_cycles, cps, SR);
            melody.extend(signal)

            // Now `carrier` is ready for synthesis
        }

        let mut rng = thread_rng();
        let value = rng.gen::<f32>();
        
        let filename = format!(
            "dev-audio/test-random-fm-synth-{}", value
        );
        engrave::samples(SR, &melody, &filename);
    }    

    }
}
