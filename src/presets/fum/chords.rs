use super::*;
use crate::analysis::melody::{eval_odr_level, LevelMacro, Levels, ODRMacro, ODR};
use crate::fm::*;

/// Creates a renderable stem using FM synthesis
pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  Renderable2::FMOp((
    melody,
    arf.clone(),
    dexed_pad,
    vec![], // Delay1
    vec![], // Delay2
    vec![], // Reverb1
    vec![], // Reverb2
  ))
}

#[test]
fn test_dexed_pad_bandwidth() {
  let conf = Conf { cps: 1.5, root: 1.23 };
  let offset_register = 4;
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

  let arf = Arf {
    mode: Mode::Melodic,
    role: Role::Chords,
    register: 10,
    visibility: Visibility::Foreground,
    energy: Energy::High,
    presence: Presence::Legato,
  };

  let mut signal: Vec<f32> = vec![];
  for (i, note) in melody[0].iter().enumerate() {
    let ops = dexed_pad(&conf, &arf, note, conf.cps, 16f32, i as f32, 1f32);
    let freq = note_to_freq(note);

    for op in &ops {
      match get_remaining_range(&op, 0f32, 0f32, MFf, NFf) {
        Some((lower_range, upper_range)) => {
          // println!("Has op {:?}", op);
          // println!("Got bandwidth lower {} upper {}", lower_range, upper_range);
        }
        None => assert!(false, "Must not design a synth that exceeds its bandwidth"),
      }
    }
    let s = render_operators_with_envelopes(ops, time::note_to_cycles(note), conf.cps, SR);
    signal.extend(s);
  }
  crate::render::engrave::samples(SR, &signal, &format!("dev-audio/test-fm-bandwidth.wav"));
}

/// Represents Algorithm 9 in DX-7 operator configuration for a synth pad
/// A9 { [2 > 2] > 1 + { 4 + 6 > 5 } > 3 }
pub fn dexed_pad(
  conf: &Conf, arf: &Arf, note: &Note, cps: f32, line_length_cycles: f32, curr_pos_cycles: f32, velocity: f32,
) -> Vec<Operator> {
  let mut rng = thread_rng();
  let freq = note_to_freq(note);
  let n_cycles: f32 = note_to_cycles(note);
  let amp = velocity * note.2;

  let op1_detune_cents = get_dexed_detune(freq, 7);
  let op2_detune_cents = get_dexed_detune(freq, 5);
  let op3_detune_cents = get_dexed_detune(freq, 1);
  let op6_detune_cents = get_dexed_detune(freq, 7);

  let op_freq = freq;
  let gain = amp * visibility_gain(arf.visibility) / 2f32;
  let mod_gain = mod_index_by_moment(note, arf);

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
          a: in_range(&mut rng, 0.8f32, 0.9f32),
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
      op_freq * 2f32.powi(-1) + op2_detune_cents,
      cascaded_gain(mod_gain, 0) * dx_to_mod_index(77.0),
    )
  };

  let mut op1 = Operator {
    mod_gain_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.6f32,
          b: in_range(&mut rng, 0.8f32, 0.95f32),
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
          a: in_range(&mut rng, 0.7f32, 0.9f32),
          b: 0.15f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles / 2f32,
      ),
    },
    modulators: single_modulator(op2),
    ..Operator::carrier2(op_freq * in_range(&mut rng, 0.99, 0.997), gain * 0.5f32)
  };

  let op6 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: in_range(&mut rng, 0.7f32, 0.9f32),
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
          a: in_range(&mut rng, 0.7f32, 0.99f32),
          b: 0.5f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    ..Operator::modulator(
      1f32 / (op_freq * (3f32 / 1.2f32)),
      cascaded_gain(mod_gain, 1) * dx_to_mod_index(46.0),
    )
  };

  let op7 = Operator {
    ..Operator::modulator(
      in_range(&mut rng, 3f32, 5f32),
      cascaded_gain(mod_gain, 1) * dx_to_mod_index(16.0),
    )
  };

  let op5 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: in_range(&mut rng, 0.75f32, 0.97f32),
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
          a: in_range(&mut rng, 0.85f32, 0.97f32),
          b: 0.5f32,
          c: 1f32,
        },
        cps,
        1f32,
        n_cycles,
      ),
    },
    modulators: vec![
      ModulationSource::Operator(op6.clone()),
      ModulationSource::Operator(op7.clone()),
    ],
    ..Operator::modulator(
      op_freq * in_range(&mut rng, 1f32 / 12f32, 1f32 / 16f32),
      cascaded_gain(mod_gain, 0) * dx_to_mod_index(49.0),
    )
  };

  let op8 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: in_range(&mut rng, 0.5f32, 0.7f32),
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
          a: in_range(&mut rng, 0.5f32, 0.7f32),
          b: 0.5f32,
          c: 1f32,
        },
        cps,
        1f32,
        n_cycles,
      ),
    },
    modulators: vec![ModulationSource::Operator(op6), ModulationSource::Operator(op7)],
    ..Operator::modulator(
      op_freq * in_range(&mut rng, 1f32 / 18f32, 1f32 / 32f32),
      cascaded_gain(mod_gain, 0) * dx_to_mod_index(59.0),
    )
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
    ..Operator::modulator(
      (2f32 / 3f32) * op_freq * in_range(&mut rng, 0.995, 1.005),
      cascaded_gain(mod_gain, 0) * dx_to_mod_index(75.0),
    )
  };

  let op3 = Operator {
    mod_gain_env_mul: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_cycle_fadein_4_16,
        &Knob {
          a: 0.25f32,
          b: 0.5f32,
          c: 1f32,
        },
        cps,
        op_freq,
        n_cycles,
      ),
    },
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
    ..Operator::carrier2(op_freq, gain * 0.5f32)
  };

  vec![op1, op3]
}
