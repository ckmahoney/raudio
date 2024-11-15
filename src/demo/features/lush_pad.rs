use super::*;
use crate::analysis::delay;
use crate::complexity;
use crate::files;

static demo_name: &str = "lush-pad";

use crate::analysis::volume::db_to_amp;
use crate::presets::Instrument;
use crate::render::{self, Renderable};
use crate::reverb;
use crate::types::render::{Feel, Melody, Stem};
use crate::types::synthesis::{
  Ampl, Bandpass, Direction, Duration, Ely, FilterPoint, Freq, Frex, GlideLen, Monae, Mote, Note, Register, Soids, Tone,
};

use crate::presets::mountain;

fn chords_melody_loop() -> Melody<Note> {
  let tala: Vec<Duration> = vec![
    (2i32, 1i32),
    (3i32, 1i32),
    (3i32, 1i32),
    (2i32, 1i32),
    (3i32, 1i32),
    (3i32, 1i32),
    (2i32, 1i32),
    (3i32, 1i32),
    (3i32, 1i32),
    (2i32, 1i32),
    (3i32, 1i32),
    (3i32, 1i32),
  ]
  .into_iter()
  .map(|(a, b)| (a * 3i32, b))
  .collect();

  let amps: Vec<Ampl> = vec![1f32; tala.len()];

  let line_1: Vec<Tone> = vec![
    (6, (1i8, 0i8, 1i8)),
    (7, (0i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
    (6, (1i8, 0i8, 1i8)),
    (7, (0i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
    (6, (1i8, 0i8, 1i8)),
    (7, (0i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
    (6, (1i8, 0i8, 1i8)),
    (7, (0i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
  ];

  let line_2: Vec<Tone> = vec![
    (6, (0i8, 0i8, 3i8)),
    (7, (0i8, 0i8, 3i8)),
    (8, (0i8, 0i8, 3i8)),
    (6, (0i8, 0i8, 3i8)),
    (7, (0i8, 0i8, 3i8)),
    (8, (0i8, 0i8, 3i8)),
    (6, (0i8, 0i8, 3i8)),
    (7, (0i8, 0i8, 3i8)),
    (8, (0i8, 0i8, 3i8)),
    (6, (0i8, 0i8, 3i8)),
    (7, (0i8, 0i8, 3i8)),
    (8, (0i8, 0i8, 3i8)),
  ];

  let line_3: Vec<Tone> = vec![
    (6, (0i8, 0i8, 5i8)),
    (7, (0i8, 0i8, 5i8)),
    (8, (0i8, 0i8, 5i8)),
    (6, (0i8, 0i8, 5i8)),
    (7, (0i8, 0i8, 5i8)),
    (8, (0i8, 0i8, 5i8)),
    (6, (0i8, 0i8, 5i8)),
    (7, (0i8, 0i8, 5i8)),
    (8, (0i8, 0i8, 5i8)),
    (6, (0i8, 0i8, 5i8)),
    (7, (0i8, 0i8, 5i8)),
    (8, (0i8, 0i8, 5i8)),
  ];

  vec![
    zip_line(tala.clone(), line_1, amps.clone()),
    zip_line(tala.clone(), line_2, amps.clone()),
    zip_line(tala.clone(), line_3, amps.clone()),
  ]
}

fn chords_melody_loop_half() -> Melody<Note> {
  let tala: Vec<Duration> = vec![
    (2i32, 1i32),
    (3i32, 1i32),
    (3i32, 1i32),
    (2i32, 1i32),
    (3i32, 1i32),
    (3i32, 1i32),
  ]
  .into_iter()
  .map(|(a, b)| (a * 3i32, b))
  .collect();

  let amps: Vec<Ampl> = vec![1f32; tala.len()];

  let line_1: Vec<Tone> = vec![
    (6, (1i8, 0i8, 1i8)),
    (7, (0i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
    (6, (1i8, 0i8, 1i8)),
    (7, (0i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
  ];

  let line_2: Vec<Tone> = vec![
    (6, (0i8, 0i8, 3i8)),
    (7, (0i8, 0i8, 3i8)),
    (8, (0i8, 0i8, 3i8)),
    (6, (0i8, 0i8, 3i8)),
    (7, (0i8, 0i8, 3i8)),
    (8, (0i8, 0i8, 3i8)),
  ];

  let line_3: Vec<Tone> = vec![
    (6, (0i8, 0i8, 5i8)),
    (7, (0i8, 0i8, 5i8)),
    (8, (0i8, 0i8, 5i8)),
    (6, (0i8, 0i8, 5i8)),
    (7, (0i8, 0i8, 5i8)),
    (8, (0i8, 0i8, 5i8)),
  ];

  vec![
    zip_line(tala.clone(), line_1, amps.clone()),
    zip_line(tala.clone(), line_2, amps.clone()),
    zip_line(tala.clone(), line_3, amps.clone()),
  ]
}

fn chords_melody_short() -> Melody<Note> {
  let tala: Vec<Duration> = vec![
    (1i32, 1i32),
    (1i32, 1i32),
    (1i32, 1i32),
    (1i32, 1i32),
    (1i32, 1i32),
    (1i32, 1i32),
    (1i32, 1i32),
    (1i32, 1i32),
  ];

  let amps: Vec<Ampl> = vec![1f32; tala.len()];

  let line_1: Vec<Tone> = vec![
    (7, (0i8, 0i8, 1i8)),
    (7, (-1i8, 0i8, 1i8)),
    (6, (0i8, 0i8, 1i8)),
    (6, (1i8, 0i8, 1i8)),
    (7, (0i8, 0i8, 1i8)),
    (7, (-1i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
    (8, (1i8, 0i8, 1i8)),
  ];

  let line_2: Vec<Tone> = vec![
    (7, (0i8, 0i8, 3i8)),
    (7, (-1i8, 0i8, 3i8)),
    (6, (0i8, 0i8, 3i8)),
    (6, (1i8, 0i8, 3i8)),
    (7, (0i8, 0i8, 3i8)),
    (7, (-1i8, 0i8, 3i8)),
    (8, (0i8, 0i8, 3i8)),
    (8, (1i8, 0i8, 3i8)),
  ];

  let line_3: Vec<Tone> = vec![
    (6, (0i8, 0i8, 5i8)),
    (6, (-1i8, 0i8, 5i8)),
    (6, (0i8, 0i8, 5i8)),
    (6, (1i8, 0i8, 5i8)),
    (7, (0i8, 0i8, 5i8)),
    (7, (-1i8, 0i8, 5i8)),
    (8, (0i8, 0i8, 5i8)),
    (8, (1i8, 0i8, 5i8)),
  ];

  vec![
    zip_line(tala.clone(), line_1, amps.clone()),
    zip_line(tala.clone(), line_2, amps.clone()),
    zip_line(tala.clone(), line_3, amps.clone()),
  ]
}

fn chords_arf(visibility: Visibility, energy: Energy, presence: Presence) -> Arf {
  Arf {
    mode: Mode::Melodic,
    role: Role::Chords,
    register: 6,
    visibility,
    energy,
    presence,
  }
}
#[test]
fn test_iter() {
  let path: String = location(demo_name);
  let cps: f32 = 2.0;
  let root: f32 = 1.12;
  let preset = Preset::Mountain;
  files::with_dir(&path);

  let label = "mountain_simple_melody";
  let melody = chords_melody_loop_half();
  let vs = vec![Visibility::Visible, Visibility::Hidden];
  let vs = vec![Visibility::Visible];
  let es = vec![Energy::High, Energy::Low];
  let es = vec![Energy::High];
  let ps = vec![Presence::Staccatto];
  let arfs = prism::iter_vep(&label, Role::Chords, Mode::Melodic, &melody, &vs, &es, &ps);

  prism::run(&path, root, cps, &melody, &arfs, &preset)
}
