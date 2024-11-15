use super::*;
use crate::druid::{self, soid_fx, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::types::synthesis::{ModifiersHolder, Soids};

/// noise component
fn stem_hidden<'render>(arf: &Arf, melody: &'render Melody<Note>) -> Stem<'render> {
  let soids = druidic_soids::id();
  let mut rng = thread_rng();
  let expr = (vec![visibility_gain(Visibility::Background)], vec![1f32], vec![0f32]);

  let feel: Feel = Feel {
    bp: (vec![MFf], vec![NFf]),
    modifiers: (vec![], vec![], vec![], vec![]),
    clippers: (0f32, 1f32),
  };
  let mut knob_mods: KnobMods = KnobMods::unit();
  // principal layer
  knob_mods.0.push((
    Knob {
      a: 0f32,
      b: 0.3f32,
      c: 0f32,
    },
    ranger::amod_pluck,
  ));
  // attenuation layer
  knob_mods.0.push((
    Knob {
      a: match arf.presence {
        Presence::Staccatto => in_range(&mut rng, 0f32, 0.33f32),
        Presence::Legato => in_range(&mut rng, 0.33f32, 0.66f32),
        Presence::Tenuto => in_range(&mut rng, 0.88f32, 1f32),
      },
      b: match arf.energy {
        Energy::High => in_range(&mut rng, 0f32, 0.33f32),
        Energy::Medium => in_range(&mut rng, 0.33f32, 0.66f32),
        Energy::Low => in_range(&mut rng, 0.88f32, 1f32),
      },
      c: 0f32,
    },
    ranger::amod_pluck,
  ));
  let noises: Vec<Soids> = (0..10).map(|register| soid_fx::noise::rank(register, NoiseColor::Violet, 0.1f32)).collect();
  let soids = soid_fx::concat(&noises);

  (melody, soids, expr, feel, knob_mods, vec![delay::passthrough])
}

/// sine component
fn stem_foreground<'render>(arf: &Arf, melody: &'render Melody<Note>) -> Stem<'render> {
  let mut rng = thread_rng();
  let peak1: f32 = 6f32;
  let peak2: f32 = 7f32;
  let peak3: f32 = (peak1.sqrt() + peak2.sqrt()).powi(2i32);

  let mut compound_ratios: Vec<(f32, f32)> = vec![
    (peak1, 0.22f32),
    (peak2, 0.1f32),
    (2f32, 0.7f32),
    (3f32, 0.4f32),
    (2f32, 0.5f32),
    (3f32, 0.4f32),
    (0.2f32, 0.6f32),
    (0.5f32, 0.6f32),
    (0.33f32, 0.5f32),
    (0.75f32, 0.3f32),
  ];

  let soids: Soids = compound_ratios.iter().fold(druidic_soids::id(), |soids, (k, gain)| {
    soid_fx::ratio::constant(&soids, *k, *gain)
  });
  let feelsoid_fx: Feel = Feel {
    bp: (vec![MFf], vec![NFf]),
    modifiers: (vec![], vec![], vec![], vec![]),
    clippers: (0f32, 1f32),
  };

  let mut knob_mods: KnobMods = KnobMods::unit();
  knob_mods.0.push((
    Knob {
      a: 0.8f32,
      b: 0.1f32,
      c: 0f32,
    },
    ranger::amod_pluck,
  ));
  knob_mods.0.push((
    Knob {
      a: 0.5f32,
      b: 0.3f32,
      c: 0f32,
    },
    ranger::amod_pluck,
  ));
  let expr = (vec![visibility_gain(Visibility::Foreground)], vec![1f32], vec![0f32]);
  // let soids = soid_fx::detune::reece(&soids, 3, 0.5f32);
  (melody, soids, expr, feelsoid_fx, knob_mods, vec![delay::passthrough])
}

pub fn renderable<'render>(melody: &'render Melody<Note>, arf: &Arf) -> Renderable<'render> {
  Renderable::Group(vec![stem_hidden(arf, melody), stem_foreground(arf, melody)])
}
