use super::*;
use crate::analysis::delay;
use crate::complexity;
use crate::files;

static demo_name: &str = "fummy";

use crate::analysis::volume::db_to_amp;
use crate::presets::Instrument;
use crate::render::{self, Renderable};
use crate::reverb;
use crate::types::render::{Feel, Melody, Stem};
use crate::types::synthesis::{
  Ampl, Bandpass, Direction, Duration, Ely, FilterPoint, Freq, Frex, GlideLen, Monae, Mote, Note, Register, Soids, Tone,
};

use presets::fum::{bass, chords, hats, kick, lead, perc};

fn bass_melody() -> Melody<Note> {
  let tala: Vec<Duration> = vec![
    (2i32, 1i32),
    (1i32, 1i32),
    (1i32, 1i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 1i32),
    (2i32, 1i32),
  ];

  let amps: Vec<Ampl> = vec![1f32, 0.66f32, 1f32, 0.66f32, 1f32, 0.66f32, 0.75f32]
    .iter()
    .map(|x| x * db_to_amp(-6f32))
    .collect::<Vec<f32>>();

  let bass_register = 6;
  let tones: Vec<Tone> = vec![
    (bass_register, (0i8, 0i8, 1i8)),
    (bass_register, (0i8, 0i8, 3i8)),
    (bass_register, (1i8, 0i8, 1i8)),
    (bass_register, (1i8, 0i8, 5i8)),
    (bass_register, (1i8, 0i8, 3i8)),
    (bass_register, (-1i8, 0i8, 5i8)),
    (bass_register, (-1i8, 0i8, 1i8)),
  ];

  vec![zip_line(tala, tones, amps)]
}

fn chords_melody() -> Melody<Note> {
  let tala: Vec<Duration> = vec![(2i32, 1i32), (2i32, 1i32), (2i32, 1i32), (2i32, 1i32)];

  let amps: Vec<Ampl> = vec![1f32, 0.66f32, 0.66f32, 1f32].iter().map(|x| x * db_to_amp(-30f32)).collect::<Vec<f32>>();

  let line_1: Vec<Tone> = vec![
    (8, (0i8, 0i8, 1i8)),
    (8, (-1i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
    (8, (1i8, 0i8, 1i8)),
  ];

  let line_2: Vec<Tone> = vec![
    (8, (0i8, 0i8, 3i8)),
    (8, (-1i8, 0i8, 3i8)),
    (8, (0i8, 0i8, 3i8)),
    (8, (1i8, 0i8, 3i8)),
  ];

  let line_3: Vec<Tone> = vec![
    (8, (0i8, 0i8, 5i8)),
    (8, (-1i8, 0i8, 5i8)),
    (8, (0i8, 0i8, 5i8)),
    (8, (1i8, 0i8, 5i8)),
  ];

  vec![
    zip_line(tala.clone(), line_1, amps.clone()),
    zip_line(tala.clone(), line_2, amps.clone()),
    zip_line(tala.clone(), line_3, amps.clone()),
  ]
}

fn kick_melody() -> Melody<Note> {
  let tala: Vec<Duration> = vec![
    (1i32, 1i32),
    (1i32, 1i32),
    (2i32, 1i32),
    (1i32, 1i32),
    (1i32, 1i32),
    (1i32, 1i32),
    (1i32, 2i32),
    (1i32, 2i32),
  ];

  let amps: Vec<Ampl> = vec![1f32, 0.66f32, 1f32, 1f32, 0.5f32, 0.75f32, 1f32, 0.66f32]
    .iter()
    .map(|x| x * db_to_amp(-12f32))
    .collect::<Vec<f32>>();

  let tones: Vec<Tone> = vec![
    (5, (0i8, 0i8, 1i8)),
    (5, (0i8, 0i8, 1i8)),
    (5, (0i8, 0i8, 1i8)),
    (5, (0i8, 0i8, 1i8)),
    (5, (0i8, 0i8, 1i8)),
    (5, (0i8, 0i8, 1i8)),
    (5, (0i8, 0i8, 1i8)),
    (5, (0i8, 0i8, 1i8)),
  ];

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
    (-1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (-1i32, 2i32),
    (1i32, 2i32),
    (-1i32, 2i32),
    (1i32, 2i32),
  ];

  let amps: Vec<Ampl> = vec![
    0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32, 0.5f32,
    0.5f32, 0.5f32,
  ]
  .iter()
  .map(|x| x * db_to_amp(-24f32))
  .collect::<Vec<f32>>();

  let tones: Vec<Tone> = vec![
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
    (12, (0i8, 0i8, 1i8)),
  ];

  vec![zip_line(tala, tones, amps)]
}

fn perc_melody() -> Melody<Note> {
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

  let amps: Vec<Ampl> = vec![0f32, 0.66f32, 0f32, 0.75f32, 0f32, 0.66f32, 0f32, 0.5f32]
    .iter()
    .map(|x| x * db_to_amp(-12f32))
    .collect::<Vec<f32>>();

  let tones: Vec<Tone> = vec![
    (8, (0i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
    (8, (0i8, 0i8, 1i8)),
  ];

  vec![zip_line(tala, tones, amps)]
}

fn lead_melody() -> Melody<Note> {
  let tala: Vec<Duration> = vec![
    (1i32, 1i32),
    (3i32, 2i32),
    (1i32, 1i32),
    (1i32, 1i32),
    (3i32, 2i32),
    (1i32, 1i32),
    (1i32, 1i32),
  ];

  let amps: Vec<Ampl> = vec![1f32, 0.5f32, 0.66f32, 0.15f32, 1f32, 0.25f32, 0.75f32]
    .iter()
    .map(|x| x * db_to_amp(-20f32))
    .collect();

  let tones: Vec<Tone> = vec![
    (7, (0i8, 0i8, 5i8)),
    (8, (0i8, 0i8, 3i8)),
    (8, (1i8, 0i8, 1i8)),
    (7, (1i8, 0i8, 5i8)),
    (8, (1i8, 0i8, 3i8)),
    (9, (-1i8, 0i8, 5i8)),
    (8, (-1i8, 0i8, 1i8)),
  ];

  vec![zip_line(tala, tones, amps)]
}

fn bass_arf() -> Arf {
  Arf {
    mode: Mode::Melodic,
    role: Role::Bass,
    register: 5,
    visibility: Visibility::Visible,
    energy: Energy::Low,
    presence: Presence::Legato,
  }
}

fn chords_arf() -> Arf {
  Arf {
    mode: Mode::Melodic,
    role: Role::Chords,
    register: 8,
    visibility: Visibility::Visible,
    energy: Energy::High,
    presence: Presence::Tenuto,
  }
}

fn kick_arf() -> Arf {
  Arf {
    mode: Mode::Enharmonic,
    role: Role::Perc,
    register: 5,
    visibility: Visibility::Foreground,
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

fn lead_arf() -> Arf {
  Arf {
    mode: Mode::Melodic,
    role: Role::Lead,
    register: 8,
    visibility: Visibility::Foreground,
    energy: Energy::High,
    presence: Presence::Legato,
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

  let cps: f32 = 1.15;
  let root: f32 = 1.9;

  let delays: Vec<DelayParams> = vec![delay::passthrough];

  let lead_melody = lead_melody();
  let hats_melody = hats_melody();
  let chords_melody = chords_melody();
  let bass_melody = bass_melody();
  let perc_melody = perc_melody();
  let kick_mel = kick_melody();
  let conf: Conf = Conf { cps, root };

  let stem_lead = lead::renderable(&conf, &lead_melody, &lead_arf());
  let stem_hats = hats::renderable(&conf, &hats_melody, &hats_arf());
  let stem_chords = chords::renderable(&conf, &chords_melody, &chords_arf());
  let stem_bass = bass::renderable(&conf, &bass_melody, &bass_arf());
  let stem_perc = perc::renderable(&conf, &perc_melody, &perc_arf());
  let stem_kick = kick::renderable(&conf, &kick_mel, &kick_arf());

  use Renderable::{Group, Instance};
  let renderables: Vec<Renderable2> = vec![
    // stem_kick,
    // stem_perc,
    // stem_hats,
    stem_bass,
    stem_chords,
    stem_lead,
  ];

  use crate::types::timbre::Enclosure;
  use crate::Distance;

  let complexity: f32 = rng.gen::<f32>().min(0.01);
  let group_reverbs = crate::inp::arg_xform::gen_reverbs(&mut rng, cps, &Distance::Near, &Enclosure::Room, complexity);
  let keep_stems = Some(path.as_str());
  let group_reverbs = vec![];
  let mix = render::combiner_with_reso2(&conf, &renderables, &vec![], &group_reverbs, keep_stems);
  let filename = format!("{}/{}.wav", location(demo_name), demo_name);
  render::engrave::samples(SR, &mix, &filename);
}

#[test]
fn test_demonstrate() {
  demonstrate()
}

#[test]
fn test_render_playbook() {
  eprintln!("There's something wrong with the mountain preset! be carefule");
  let filepath: &str = &format!("{}/demo/ambien/test_ambien_playbook", crate::demo::out_dir);
  crate::render_playbook(
    filepath,
    "mountain",
    "src/demo/playbook-demo-ambien.json",
    "test-preset-ambien",
  )
}

#[test]
fn test_render_deck_the_hall() {
  let filepath: &str = &format!("{}/demo/fum", crate::demo::out_dir);
  crate::render_playbook(
    filepath,
    "fum",
    "src/demo/playbooks/deck-the-hall-house-playbook.json",
    "test-fum-deck-the-hall",
  )
}
