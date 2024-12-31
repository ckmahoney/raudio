use super::*;

/// A rich bass synth evoking distorted bassoon.  
///
/// Uses Algorithm 18 in DX-7 operator configuration.
/// A18 { { 2 + [3 > 3] + 6 > 5 > 4 } > 1 }
/// op2 provides the basis of the bass woodwind timbre
/// op3 offers the upper body of the woodwind timbre
/// op4 gives it the bite on attack
pub fn dexed_bassoon(p: f32, n_cycles: f32, cps: f32, freq: f32, mod_gain: f32) -> Vec<Operator> {
  let mut rng = thread_rng();

  let op3_detune_cents = get_dexed_detune(freq, 2);
  let op5_detune_cents = get_dexed_detune(freq, -2);
  let op6_detune_cents = get_dexed_detune(freq, 3);
  let max_fmod_mul = 2f32;

  let op_frequency = freq / max_fmod_mul;

  let mut op2 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_burp,
        &Knob {
          a: 1f32,
          b: 1f32,
          c: 0f32,
        },
        cps,
        freq,
        4f32,
      ),
    },
    ..Operator::modulator(op_frequency * 1.0f32, mod_gain * dx_to_mod_index(93.0))
  };

  let op3 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: eval_odr_level(
        cps,
        n_cycles,
        &LevelMacro {
          stable: [0.8, 0.9f32],
          peak: [0.98, 1f32],
          sustain: [0.95, 0.9f32],
        },
        &ODRMacro {
          onset: [20.0, 30.0],
          decay: [150.0, 200.0],
          release: [1500.0, 2100.0],
          mo: vec![MacroMotion::Constant],
          md: vec![MacroMotion::Constant],
          mr: vec![MacroMotion::Constant],
        },
      ),
    },
    modulators: vec![ModulationSource::Feedback(0.5)],
    ..Operator::modulator(
      op_frequency * 1.0f32 + op3_detune_cents,
      mod_gain * dx_to_mod_index(81.0),
    )
  };

  let op6 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: eval_odr_level(
        cps,
        n_cycles,
        &LevelMacro {
          stable: [0.1, 0.1f32],
          peak: [0.98, 1f32],
          sustain: [0.95, 0.995f32],
        },
        &ODRMacro {
          onset: [20.0, 50.0],
          decay: [500.0, 1200.0],
          release: [30.0, 50.0],
          mo: vec![MacroMotion::Constant],
          md: vec![MacroMotion::Constant],
          mr: vec![MacroMotion::Constant],
        },
      ),
    },
    ..Operator::modulator(op_frequency * 2.0f32 + op6_detune_cents, dx_to_mod_index(99.0))
  };

  let op5 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: eval_odr_level(
        cps,
        n_cycles,
        &LevelMacro {
          stable: [0.3, 0.4f32],
          peak: [0.98, 1f32],
          sustain: [0.95, 0.995f32],
        },
        &ODRMacro {
          onset: [10.0, 30.0],
          decay: [1500.0, 2200.0],
          release: [30.0, 50.0],
          mo: vec![MacroMotion::Constant],
          md: vec![MacroMotion::Constant],
          mr: vec![MacroMotion::Constant],
        },
      ),
    },
    modulators: single_modulator(op6),
    ..Operator::modulator(op_frequency * 2.0f32 + op5_detune_cents, dx_to_mod_index(57.0))
  };

  // burst of harmonics on note entry
  let op4 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: eval_odr_level(
        cps,
        n_cycles,
        &LevelMacro {
          stable: [1.0, 1f32],
          peak: [1.0, 1f32],
          sustain: [0.65, 0.75f32],
        },
        &ODRMacro {
          onset: [20.0, 50.0],
          decay: [1500.0, 3200.0],
          release: [30.0, 100.0],
          mo: vec![MacroMotion::Constant],
          md: vec![MacroMotion::Constant],
          mr: vec![MacroMotion::Constant],
        },
      ),
    },
    modulators: single_modulator(op5),
    ..Operator::modulator(op_frequency * 1.0f32, mod_gain * dx_to_mod_index(75.0))
  };

  let op1 = Operator {
    modulators: vec![
      ModulationSource::Operator(op2),
      ModulationSource::Operator(op3),
      ModulationSource::Operator(op4),
    ],
    ..Operator::carrier(freq)
  };

  vec![op1]
}

/// Represents Algorithm 18 in DX-7 operator configuration.
/// A18 { { 2 + [3 > 3] + { 6 > 5 > 4 } } > 1 }
/// mod op2
/// mod op3
/// mod op4
pub fn dexed_brass(p: f32, n_cycles: f32, cps: f32, freq: f32, mod_gain: f32) -> Vec<Operator> {
  let mut rng = thread_rng();

  let op2_detune_cents = get_dexed_detune(freq, -6);
  let op3_detune_cents = get_dexed_detune(freq, 7);
  let op4_detune_cents = get_dexed_detune(freq, 2);
  let op5_detune_cents = get_dexed_detune(freq, -2);

  let max_fmod_mul = 2f32;

  let op_frequency = freq / max_fmod_mul;

  let mut op2 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.25f32,
          b: 0.5f32,
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
          a: 0.25f32,
          b: 0.15f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles / 2f32,
      ),
    },
    ..Operator::modulator(
      op_frequency * 2f32.powi(0) + op2_detune_cents,
      mod_gain * dx_to_mod_index(93.0),
    )
  };

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

  let op3 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: fadein.clone(),
    },
    mod_index_env_mul: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.25f32,
          b: 0.15f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    modulators: vec![ModulationSource::Feedback(0.95)],
    ..Operator::modulator(
      op_frequency * 2f32.powi(0) + op3_detune_cents,
      mod_gain * dx_to_mod_index(81.0),
    )
  };

  let op6 = Operator {
    mod_index_env_sum: Envelope::SampleBased { samples: fadein },
    mod_index_env_mul: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.19f32,
          b: 0.75f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    ..Operator::modulator(op_frequency * 2f32.powi(0), mod_gain * dx_to_mod_index(99.0))
  };

  let op5 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.5f32,
          b: 0f32,
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
          a: 0.5f32,
          b: 0.85f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    modulators: single_modulator(op6),
    ..Operator::modulator(
      op_frequency * 2f32.powi(-1) + op5_detune_cents,
      mod_gain * dx_to_mod_index(57.0),
    )
  };

  // burst of harmonics on note entry
  let op4 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: eval_odr_level(
        cps,
        n_cycles,
        &LevelMacro {
          stable: [1.0, 1f32],
          peak: [1.0, 1f32],
          sustain: [0.65, 0.75f32],
        },
        &ODRMacro {
          onset: [20.0, 50.0],
          decay: [500.0, 1200.0],
          release: [30.0, 100.0],
          mo: vec![MacroMotion::Constant],
          md: vec![MacroMotion::Constant],
          mr: vec![MacroMotion::Constant],
        },
      ),
    },
    mod_index_env_mul: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.7f32,
          b: 0.75f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    modulators: single_modulator(op5),
    ..Operator::modulator(
      op_frequency * 2f32.powi(0) + op4_detune_cents,
      mod_gain * dx_to_mod_index(75.0),
    )
  };

  let op1 = Operator {
    modulators: vec![
      ModulationSource::Operator(op2),
      ModulationSource::Operator(op3),
      ModulationSource::Operator(op4),
    ],
    mod_gain_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.85f32,
          b: 0.25f32,
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
          a: 0.75f32,
          b: 0.75f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    ..Operator::carrier(freq)
  };

  vec![op1]
}

/// Represents Algorithm 9 in DX-7 operator configuration for a synth pad
/// A9 { [2 > 2] > 1 + { 4 + 6 > 5 } > 3 }
pub fn dexed_pad(p: f32, n_cycles: f32, cps: f32, freq: f32, mod_gain: f32) -> Vec<Operator> {
  let mut rng = thread_rng();

  let op1_detune_cents = get_dexed_detune(freq, 7);
  let op2_detune_cents = get_dexed_detune(freq, 5);
  let op3_detune_cents = get_dexed_detune(freq, 1);
  let op6_detune_cents = get_dexed_detune(freq, 7);

  let max_fmod_mul = 17.38f32;

  let op_frequency = freq / max_fmod_mul;
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
    ..Operator::modulator(
      op_frequency * 2f32.powi(-1) + op2_detune_cents,
      mod_gain * dx_to_mod_index(77.0),
    )
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
    ..Operator::carrier(op_frequency * 2f32.powi(0) + op1_detune_cents)
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
    ..Operator::modulator(
      op_frequency * 10f32 + op6_detune_cents,
      mod_gain * dx_to_mod_index(86.0),
    )
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

/// Represents Algorithm 2 in DX-7 operator configuration.
/// First carrier has a static LFO and is modulated by a feedback loop.
/// Second carrier is octave with rich modulation.
/// [2 -> 2] -> 1
/// 6 -> 5 -> 4 -> 3
pub fn dexed_mushstring(n_cycles: f32, cps: f32, base_frequency: f32, const_a: f32) -> Vec<Operator> {
  let mut rng = thread_rng();

  let op2_detune_cents = -2f32 * base_frequency.log2() / 15f32;
  let op3_detune_cents = 2f32 * base_frequency.log2() / 15f32;
  let op4_detune_cents = -3f32 * base_frequency.log2() / 15f32;
  let op6_detune_cents = -2.5f32 * base_frequency.log2() / 15f32;

  let base_frequency = base_frequency / 8f32;

  // Carrier 1 (op1) and its modulator (op2)

  let knob: Knob = Knob {
    a: in_range(&mut rng, 0f32, 0.125f32),
    b: 0.75f32,
    c: 1.0f32,
  };
  let onset1 = ranger::eval_knob_mod(ranger::amod_cycle_fadein_1_4, &knob, cps, 1f32, 4f32);
  let onset2 = ranger::eval_knob_mod(
    ranger::amod_cycle_fadein_1_4,
    &Knob {
      a: in_range(&mut rng, 0f32, 0.0125f32),
      ..knob
    },
    cps,
    1f32,
    4f32,
  );

  let env_1 = mul_envelopes(
    onset1,
    ranger::eval_knob_mod(
      ranger::amod_fadeout,
      &Knob {
        a: in_range(&mut rng, 0.7f32, 1f32),
        b: 1f32,
        c: 0f32,
      },
      cps,
      1f32,
      4f32,
    ),
    true,
  );

  let op2 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_burp,
        &Knob {
          a: 1f32,
          b: 1f32,
          c: 0f32,
        },
        cps,
        1f32,
        4f32,
      ),
    },
    modulators: vec![ModulationSource::Feedback(0.5f32)],
    ..Operator::modulator(2.02f32 + op2_detune_cents, dx_to_mod_index(50.0))
  };
  let op1 = Operator {
    modulators: single_modulator(op2),
    mod_gain_env_mul: Envelope::SampleBased { samples: (env_1) },
    ..Operator::carrier(const_a)
  };

  // Carrier 3 (op1) and its cascde of modulators (op2)
  let op6 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_burp,
        &Knob {
          a: 1f32,
          b: 1f32,
          c: 0f32,
        },
        cps,
        1f32,
        4f32,
      ),
    },
    ..Operator::modulator(
      op6_detune_cents + base_frequency * 4f32,
      dx_to_mod_index(in_range(&mut rng, 30.0, 80.0)),
    )
  };
  let op5 = Operator {
    modulators: single_modulator(op6),
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_burp,
        &Knob {
          a: 1f32,
          b: 1f32,
          c: 0f32,
        },
        cps,
        1f32,
        4f32,
      ),
    },
    ..Operator::modulator(base_frequency * 8f32, dx_to_mod_index(in_range(&mut rng, 30.0, 80.0)))
  };
  let op4 = Operator {
    modulators: single_modulator(op5),
    mod_gain_env_mul: Envelope::SampleBased {
      samples: gen_organic_amplitude(10, ((n_cycles * SRf) as f32 * cps) as usize, Visibility::Visible),
    },
    ..Operator::modulator(
      op4_detune_cents + base_frequency * 2f32,
      dx_to_mod_index(in_range(&mut rng, 70.0, 80.0)),
    )
  };
  let onset3 = mul_envelopes(
    onset2,
    ranger::eval_knob_mod(
      ranger::amod_fadeout,
      &Knob {
        a: in_range(&mut rng, 0.7f32, 1f32),
        b: 1f32,
        c: 0f32,
      },
      cps,
      1f32,
      4f32,
    ),
    true,
  );

  let op3 = Operator {
    modulators: single_modulator(op4),
    mod_gain_env_mul: Envelope::SampleBased { samples: onset3 },
    ..Operator::carrier(op3_detune_cents + base_frequency * 2.0)
  };
  vec![op1, op3]
}

/// applies what I know about the tune knob
pub fn get_dexed_detune(base_frequency: f32, tune: i32) -> f32 {
  let v = tune as f32 / 7i32 as f32; // range of knob in dexed is -7 to 7
  v * base_frequency.log2() / pi
}

mod tests {
  use super::*;

  #[test]
  fn test_dx_string() {
    let freqs: Vec<f32> = (0..12).map(|i| 330f32 + 330f32 * 2f32.powf(i as f32 / 12f32)).collect();
    let mut melody: Vec<f32> = vec![];
    let c: f32 = 1.62181;
    let n_cycles = 3f32;
    let cps: f32 = 1.5f32;
    for carrier_frequency in &freqs {
      let modulator_playback_rate = 1.0;

      let operators = dexed_mushstring(n_cycles, cps, *carrier_frequency, c);
      let mut signal = render_operators(operators, n_cycles, cps, SR);

      assert!(!signal.is_empty());
      melody.extend(signal)
    }

    let filename = format!("dev-audio/test-dx-strings");
    engrave::samples(SR, &melody, &filename);
  }

  #[test]
  fn test_dx_bassoon() {
    let label = "dx-bass";
    let start_freq = 333f32 / 8f32;
    let freqs: Vec<f32> = (0..12).map(|i| start_freq + start_freq * 2f32.powf(i as f32 / 12f32)).collect();
    let mut melody: Vec<f32> = vec![];
    let c: f32 = 1.62181;
    let n_cycles = 3f32;
    let cps: f32 = 1.5f32;

    let l = freqs.len() as f32;
    for (i, carrier_frequency) in freqs.iter().enumerate() {
      let p = i as f32 * l / l;
      let modulator_playback_rate = 1.0;
      let operators = dexed_bassoon(p, n_cycles, cps, *carrier_frequency, 0.15f32);
      let mut signal = render_operators(operators, n_cycles, cps, SR);

      assert!(!signal.is_empty());
      melody.extend(signal)
    }

    let filename = format!("dev-audio/{}", label);
    engrave::samples(SR, &melody, &filename);
  }

  #[test]
  fn test_dx_brass() {
    let label = "dx-brass";
    let start_freq = 333f32 / 1f32;
    let freqs: Vec<f32> = (0..12).map(|i| start_freq + start_freq * 2f32.powf(i as f32 / 12f32)).collect();
    let mut melody: Vec<f32> = vec![];
    let c: f32 = 1.62181;
    let n_cycles = 3f32;
    let cps: f32 = 1.5f32;

    let l = freqs.len() as f32;
    for (i, carrier_frequency) in freqs.iter().enumerate() {
      let p = i as f32 * l / l;
      let modulator_playback_rate = 1.0;
      let operators = dexed_brass(p, n_cycles, cps, *carrier_frequency, 1f32);
      let mut signal = render_operators(operators, n_cycles, cps, SR);

      assert!(!signal.is_empty());
      melody.extend(signal)
    }

    let filename = format!("dev-audio/{}", label);
    engrave::samples(SR, &melody, &filename);
  }

  #[test]
  fn test_dx_lead() {
    let label = "dx-lead";
    let start_freq = 333f32 / 1f32;
    let freqs: Vec<f32> = (0..12).map(|i| start_freq + start_freq * 2f32.powf(i as f32 / 12f32)).collect();
    let c: f32 = 1.62181;
    let n_cycles = 12f32;
    let cps: f32 = 1.5f32;

    let l = freqs.len() as f32;

    for mod_i in (2..=10).step_by(2) {
      let mod_gain = mod_i as f32 / 10f32;
      let mut melody: Vec<f32> = vec![];

      for (i, carrier_frequency) in freqs.iter().enumerate() {
        let p = i as f32 * l / l;
        let modulator_playback_rate = 1.0;
        let operators = dexed_brass(p, n_cycles, cps, *carrier_frequency, mod_gain);
        let mut chords_signal = render_operators(operators, n_cycles, cps, SR);

        for mul in vec![1.2f32, 1.5f32] {
          let operators = dexed_brass(p, n_cycles, cps, *carrier_frequency * mul, mod_gain);
          let mut add_signal = render_operators(operators, n_cycles, cps, SR);
          for (ii, y) in add_signal.iter().enumerate() {
            chords_signal[ii] += y;
          }
        }
        chords_signal.iter_mut().for_each(|v| *v /= 3f32);
        melody.extend(chords_signal)
      }

      let filename = format!("dev-audio/{}-mod-gain-{}", label, mod_gain);
      engrave::samples(SR, &melody, &filename);
    }
  }

  #[test]
  fn test_dx_chords_pad() {
    let label = "dx-pad";
    let start_freq = 333f32 / 1f32;
    let freqs: Vec<f32> = (0..12).map(|i| start_freq + start_freq * 2f32.powf(i as f32 / 12f32)).collect();
    let n_cycles = 12f32;
    let cps: f32 = 1.5f32;

    let l = freqs.len() as f32;

    for mod_i in (2..=10).step_by(2) {
      let mod_gain = mod_i as f32 / 10f32;
      let mut melody: Vec<f32> = vec![];

      for (i, carrier_frequency) in freqs.iter().enumerate() {
        let p = i as f32 * l / l;
        let modulator_playback_rate = 1.0;
        let operators = dexed_pad(p, n_cycles, cps, *carrier_frequency, mod_gain);
        let mut chords_signal = render_operators(operators, n_cycles, cps, SR);

        for mul in vec![1.2f32, 1.5f32] {
          let operators = dexed_pad(p, n_cycles, cps, *carrier_frequency * mul, mod_gain);
          let mut add_signal = render_operators(operators, n_cycles, cps, SR);
          for (ii, y) in add_signal.iter().enumerate() {
            chords_signal[ii] += y;
          }
        }
        chords_signal.iter_mut().for_each(|v| *v /= 3f32);
        melody.extend(chords_signal)
      }

      let filename = format!("dev-audio/{}-mod-gain-{}", label, mod_gain);
      engrave::samples(SR, &melody, &filename);
    }
  }

  fn dx7_simplified_5_8(base_frequency: f32) -> Vec<Operator> {
    // Carrier 1 (op1) and its modulator (op2)
    let op2 = Operator::modulator(base_frequency * 1.0, dx_to_mod_index(84.0)); // Ratio: 1.0, Mod Index: 84
    let op1 = Operator {
      frequency: base_frequency * 1.0, // Carrier: 1.0 Hz
      modulators: single_modulator(op2),
      ..Operator::carrier(base_frequency * 1.0)
    };

    // Carrier 2 (op3) and its modulator (op4)
    let op4 = Operator::modulator(base_frequency * 2.0, dx_to_mod_index(84.0)); // Ratio: 2.0, Mod Index: 84
    let op3 = Operator {
      frequency: base_frequency * 1.288, // Carrier: 1.288 Hz
      modulators: single_modulator(op4),
      ..Operator::carrier(base_frequency * 1.288)
    };

    // Carrier 3 (op5) and its modulator (op6)
    let op6 = Operator::modulator(base_frequency * 3.0, dx_to_mod_index(84.0)); // Ratio: 3.0, Mod Index: 84
    let op5 = Operator {
      frequency: base_frequency * 2.042, // Carrier: 2.042 Hz
      modulators: single_modulator(op6),
      ..Operator::carrier(base_frequency * 2.042)
    };

    // Return all carriers as independent patches
    vec![op1, op3, op5]
  }

  #[test]
  fn test_dx_clone() {
    let freqs: Vec<f32> = (1..11).map(|i| i as f32 * 110f32).collect();
    let mut melody: Vec<f32> = vec![];
    for carrier_frequency in &freqs {
      let modulator_playback_rate = 1.0;

      let operators = dx7_simplified_5_8(*carrier_frequency);
      let mut signal = render_operators(operators, 12f32, 1.5, SR);

      assert!(!signal.is_empty());
      melody.extend(signal)
    }

    let filename = format!("dev-audio/test-strings-clone");
    engrave::samples(SR, &melody, &filename);
  }
}
