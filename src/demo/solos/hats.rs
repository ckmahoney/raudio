use super::*;
use crate::analysis::delay;
use crate::complexity;
use crate::files;

static demo_name: &str = "just-hats";

use crate::analysis::volume::db_to_amp;
use crate::presets::Instrument;
use crate::render::{self, Renderable};
use crate::reverb;
use crate::types::render::{Feel, Melody, Stem};
use crate::types::synthesis::{
  Ampl, Bandpass, Direction, Duration, Ely, FilterPoint, Freq, Frex, GlideLen, Monae, Mote, Note, Register, Soids, Tone,
};

fn hats_melody() -> Melody<Note> {
  let tala: Vec<Duration> = vec![
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 1i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 1i32),
    (1i32, 2i32),
    (2i32, 1i32),
    (-1i32, 1i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (-1i32, 1i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
  ];
  let mut rng = thread_rng();
  let amps: Vec<Ampl> = (0..tala.len()).map(|x| rng.gen::<f32>() * 0.2f32 + 0.4f32).collect();

  let tones: Vec<Tone> = vec![(12, (0i8, 0i8, 1i8)); tala.len()];

  vec![zip_line(tala, tones, amps)]
}

#[test]
fn test_arf() {
  let path: String = location(demo_name);
  let cps: f32 = 2.0;
  let root: f32 = 1.12;
  let preset = Preset::Hill;
  files::with_dir(&path);

  let arf: Arf = Arf {
    mode: Mode::Enharmonic,
    register: 10,
    role: Role::Hats,
    visibility: Visibility::Visible,
    energy: Energy::High,
    presence: Presence::Tenuto,
  };

  let label = "hill_simple_melody";
  let melody = hats_melody();
  let arfs: Vec<(String, Arf)> = vec![("the-one".to_string(), arf)];

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

#[test]
fn test_slice() {
  let vs: Vec<Visibility> = prism::VISIBILTYS.to_vec();
  let es: Vec<Energy> = prism::ENERGYS.to_vec();
  let ps: Vec<Presence> = prism::PRESENCES.to_vec();

  let ps = vec![Presence::Tenuto];

  let path: String = location(demo_name);
  let cps: f32 = 2.0;
  let root: f32 = 1.12;
  let preset = Preset::Hill;
  files::with_dir(&path);

  let label = "hill_simple_melody";
  let melody = hats_melody();
  let arfs = prism::iter_all_vep(&label, Role::Hats, Mode::Enharmonic, &melody);

  let num_threads = prism::get_par_thread_count().min(8);

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

#[test]
fn test_iter() {
  let path: String = location(demo_name);
  let cps: f32 = 2.0;
  let root: f32 = 1.12;
  let preset = Preset::Hill;
  files::with_dir(&path);

  let label = "hill_simple_melody";
  let melody = hats_melody();
  let arfs = prism::iter_all_vep(&label, Role::Hats, Mode::Enharmonic, &melody);

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
