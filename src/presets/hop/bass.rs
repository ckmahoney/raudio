use super::*;

use crate::druid::{self, soid_fx, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::types::synthesis::{ModifiersHolder, Soids};

pub fn expr(arf: &Arf, n_samples: usize) -> Expr {
  let dynamics = dynamics::gen_organic_amplitude(10, n_samples, arf.visibility);
  (dynamics, vec![1f32], vec![0f32])
}

/// Create bandpass automations with respect to Arf and Melody
fn bp<'render>(melody: &'render Melody<Note>, arf: &Arf, len_cycles: f32) -> (SampleBuffer, SampleBuffer) {
  let size = len_cycles.log2() - 1f32; // offset 1 to account for lack of CPC. -1 assumes CPC=2
  let rate_per_size = match arf.energy {
    Energy::Low => 0.25f32,
    Energy::Medium => 0.5f32,
    Energy::High => 1f32,
  };

  let mut highest_register: i8 = arf.register;
  let mut lowest_register: i8 = arf.register;
  for line in melody.iter() {
    for (_, (register, _), _) in line.iter() {
      highest_register = (*register).max(highest_register);
      lowest_register = (*register).min(lowest_register);
    }
  }
  let n_samples: usize = (len_cycles / 2f32) as usize * SR;

  let (highpass, lowpass): (Vec<f32>, Vec<f32>) = if let Visibility::Visible = arf.visibility {
    match arf.energy {
      Energy::Low => (
        filter_contour_triangle_shape_highpass(
          lowest_register - 2,
          highest_register - 2,
          n_samples,
          size * rate_per_size,
        ),
        vec![NFf],
      ),
      _ => (
        vec![MFf],
        filter_contour_triangle_shape_lowpass(lowest_register, n_samples, size * rate_per_size),
      ),
    }
  } else {
    (vec![MFf], vec![NFf / 8f32])
  };

  (highpass, lowpass)
}

fn amp_knob_principal(rng: &mut ThreadRng, arf: &Arf) -> KnobPair {
  (
    KnobMacro {
      a: match arf.presence {
        Presence::Staccatto => [0.11f32, 0.3f32], // Using range values directly
        Presence::Legato => [0.33f32, 0.5f32],
        Presence::Tenuto => [0.7f32, 0.9f32],
      },
      b: match arf.visibility {
        Visibility::Visible => [0f32, 0.2f32],
        Visibility::Foreground => [0.2f32, 0.3f32],
        _ => [0.3f32, 0.5f32],
      },
      c: [0.0, 0.0], // Static value as [0, 0]
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    ranger::amod_burp,
  )
}

fn amp_knob_attenuation(rng: &mut ThreadRng, arf: &Arf) -> KnobPair {
  (
    KnobMacro {
      a: match arf.energy {
        Energy::High => [0.34f32, 0.5f32],
        Energy::Medium => [0.23f32, 0.34f32],
        Energy::Low => [0.0f32, 0.21f32],
      },
      b: [1f32, 1f32], // Static value as [1, 1]
      c: [0.0, 0.0],   // Static value as [0, 0]
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    ranger::amod_detune,
  )
}

pub fn renderable<'render>(cps: f32, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  let mut rng = thread_rng();
  let len_cycles: f32 = time::count_cycles(&melody[0]);
  let n_samples = (SRf * len_cycles / 2f32) as usize;

  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  knob_mods.0.push(amp_microtransient(arf.visibility, arf.energy, arf.presence));
  knob_mods.0.push(amp_knob_principal(&mut rng, &arf));
  knob_mods.0.push(amp_knob_attenuation(&mut rng, &arf));

  let soids = druidic_soids::overs_square(2f32.powi(7i32));

  let delays_note = vec![delay::passthrough];
  let delays_room = vec![];
  let reverbs_note: Vec<ReverbParams> = vec![];
  let reverbs_room: Vec<ReverbParams> = vec![];

  let stem = (
    melody,
    soids,
    expr(arf, n_samples),
    get_bp(cps, melody, arf, len_cycles),
    knob_mods,
    delays_note,
    delays_room,
    reverbs_note,
    reverbs_room,
  );

  Renderable2::Group(vec![stem])
}
