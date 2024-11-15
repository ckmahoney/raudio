use super::*;
use crate::druid::{self, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::types::synthesis::{ModifiersHolder, Soids};

pub mod v1 {
  use super::*;

  pub fn expr(arf: &Arf) -> Expr {
    (vec![visibility_gain(arf.visibility)], vec![1f32], vec![0f32])
  }

  /// Selects the number of resonant nodes to add
  fn amp_reso_gen(modders: &mut KnobMods, visibility: Visibility, energy: Energy, presence: Presence) {
    let n = match energy {
      Energy::Low => 2,
      Energy::Medium => 3,
      Energy::High => 5,
    };
    let mut rng = thread_rng();
    for i in 0..n {
      let a: f32 = match visibility {
        Visibility::Visible => rng.gen::<f32>() / 5f32,
        Visibility::Foreground => rng.gen::<f32>() / 3f32,
        Visibility::Background => rng.gen::<f32>() / 2f32,
        Visibility::Hidden => 0.5f32 + rng.gen::<f32>() / 2f32,
      };
      let b: f32 = match energy {
        Energy::Low => rng.gen::<f32>() / 2f32,
        Energy::Medium => rng.gen::<f32>() / 3f32,
        Energy::High => rng.gen::<f32>() / 5f32,
      };
      modders.0.push((Knob { a: a, b: b, c: 0.0 }, ranger::amod_peak))
    }
  }

  pub fn renderable<'render>(cps: f32, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
    let mut knob_mods: KnobMods2 = KnobMods2::unit();
    let mut knob_mods2: KnobMods2 = KnobMods2::unit();
    let mut rng: ThreadRng = thread_rng();

    // Principal layer
    knob_mods.0.push((
      KnobMacro {
        a: match arf.energy {
          Energy::High => [0.8f32, 1f32],
          Energy::Medium => [0.5f32, 0.8f32],
          Energy::Low => [0.35f32, 0.5f32],
        },
        b: [0.1f32, 0.2f32], // Using the arguments to in_range directly
        c: [0f32, 0f32],     // Static value as [0, 0]
        ma: MacroMotion::Constant,
        mb: MacroMotion::Constant,
        mc: MacroMotion::Random,
      },
      ranger::amod_burp,
    ));

    // // double
    knob_mods2.0.push((
      KnobMacro {
        a: match arf.energy {
          Energy::High => [0.7f32, 0.8f32],
          Energy::Medium => [0.5f32, 0.7f32],
          Energy::Low => [0.3f32, 0.5f32],
        },
        b: [0.75f32, 1f32],
        c: [0f32, 0f32],
        ma: MacroMotion::Random,
        mb: MacroMotion::Random,
        mc: MacroMotion::Random,
      },
      ranger::amod_burp,
    ));

    knob_mods2.0.push((
      KnobMacro {
        a: match arf.energy {
          Energy::High => [0.7f32, 0.8f32],
          Energy::Medium => [0.5f32, 0.7f32],
          Energy::Low => [0.3f32, 0.5f32],
        },
        b: [0.75f32, 1f32],
        c: [0f32, 0f32],
        ma: MacroMotion::Random,
        mb: MacroMotion::Random,
        mc: MacroMotion::Random,
      },
      ranger::amod_burp,
    ));

    // Attenuation layer
    knob_mods.0.push((
      KnobMacro {
        a: match arf.energy {
          Energy::High => [0.3f32, 1f32],
          Energy::Medium => [0.5f32, 0.8f32],
          Energy::Low => [0.35f32, 0.5f32],
        },
        b: match arf.visibility {
          Visibility::Visible => [0.15f32, 0.5f32 + 0.5f32 * rng.gen::<f32>() / 5f32],
          Visibility::Foreground => [0.25f32, 0.25f32 + 0.5f32 * rng.gen::<f32>() / 3f32],
          Visibility::Background => [0.35f32, 0.15f32 + 0.5f32 * rng.gen::<f32>() / 2f32],
          Visibility::Hidden => [0.45f32, 0.05f32 + 0.5f32 * rng.gen::<f32>() / 2f32],
        },
        c: [0f32, 0f32],
        ma: MacroMotion::Random,
        mb: MacroMotion::Random,
        mc: MacroMotion::Random,
      },
      ranger::amod_fadeout,
    ));

    // Attenuation layer
    knob_mods.0.push((
      KnobMacro {
        a: match arf.energy {
          Energy::High => [0.5f32, 1f32],
          Energy::Medium => [0.5f32, 0.8f32],
          Energy::Low => [0.5f32, 0.5f32],
        },
        b: match arf.visibility {
          Visibility::Visible => [0.15f32, 0.5f32 + 0.5f32 * rng.gen::<f32>() / 5f32],
          Visibility::Foreground => [0.25f32, 0.25f32 + 0.5f32 * rng.gen::<f32>() / 3f32],
          Visibility::Background => [0.35f32, 0.15f32 + 0.5f32 * rng.gen::<f32>() / 2f32],
          Visibility::Hidden => [0.45f32, 0.05f32 + 0.5f32 * rng.gen::<f32>() / 2f32],
        },
        c: [0f32, 0f32],
        ma: MacroMotion::Random,
        mb: MacroMotion::Random,
        mc: MacroMotion::Random,
      },
      ranger::amod_fadeout,
    ));

    // Detail layer
    knob_mods.0.push((
      KnobMacro {
        a: match arf.presence {
          Presence::Staccatto => [0.33f32, 0.66f32], // Using the arguments to in_range directly
          Presence::Legato => [0.53f32, 0.76f32],    // Using the arguments to in_range directly
          Presence::Tenuto => [0.88f32, 1f32],       // Using the arguments to in_range directly
        },
        b: match arf.energy {
          Energy::High => [0f32, 0.33f32],     // Using the arguments to in_range directly
          Energy::Medium => [0.33f32, 0.5f32], // Using the arguments to in_range directly
          Energy::Low => [0.5f32, 1f32],       // Using the arguments to in_range directly
        },
        c: [0f32, 0f32], // Static value as [0, 0]
        ma: MacroMotion::Random,
        mb: MacroMotion::Random,
        mc: MacroMotion::Random,
      },
      ranger::amod_oscillation_sine,
    ));

    let len_cycles: f32 = time::count_cycles(&melody[0]);

    let height = 12i32;
    let noise_type = match arf.energy {
      Energy::High => druidic_soids::NoiseType::Pink,
      Energy::Medium => druidic_soids::NoiseType::Equal,
      Energy::Low => druidic_soids::NoiseType::Violet,
    };

    let soids = druidic_soids::noise(2f32.powi(height), noise_type);
    let soids2 = soid_fx::noise::rank(0, NoiseColor::Violet, 0.3f32);

    let delays_note = vec![delay::passthrough];
    let delays_room = vec![];
    let reverbs_note: Vec<ReverbParams> = vec![];
    let reverbs_room: Vec<ReverbParams> = vec![];

    let stem = (
      melody,
      soids,
      expr(arf),
      get_bp(cps, melody, arf, len_cycles),
      knob_mods,
      delays_note.clone(),
      delays_room.clone(),
      reverbs_note.clone(),
      reverbs_room.clone(),
    );

    let stem2 = (
      melody,
      soids2,
      (
        vec![visibility_gain(Visibility::Hidden) * visibility_gain(arf.visibility)],
        vec![1f32],
        vec![0f32],
      ),
      get_bp(cps, melody, arf, len_cycles),
      knob_mods2,
      delays_note,
      delays_room,
      reverbs_note,
      reverbs_room,
    );

    Renderable2::Group(vec![stem, stem2])
  }
}

// pub mod v2 {

use super::*;

pub fn expr(arf: &Arf) -> Expr {
  (vec![visibility_gain(arf.visibility)], vec![1f32], vec![0f32])
}

/// Selects the number of resonant nodes to add
fn amp_reso_gen(modders: &mut KnobMods, visibility: Visibility, energy: Energy, presence: Presence) {
  let n = match energy {
    Energy::Low => 2,
    Energy::Medium => 3,
    Energy::High => 5,
  };
  let mut rng = thread_rng();
  for i in 0..n {
    let a: f32 = match visibility {
      Visibility::Visible => rng.gen::<f32>() / 5f32,
      Visibility::Foreground => rng.gen::<f32>() / 3f32,
      Visibility::Background => rng.gen::<f32>() / 2f32,
      Visibility::Hidden => 0.5f32 + rng.gen::<f32>() / 2f32,
    };
    let b: f32 = match energy {
      Energy::Low => rng.gen::<f32>() / 2f32,
      Energy::Medium => rng.gen::<f32>() / 3f32,
      Energy::High => rng.gen::<f32>() / 5f32,
    };
    modders.0.push((Knob { a: a, b: b, c: 0.0 }, ranger::amod_peak))
  }
}

pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  let mut rng: ThreadRng = thread_rng();

  // Principal layer
  knob_mods.0.push((
    KnobMacro {
      a: match arf.energy {
        Energy::High => [0.8f32, 1f32],
        Energy::Medium => [0.5f32, 0.8f32],
        Energy::Low => [0.35f32, 0.5f32],
      },
      b: [0.1f32, 0.2f32], // Using the arguments to in_range directly
      c: [0f32, 0f32],     // Static value as [0, 0]
      ma: MacroMotion::Constant,
      mb: MacroMotion::Constant,
      mc: MacroMotion::Random,
    },
    ranger::amod_burp,
  ));

  // Attenuation layer
  knob_mods.0.push((
    KnobMacro {
      a: match arf.energy {
        Energy::High => [0.3f32, 1f32],
        Energy::Medium => [0.5f32, 0.8f32],
        Energy::Low => [0.35f32, 0.5f32],
      },
      b: [0.2f32, 0.8f32], // Using the arguments to in_range directly
      c: [0f32, 0f32],     // Static value as [0, 0]
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    ranger::amod_burp,
  ));

  // Attenuation layer
  knob_mods.0.push((
    KnobMacro {
      a: match arf.energy {
        Energy::High => [0.3f32, 1f32],
        Energy::Medium => [0.5f32, 0.8f32],
        Energy::Low => [0.35f32, 0.5f32],
      },
      b: match arf.visibility {
        Visibility::Visible => [0.15f32, 0.5f32 + 0.5f32 * rng.gen::<f32>() / 5f32],
        Visibility::Foreground => [0.25f32, 0.25f32 + 0.5f32 * rng.gen::<f32>() / 3f32],
        Visibility::Background => [0.35f32, 0.15f32 + 0.5f32 * rng.gen::<f32>() / 2f32],
        Visibility::Hidden => [0.45f32, 0.05f32 + 0.5f32 * rng.gen::<f32>() / 2f32],
      },
      c: [0f32, 0f32],
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    ranger::amod_fadeout,
  ));

  // Detail layer
  knob_mods.0.push((
    KnobMacro {
      a: match arf.presence {
        Presence::Staccatto => [0.33f32, 0.5f32], // Using the arguments to in_range directly
        Presence::Legato => [0.23f32, 0.3f32],    // Using the arguments to in_range directly
        Presence::Tenuto => [0.08f32, 0.2f32],    // Using the arguments to in_range directly
      },
      b: match arf.energy {
        Energy::High => [0.7f32, 1f32],      // Using the arguments to in_range directly
        Energy::Medium => [0.33f32, 0.7f32], // Using the arguments to in_range directly
        Energy::Low => [0.15f32, 0.33f32],   // Using the arguments to in_range directly
      },
      c: [0f32, 0f32], // Static value as [0, 0]
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    ranger::amod_oscillation_sine,
  ));

  let len_cycles: f32 = time::count_cycles(&melody[0]);

  let height = match arf.energy {
    Energy::High => 10i32,
    Energy::Medium => 11i32,
    Energy::Low => 12i32,
  };

  let noise_type = druidic_soids::NoiseType::Violet;

  let soids = soid_fx::concat(&vec![
    druidic_soids::noise(2f32.powi(height), noise_type),
    soid_fx::noise::resof(2f32 * rng.gen::<f32>()),
    soid_fx::noise::resof(1f32 + 2f32 * rng.gen::<f32>()),
  ]);

  let delays_note = vec![delay::passthrough];
  let delays_room = vec![];
  let reverbs_note: Vec<ReverbParams> = vec![];
  let reverbs_room: Vec<ReverbParams> = vec![];

  let stem = (
    melody,
    soids,
    expr(arf),
    get_bp(conf.cps, melody, arf, len_cycles),
    knob_mods,
    delays_note,
    delays_room,
    reverbs_note,
    reverbs_room,
  );
  Renderable2::Instance(stem)
}
// }
