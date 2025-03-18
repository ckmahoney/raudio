use super::*;
use crate::analysis::melody::{eval_odr_level, LevelMacro, Levels, ODRMacro, ODR};
use crate::fm::*;

/// Creates a renderable stem using FM synthesis
pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  Renderable2::FMOp((
    melody,
    arf.clone(),
    dexed_brass,
    vec![], // Delay1
    vec![], // Delay2
    vec![], // Reverb1
    vec![], // Reverb2
  ))
}

#[test]
fn test_dexed_brass_bandwidth() {
  let conf = Conf { cps: 1.5, root: 1.23 };
  let offset_register = 0;
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
    role: Role::Lead,
    register: 7,
    visibility: Visibility::Foreground,
    energy: Energy::High,
    presence: Presence::Legato,
  };

  let mut signal: Vec<f32> = vec![];
  for (i, note) in melody[0].iter().enumerate() {
    let ops = dexed_brass(&conf, &arf, note, conf.cps, 16f32, i as f32, 1f32);
    let freq = note_to_freq(note);

    for op in &ops {
      match get_remaining_range(&op, 0f32, 0f32, MFf, NFf) {
        Some((lower_range, upper_range)) => {
          println!("Has op {:?}", op);
          println!("Got bandwidth lower {} upper {}", lower_range, upper_range);
        }
        None => assert!(false, "Must not design a synth that exceeds its bandwidth"),
      }
    }
    let s = render_operators_with_envelopes(ops, time::note_to_cycles(note), conf.cps, SR);
    signal.extend(s);
  }
  crate::render::engrave::samples(SR, &signal, &format!("dev-audio/test-fm-bandwidth.wav"));
}

/// Represents Algorithm 18 in DX-7 operator configuration.
/// A18 { { 2 + [3 > 3] + { 6 > 5 > 4 } } > 1 }
/// mod op2
/// mod op3
/// mod op4
pub fn dexed_brass(
  conf: &Conf, arf: &Arf, note: &Note, cps: f32, line_length_cycles: f32, curr_pos_cycles: f32, velocity: f32,
) -> Vec<Operator> {
  let p: f32 = curr_pos_cycles / line_length_cycles;
  let freq = note_to_freq(note);
  let n_cycles: f32 = note_to_cycles(note);
  // The percentage p that will be completed from start to end of this note
  let note_inc_p: f32 = n_cycles / curr_pos_cycles;
  let amp = note.2;

  let mut rng = thread_rng();

  let op2_detune_cents = get_dexed_detune(freq, -6);
  let op3_detune_cents = get_dexed_detune(freq, 7);
  let op4_detune_cents = get_dexed_detune(freq, 2);
  let op5_detune_cents = get_dexed_detune(freq, -2);

  let max_fmod_mul = 2f32;

  let mod_gain = mod_index_by_moment(note, arf);
  let op_frequency = freq;
  let gain = amp * visibility_gain(arf.visibility);

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
      op_frequency / 3f32 + op2_detune_cents,
      cascaded_gain(mod_gain, 0) * dx_to_mod_index(93.0),
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
      op_frequency / 5f32 + op3_detune_cents,
      cascaded_gain(mod_gain, 1) * dx_to_mod_index(81.0),
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
    ..Operator::modulator(
      op_frequency * 2f32.powi(0),
      cascaded_gain(mod_gain, 1) * dx_to_mod_index(99.0),
    )
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
      cascaded_gain(mod_gain, 0) * dx_to_mod_index(57.0),
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
      cascaded_gain(mod_gain, 0) * dx_to_mod_index(75.0),
    )
  };

  let op1 = Operator {
    modulators: vec![
      ModulationSource::Operator(op2),
      ModulationSource::Operator(op3),
      ModulationSource::Operator(op4),
    ],
    mod_gain_env_sum: Envelope::SampleBased {
      samples: mul_envelopes(
        vec![gain],
        ranger::eval_knob_mod(
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
        false,
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
