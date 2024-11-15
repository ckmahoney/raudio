use super::*;
use crate::druid::{self, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::types::synthesis::{ModifiersHolder, Soids};

// @art-choice This module would benefit from dynamic selection of knob params
// from the given VEP parameters

fn amp_knob_noise() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  let sustain = 0f32;
  let decay_mode = 0.5f32;
  let time_scaling_enabled = 0f32;

  (
    Knob {
      a: sustain,
      b: decay_mode,
      c: time_scaling_enabled,
    },
    ranger::amod_microtransient_4_20,
  )
}

fn amp_knob_tonal() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  let sustain = 0.5f32;
  (
    Knob {
      a: sustain,
      b: 0.0f32,
      c: 0.0,
    },
    ranger::amod_breath,
  )
}

fn pmod_knob_tonal(v: Visibility, e: Energy, p: Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  let mut rng = thread_rng();

  let modulation_depth: f32 = match v {
    Visibility::Hidden => 0.33f32,
    Visibility::Background => 0.5,
    Visibility::Foreground => 0.75,
    Visibility::Visible => 1f32,
  };

  let chorus_visibility: f32 = match v {
    Visibility::Hidden => 0f32,
    Visibility::Background => 0.1f32 + 0.5f32 * rng.gen::<f32>(),
    Visibility::Foreground => 0.6f32 + 0.2f32 * rng.gen::<f32>(),
    Visibility::Visible => 0.8f32 + 0.1f32 * rng.gen::<f32>(),
  };

  (
    Knob {
      a: modulation_depth,
      b: chorus_visibility,
      c: 0.0,
    },
    ranger::pmod_chorus,
  )
}

/// Defines the constituent stems to create a simple closed hat drum
/// Components include:
///  - an impulse of staccato undertone voicing
///  - a pluck of pink noise
pub fn renderable<'render>(melody: &'render Melody<Note>, arf: &Arf) -> Renderable<'render> {
  //# microtransient that indicates the note has started
  let soids_noise = druidic_soids::noise(1024f32, druidic_soids::NoiseType::Pink);
  let feel_noise: Feel = Feel {
    bp: (vec![4000f32], vec![6000f32]),
    modifiers: (vec![], vec![], vec![], vec![]),
    clippers: (0f32, 1f32),
  };
  let mut knob_mods_noise: KnobMods = KnobMods::unit();
  let expr = (vec![visibility_gain(Visibility::Hidden)], vec![1f32], vec![0f32]);

  knob_mods_noise.0.push(amp_knob_noise());

  let stem_noise = (
    melody,
    soids_noise,
    expr,
    feel_noise,
    knob_mods_noise,
    vec![delay::passthrough],
  );

  //# melodic component

  let soids_tonal = druidic_soids::under_square(2f32.powi(10i32));
  let modifiers_tonal: ModifiersHolder = (vec![], vec![], vec![], vec![]);
  let feel_tonal: Feel = Feel {
    bp: (vec![MFf], vec![NFf]),
    modifiers: modifiers_tonal,
    clippers: (0f32, 1f32),
  };

  let mut knob_mods_tonal: KnobMods = KnobMods::unit();
  knob_mods_tonal.0.push(amp_knob_tonal());
  knob_mods_tonal.2.push(pmod_knob_tonal(arf.visibility, arf.energy, arf.presence));
  let expr = (vec![visibility_gain(arf.visibility)], vec![1f32], vec![0f32]);
  let stem_tonal = (
    melody,
    soids_tonal,
    expr,
    feel_tonal,
    knob_mods_tonal,
    vec![delay::passthrough],
  );

  Renderable::Group(vec![stem_noise, stem_tonal])
}
