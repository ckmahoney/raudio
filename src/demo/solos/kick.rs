use super::*;
use crate::analysis::delay;
use crate::complexity;
use crate::files;

static demo_name: &str = "just-kick";

use crate::analysis::volume::db_to_amp;
use crate::presets::Instrument;
use crate::render::{self, Renderable};
use crate::reverb;
use crate::types::render::{Feel, Melody, Stem};
use crate::types::synthesis::{
  Ampl, Bandpass, Direction, Duration, Ely, FilterPoint, Freq, Frex, GlideLen, Monae, Mote, Note, Register, Soids, Tone,
};

fn kick_melody() -> Melody<Note> {
  let tala: Vec<Duration> = vec![
    (1i32, 1i32),
    (1i32, 1i32),
    (1i32, 1i32),
    (1i32, 1i32),
    (1i32, 1i32),
    (3i32, 4i32),
    (5i32, 4i32),
    (1i32, 1i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 1i32),
    (2i32, 1i32),
  ];

  let amps: Vec<Ampl> = vec![
    1f32, 1f32, 1f32, 1f32, 1f32, 0.66f32, 0.66f32, 1f32, 0.66f32, 1f32, 0.66f32, 0.75f32,
  ]
  .iter()
  .map(|x| x * db_to_amp(-6f32))
  .collect::<Vec<f32>>();

  let tones: Vec<Tone> = vec![
    (5, (0i8, 0i8, 1i8)),
    (5, (0i8, 0i8, 1i8)),
    (5, (0i8, 0i8, 1i8)),
    (5, (0i8, 0i8, 1i8)),
    (5, (0i8, 0i8, 1i8)),
    (5, (0i8, 0i8, 5i8)),
    (5, (0i8, 0i8, 3i8)),
    (5, (1i8, 0i8, 1i8)),
    (5, (1i8, 0i8, 5i8)),
    (5, (1i8, 0i8, 3i8)),
    (5, (-1i8, 0i8, 5i8)),
    (5, (-1i8, 0i8, 1i8)),
  ];

  vec![zip_line(tala, tones, amps)]
}

#[test]
fn test_iter() {
  let path: String = location(demo_name);
  let cps: f32 = 2.0;
  let root: f32 = 1.12;
  let preset = Preset::Mountain;
  files::with_dir(&path);

  let label = "mountain_simple_melody";
  let melody = kick_melody();
  let arfs = prism::iter_all_vep(&label, Role::Kick, Mode::Enharmonic, &melody);

  let num_threads = prism::get_par_thread_count().min(4);

  if num_threads > 1 {
    let pool = ThreadPoolBuilder::new().num_threads(num_threads).build().unwrap();

    pool.install(|| {
      arfs.par_iter().for_each(|arf| {
        prism::render_labelled_arf(&path, root, cps, &melody, arf, preset.clone());
      });
    });
  } else {
    for arf in arfs {
      prism::render_labelled_arf(&path, root, cps, &melody, &arf, preset.clone());
    }
  }
}
