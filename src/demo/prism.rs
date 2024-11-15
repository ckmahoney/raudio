use render::engrave;

/// Methods for examining a preset from any desired angle
use super::*;
use crate::analysis::melody::find_reach;

/// iterations happen from first to last.
/// so sort these in an order that matches which stems you want to read first.

pub const VISIBILTYS: [Visibility; 4] = [
  Visibility::Visible,
  Visibility::Background,
  Visibility::Foreground,
  Visibility::Hidden,
];

pub const ENERGYS: [Energy; 3] = [Energy::Medium, Energy::Low, Energy::High];

pub const PRESENCES: [Presence; 3] = [Presence::Staccatto, Presence::Legato, Presence::Tenuto];

pub type LabelledArf = (String, Arf);

/// Given a melody, role, and mode,
/// Create all variations possible (with respect to VEP parameters)
pub fn iter_all_vep<'render>(
  label: &'render str, role: Role, mode: Mode, melody: &'render Melody<Note>,
) -> Vec<LabelledArf> {
  let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(melody);
  let n_coverages = VISIBILTYS.len() * ENERGYS.len() * PRESENCES.len();
  let mut sources: Vec<LabelledArf> = Vec::with_capacity(n_coverages);
  let mut i = 0;

  for &visibility in &VISIBILTYS {
    for &energy in &ENERGYS {
      for &presence in &PRESENCES {
        let sample_name = format!("{}_{}_{}_v={}_e={}_p={}", label, role, i, visibility, energy, presence);
        sources.push((
          sample_name,
          Arf {
            mode,
            role,
            register: lowest_register,
            visibility,
            energy,
            presence,
          },
        ));
        i += 1;
      }
    }
  }

  sources
}

/// Given a melody, role, and mode,
/// Create all variations possible (with respect to VEP parameters)
pub fn iter_vep<'render>(
  label: &'render str, role: Role, mode: Mode, melody: &'render Melody<Note>, vs: &Vec<Visibility>, es: &Vec<Energy>,
  ps: &Vec<Presence>,
) -> Vec<LabelledArf> {
  let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(melody);
  let n_coverages = VISIBILTYS.len() * ENERGYS.len() * PRESENCES.len();
  let mut sources: Vec<LabelledArf> = Vec::with_capacity(n_coverages);
  let mut i = 0;

  for &visibility in vs {
    for &energy in es {
      for &presence in ps {
        let sample_name = format!("{}_{}_{}_v={}_e={}_p={}", label, role, i, visibility, energy, presence);
        sources.push((
          sample_name,
          Arf {
            mode,
            role,
            register: lowest_register,
            visibility,
            energy,
            presence,
          },
        ));
        i += 1;
      }
    }
  }

  sources
}

/// Given a melody, Labelled Arfs, and a preset to splay,
/// Render each labelled arf using the preset into destination_dir.
pub fn render_labelled_arf(
  destination_dir: &str, root: f32, cps: f32, melody: &Melody<Note>, (label, arf): &LabelledArf, preset: Preset,
) {
  let conf: Conf = Conf { root, cps };

  let group_reverbs: Vec<ReverbParams> = vec![];
  let keep_stems = Some(destination_dir);
  let stems: Vec<Renderable2> = vec![Preset::create_stem(&conf, melody, arf, preset)];

  let samples = render::combiner_with_reso(&conf, &stems, &group_reverbs, keep_stems);
  let filename = format!("{}/{}.wav", destination_dir, label);
  engrave::samples(SR, &samples, &filename);
}

use std::env;
use std::thread;
use sysinfo::System;

pub fn get_par_thread_count() -> usize {
  let available_threads = thread::available_parallelism().map(|n| n.get()).unwrap_or(12);

  let mut sys = System::new_all();
  sys.refresh_cpu_all();

  let idle_cores = sys.cpus().iter().filter(|cpu| cpu.cpu_usage() < 50.0).count().max(12);

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

pub fn run(
  path: &str, root: f32, cps: f32, melody: &Melody<Note>, labelled_arfs: &Vec<(String, Arf)>, preset: &Preset,
) {
  let num_threads = get_par_thread_count();

  if num_threads > 1 {
    let pool = ThreadPoolBuilder::new().num_threads(num_threads).build().unwrap();

    pool.install(|| {
      (labelled_arfs).par_iter().for_each(|arf| {
        prism::render_labelled_arf(path, root, cps, &melody, arf, preset.clone());
      });
    });
  } else {
    for arf in labelled_arfs {
      prism::render_labelled_arf(path, root, cps, &melody, &arf, preset.clone());
    }
  }
}
