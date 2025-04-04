use super::*;
use std::iter::FromIterator;

use crate::analysis::delay::{self, DelayParams};
use crate::files;
use crate::presets::Instrument;
use crate::reverb;
use crate::synth::{pi, pi2, SampleBuffer, MF, NF, SR};
use crate::time;
use crate::types::render::{Melody, Stem};
use crate::types::synthesis::{
  Ampl, Bandpass, Direction, Duration, FilterPoint, Freq, Frex, GlideLen, Monae, Mote, Note, Register, Tone,
};
use crate::types::timbre::{
  AmpContour, AmpLifespan, Arf, Energy, FilterMode, Mode, Phrasing, Presence, Role, Sound, Sound2, Timeframe,
  Visibility,
};
use crate::{presets, render};

use crate::druid::{inflect, melody_frexer, ApplyAt, Element, Elementor};
use crate::phrasing::{
  lifespan,
  ranger::{self, Knob, KnobMods},
};
use presets::hop::{bass, chords, lead};

static demo_name: &str = "trio-hop";

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

  let amps: Vec<Ampl> = vec![1f32, 0.66f32, 0.66f32, 1f32].iter().map(|x| x * db_to_amp(-6f32)).collect::<Vec<f32>>();

  let line_1: Vec<Tone> = vec![
    (7, (0i8, 0i8, 1i8)),
    (7, (-1i8, 0i8, 1i8)),
    (7, (0i8, 0i8, 1i8)),
    (7, (1i8, 0i8, 1i8)),
  ];

  let line_2: Vec<Tone> = vec![
    (7, (0i8, 0i8, 3i8)),
    (7, (-1i8, 0i8, 3i8)),
    (7, (0i8, 0i8, 3i8)),
    (7, (1i8, 0i8, 3i8)),
  ];

  let line_3: Vec<Tone> = vec![
    (7, (0i8, 0i8, 5i8)),
    (7, (-1i8, 0i8, 5i8)),
    (7, (0i8, 0i8, 5i8)),
    (7, (1i8, 0i8, 5i8)),
  ];

  vec![
    zip_line(tala.clone(), line_1, amps.clone()),
    zip_line(tala.clone(), line_2, amps.clone()),
    zip_line(tala.clone(), line_3, amps.clone()),
  ]
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
    .map(|x| x * db_to_amp(-3f32))
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
    presence: Presence::Tenuto,
  }
}

fn chords_arf() -> Arf {
  Arf {
    mode: Mode::Melodic,
    role: Role::Chords,
    register: 8,
    visibility: Visibility::Visible,
    energy: Energy::Medium,
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
    presence: Presence::Tenuto,
  }
}

fn demonstrate() {
  let path: String = location(demo_name);
  files::with_dir(&path);

  use rand::Rng;
  let mut rng = rand::thread_rng();

  let cps: f32 = 1.15;
  let root: f32 = 1.9;

  let lead_melody = lead_melody();
  let chords_melody = chords_melody();
  let bass_melody = bass_melody();
  let conf: Conf = Conf { cps, root };

  let stem_lead = lead::renderable(&conf, &lead_melody, &lead_arf());
  let stem_chords = chords::renderable(&conf, &chords_melody, &chords_arf());
  let stem_bass = bass::renderable(&conf, &bass_melody, &bass_arf());

  let renderables: Vec<(Arf,Renderable2)> = vec![
    // (kick_arf(), stem_kick), 
    // (perc_arf(), stem_perc), 
    // (hats_arf(), stem_hats),
    (bass_arf(), stem_bass),
    (chords_arf(), stem_chords),
    (lead_arf(), stem_lead),
  ];

  use crate::types::timbre::Enclosure;
  use crate::Distance;

  let complexity: f32 = rng.gen::<f32>().min(0.01);
  let group_reverbs = vec![];
  let keep_stems = Some(path.as_str());
  let mix = render::combiner_with_reso2(&Conf { cps, root }, &renderables, &vec![], &group_reverbs, keep_stems);
  let filename = format!("{}/{}.wav", location(demo_name), demo_name);
  render::engrave::samples(SR, &mix, &filename);
}

#[test]
fn test_demonstrate() {
  demonstrate()
}
