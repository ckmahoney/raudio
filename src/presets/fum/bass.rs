use super::*;
use crate::analysis::melody::{eval_odr_level, LevelMacro, Levels, ODRMacro, ODR};
use crate::fm::*;

/// Creates a renderable stem using FM synthesis
pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  Renderable2::FMOp((
    melody,
    arf.clone(),
    dexed_bass,
    vec![], // Delay1
    vec![], // Delay2
    vec![], // Reverb1
    vec![], // Reverb2
  ));
  simple_tacet(melody)
}

/// A rich bass synth evoking distorted bassoon.  
///
/// Uses Algorithm 18 in DX-7 operator configuration.
/// A18 { { 2 + [3 > 3] + 6 > 5 > 4 } > 1 }
/// op2 provides the basis of the bass woodwind timbre
/// op3 offers the upper body of the woodwind timbre
/// op4 gives it the bite on attack
pub fn dexed_bass(
  conf: &Conf, arf: &Arf, note: &Note, cps: f32, line_length_cycles: f32, curr_pos_cycles: f32, velocity: f32,
) -> Vec<Operator> {
  let amp = note.2;

  let p: f32 = curr_pos_cycles / line_length_cycles;
  let freq = note_to_freq(note);
  let n_cycles: f32 = note_to_cycles(note);

  // The percentage p that will be completed from start to end of this note
  let note_inc_p: f32 = n_cycles / curr_pos_cycles;
  let mut rng = thread_rng();

  let op3_detune_cents = get_dexed_detune(freq, 2);
  let op5_detune_cents = get_dexed_detune(freq, -2);
  let op6_detune_cents = get_dexed_detune(freq, 3);
  let max_fmod_mul = 2f32;

  let op_freq = freq;
  let mod_gain = mod_index_by_moment(note, arf);
  let gain = amp * visibility_gain(arf.visibility);

  let mut gen_mul = || -> Envelope {
    let mut rng = thread_rng();
    Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: match arf.presence {
            Presence::Staccatto => in_range(&mut rng, 0.1f32, 0.2f32),
            Presence::Legato => in_range(&mut rng, 0.5f32, 0.8f32),
            Presence::Tenuto => in_range(&mut rng, 0.8f32, 1f32),
          },
          b: match arf.visibility {
            Visibility::Visible => 0.5f32,
            Visibility::Foreground => 0.25f32,
            Visibility::Background => 0f32,
            Visibility::Hidden => 0f32,
          },
          c: 0f32,
        },
        cps,
        op_freq,
        n_cycles,
      ),
    }
  };

  let mut op2 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_burp,
        &Knob {
          a: in_range(&mut rng, 0.8f32, 1f32),
          b: 0.8f32,
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    modulators: single_modulator(Operator {
      mod_index_env_mul: Envelope::SampleBased {
        samples: ranger::eval_knob_mod(
          ranger::amod_unit,
          &Knob {
            a: in_range(&mut rng, 0.4f32, 0.5f32),
            b: in_range(&mut rng, 0.2f32, 0.4f32),
            c: 0f32,
          },
          cps,
          freq,
          n_cycles,
        ),
      },
      mod_index_env_sum: Envelope::SampleBased {
        samples: ranger::eval_knob_mod(
          ranger::amod_unit,
          &Knob {
            a: in_range(&mut rng, 0.05f32, 0.1f32),
            b: in_range(&mut rng, 0.1f32, 0.3f32),
            c: 0f32,
          },
          cps,
          freq,
          n_cycles,
        ),
      },
      ..Operator::modulator(3f32, cascaded_gain(mod_gain, 2) * dx_to_mod_index(75.0))
    }),
    ..Operator::modulator(
      in_range(&mut rng, 1f32, 3f32),
      cascaded_gain(mod_gain, 1) * dx_to_mod_index(75.0),
    )
  };

  let op3 = Operator {
    mod_index_env_mul: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: 0.5f32,
          b: 1f32,
          c: 0f32,
        },
        cps,
        op_freq,
        n_cycles,
      ),
    },
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: in_range(&mut rng, 0.17f32, 0.29f32),
          b: in_range(&mut rng, 0.1f32, 0.2f32),
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    modulators: vec![ModulationSource::Feedback(0.75)],
    ..Operator::modulator(
      in_range(&mut rng, 2.9, 3.013) + op3_detune_cents,
      cascaded_gain(mod_gain, 1) * dx_to_mod_index(61.0),
    )
  };

  let op6 = Operator {
    mod_index_env_mul: gen_mul(),
    mod_index_env_sum: Envelope::SampleBased {
      samples: eval_odr_level(
        cps,
        n_cycles,
        &LevelMacro {
          stable: [0.1, 0.1f32],
          peak: [0.98, 1f32],
          sustain: [0.15, 0.25f32],
        },
        &ODRMacro {
          onset: [20.0, 50.0],
          decay: [90.0, 200.0],
          release: [30.0, 50.0],
          mo: vec![MacroMotion::Constant],
          md: vec![MacroMotion::Constant],
          mr: vec![MacroMotion::Constant],
        },
      ),
    },
    ..Operator::modulator(
      op_freq * 2.0f32 + op6_detune_cents,
      cascaded_gain(mod_gain, 2) * dx_to_mod_index(70.0),
    )
  };

  let op5 = Operator {
    mod_index_env_mul: gen_mul(),
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
          onset: [100.0, 300.0],
          decay: [50.0, 90.0],
          release: [100.0, 500.0],
          mo: vec![MacroMotion::Constant],
          md: vec![MacroMotion::Constant],
          mr: vec![MacroMotion::Constant],
        },
      ),
    },
    modulators: single_modulator(op6),
    ..Operator::modulator(
      op_freq * 2.0f32 + op5_detune_cents,
      cascaded_gain(mod_gain, 1) * dx_to_mod_index(60.0),
    )
  };

  // burst of harmonics on note entry
  let op4 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: in_range(&mut rng, 0.7f32, 0.9f32),
          b: in_range(&mut rng, 0.7f32, 0.95f32),
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    modulators: single_modulator(op5.clone()),
    ..Operator::modulator(
      op_freq * in_range(&mut rng, 0.997, 1.003),
      cascaded_gain(mod_gain, 0) * dx_to_mod_index(75.0),
    )
  };
  // burst of harmonics on note entry
  let op7 = Operator {
    mod_index_env_sum: Envelope::SampleBased {
      samples: ranger::eval_knob_mod(
        ranger::amod_unit,
        &Knob {
          a: in_range(&mut rng, 0.05f32, 0.1f32),
          b: in_range(&mut rng, 0.3f32, 0.5f32),
          c: 0f32,
        },
        cps,
        freq,
        n_cycles,
      ),
    },
    ..Operator::modulator(
      in_range(&mut rng, 0.997, 1.003),
      cascaded_gain(mod_gain, 0) * dx_to_mod_index(75.0),
    )
  };

  let mut op1 = Operator {
    modulators: vec![
      ModulationSource::Operator(op2),
      ModulationSource::Operator(op4),
      ModulationSource::Operator(op7),
    ],
    ..Operator::carrier2(freq, gain)
  };

  if let Energy::High = arf.energy {
    op1.modulators.push(ModulationSource::Operator(op3));
  };

  vec![op1]
}
