use std::os::unix::thread;

use super::*;

pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  let soids = druidic_soids::upto(2);

  let expr = (
    vec![db_to_amp(-4.5f32) * visibility_gain(arf.visibility)],
    vec![1f32],
    vec![0f32],
  );

  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  let mut rng: ThreadRng = thread_rng();

  // Principal layer
  knob_mods.0.push((
    KnobMacro {
      a: match arf.presence {
        Presence::Staccatto => [0.3f32, 0.5f32],
        Presence::Legato => [0.5f32, 0.9f32],
        Presence::Tenuto => [0.9f32, 1f32],
      },
      b: match arf.energy {
        Energy::High => [0f32, 0.33f32],
        Energy::Medium => [0.33f32, 0.5f32],
        Energy::Low => [0.5f32, 0.66f32],
      },
      c: [0f32, 0f32],
      ma: grab_variant(vec![MacroMotion::Constant]),
      mb: grab_variant(vec![
        MacroMotion::Forward,
        MacroMotion::Reverse,
        MacroMotion::Random,
        MacroMotion::Constant,
      ]),
      mc: MacroMotion::Random,
    },
    if let Presence::Tenuto = arf.presence {
      ranger::amod_burp
    } else {
      ranger::amod_pluck2
    },
  ));

  // Attenuation layer
  knob_mods.0.push((
    KnobMacro {
      a: match arf.presence {
        Presence::Staccatto => [0.5f32, 0.7f32],
        Presence::Legato => [0.7f32, 0.85f32],
        Presence::Tenuto => [0.85f32, 1f32],
      },
      b: match arf.energy {
        Energy::High => [0f32, 0.11f32],
        Energy::Medium => [0.11f32, 0.33f32],
        Energy::Low => [0.33f32, 0.5f32],
      },
      c: [0f32, 0f32],
      ma: grab_variant(vec![MacroMotion::Constant]),
      mb: grab_variant(vec![
        MacroMotion::Forward,
        MacroMotion::Reverse,
        MacroMotion::Random,
        MacroMotion::Constant,
      ]),
      mc: MacroMotion::Random,
    },
    ranger::amod_burp,
  ));

  // Secondary layer
  knob_mods.1.push((
    KnobMacro {
      a: match arf.energy {
        Energy::High => [0f32, 0.75f32],
        Energy::Medium => [0f32, 0.5f32],
        Energy::Low => [0.01f32, 0.1f32],
      },
      b: match arf.presence {
        Presence::Staccatto => [0.1f32, 0.2f32],
        Presence::Legato => [0.1f32, 0.2f32],
        Presence::Tenuto => [0.1f32, 0.4f32],
      },
      c: [0f32, 0f32],
      ma: grab_variant(vec![MacroMotion::Constant]),
      mb: grab_variant(vec![
        MacroMotion::Forward,
        MacroMotion::Reverse,
        MacroMotion::Random,
        MacroMotion::Constant,
      ]),
      mc: MacroMotion::Random,
    },
    ranger::fmod_sweepdown,
  ));

  let delays_note = vec![delay::passthrough];
  let delays_room = vec![];
  let reverbs_note: Vec<ReverbParams> = vec![];
  let reverbs_room: Vec<ReverbParams> = vec![];

  Renderable2::Instance((
    melody,
    soids,
    expr,
    bp2_unit(),
    knob_mods,
    delays_note,
    delays_room,
    reverbs_note,
    reverbs_room,
  ))
}
