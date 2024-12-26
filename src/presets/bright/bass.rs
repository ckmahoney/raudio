use super::*;
use crate::analysis::in_range_usize;
use crate::druid::{self, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::types::synthesis::{ModifiersHolder, Soids};
use std::os::unix::thread;

pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  let mut rng = thread_rng();
  // use a short mullet here and extend it with extra soids next
  let mullet = get_mullet(11i8, arf.energy);

  let soids = match arf.visibility {
    Visibility::Hidden => druidic_soids::octave(mullet),
    Visibility::Background => druidic_soids::octave(mullet),
    Visibility::Foreground => druidic_soids::overs_square(mullet),
    Visibility::Visible => druidic_soids::overs_square(mullet),
  };

  let soids = soid_fx::fmod::sawtooth(&soids, 3);
  let soids = soid_fx::fmod::triangle(&soids, 3);

  let detune = |soids: &Soids| -> Soids {
    let mut rng = thread_rng();
    soid_fx::fmod::reece2(soids, in_range(&mut rng, 0.005, 0.01))
  };
  let detune2 = |soids: &Soids| -> Soids {
    let mut rng = thread_rng();
    soid_fx::fmod::reece2(soids, in_range(&mut rng, 0.01, 0.02))
  };
  let soids = soid_fx::filter_do(&soids, detune, |soid| soid.1.log2() < 2f32);
  let soids = soid_fx::filter_do(&soids, detune, |soid| soid.1.log2() > 7f32);
  let soids = soid_fx::amod::gain(&soids, 0.1);

  // let soids = soid_fx::fmod::triangle(&soids, 3);

  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  let expr = (vec![0.2f32], vec![1f32], vec![0f32]);
  let delays_note = vec![];
  let delays_room = vec![];
  let reverbs_note: Vec<ReverbParams> = vec![];
  let reverbs_room: Vec<ReverbParams> = vec![];

  let bp = BrightCon::get_bp(conf.cps, melody, arf);

  let stem = (
    melody,
    soids,
    expr,
    BrightCon::get_bp(conf.cps, melody, arf),
    knob_mods,
    delays_note,
    delays_room,
    reverbs_note,
    reverbs_room,
  );

  Renderable2::Instance(stem)
}
