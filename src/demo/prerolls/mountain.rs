use super::*;
use crate::analysis::delay;
use crate::complexity;
use crate::files;

static demo_name: &str = "mountain";

use crate::analysis::volume::db_to_amp;
use crate::presets::Instrument;
use crate::render::{self, Renderable};
use crate::reverb;
use crate::types::render::{Feel, Melody, Stem};
use crate::types::synthesis::{
  Ampl, Bandpass, Direction, Duration, Ely, FilterPoint, Freq, Frex, GlideLen, Monae, Mote, Note, Register, Soids, Tone,
};

use presets::mountain::{bass, chords, hats, kick, lead, perc};

fn bass_melody() -> Melody<Note> {
  let tala: Vec<Duration> = vec![
    (1i32, 1i32),
    (3i32, 4i32),
    (5i32, 4i32),
    (1i32, 1i32),
    (1i32, 2i32),
    (1i32, 2i32),
    (1i32, 1i32),
    (2i32, 1i32),
  ];

  let amps: Vec<Ampl> = vec![1f32, 0.66f32, 0.66f32, 1f32, 0.66f32, 1f32, 0.66f32, 0.75f32]
    .iter()
    .map(|x| x * db_to_amp(-6f32))
    .collect::<Vec<f32>>();

  let tones: Vec<Tone> = vec![
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

  let amps: Vec<Ampl> = vec![1f32, 0.5f32, 0.66f32, 0.5f32, 1f32, 0.5f32, 0.75f32]
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
    energy: Energy::High,
    presence: Presence::Legato,
  }
}

fn chords_arf() -> Arf {
  Arf {
    mode: Mode::Melodic,
    role: Role::Chords,
    register: 8,
    visibility: Visibility::Foreground,
    energy: Energy::Medium,
    presence: Presence::Tenuto,
  }
}

fn kick_arf() -> Arf {
  Arf {
    mode: Mode::Enharmonic,
    role: Role::Kick,
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
    visibility: Visibility::Foreground,
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
  let root: f32 = 1.2;

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
  
  let renderables: Vec<(Arf,Renderable2)> = vec![
    (kick_arf(), stem_kick), 
    (perc_arf(), stem_perc), 
    (hats_arf(), stem_hats),
    (bass_arf(), stem_bass),
    (chords_arf(), stem_chords),
    (lead_arf(), stem_lead),
  ];

  use crate::types::timbre::Enclosure;
  use crate::Distance;

  let complexity: f32 = rng.gen::<f32>();
  let complexity: f32 = 1f32;
  let len_cycles = time::count_cycles(&hats_melody[0]);
  let len_seconds = len_cycles / cps;
  let group_reverbs = vec![crate::inp::arg_xform::reverb_params(
    &mut rng,
    len_seconds,
    cps,
    &Distance::Near,
    &Enclosure::Hall,
    complexity,
  )];
  let keep_stems = Some(path.as_str());

  let mix = render::combiner_with_reso2(&Conf { cps, root }, &renderables, &vec![], &group_reverbs, keep_stems);
  let filename = format!("{}/{}.wav", location(demo_name), demo_name);
  render::engrave::samples(SR, &mix, &filename);
}

#[test]
fn test_demonstrate() {
  demonstrate()
}
