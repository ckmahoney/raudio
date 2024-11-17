use super::*;
use crate::druid::{self, noise::NoiseColor, soid_fx, soids as druidic_soids};
use crate::phrasing::ranger::KnobMods2;
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::types::synthesis::{ModifiersHolder, Soids};

use super::*;

/// Noise component
pub fn stem_visible<'render>(cps: f32, arf: &Arf, melody: &'render Melody<Note>) -> Stem2<'render> {
  let expr: Expr = (
    vec![db_to_amp(-30f32) * visibility_gain(arf.visibility)],
    vec![1f32],
    vec![0f32],
  );

  let height = 10i32;
  let noise_type = match arf.energy {
    Energy::High => druidic_soids::NoiseType::Pink,
    Energy::Medium => druidic_soids::NoiseType::Equal,
    Energy::Low => druidic_soids::NoiseType::Violet,
  };

  let soids = druidic_soids::noise(2f32.powi(height), noise_type);

  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  let mut rng = thread_rng();

  knob_mods.0.push(microtransient2());
  // Principal layer
  knob_mods.0.push((
    KnobMacro {
      a: match arf.presence {
        Presence::Staccatto => [0.2f32, 0.3f32],
        Presence::Legato => [0.4f32, 0.5f32],
        Presence::Tenuto => [0.6f32, 0.8f32],
      },
      b: [0.8f32, 1f32],
      c: [0f32, 0f32],
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Constant,
    },
    ranger::amod_fadeout,
  ));

  // Principal layer
  knob_mods.0.push((
    KnobMacro {
      a: match arf.presence {
        Presence::Staccatto => [0.2f32, 0.3f32],
        Presence::Legato => [0.4f32, 0.5f32],
        Presence::Tenuto => [0.6f32, 0.8f32],
      },
      b: match arf.energy {
        Energy::High => [0.1f32, 0.3f32],
        Energy::Medium => [0.3f32, 0.7f32],
        Energy::Low => [0.7f32, 1f32],
      },
      c: [0f32, 0f32],
      ma: MacroMotion::Constant,
      mb: MacroMotion::Constant,
      mc: MacroMotion::Constant,
    },
    ranger::amod_fadeout,
  ));

  let len_cycles: f32 = time::count_cycles(&melody[0]);

  let delays_note = vec![delay::passthrough];
  let delays_room = vec![];
  let reverbs_note: Vec<ReverbParams> = vec![];
  let reverbs_room: Vec<ReverbParams> = vec![];

  (
    melody,
    soids,
    expr,
    bp2_unit(),
    knob_mods,
    delays_note,
    delays_room,
    reverbs_note,
    reverbs_room,
  )
}

/// Defines the constituent stems to create a simple closed hat drum
/// Components include:
///  - a pluck of pink noise
pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  Renderable2::Instance(stem_visible(conf.cps, arf, melody))
}
