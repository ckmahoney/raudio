use super::*;
use crate::druid::{self, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::types::synthesis::{ModifiersHolder, Soids};

pub fn expr_tonal(arf: &Arf) -> Expr {
  (vec![visibility_gain(arf.visibility)], vec![1f32], vec![0f32])
}

// @art-choice This module would benefit from dynamic selection of knob params
// from the given VEP parameters

fn amp_knob_monic_delay(
  visibility: Visibility, energy: Energy, presence: Presence,
) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  let mut rng = thread_rng();
  let env_length = match presence {
    Presence::Tenuto => 0.5f32 + 0.5f32 * rng.gen::<f32>(),
    Presence::Legato => 0.2f32 + 0.3f32 * rng.gen::<f32>(),
    Presence::Staccatto => 0.1f32 * rng.gen::<f32>(),
  };

  let dynamic_range = match energy {
    Energy::Low => rng.gen::<f32>() / 6f32,
    Energy::Medium => rng.gen::<f32>() / 3f32,
    Energy::High => rng.gen::<f32>(),
  };

  let multiplier_rate = match visibility {
    Visibility::Visible => 0.05f32 * rng.gen::<f32>(),
    Visibility::Foreground => 0.25 + 0.25 * rng.gen::<f32>(),
    Visibility::Background => 0.5 + 0.25 * rng.gen::<f32>(),
    Visibility::Hidden => 0.77 + 0.23 * rng.gen::<f32>(),
  };

  (
    Knob {
      a: env_length,
      b: dynamic_range,
      c: multiplier_rate,
    },
    ranger::amod_fadein,
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

fn amp_knob_detune(
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
    Visibility::Background => 0.1 + 0.1 * rng.gen::<f32>(),
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

fn freq_knob_tonal(v: Visibility, e: Energy, p: Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  let mut rng = thread_rng();
  let modulation_amount = match e {
    Energy::Low => 0.005f32 + 0.003 * rng.gen::<f32>(),
    Energy::Medium => 0.008f32 + 0.012f32 * rng.gen::<f32>(),
    Energy::High => 0.1f32 + 0.2f32 * rng.gen::<f32>(),
  };
  (
    Knob {
      a: modulation_amount,
      b: 0f32,
      c: 0.0,
    },
    ranger::fmod_warble,
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

pub fn renderable<'render>(melody: &'render Melody<Note>, arf: &Arf) -> Renderable<'render> {
  //# melodic component

  let mullet = match arf.energy {
    Energy::Low => 256f32,
    Energy::Medium => 64f32,
    Energy::High => 32f32,
  };

  let soids_tonal = match arf.visibility {
    Visibility::Hidden => druidic_soids::under_octave(mullet),
    Visibility::Background => druidic_soids::overs_triangle(mullet),
    Visibility::Foreground => druidic_soids::overs_square(mullet),
    Visibility::Visible => druidic_soids::overs_sawtooth(mullet),
  };
  let modifiers_tonal: ModifiersHolder = (vec![], vec![], vec![], vec![]);
  let feel_tonal: Feel = Feel {
    bp: (vec![MFf], vec![NFf]),
    modifiers: modifiers_tonal,
    clippers: (0f32, 1f32),
  };

  let mut knob_mods_tonal: KnobMods = KnobMods::unit();
  knob_mods_tonal.0.push(amp_knob_tonal());
  knob_mods_tonal.0.push(amp_knob_monic_delay(arf.visibility, arf.energy, arf.presence));
  // knob_mods_tonal.1.push(freq_knob_tonal(arf.visibility, arf.energy, arf.presence));
  knob_mods_tonal.2.push(pmod_knob_tonal(arf.visibility, arf.energy, arf.presence));
  let stem_tonal = (
    melody,
    soids_tonal,
    expr_tonal(arf),
    feel_tonal,
    knob_mods_tonal,
    vec![delay::passthrough],
  );

  Renderable::Group(vec![stem_tonal])
}
