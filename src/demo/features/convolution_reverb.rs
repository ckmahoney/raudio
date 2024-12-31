use super::*;
use crate::analysis::delay;
use crate::complexity;
use crate::files;

static demo_name: &str = "convolution-reverb";

use crate::analysis::volume::db_to_amp;
use crate::presets::Instrument;
use crate::render::{self, Renderable};
use crate::reverb;
use crate::reverb::convolution::ReverbParams;
use crate::types::render::{Feel, Melody, Stem};
use crate::types::synthesis::{
  Ampl, Bandpass, Direction, Duration, Ely, FilterPoint, Freq, Frex, GlideLen, Monae, Mote, Note, Register, Soids, Tone,
};

use presets::ambien::{hats, kick, perc};

// static cps:f32 = 1.8f32;
static cps: f32 = 2.1f32;
static root: f32 = 1f32;

fn kick_melody() -> Melody<Note> {
  let tala: Vec<Duration> = vec![
    // group A
    (1i32, 2i32),
    (1i32, 2i32),
    (-3i32, 2i32),
    (1i32, 4i32),
    (1i32, 4i32),
    (-1i32, 1i32),
    // group A again
    (1i32, 2i32),
    (1i32, 2i32),
    (-3i32, 2i32),
    (1i32, 4i32),
    (1i32, 4i32),
    (-1i32, 1i32),
    // group B
    (1i32, 2i32),
    (1i32, 2i32),
    (-3i32, 2i32),
    (1i32, 2i32),
    (-1i32, 1i32),
    // group C
    (-1i32, 2i32),
    (1i32, 4i32),
    (1i32, 4i32),
    (-3i32, 2i32),
    (1i32, 2i32),
    (-1i32, 1i32),
  ];

  let amps: Vec<Ampl> = vec![
    1f32, 0.8f32, 0f32, 0.7f32, 1f32, 0f32, 1f32, 0.8f32, 0f32, 0.7f32, 1f32, 0f32, 1f32, 1f32, 0f32, 1f32, 0f32, 0f32,
    1f32, 1f32, 0f32, 1f32, 0f32,
  ]
  .iter()
  .map(|x| x * db_to_amp(-12f32))
  .collect::<Vec<f32>>();

  let tones: Vec<Tone> = vec![(5, (0i8, 0i8, 1i8)); tala.len()];

  vec![zip_line(tala, tones, amps)]
}

fn hats_melody() -> Melody<Note> {
  let tala: Vec<Duration> = vec![
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
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
  ];

  let amps: Vec<Ampl> = tala.iter().map(|x| db_to_amp(-10f32)).collect::<Vec<f32>>();
  let tones: Vec<Tone> = vec![(12, (0i8, 0i8, 1i8)); tala.len()];

  vec![zip_line(tala, tones, amps)]
}

fn perc_melody() -> Melody<Note> {
  let tala: Vec<Duration> = vec![
    // group A
    (-1i32, 1i32),
    (3i32, 4i32),
    (1i32, 2i32),
    (3i32, 4i32),
    (3i32, 4i32),
    (1i32, 4i32),
    // group A
    (-1i32, 1i32),
    (3i32, 4i32),
    (1i32, 2i32),
    (3i32, 4i32),
    (3i32, 4i32),
    (1i32, 4i32),
    // group B
    (-1i32, 1i32),
    (3i32, 4i32),
    (1i32, 4i32),
    (-1i32, 4i32),
    (1i32, 4i32),
    (-1i32, 1i32),
    (1i32, 2i32),
    // group C
    (-1i32, 4i32),
    (1i32, 4i32),
    (-1i32, 2i32),
    (3i32, 4i32),
    (1i32, 2i32),
    (1i32, 4i32),
    (-3i32, 2i32),
  ];

  let amps: Vec<Ampl> = vec![
    0f32, 1f32, 0.21f32, 0.5f32, 1f32, 0.6f32, 0f32, 0.8f32, 0.41f32, 0.5f32, 1f32, 0.4f32, 0f32, 1f32, 0.7f32, 0f32,
    1f32, 0f32, 0.7f32, 0f32, 0.5f32, 0f32, 0.8f32, 0.5f32, 0.7f32, 0f32,
  ]
  .iter()
  .map(|x| if *x == 0f32 { 0f32 } else { 1f32 })
  .collect::<Vec<f32>>();

  let tones: Vec<Tone> = vec![(8, (0i8, 0i8, 1i8)); tala.len()];

  vec![zip_line(tala, tones, amps)]
}

fn kick_arf() -> Arf {
  Arf {
    mode: Mode::Enharmonic,
    role: Role::Perc,
    register: 5,
    visibility: Visibility::Visible,
    energy: Energy::Medium,
    presence: Presence::Tenuto,
  }
}

fn perc_arf() -> Arf {
  Arf {
    mode: Mode::Enharmonic,
    role: Role::Perc,
    register: 7,
    visibility: Visibility::Visible,
    energy: Energy::Low,
    presence: Presence::Staccatto,
  }
}
fn hats_arf() -> Arf {
  Arf {
    mode: Mode::Enharmonic,
    role: Role::Hats,
    register: 12,
    visibility: Visibility::Foreground,
    energy: Energy::Medium,
    presence: Presence::Legato,
  }
}

fn demonstrate() {
  let path: String = location(demo_name);
  files::with_dir(&path);

  use rand::Rng;
  let mut rng = rand::thread_rng();

  let delays: Vec<DelayParams> = vec![delay::passthrough];

  let hats_melody = hats_melody();
  let perc_melody = perc_melody();
  println!("pm at zero {:?}", perc_melody[0]);
  let kick_mel = kick_melody();

  let stem_hats = hats::renderable(&hats_melody, &hats_arf());
  let stem_perc = perc::renderable(&perc_melody, &perc_arf());
  let stem_kick = kick::renderable(&kick_mel, &kick_arf());

  use Renderable::{Group, Instance};
  let renderables: Vec<Renderable> = vec![stem_kick, stem_perc, stem_hats];

  use crate::types::timbre::Enclosure;
  use crate::Distance;

  let complexity: f32 = rng.gen::<f32>();
  let group_reverbs = crate::inp::arg_xform::gen_reverbs(&mut rng, cps, &Distance::Near, &Enclosure::Vast, complexity);
  let mixes: Vec<f32> = (1..10).map(|i| i as f32 / 10f32).collect();
  let durs: Vec<f32> = vec![0.5f32, 1f32, 2f32, 4f32];
  // let durs:Vec<f32> = vec![8f32, 16f32];
  let rates: Vec<f32> = (0..10).map(|i| i as f32 / 10f32).collect();
  let amps: Vec<f32> = (1..5).map(|i| i as f32 / 5f32).collect();
  let keep_stems = None;

  for mix in &mixes {
    for dur in &durs {
      for rate in &rates {
        for amp in &amps {
          let reverbs: Vec<ReverbParams> = vec![ReverbParams {
            mix: *mix,
            dur: *dur,
            rate: *rate,
            amp: *amp,
          }];
          let samples = render::combiner(cps, root, &renderables, &reverbs, keep_stems);
          let filename = format!(
            "{}/reverb_amp_{}_mix_{}_rate_{}_dur_{}.wav",
            location(demo_name),
            amp,
            mix,
            rate,
            dur
          );
          render::engrave::samples(SR, &samples, &filename);
        }
      }
    }
  }
}

fn samp(c: f32, r: f32) -> SampleBuffer {
  use rand::Rng;
  let mut rng = rand::thread_rng();

  let delays: Vec<DelayParams> = vec![delay::passthrough];

  let hats_melody = hats_melody();
  let perc_melody = perc_melody();
  let kick_mel = kick_melody();

  let stem_hats = hats::renderable(&hats_melody, &hats_arf());
  let stem_perc = perc::renderable(&perc_melody, &perc_arf());
  let stem_kick = kick::renderable(&kick_mel, &kick_arf());

  use Renderable::{Group, Instance};
  let renderables: Vec<Renderable> = vec![stem_kick, stem_perc, stem_hats];

  use crate::types::timbre::Enclosure;
  use crate::Distance;

  let complexity: f32 = rng.gen::<f32>();
  let group_reverbs = crate::inp::arg_xform::gen_reverbs(&mut rng, cps, &Distance::Near, &Enclosure::Vast, complexity);

  render::combiner(c, r, &renderables, &group_reverbs, None)
}

#[test]
fn test_demonstrate1x() {
  demonstrate()
}

#[test]
fn test_hypnosis() {
  let path: String = location(demo_name);
  files::with_dir(&path);

  let mut track: SampleBuffer = vec![];

  let n_versions = 8;
  let n_loops = 4;

  use rand::Rng;
  let mut rng = rand::thread_rng();

  let mut roott: f32 = rng.gen::<f32>();
  let base_cps: f32 = 1.2f32 + rng.gen::<f32>();
  let mut cpss: f32 = base_cps;

  for i in 0..n_versions {
    let mut ring = samp(cpss, root);
    for j in 0..n_loops {
      track.extend(&ring)
    }

    roott *= 1.5f32;
    if roott > 2f32 {
      roott /= 2f32;
    };

    cpss *= 1.5f32;
    if cpss > base_cps * 3f32 {
      cpss /= 3f32;
    };
  }

  let filename = format!("{}/hypnoloop_{}.wav", location(demo_name), demo_name);
  render::engrave::samples(SR, &track, &filename);
}

fn render_group(n_versions: usize, n_loops: usize, label: &str) {
  let mut track: SampleBuffer = vec![];

  for i in 0..n_versions {
    let mut ring = samp(cps, root);
    for j in 0..n_loops {
      track.extend(&ring)
    }
  }

  let filename = format!("{}/hypnoloop_{}_{}.wav", location(demo_name), demo_name, label);
  render::engrave::samples(SR, &track, &filename);
}

#[test]
fn test_demo() {
  let path: String = location(demo_name);
  files::with_dir(&path);
  let n_versions = 1;
  let n_loops = 1;
  let label = "mantest";
  let filename = format!("{}/hypnoloop_{}.wav", location(demo_name), demo_name);
  render_group(n_versions, n_loops, label)
}

#[test]
fn test_render_playbook() {
  let filepath: &str = &format!("{}/demo/reverb/test_reverb_playbook", crate::demo::out_dir);

  crate::render_playbook(
    filepath,
    "mountain",
    "src/demo/playbook-demo-ambien.json",
    "test-preset-ambien",
  )
}
