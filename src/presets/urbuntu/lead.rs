use std::os::unix::thread;

use super::*;
use crate::druid::{self, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::types::synthesis::{ModifiersHolder, Soids};

fn amp_knob_breath(
  visibility: Visibility, energy: Energy, presence: Presence,
) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  let mut rng = thread_rng();

  let breath_rate = match energy {
    Energy::Low => rng.gen::<f32>(),
    Energy::Medium => rng.gen::<f32>() / 2f32,
    Energy::High => rng.gen::<f32>() / 3f32,
  };

  return (
    Knob {
      a: breath_rate,
      b: 0f32,
      c: 0.0,
    },
    ranger::amod_breath,
  );
}

fn amp_knob_experiement(
  visibility: Visibility, energy: Energy, presence: Presence,
) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  let mut rng = thread_rng();

  let detune_rate = match energy {
    Energy::Low => rng.gen::<f32>() / 6f32,
    Energy::Medium => rng.gen::<f32>() / 3f32,
    Energy::High => rng.gen::<f32>(),
  };
  let detune_mix = match visibility {
    Visibility::Visible => 0.33 + 0.47 * rng.gen::<f32>(),
    Visibility::Foreground => 0.2 + 0.13 * rng.gen::<f32>(),
    Visibility::Background => 0.1 * 0.1 * rng.gen::<f32>(),
    Visibility::Hidden => 0.05f32 * rng.gen::<f32>(),
  };

  return (
    Knob {
      a: detune_rate,
      b: detune_mix,
      c: 0.0,
    },
    ranger::amod_detune,
  );
}

pub fn renderable<'render>(melody: &'render Melody<Note>, arf: &Arf) -> Renderable<'render> {
  let mullet = match arf.energy {
    Energy::Low => 256f32,
    Energy::Medium => 64f32,
    Energy::High => 32f32,
  };
  let soids = match arf.visibility {
    Visibility::Hidden => druidic_soids::octave(mullet),
    Visibility::Background => druidic_soids::overs_triangle(mullet),
    Visibility::Foreground => druidic_soids::overs_square(mullet),
    Visibility::Visible => druidic_soids::overs_sawtooth(mullet),
  };
  let mut knob_mods: KnobMods = KnobMods::unit();

  knob_mods.0.push(amp_knob_experiement(arf.visibility, arf.energy, arf.presence));
  knob_mods.0.push(amp_knob_breath(arf.visibility, arf.energy, arf.presence));

  let feel: Feel = Feel {
    bp: (vec![MFf], vec![NFf]),
    modifiers: (vec![], vec![], vec![], vec![]),
    clippers: (0f32, 1f32),
  };
  let expr = (vec![visibility_gain(arf.visibility)], vec![1f32], vec![0f32]);

  let stem = (melody, soids, expr, feel, knob_mods, vec![delay::passthrough]);
  Renderable::Instance(stem)
}
