use super::*;
use crate::druid::{self, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::types::synthesis::{ModifiersHolder, Soids};

pub fn expr_noise(arf: &Arf) -> Expr {
  (vec![db_to_amp(-55f32)], vec![1f32], vec![0f32])
}

pub fn expr_tonal(arf: &Arf) -> Expr {
  (vec![db_to_amp(-55f32)], vec![1f32], vec![0f32])
}

// @art-choice This module would benefit from dynamic selection of knob params
// from the given VEP parameters

/// Selects a short lived impulse for the pink noise component of a closed hi hat
fn amp_knob_noise(
  visibility: Visibility, energy: Energy, presence: Presence,
) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
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
    Knob {
      a: env_length,
      b: decay_rate,
      c: 0.0,
    },
    ranger::amod_pluck,
  )
}

/// Selects a short lived impulse for the pink noise component of a closed hi hat
fn amp_knob_tonal() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  let mut rng = thread_rng();
  let decay_rate = 0.2f32 * rng.gen::<f32>();
  (
    Knob {
      a: decay_rate,
      b: 0.0,
      c: 0.0,
    },
    ranger::amod_impulse,
  )
}

/// Defines the constituent stems to create a simple closed hat drum
/// Components include:
///  - an impulse of staccato undertone voicing
///  - a pluck of pink noise
pub fn renderable<'render>(melody: &'render Melody<Note>, arf: &Arf) -> Renderable<'render> {
  //# noise component

  let soids_noise = druidic_soids::noise(1024f32, druidic_soids::NoiseType::Pink);
  let modifiers_noise: ModifiersHolder = (vec![], vec![], vec![], vec![]);
  let feel_noise: Feel = Feel {
    bp: (vec![MFf], vec![NFf]),
    modifiers: modifiers_noise,
    clippers: (0f32, 1f32),
  };

  let mut knob_mods_noise: KnobMods = KnobMods::unit();
  knob_mods_noise.0.push(amp_knob_noise(arf.visibility, arf.energy, arf.presence));
  let stem_noise = (
    melody,
    soids_noise,
    expr_noise(arf),
    feel_noise,
    knob_mods_noise,
    vec![delay::passthrough],
  );

  //# tonal component

  let soids_tonal = druidic_soids::under_square(2f32.powi(10i32));
  let modifiers_tonal: ModifiersHolder = (vec![], vec![], vec![], vec![]);
  let feel_tonal: Feel = Feel {
    bp: (vec![MFf], vec![NFf]),
    modifiers: modifiers_tonal,
    clippers: (0f32, 1f32),
  };

  let mut knob_mods_tonal: KnobMods = KnobMods::unit();
  knob_mods_tonal.0.push(amp_knob_tonal());
  let stem_tonal = (
    melody,
    soids_tonal,
    expr_tonal(arf),
    feel_tonal,
    knob_mods_tonal,
    vec![delay::passthrough],
  );

  Renderable::Group(vec![stem_noise, stem_tonal])
}

mod a2 {

  // from urbuntu

  use super::*;
  use super::*;
  use crate::phrasing::ranger::KnobMods2;

  pub fn expr_noise(arf: &Arf) -> Expr {
    (vec![db_to_amp(-55f32)], vec![1f32], vec![0f32])
  }

  pub fn expr_tonal(arf: &Arf) -> Expr {
    (vec![db_to_amp(-55f32)], vec![1f32], vec![0f32])
  }

  /// Selects a short-lived impulse for the pink noise component of a closed hi-hat
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

  /// Selects a short-lived impulse for the tonal component of a closed hi-hat
  fn amp_knob_tonal() -> KnobPair {
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

  /// Defines the constituent stems to create a simple closed hi-hat drum
  /// Components include:
  ///  - an impulse of staccato undertone voicing
  ///  - a pluck of pink noise
  pub fn renderable<'render>(cps: f32, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
    // Noise component
    let soids_noise = druidic_soids::noise(1024f32, druidic_soids::NoiseType::Pink);
    let modifiers_noise: ModifiersHolder = (vec![], vec![], vec![], vec![]);
    let bp2: Bp2 = (vec![MFf], vec![NFf], vec![]);

    let mut knob_mods_noise: KnobMods2 = KnobMods2::unit();
    knob_mods_noise.0.push(amp_knob_noise(arf.visibility, arf.energy, arf.presence));
    let stem_noise: Stem2 = (
      melody,
      soids_noise,
      expr_noise(arf),
      bp2,
      knob_mods_noise,
      vec![delay::passthrough],
      vec![],
      vec![],
      vec![],
    );

    // Tonal component
    let soids_tonal = druidic_soids::under_square(2f32.powi(10i32));
    let modifiers_tonal: ModifiersHolder = (vec![], vec![], vec![], vec![]);
    let bp2 = (vec![MFf], vec![NFf], vec![]);

    let mut knob_mods_tonal: KnobMods2 = KnobMods2::unit();
    knob_mods_tonal.0.push(amp_knob_tonal());
    let stem_tonal: Stem2 = (
      melody,
      soids_tonal,
      expr_tonal(arf),
      bp2,
      knob_mods_tonal,
      vec![delay::passthrough],
      vec![],
      vec![],
      vec![],
    );

    Renderable2::Group(vec![stem_noise, stem_tonal])
  }
}
