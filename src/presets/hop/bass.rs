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
        Presence::Staccatto => [0.2f32, 0.5f32],
        Presence::Legato => [0.4f32, 0.8f32],
        Presence::Tenuto => [0.85f32, 1f32],
      },
      b: match arf.visibility {
        Visibility::Visible => [0.2f32, 0.4f32],
        Visibility::Foreground => [0.5f32, 0.7f32],
        _ => [0.6f32, 1f32],
      },
      c: [0.0, 0.0],
      ma: grab_variant(vec![MacroMotion::Forward, MacroMotion::Reverse, MacroMotion::Constant]),
      mb: grab_variant(vec![
        MacroMotion::Forward,
        MacroMotion::Reverse,
        MacroMotion::Random,
        MacroMotion::Constant,
      ]),
      mc: MacroMotion::Random,
    },
    ranger::amod_pluck2,
  )
}

fn amp_knob_detune(rng: &mut ThreadRng, arf: &Arf) -> KnobPair {
  (
    KnobMacro {
      a: match arf.energy {
        Energy::High => [0.34f32, 0.5f32],
        Energy::Medium => [0.23f32, 0.34f32],
        Energy::Low => [0.0f32, 0.21f32],
      },
      b: [1f32, 1f32],
      c: [0.0, 0.0],
      ma: grab_variant(vec![
        MacroMotion::Forward,
        MacroMotion::Reverse,
        MacroMotion::Min,
        MacroMotion::Constant,
      ]),
      mb: MacroMotion::Min,
      mc: MacroMotion::Min,
    },
    ranger::amod_detune,
  )
}

pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  let mut rng = thread_rng();
  let len_cycles: f32 = time::count_cycles(&melody[0]);
  let n_samples = (SRf * len_cycles / 2f32) as usize;

  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  if let Visibility::Visible = arf.visibility {
    // don't add it
  } else {
    knob_mods.0.push(amp_onset(arf.visibility, arf.energy, arf.presence));
  }
  knob_mods.0.push(amp_knob_principal(&mut rng, &arf));
  knob_mods.0.push(amp_knob_detune(&mut rng, &arf));

  let height = mul_it(&arf, 10f32, 9f32, 7f32);

  let soids = match arf.visibility {
    Visibility::Visible => druidic_soids::overs_sawtooth(height),
    Visibility::Foreground => druidic_soids::overs_sawtooth(height),
    _ => druidic_soids::overs_triangle(height),
  };
  let delays_note = vec![];
  let delays_room = vec![];
  let reverbs_note: Vec<ReverbParams> = vec![];
  let reverbs_room: Vec<ReverbParams> = vec![];

  let stem = (
    melody,
    soids,
    expr(arf, n_samples),
    HopCon::get_bp(conf.cps, melody, arf),
    knob_mods,
    delays_note,
    delays_room,
    reverbs_note,
    reverbs_room,
  );

  Renderable2::Instance(stem)
}
