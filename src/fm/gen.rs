use super::*;
use crate::{Arf, Conf, Energy, Melody, Mode, Note, Presence, Role};
use rand::distributions::Uniform;
use rand::{distributions::Distribution, thread_rng, Rng};

/// ---------------------------------------------------------------------------
/// 2. Abstract frequency relationships
/// ---------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub enum FreqRelation {
  /// A fixed frequency (Hz).
  Fixed(f32),
  /// A ratio applied to the parent’s frequency.
  Ratio(f32),
  /// Inversion of some sub-relation => 1 / freq.
  Invert(Box<FreqRelation>),
}

/// ---------------------------------------------------------------------------
/// 3. Lifetime: We separate shape (burp, pluck, fall, rise) from
///    a float `lifetime_value` in [0..1].
/// ---------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub enum LifetimeSpec {
  /// Terminal "burp" shape
  Burp,
  /// Terminal "pluck" shape
  Pluck,
  /// Infinite "fall"
  Fall,
  /// Infinite "rise"
  Rise,
}

/// ---------------------------------------------------------------------------
/// 4. OperatorSpec: structure + mod_index + lifetime_value + shape
/// ---------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct OperatorSpec {
  pub freq_relation: FreqRelation,
  /// Baseline modulation index
  pub mod_index: f32,
  /// Float in [0..1]. 0 => very short, 1 => infinite sustain.
  /// You can interpret it as a rough timescale factor.
  pub lifetime_value: f32,
  /// Shape or “curve style” for the envelope
  pub lifetime_shape: LifetimeSpec,

  /// Child modulators
  pub modulators: Vec<OperatorSpec>,
}

impl OperatorSpec {
  /// Recursively convert this OperatorSpec (which now has a `lifetime_value` + `lifetime_shape`)
  /// into a concrete `Operator`.
  pub fn to_operator(&self, parent_freq: f32) -> Operator {
    // 1) Resolve frequency
    let my_freq = match &self.freq_relation {
      FreqRelation::Fixed(f) => *f,
      FreqRelation::Ratio(r) => parent_freq * r,
      FreqRelation::Invert(inner) => {
        let base_freq = match &**inner {
          FreqRelation::Fixed(f) => *f,
          FreqRelation::Ratio(r2) => parent_freq * r2,
          FreqRelation::Invert(_) => 1.0, // keep it simple
        };
        1.0 / base_freq.max(0.0001)
      }
    };

    // 2) Build an envelope from the shape + lifetime_value
    //    Here we put it in mod_index_env_mul, but you could also use mod_gain_env_mul, etc.
    let mod_index_env_mul = build_envelope_from_lifetime(self.lifetime_shape.clone(), self.lifetime_value);

    // 3) Recurse on children
    let child_ops = self
      .modulators
      .iter()
      .map(|child| child.to_operator(my_freq))
      .map(ModulationSource::Operator)
      .collect::<Vec<_>>();

    // 4) Construct final Operator
    Operator {
      frequency: my_freq,
      modulation_index: self.mod_index,
      modulators: child_ops,
      mod_index_env_mul,
      ..Operator::default()
    }
  }
}

/// ---------------------------------------------------------------------------
/// 5. Envelope generation from shape + lifetime_value
///    This is just a demonstration using a 512-sample buffer. Tweak as needed.
/// ---------------------------------------------------------------------------
pub fn build_envelope_from_lifetime(shape: LifetimeSpec, lf: f32) -> Envelope {
  // The `lf` parameter in [0..1] can define how long or short the envelope is.
  // For example:
  //   0.0 => ~60 ms or so
  //   0.5 => ~some moderate length
  //   1.0 => infinite sustain
  //
  // We’ll just produce a 512-sample buffer, but scale the shape accordingly.
  let sample_count = 512;
  let mut samples = Vec::with_capacity(sample_count);

  // approximate "time scale" factor
  // e.g. lf=0 => fast decay, lf=1 => never decays
  // Below is just an example function, do any mapping you like.
  let decay_factor = if lf >= 0.99 {
    // Essentially infinite
    9999.0
  } else {
    // Map lf in [0..1] => [0.06..2.0] seconds for demonstration
    // so 0 => 0.06, 1 => 2.0
    0.06 + 1.94 * lf
  };

  // Use shape to define the curve. A typical approach is:
  //   for i in 0..sample_count => t = i/(sample_count-1) => envelope = ...
  //   but you incorporate "decay_factor" into the formula somehow.

  match shape {
    LifetimeSpec::Burp => {
      // e.g. x^(1/2) from x=0..1, then we accelerate the time based on `decay_factor`
      for i in 0..sample_count {
        let time_norm = i as f32 / (sample_count as f32 - 1.0);
        // accelerate or compress time
        let scaled_t = (time_norm / decay_factor).min(1.0);
        let val = scaled_t.sqrt();
        samples.push(val);
      }
    }
    LifetimeSpec::Pluck => {
      // e.g. e^(-5 * t), scaled by decay_factor
      for i in 0..sample_count {
        let time_norm = i as f32 / (sample_count as f32 - 1.0);
        let scaled_t = time_norm * 5.0 * (1.0 / decay_factor);
        // shorter => bigger exponent => faster decay
        let val = (-scaled_t).exp();
        samples.push(val);
      }
    }
    LifetimeSpec::Fall => {
      // “Infinite fall”: never fully zero. e.g. from 1.0 down to 0.5
      // If lf is near 1, the fall is extremely slow. If lf=0 => quick fall.
      let floor = 0.5;
      for i in 0..sample_count {
        let time_norm = i as f32 / (sample_count as f32 - 1.0);
        // scale the time => if decay_factor is large => slower descent
        let scaled_t = time_norm / decay_factor;
        let val = 1.0 - (scaled_t.min(1.0)) * (1.0 - floor);
        samples.push(val);
      }
    }
    LifetimeSpec::Rise => {
      // “Infinite rise”: from 0.7 to 1.0
      for i in 0..sample_count {
        let time_norm = i as f32 / (sample_count as f32 - 1.0);
        let scaled_t = time_norm / decay_factor;
        let val = 0.7 + 0.3 * scaled_t.min(1.0);
        samples.push(val);
      }
    }
  }

  Envelope::SampleBased { samples }
}

/// ---------------------------------------------------------------------------
/// 6. A function that assigns lifetime_value in [0..1] to each node,
///    after the structure is built. For example, deeper nodes are
///    more likely to be short => small lifetime_value.
/// ---------------------------------------------------------------------------
pub fn assign_lifetimes(root: &mut OperatorSpec, depth: usize, max_depth: usize, rng: &mut impl Rng) {
  // Example heuristic: deeper => more likely short.
  // roll = random in [0..1]
  let roll = rng.gen::<f32>();
  // p_short => fraction that depends on depth
  let p_short = (depth as f32 / max_depth as f32).min(1.0);

  // If roll < p_short => small lifetime => ~0..0.3
  // else => ~0.3..1.0
  let new_lf = if roll < p_short {
    rng.gen_range(0.0..0.3)
  } else {
    rng.gen_range(0.3..1.0)
  };
  root.lifetime_value = new_lf;

  // Recurse for children
  for child in &mut root.modulators {
    assign_lifetimes(child, depth + 1, max_depth, rng);
  }
}

/// ---------------------------------------------------------------------------
/// 7. You can still define simpler “random_operator_spec” to pick
///    freq relations, shapes, etc. Then run `assign_lifetimes` as a second pass.
/// ---------------------------------------------------------------------------
pub fn random_operator_structure(depth: usize, max_depth: usize) -> OperatorSpec {
  let mut rng = thread_rng();

  // Example random freq relation in [0.5..2.0].
  let freq_rel = FreqRelation::Ratio(rng.gen_range(0.5..2.0));
  // Example random mod_index
  let mod_idx = rng.gen_range(0.5..3.0);
  // For now, default lifetime_value=1.0 => we’ll adjust later
  let lf_val = 1.0;
  // Random shape
  let shapes = [
    LifetimeSpec::Burp,
    LifetimeSpec::Pluck,
    LifetimeSpec::Fall,
    LifetimeSpec::Rise,
  ];
  let shape = shapes[rng.gen_range(0..shapes.len())].clone();

  // Build children if not at max depth
  let mut children = vec![];
  if depth < max_depth {
    let child_count = rng.gen_range(0..=2);
    for _ in 0..child_count {
      children.push(random_operator_structure(depth + 1, max_depth));
    }
  }

  OperatorSpec {
    freq_relation: freq_rel,
    mod_index: mod_idx,
    lifetime_value: lf_val,
    lifetime_shape: shape,
    modulators: children,
  }
}

/// ---------------------------------------------------------------------------
/// 8. Demo test
/// ---------------------------------------------------------------------------
#[test]
fn test_lifetimes() {
  let mut root_spec = random_operator_structure(0, 3);
  let mut rng = thread_rng();

  // In a second pass, set each node’s lifetime_value
  assign_lifetimes(&mut root_spec, 0, 3, &mut rng);

  // Now convert to an actual Operator at e.g. 220 Hz
  let op_220 = root_spec.to_operator(220.0);
  println!("Operator @220Hz => {:#?}", op_220);

  // Retune to 2000 Hz
  let op_2k = root_spec.to_operator(2000.0);
  println!("Operator @2000Hz => {:#?}", op_2k);
}

/// ---------------------------------------------------------------------------
/// 9. If you have existing “ModSpec” logic, you can still map it to
///    an OperatorSpec with `lifetime_value=1.0` or something, then
///    call `assign_lifetimes` afterwards. For brevity, not repeated here.
/// ---------------------------------------------------------------------------

/// ---------------------------------------------------------------------------
/// 10. Usage in main
/// ---------------------------------------------------------------------------
fn main() {
  let mut rng = thread_rng();

  // Step A: Build an operator tree structure (freq relations, shapes, etc.), ignoring lifetimes.
  let mut spec = random_operator_structure(0, 2);

  // Step B: Post-process to assign lifetime_value in [0..1], deeper => more likely short
  assign_lifetimes(&mut spec, 0, 2, &mut rng);

  // Step C: Convert to a final Operator at your chosen carrier freq
  let root_op = spec.to_operator(440.0);
  println!("Final operator @440Hz:\n{:#?}", root_op);
}

/// Picks the next modulator frequency (and whether to invert) based on the current `carf`.
/// Uses a global `NFf` for Nyquist-based clamping.  
///
/// - If `carf < 100 Hz`, picks a fixed freq in [1..12].
/// - If `100 <= carf < 400`, 75% chance fixed in [40..300], else ratio up to 3×, maybe invert.
/// - If `400 <= carf < 2000`, half fixed in [50..1200], half ratio up to 3×, ~50% chance invert.
/// - If `2000 <= carf < 12000`, ratio in [1..10], always invert.
/// - If `carf >= 12000`, ratio in [1..2], always invert.
///
/// After picking the frequency, we:
///   - Apply inversion if needed.
///   - Apply a ratio-based clamp: freq <= carf * 4.0
///   - Apply an absolute clamp to the global `NFf`.
pub fn pick_mod_frequency(carf: f32) -> (f32, bool) {
  let mut rng = thread_rng();
  let (raw_freq, invert) = match carf {
    f if f < 100.0 => {
      let fixed_range = Uniform::new(1.0, 12.0);
      (fixed_range.sample(&mut rng), false)
    }
    f if f < 400.0 => {
      if rng.gen_bool(0.75) {
        let fixed_range = Uniform::new(40.0, 300.0);
        (fixed_range.sample(&mut rng), rng.gen_bool(0.4))
      } else {
        let ratio = Uniform::new(1.0, 3.0).sample(&mut rng);
        (f * ratio, rng.gen_bool(0.5))
      }
    }
    f if f < 2000.0 => {
      if rng.gen_bool(0.5) {
        let fixed_range = Uniform::new(50.0, 1200.0);
        (fixed_range.sample(&mut rng), rng.gen_bool(0.5))
      } else {
        let ratio = Uniform::new(1.0, 3.0).sample(&mut rng);
        (f * ratio, rng.gen_bool(0.5))
      }
    }
    f if f < 12000.0 => {
      let ratio = Uniform::new(1.0, 10.0).sample(&mut rng);
      (f * ratio, true)
    }
    f => {
      let ratio = Uniform::new(12.0, 42.0).sample(&mut rng);
      (f * ratio, true)
    }
  };

  let clamped_freq = raw_freq
        .max(0.0001)                // Avoid zero or negative frequencies
        .min(carf * 4.0)            // Ratio clamp
        .min(NFf); // Absolute Nyquist clamp

  println!(
    "[DEBUG] pick_mod_frequency: carf={:.2}, raw_freq={:.2}, clamped_freq={:.2}, invert={}",
    carf, raw_freq, clamped_freq, invert
  );

  (clamped_freq, invert)
}

/// Recursively constructs an `Operator` by picking random frequencies for children
/// up to `max_depth`.  
///
/// If a child's bandwidth check fails, we skip that child.
pub fn build_operator(carf: f32, depth: usize, max_depth: usize, is_first_order: bool, child_index: usize) -> Operator {
  let mut rng = thread_rng();

  // Base operator
  let mut op = Operator {
    frequency: carf,
    modulation_index: rng.gen_range(0.5..2.5),
    ..Operator::default()
  };

  // Decide how many children
  let max_children = match carf {
    f if f < 100.0 => 3,
    f if f < 400.0 => 4,
    f if f < 2000.0 => 2,
    f if f < 12000.0 => 2,
    _ => 1,
  };

  if depth >= max_depth {
    return op;
  }

  let child_count = rng.gen_range(0..=max_children);
  let mut modulators = Vec::new();

  for i in 0..child_count {
    // 1. pick freq + invert
    let (mut freq, suggested_invert) = pick_mod_frequency(carf);

    // every 2nd first-order child => forced invert
    let invert = if is_first_order && (child_index + i) % 2 == 1 {
      true
    } else {
      suggested_invert
    };

    if invert {
      freq = 1.0 / freq.max(0.0001);
    } else {
      freq = freq.max(0.0001);
    }

    // 2. build the child operator
    let child_op = build_operator(
      freq,
      depth + 1,
      max_depth,
      false, // only depth=0 => first_order
      0,
    );

    // 3. run bandwidth check. If it fails, skip
    match get_remaining_range(&child_op, 0f32, 0f32, crate::synth::MFf, NFf) {
      Some((_low, _high)) => {
        // all good => add
        modulators.push(ModulationSource::Operator(child_op));
      }
      None => {
        // bandwidth exceeded => skip
        eprintln!("Skipping child due to exceeded bandwidth (freq={freq} at depth={depth}).");
      }
    }
  }

  op.modulators = modulators;
  op
}

/// Top-level convenience function to build a random operator tree
/// at `carf` with up to `max_depth`.
pub fn generate_top_operator(carf: f32, max_depth: usize) -> Operator {
  let mut rng = thread_rng();

  // Root operator
  let mut root = Operator {
    frequency: carf,
    modulation_index: 0.0,
    ..Operator::default()
  };

  // top children in [0..3]
  let top_children = rng.gen_range(0..=3);
  let mut top_modulators = Vec::new();

  for i in 0..top_children {
    // pick freq
    let (mut next_freq, _) = pick_mod_frequency(carf);
    next_freq = next_freq.max(0.0001);

    let child_op = build_operator(
      next_freq, 1, max_depth, true, // is_first_order
      i,
    );

    // optionally, do a bandwidth check on the child op
    match get_remaining_range(&child_op, 0f32, 0f32, crate::synth::MFf, NFf) {
      Some(_) => {
        top_modulators.push(ModulationSource::Operator(child_op));
      }
      None => {
        eprintln!("Skipping a top-level child => bandwidth exceeded at freq={next_freq}.");
      }
    }
  }

  root.modulators = top_modulators;
  root
}

#[test]
fn test_build_operator() {
  // Step 1: build a top-level OperatorSpec
  let mut root_spec = random_operator_structure(0, 3);

  // Step 2: assign lifetimes
  let mut rng = thread_rng();
  assign_lifetimes(&mut root_spec, 0, 3, &mut rng);

  // Step 3: Convert to an Operator at 220.0 Hz
  let op_220 = root_spec.to_operator(220.0);

  // Step 4: Possibly do a bandwidth check or just print
  match get_remaining_range(&op_220, 0f32, 0f32, crate::synth::MFf, NFf) {
    Some((low, high)) => {
      println!("OperatorSpec => Operator @220Hz: range [{low}..{high}] => {op_220:#?}");
    }
    None => {
      eprintln!("Bandwith exceeded => using partial operator anyway.");
    }
  }
}

/// Recursively prunes the deepest modulator from an `Operator`.
///
/// This function identifies the deepest `Operator` in the modulation tree and removes it.
/// If there are multiple modulators at the same depth, it removes the first one encountered.
///
/// # Arguments
///
/// * `op` - The root `Operator` to prune.
///
/// # Returns
///
/// A new `Operator` with the deepest modulator removed. If no modulators are present, the function
/// returns a simplified sine operator (modulators list empty, modulation index set to zero).
///
/// # Behavior
///
/// - If the `Operator` has no modulators, it is returned unchanged with modulation index zero.
/// - The traversal dynamically calculates the depth of each modulator subtree.
/// - The `modulators` list is rebuilt at each recursion level, excluding the deepest modulator.
///
/// # Example
///
/// ```rust
/// let op = Operator {
///     frequency: 440.0,
///     modulation_index: 1.5,
///     modulators: vec![
///         ModulationSource::Operator(Operator {
///             frequency: 220.0,
///             modulation_index: 1.0,
///             modulators: vec![
///                 ModulationSource::Operator(Operator {
///                     frequency: 110.0,
///                     modulation_index: 0.5,
///                     modulators: vec![],
///                     ..Operator::default()
///                 })
///             ],
///             ..Operator::default()
///         })
///     ],
///     ..Operator::default()
/// };
///
/// let pruned_op = prune_operator(op);
/// ```
pub fn prune_operator(op: Operator) -> Operator {
  // If the operator has no modulators, return it as a simplified sine operator
  if op.modulators.is_empty() {
    return Operator {
      frequency: op.frequency,
      modulation_index: 0.0,
      ..Operator::default()
    };
  }

  // Variables to track the deepest modulator and its index
  let mut max_depth = 0;
  let mut deepest_index = None;

  // Iterate through the modulators to calculate depth
  for (i, modulator) in op.modulators.iter().enumerate() {
    if let ModulationSource::Operator(inner_op) = modulator {
      let depth = calculate_depth(inner_op);
      if depth > max_depth {
        max_depth = depth;
        deepest_index = Some(i);
      }
    }
  }

  // If a deepest modulator was found, prune it
  if let Some(idx) = deepest_index {
    let mut new_modulators = op.modulators.clone();
    if let ModulationSource::Operator(inner_op) = &new_modulators[idx] {
      new_modulators[idx] = ModulationSource::Operator(prune_operator(inner_op.clone()));
    }
    return Operator {
      modulators: new_modulators,
      ..op
    };
  }

  // If no deep modulators exist, return the operator unchanged
  op
}

/// Calculates the depth of an `Operator` tree.
///
/// This helper function determines the depth of the deepest modulation chain.
///
/// # Arguments
///
/// * `op` - The `Operator` to evaluate.
///
/// # Returns
///
/// The depth of the deepest modulator. A depth of `0` means no modulators.
fn calculate_depth(op: &Operator) -> usize {
  if op.modulators.is_empty() {
    return 0;
  }
  op.modulators
    .iter()
    .filter_map(|modulator| {
      if let ModulationSource::Operator(inner_op) = modulator {
        Some(calculate_depth(inner_op))
      } else {
        None
      }
    })
    .max()
    .unwrap_or(0)
    + 1
}

#[test]
fn test_my_rendered_synth() {
  // 1) Melody configuration with offset_register
  let offset_register = 1;
  let melody: Melody<Note> = vec![vec![
    ((3, 2), (offset_register + 6, (1, 0, 3)), 1.0),
    ((3, 2), (offset_register + 6, (1, 0, 1)), 1.0),
    ((2, 2), (offset_register + 6, (1, 0, 5)), 1.0),
    ((3, 2), (offset_register + 7, (1, 0, 3)), 1.0),
    ((3, 2), (offset_register + 7, (1, 0, 1)), 1.0),
    ((2, 2), (offset_register + 7, (1, 0, 5)), 1.0),
    ((3, 2), (offset_register + 8, (1, 0, 3)), 1.0),
    ((3, 2), (offset_register + 8, (1, 0, 1)), 1.0),
    ((2, 2), (offset_register + 8, (1, 0, 5)), 1.0),
    ((2, 2), (offset_register + 9, (1, 0, 3)), 1.0),
    ((2, 2), (offset_register + 8, (1, 0, 5)), 1.0),
    ((2, 2), (offset_register + 7, (1, 0, 1)), 1.0),
    ((2, 2), (offset_register + 6, (1, 0, 3)), 1.0),
  ]];

  // 2) Config and ARF setup
  let conf = Conf { cps: 1.5, root: 1.23 };
  let arf = Arf {
    mode: Mode::Melodic,
    role: Role::Chords,
    register: 6,
    visibility: Visibility::Foreground,
    energy: Energy::High,
    presence: Presence::Legato,
  };

  // 3) Generate a single OperatorSpec for the entire melody
  let mut root_spec = random_operator_structure(0, 2);
  let mut rng = rand::thread_rng();
  assign_lifetimes(&mut root_spec, 0, 2, &mut rng);

  // Convert the OperatorSpec to an Operator at an initial frequency
  let mut root_operator = root_spec.to_operator(440.0);

  // 4) Initialize the audio signal buffer
  let mut signal: Vec<f32> = Vec::new();

  // 5) Render the melody using the single operator
  for (i, note) in melody[0].iter().enumerate() {
    let freq = note_to_freq(note);

    // Adjust the operator's frequency for the current note
    let mut op = Operator {
      frequency: freq,
      ..root_operator.clone()
    };

    // 5a) Bandwidth check and pruning loop
    while let None = get_remaining_range(&op, 0f32, 0f32, crate::synth::MFf, NFf) {
      eprintln!("Note {} @{}Hz: Bandwidth exceeded. Pruning operator.", i, freq);
      op = prune_operator(op.clone());
    }

    // Log the successful rendering
    println!("Note {} @{}Hz: Operator valid. Rendering...", i, freq);

    // 5b) Render the samples for note duration
    let dur_cycles = crate::time::note_to_cycles(note); // duration in cycles
    let samples = render_operators(
      vec![op],
      dur_cycles,
      conf.cps, // control rate
      SR,       // sample rate
    );

    signal.extend(samples);
  }

  // 6) Write audio signal to a file
  if !signal.is_empty() {
    crate::render::engrave::samples(SR, &signal, "dev-audio/test-melody-single-operator.wav");
    println!(
      "Successfully wrote {} samples to dev-audio/test-melody-single-operator.wav",
      signal.len()
    );
  } else {
    eprintln!("No audio generated; all operators were skipped.");
  }
}
