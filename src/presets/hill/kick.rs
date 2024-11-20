use super::*;

fn gain(arf: &Arf) -> f32 {
  let x = match arf.presence {
    Presence::Tenuto => 2f32,
    Presence::Legato => 1.5f32,
    Presence::Staccatto => 1f32,
  };

  let mul = match arf.energy {
    Energy::High => 2f32,
    Energy::Medium => 1.5f32,
    Energy::Low => 1f32,
  };

  x * mul
}

pub fn stem_bass<'render>(arf: &Arf, melody: &'render Melody<Note>) -> Stem2<'render> {
  let soids = druidic_soids::upto(2);

  let expr = (vec![db_to_amp(-4.5f32) * gain(arf)], vec![1f32], vec![0f32]);

  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  let mut rng: ThreadRng = thread_rng();

  // Principal layer
  knob_mods.0.push((
    KnobMacro {
      a: match arf.presence {
        Presence::Staccatto => [0.1f32, 0.33f32], // Using in_range arguments directly
        Presence::Legato => [0.7f32, 0.94f32],
        Presence::Tenuto => [0.9f32, 1f32],
      },
      b: match arf.energy {
        Energy::High => [0f32, 0.33f32], // Using in_range arguments directly
        Energy::Medium => [0.33f32, 0.5f32],
        Energy::Low => [0.5f32, 0.66f32],
      },
      c: [0f32, 0f32], // Static value as [0, 0]
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    ranger::amod_burp,
  ));

  // Attenuation layer
  knob_mods.0.push((
    KnobMacro {
      a: [0.95f32, 1f32], // Using in_range arguments directly
      b: match arf.energy {
        Energy::High => [0f32, 0.33f32], // Using in_range arguments directly
        Energy::Medium => [0.33f32, 0.5f32],
        Energy::Low => [0.5f32, 0.66f32],
      },
      c: [0f32, 0f32], // Static value as [0, 0]
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    if let Energy::High = arf.energy {
      ranger::amod_burp
    } else {
      ranger::amod_pluck
    },
  ));

  // Secondary layer
  knob_mods.1.push((
    KnobMacro {
      a: match arf.energy {
        Energy::High => [0.33f32, 0.5f32], 
        Energy::Medium => [0.1f32, 0.3f32],
        Energy::Low => [0.01f32, 0.1f32],
      },
      b: match arf.presence {
        Presence::Staccatto => [0.4f32, 0.7f32], 
        Presence::Legato => [0.3f32, 0.3f32], 
        Presence::Tenuto => [0.4f32, 0.4f32],
      },
      c: [0f32, 0f32],
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    ranger::fmod_sweepdown,
  ));

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

/**
 *
 * Primary concepts to cover in a synth
 * Can be combined as a list of layers
 *
 * lifespan
 * centroid
 * height
 * filters
 * spectral distribution
 *
 *
 * distortion
 * saturation
 * reverb
 */

/*
needs
to have punch, decay, and body as primary facets
*/
pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  Renderable2::Group(vec![
    stem_bass(arf, melody),
  ])
}
