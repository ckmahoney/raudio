use super::*;
pub use crate::phrasing::dynamics::gen_organic_amplitude;
pub use crate::types::timbre::Visibility;

use crate::phrasing::ranger::{self, Knob, Ranger};
use rand::{self, thread_rng, Rng};

/// Generates a random knob with parameters `a`, `b`, and `c`.
pub fn random_knob() -> Knob {
  let mut rng = rand::thread_rng();
  Knob {
    a: rng.gen_range(0.0..1.0),
    b: rng.gen_range(0.0..1.0),
    c: rng.gen_range(0.0..1.0),
  }
}

/// Selects a random modulation function from the available options.
pub fn random_modulation_function() -> Ranger {
  let mut rng = rand::thread_rng();
  let modulation_functions: Vec<fn(&Knob, f32, f32, f32, f32, f32) -> f32> = vec![
    ranger::amod_fadeout,
    ranger::amod_cycle_fadein_1_4,
    ranger::amod_cycle_fadein_4_16,
    ranger::amod_stab,
    ranger::amod_fadeout,
    ranger::amod_burp,
    ranger::amod_fall,
    ranger::amod_slowest,
    ranger::amod_pluck2,
  ];
  modulation_functions[rng.gen_range(0..modulation_functions.len())]
}
/// Selects a random modulation function from the available options.
pub fn random_continuous_function() -> Ranger {
  let mut rng = rand::thread_rng();
  let modulation_functions: Vec<fn(&Knob, f32, f32, f32, f32, f32) -> f32> = vec![
    ranger::amod_fadeout,
    ranger::amod_oscillation_sine,
    ranger::amod_oscillation_tri,
    ranger::amod_oscillation_sine,
  ];
  modulation_functions[rng.gen_range(0..modulation_functions.len())]
}

/// Generates a random modulator with an envelope.
pub fn random_modulator(cps: f32, n_cycles: f32, fund: f32, z_index: usize) -> Operator {
  let mut rng = rand::thread_rng();

  // Generate random properties
  let knob = random_knob();
  let mod_func: Ranger = if rng.gen::<f32>() < 0.5f32 {
    random_modulation_function()
  } else {
    random_continuous_function()
  };
  let mod_freq = fund * rng.gen_range(1..12) as f32; // Random modulator frequency
  let mod_index_env = if rng.gen::<f32>() < 0.5f32 {
    vec![
      ranger::eval_knob_mod(
        random_modulation_function(),
        &random_knob(),
        cps,
        mod_freq,
        n_cycles / 3f32,
      ),
      ranger::eval_knob_mod(
        random_modulation_function(),
        &random_knob(),
        cps,
        mod_freq,
        n_cycles / 3f32,
      ),
      ranger::eval_knob_mod(
        random_modulation_function(),
        &random_knob(),
        cps,
        mod_freq,
        n_cycles / 3f32,
      ),
    ]
    .concat()
  } else {
    gen_organic_amplitude(10, ((n_cycles * SRf) as f32 * cps) as usize, Visibility::Visible)
  };
  let modulators = if z_index > 0 {
    vec![ModulationSource::Operator(random_modulator(
      cps,
      n_cycles,
      fund,
      z_index - 1,
    ))]
  } else {
    vec![]
  };
  // Construct modulator
  Operator {
    frequency: mod_freq,
    modulation_index: rng.gen_range(0.5..2.0), // Random modulation index
    modulators,                                // No nested modulators
    mod_index_env_mul: Envelope::from_samples(&mod_index_env),
    ..Default::default()
  }
}

/// Generates a random modulator with an envelope.
pub fn random_finite_envelope(cps: f32, n_cycles: f32, freq: f32) -> Envelope {
  let mut rng = rand::thread_rng();
  let mod_index_env = ranger::eval_knob_mod(random_modulation_function(), &random_knob(), cps, freq, n_cycles);
  let mod_index_env: Vec<f32> = mod_index_env.iter().map(|x| x.powi(8i32)).collect();
  Envelope::from_samples(&mod_index_env)
}

/// Generates a random modulator with an envelope.
pub fn random_continuous_envelope(cps: f32, n_cycles: f32, freq: f32) -> Envelope {
  let mut rng = rand::thread_rng();
  let mod_index_env = ranger::eval_knob_mod(random_continuous_function(), &random_knob(), cps, freq, n_cycles);
  let mod_index_env: Vec<f32> = mod_index_env.iter().map(|x| x.powi(8i32)).collect();
  Envelope::from_samples(&mod_index_env)
}

/// Generates a random modulator with an envelope.
pub fn random_envelope(cps: f32, n_cycles: f32, freq: f32) -> Envelope {
  let mut rng = rand::thread_rng();
  let mod_index_env = if rng.gen::<f32>() < 0.5f32 {
    ranger::eval_knob_mod(random_modulation_function(), &random_knob(), cps, freq, n_cycles)
  } else {
    gen_organic_amplitude(10, ((n_cycles * SRf) as f32 * cps) as usize, Visibility::Visible)
  };
  let mod_index_env: Vec<f32> = mod_index_env.iter().map(|x| x.powi(8i32)).collect();
  Envelope::from_samples(&mod_index_env)
}

/// Generates multiple random modulators with envelopes.
pub fn generate_modulators_with_envelopes(num_modulators: usize, cps: f32, n_cycles: f32, fund: f32) -> Vec<Operator> {
  (0..num_modulators).map(|_| random_modulator(cps, n_cycles, fund, 4)).collect()
}
/// Generates multiple random modulators with envelopes.
pub fn generate_envelopes(num_modulators: usize, cps: f32, n_cycles: f32, fund: f32) -> Vec<Envelope> {
  (0..num_modulators).map(|_| random_envelope(cps, n_cycles, fund)).collect()
}

pub fn random_modulator_with_envelope(cps: f32, n_cycles: f32, base_freq: f32, depth: usize) -> Operator {
  let mut rng = rand::thread_rng();

  // Random modulator frequency
  let mod_freq = base_freq * rng.gen_range(1..12) as f32;

  // Create a random envelope for modulation index
  let mod_index_env = vec![
    ranger::eval_knob_mod(
      random_modulation_function(),
      &random_knob(),
      cps,
      mod_freq,
      n_cycles / 3.0,
    ),
    ranger::eval_knob_mod(
      random_modulation_function(),
      &random_knob(),
      cps,
      mod_freq,
      n_cycles / 3.0,
    ),
    ranger::eval_knob_mod(
      random_modulation_function(),
      &random_knob(),
      cps,
      mod_freq,
      n_cycles / 3.0,
    ),
  ]
  .concat();

  // Recursively attach nested modulators if depth > 0
  let modulators = if depth > 0 {
    vec![ModulationSource::Operator(random_modulator_with_envelope(
      cps,
      n_cycles,
      base_freq,
      depth - 1,
    ))]
  } else {
    vec![]
  };

  // Return the modulator
  Operator {
    frequency: mod_freq,
    modulation_index: rng.gen_range(0.5..2.0), // Random modulation index
    modulators,
    mod_index_env_mul: Envelope::from_samples(&mod_index_env),
    ..Default::default()
  }
}
