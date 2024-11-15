use super::*;
use crate::druid::{self, noise::NoiseColor, soid_fx, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods2};
use crate::types::synthesis::{ModifiersHolder, Soids};

// Selects a short-lived impulse for the pink noise component of a closed hi-hat
fn amp_knob_noise(visibility: Visibility, energy: Energy, presence: Presence) -> KnobPair {
  let sustain = 0.1f32;
  let mut rng = thread_rng();
  let decay_rate = match energy {
    Energy::Low => 0.8 + 0.2f32 * rng.gen::<f32>(),
    Energy::Medium => 0.3 + 0.4f32 * rng.gen::<f32>(),
    Energy::High => 0.05 + 0.2f32 * rng.gen::<f32>(),
  };
  let env_length = match presence {
    Presence::Tenuto => 0.66f32 + 0.2f32 * rng.gen::<f32>(),
    Presence::Legato => 0.2f32 + 0.3f32 * rng.gen::<f32>(),
    Presence::Staccatto => 0.2f32 * rng.gen::<f32>(),
  };

  (
    KnobMacro {
      a: [env_length, env_length],
      b: [decay_rate, decay_rate],
      c: [0.0, 0.0],
      ma: MacroMotion::Constant,
      mb: MacroMotion::Constant,
      mc: MacroMotion::Constant,
    },
    ranger::amod_pluck,
  )
}

// Selects a short-lived impulse for the pink noise component of a closed hi-hat
fn amp_knob() -> KnobPair {
  let mut rng = thread_rng();
  let decay_rate = 0.2f32 * rng.gen::<f32>();
  (
    KnobMacro {
      a: [decay_rate, decay_rate],
      b: [0.0, 0.0],
      c: [0.0, 0.0],
      ma: MacroMotion::Constant,
      mb: MacroMotion::Constant,
      mc: MacroMotion::Constant,
    },
    ranger::amod_impulse,
  )
}

// Noise component
pub fn stem_visible<'render>(arf: &Arf, melody: &'render Melody<Note>) -> Stem2<'render> {
  let bp2: Bp2 = (vec![MFf], vec![NFf], vec![]);

  let soids = soid_fx::concat(&vec![
    soid_fx::noise::rank(0, NoiseColor::Pink, 1f32 / 3f32),
    soid_fx::noise::rank(1, NoiseColor::Equal, 1f32 / 5f32),
    soid_fx::noise::rank(2, NoiseColor::Violet, 1f32 / 11f32),
    soid_fx::noise::rank(3, NoiseColor::Violet, 1f32 / 9f32),
  ]);

  let expr: Expr = (vec![db_to_amp(-15f32)], vec![1f32], vec![0f32]);

  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  let mut rng = thread_rng();

  knob_mods.0.push((
    KnobMacro {
      a: [0.0, 0.0],
      b: match arf.energy {
        Energy::High => [0f32, 0.1f32],
        Energy::Medium => [0.1f32, 0.3f32],
        Energy::Low => [0.2f32, 0.4f32],
      },
      c: [0f32, 0f32],
      ma: MacroMotion::Constant,
      mb: MacroMotion::Constant,
      mc: MacroMotion::Constant,
    },
    ranger::amod_pluck,
  ));

  knob_mods.0.push((
    KnobMacro {
      a: match arf.presence {
        Presence::Staccatto => [0f32, 0.15f32],
        Presence::Legato => [0.23f32, 0.77f32],
        Presence::Tenuto => [0.88f32, 1f32],
      },
      b: match arf.energy {
        Energy::High => [0f32, 0.2f32],
        Energy::Medium => [0.2f32, 0.3f32],
        Energy::Low => [0.3f32, 0.5f32],
      },
      c: [0f32, 0f32],
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Constant,
    },
    ranger::amod_pluck,
  ));

  (
    melody,
    soids,
    expr,
    bp2,
    knob_mods,
    vec![delay::passthrough],
    vec![],
    vec![],
    vec![],
  )
}

// Tonal component
pub fn stem_foreground<'render>(arf: &Arf, melody: &'render Melody<Note>) -> Stem2<'render> {
  let soids_base = soid_fx::concat(&vec![
    druidic_soids::under_sawtooth(2f32.powi(11i32)),
    druidic_soids::overs_square(2f32.powi(11i32)),
  ]);
  let soids = soid_fx::concat(&vec![
    soid_fx::ratio::constant(&soids_base, 0.8f32, 0.15f32),
    soid_fx::ratio::constant(&soids_base, 0.666f32, 0.25f32),
  ]);

  let expr: Expr = (vec![db_to_amp(-15f32)], vec![1f32], vec![0f32]);

  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  let mut rng = thread_rng();
  knob_mods.0.push((
    KnobMacro {
      a: [rng.gen::<f32>(), rng.gen::<f32>()],
      b: [0f32, 0.2f32 * rng.gen::<f32>()],
      c: [0f32, 0f32],
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Constant,
    },
    ranger::amod_pluck,
  ));

  (
    melody,
    soids,
    expr,
    bp2_unit(),
    knob_mods,
    vec![delay::passthrough],
    vec![],
    vec![],
    vec![],
  )
}

// Defines the constituent stems to create a simple closed hi-hat drum
pub fn renderable<'render>(cps: f32, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  Renderable2::Group(vec![stem_visible(arf, melody), stem_foreground(arf, melody)])
}
