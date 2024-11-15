use super::*;
use crate::analysis::delay;
use crate::complexity;
use crate::files;

static demo_name: &str = "blip-lead";

use crate::analysis::volume::db_to_amp;
use crate::presets::Instrument;
use crate::render::{self, Renderable};
use crate::reverb;
use crate::types::render::{Feel, Melody, Stem};
use crate::types::synthesis::{
  Ampl, Bandpass, Direction, Duration, Ely, FilterPoint, Freq, Frex, GlideLen, Monae, Mote, Note, Register, Soids, Tone,
};

use crate::presets::mountain;

fn lead_melody_long() -> Melody<Note> {
  let tala: Vec<Duration> = vec![
    (2i32, 1i32),
    (3i32, 1i32),
    (3i32, 1i32),
    (4i32, 1i32),
    (4i32, 1i32),
    (8i32, 1i32),
    (8i32, 1i32),
    (4i32, 1i32),
  ];

  let amps: Vec<Ampl> = vec![1f32; tala.len()];

  let line_1: Vec<Tone> = vec![
    (7, (0i8, 0i8, 1i8)),
    (6, (-1i8, 0i8, 1i8)),
    (5, (0i8, 0i8, 1i8)),
    (6, (1i8, 0i8, 1i8)),
    (7, (0i8, 0i8, 1i8)),
    (8, (-1i8, 0i8, 1i8)),
    (7, (0i8, 0i8, 1i8)),
    (8, (1i8, 0i8, 1i8)),
  ];

  vec![zip_line(tala.clone(), line_1, amps.clone())]
}

#[test]
fn test_iter() {
  let path: String = location(demo_name);
  let cps: f32 = 2.0;
  let root: f32 = 1.12;
  let preset = Preset::Mountain;
  files::with_dir(&path);

  let label = "mountain_lead_melody";
  let melody = lead_melody_long();
  let vs = vec![Visibility::Visible, Visibility::Hidden];
  let vs = vec![Visibility::Visible];
  let es = vec![Energy::High, Energy::Low];
  let es = vec![Energy::High];
  let ps = vec![Presence::Staccatto];
  // let arfs = prism::iter_vep(&label, Role::Lead, Mode::Melodic, &melody, &vs, &es, &ps);
  let arfs = prism::iter_all_vep(&label, Role::Lead, Mode::Melodic, &melody);

  prism::run(&path, root, cps, &melody, &arfs, &preset)
}
