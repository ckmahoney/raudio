use super::*;
use crate::analysis::delay;
use crate::complexity;
use crate::files;

static demo_name: &str = "just-lead";

use crate::analysis::volume::db_to_amp;
use crate::presets::Instrument;
use crate::render::{self, Renderable};
use crate::reverb;
use crate::types::render::{Feel, Melody, Stem};
use crate::types::synthesis::{
  Ampl, Bandpass, Direction, Duration, Ely, FilterPoint, Freq, Frex, GlideLen, Monae, Mote, Note, Register, Soids, Tone,
};

use presets::hop;
use presets::valley;

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
    (5, (0i8, 0i8, 1i8)),
    (5, (-1i8, 0i8, 1i8)),
    (6, (0i8, 0i8, 1i8)),
    (6, (1i8, 0i8, 1i8)),
    (7, (0i8, 0i8, 1i8)),
    (7, (-1i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
    (8, (1i8, 0i8, 1i8)),
  ];

  vec![zip_line(tala.clone(), line_1, amps.clone())]
}

fn lead_melody_short() -> Melody<Note> {
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

  let line_3: Vec<Tone> = vec![
    (5, (0i8, 0i8, 5i8)),
    (5, (-1i8, 0i8, 5i8)),
    (6, (0i8, 0i8, 5i8)),
    (6, (1i8, 0i8, 5i8)),
    (7, (0i8, 0i8, 5i8)),
    (7, (-1i8, 0i8, 5i8)),
    (8, (0i8, 0i8, 5i8)),
    (8, (1i8, 0i8, 5i8)),
  ];

  vec![zip_line(tala.clone(), line_3, amps.clone())]
}

fn lead_arf(visibility: Visibility, energy: Energy, presence: Presence) -> Arf {
  Arf {
    mode: Mode::Melodic,
    role: Role::Lead,
    register: 6,
    visibility,
    energy,
    presence,
  }
}

fn demonstrate() {
  let path: String = location(demo_name);
  files::with_dir(&path);

  use rand::Rng;
  let mut rng = rand::thread_rng();

  let cps: f32 = 2f32;
  let root: f32 = 1.02;

  let delays: Vec<DelayParams> = vec![delay::passthrough];

  let lead_melody = lead_melody_short();
  let conf = Conf { cps, root };

  let stem_lead2 = valley::lead::renderable(
    &conf,
    &lead_melody,
    &lead_arf(Visibility::Visible, Energy::Low, Presence::Staccatto),
  );
  let stem_lead1 = valley::lead::renderable(
    &conf,
    &lead_melody,
    &lead_arf(Visibility::Foreground, Energy::Medium, Presence::Staccatto),
  );
  let stem_lead3 = valley::lead::renderable(
    &conf,
    &lead_melody,
    &lead_arf(Visibility::Background, Energy::High, Presence::Staccatto),
  );
  let stem_lead4 = valley::lead::renderable(
    &conf,
    &lead_melody,
    &lead_arf(Visibility::Hidden, Energy::High, Presence::Staccatto),
  );

  use Renderable2::{Group, Instance};
  let renderables: Vec<Renderable2> = vec![stem_lead1, stem_lead2, stem_lead3, stem_lead4];

  use crate::types::timbre::Enclosure;
  use crate::Distance;

  let complexity: f32 = rng.gen::<f32>().min(0.01);
  // let group_reverbs = crate::inp::arg_xform::gen_reverbs(&mut rng, cps, &Distance::Near, &Enclosure::Room, complexity);
  let group_reverbs: Vec<crate::reverb::convolution::ReverbParams> = vec![];
  let keep_stems = Some(path.as_str());
  let group_reverbs = vec![];
  let mix = render::combiner_with_reso(&Conf { cps, root }, &renderables, &group_reverbs, keep_stems);
  let filename = format!("{}/{}.wav", location(demo_name), demo_name);
  render::engrave::samples(SR, &mix, &filename);
}

use std::env;
use std::thread;
use sysinfo::System;

use crate::demo::prism;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;

#[test]
fn test_demonstrate() {
  demonstrate()
}

fn get_par_thread_count() -> usize {
  let available_threads = thread::available_parallelism().map(|n| n.get()).unwrap_or(1);

  let mut sys = System::new_all();
  sys.refresh_cpu_all();

  let idle_cores = sys.cpus().iter().filter(|cpu| cpu.cpu_usage() < 50.0).count().max(1);

  let actual_available_threads = available_threads.min(idle_cores);

  let max_par_threads = env::var("MAX_PAR_THREADS")
    .ok()
    .and_then(|val| val.parse::<usize>().ok())
    .unwrap_or(actual_available_threads);

  let num_threads = actual_available_threads.min(max_par_threads);

  if num_threads > 1 {
    num_threads - 1
  } else {
    1
  }
}

#[test]
fn test_iter() {
  let path: String = location(demo_name);
  let cps: f32 = 2.0;
  let root: f32 = 1.12;
  let preset = Preset::Valley;
  files::with_dir(&path);

  let label = "valley_simple_melody";
  let melody = lead_melody_short();
  let arfs = prism::iter_all_vep(&label, Role::Lead, Mode::Melodic, &melody);

  let num_threads = get_par_thread_count().min(4);

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
