use super::*;
use crate::druid::{self, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::types::synthesis::{ModifiersHolder, Soids};

pub fn expr(arf: &Arf) -> Expr {
  (vec![visibility_gain(arf.visibility)], vec![1f32], vec![0f32])
}

fn amp_knob(visibility: Visibility, energy: Energy, presence: Presence) -> KnobPair {
  let sustain = match presence {
    Presence::Staccatto => [0f32, 0f32],  // Static value as [0, 0]
    Presence::Legato => [0.1f32, 0.1f32], // Static value as [0.1, 0.1]
    Presence::Tenuto => [0.3f32, 0.3f32], // Static value as [0.3, 0.3]
  };
  let decay_rate = match energy {
    Energy::Low => [0.5f32, 0.5f32],      // Static value as [0.5, 0.5]
    Energy::Medium => [0.75f32, 0.75f32], // Static value as [0.75, 0.75]
    Energy::High => [1f32, 1f32],         // Static value as [1, 1]
  };
  (
    KnobMacro {
      a: sustain,
      b: decay_rate,
      c: [0f32, 0f32], // Static value as [0, 0]
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    ranger::amod_pluck,
  )
}

/// Selects the number of resonant nodes to add
fn amp_reso_gen(modders: &mut KnobMods2, visibility: Visibility, energy: Energy, presence: Presence) {
  let n = match energy {
    Energy::Low => 2,
    Energy::Medium => 3,
    Energy::High => 5,
  };
  let mut rng = thread_rng();
  for i in 0..n {
    let a: [f32; 2] = match visibility {
      Visibility::Visible => [0f32, rng.gen::<f32>() / 5f32],
      Visibility::Foreground => [0f32, rng.gen::<f32>() / 3f32],
      Visibility::Background => [0f32, rng.gen::<f32>() / 2f32],
      Visibility::Hidden => [0.5f32, 0.5f32 + rng.gen::<f32>() / 2f32],
    };
    let b: [f32; 2] = match energy {
      Energy::Low => [0f32, rng.gen::<f32>() / 2f32],
      Energy::Medium => [0f32, rng.gen::<f32>() / 3f32],
      Energy::High => [0f32, rng.gen::<f32>() / 5f32],
    };
    modders.0.push((
      KnobMacro {
        a,
        b,
        c: [0f32, 0f32], // Static value as [0, 0]
        ma: MacroMotion::Random,
        mb: MacroMotion::Random,
        mc: MacroMotion::Random,
      },
      ranger::amod_peak,
    ));
  }
}

pub fn renderable<'render>(cps: f32, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  let mut rng: ThreadRng = thread_rng();

  // Principal layer
  knob_mods.0.push((
    KnobMacro {
      a: [0.2f32, 0.8f32],
      b: [0.2f32, 0.8f32],
      c: [0f32, 0f32], // Static value as [0, 0]
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    ranger::amod_pluck,
  ));

  // Attenuation layer
  knob_mods.0.push((
    KnobMacro {
      a: match arf.presence {
        Presence::Staccatto => [0.33f32, 0.66f32],
        Presence::Legato => [0.53f32, 0.76f32],
        Presence::Tenuto => [0.88f32, 1f32],
      },
      b: match arf.energy {
        Energy::High => [0f32, 0.33f32],
        Energy::Medium => [0.33f32, 0.5f32],
        Energy::Low => [0.5f32, 1f32],
      },
      c: [0f32, 0f32], // Static value as [0, 0]
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    ranger::amod_oscillation_sine,
  ));

  let len_cycles: f32 = time::count_cycles(&melody[0]);
  let soids = soid_fx::concat(&vec![
    soid_fx::noise::rank(1, NoiseColor::Equal, 1f32),
    soid_fx::noise::rank(0, NoiseColor::Blue, 1f32),
    soid_fx::noise::rank(1, NoiseColor::Blue, 0.5f32),
  ]);
  let reverbs: Vec<ReverbParams> = vec![];

  let delays_note = vec![delay::passthrough];
  let delays_room = vec![];
  let reverbs_note: Vec<ReverbParams> = vec![];
  let reverbs_room: Vec<ReverbParams> = vec![];

  let stem: Stem2 = (
    melody,
    soids,
    expr(arf),
    get_bp(cps, melody, arf, len_cycles),
    knob_mods,
    delays_note,
    delays_room,
    reverbs_note,
    reverbs_room,
  );

  Renderable2::Instance(stem)
}
