use std::os::unix::thread;

use super::*;

fn knob_amp() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  (
    Knob {
      a: 1f32,
      b: 1f32,
      c: 0f32,
    },
    ranger::amod_pluck,
  )
}

/// Featured component
pub fn stem_visible<'render>(arf: &Arf, melody: &'render Melody<Note>) -> Stem<'render> {
  let mut rng = thread_rng();

  let soids: Soids = soid_fx::concat(&vec![
    soid_fx::noise::rank(0, NoiseColor::Violet, 1f32 / 7f32),
    soid_fx::noise::rank(1, NoiseColor::Pink, 1f32 / 11f32),
  ]);

  let expr = (
    vec![visibility_gain(Visibility::Foreground) * visibility_gain(arf.visibility)],
    vec![1f32],
    vec![0f32],
  );
  let feel: Feel = Feel {
    bp: (vec![MFf], vec![NFf]),
    modifiers: (vec![], vec![], vec![], vec![]),
    clippers: (0f32, 1f32),
  };

  let mut knob_mods: KnobMods = KnobMods::unit();
  knob_mods.0.push((
    Knob {
      a: 0.05f32,
      b: 1f32,
      c: 0f32,
    },
    ranger::amod_pluck,
  ));
  knob_mods.0.push((
    Knob {
      a: 0.5f32,
      b: 1f32,
      c: 0f32,
    },
    ranger::amod_pluck,
  ));
  knob_mods.0.push((
    Knob {
      a: 0.15f32,
      b: 1f32,
      c: 0f32,
    },
    ranger::amod_pluck,
  ));
  (melody, soids, expr, feel, knob_mods, vec![delay::passthrough])
}

/// Defines the constituent stems to create a complex kick drum
/// Components include:
///  - a transient id element
pub fn renderable<'render>(melody: &'render Melody<Note>, arf: &Arf) -> Renderable<'render> {
  Renderable::Group(vec![stem_visible(&arf, &melody)])
}
