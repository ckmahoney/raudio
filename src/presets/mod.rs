use crate::analysis::{
  in_range,
  melody::{find_reach, mask_sigh, mask_wah, LevelMacro, Levels, ODRMacro, ODR},
  trig,
  volume::db_to_amp,
};
use crate::phrasing::older_ranger::{Modders, OldRangerDeprecated, WOldRangerDeprecateds};
use crate::phrasing::{dynamics, lifespan, micro};
use crate::synth::{pi, pi2, MFf, NFf, SRf, SampleBuffer, MAX_REGISTER, MIN_REGISTER, SR};
use crate::{render, AmpLifespan};
use rand;
use rand::{prelude::SliceRandom, rngs::ThreadRng, Rng};
use std::marker::PhantomData;

use crate::analysis::delay::{self, DelayParams, DelayParamsMacro, StereoField};
use crate::analysis::sampler::read_audio_file;
use crate::druid::{self, noise::NoiseColor, soid_fx, soids as druidic_soids};
use crate::druid::{bell, melodic, noise, Element, Elementor};
use crate::phrasing::contour::expr_none;
use crate::phrasing::contour::Expr;
use crate::phrasing::ranger::{self, Knob, KnobMacro, KnobMods, KnobMods2};
use crate::render::{Renderable, Renderable2};
use crate::reverb::convolution::ReverbParams;
use crate::time;
use crate::types::render::{Conf, Feel, Melody, Stem, Stem2, Stem3};
use crate::types::synthesis::{
  bp2_unit, BoostGroup, Bp2, Direction, Ely, Freq, ModulationEffect, Note, PhaseModParams,
};
use crate::types::synthesis::{BoostGroupMacro, MacroMotion, ModifiersHolder, Soids};
use crate::types::timbre::{Arf, Energy, Mode, Phrasing, Presence, Role, Sound, Sound2, Visibility};
use crate::types::{Radian, Range};
use rand::thread_rng;
use std::fs::read_dir;

pub mod ambien;
pub mod valley;
pub mod hop;
pub mod kuwuku;
pub mod mountain;
pub mod urbuntu;

pub type KnobPair = (KnobMacro, fn(&Knob, f32, f32, f32, f32, f32) -> f32);

use std::collections::HashMap;
/// Base directory for audio samples.
const SAMPLE_SOURCE_DIR: &str = "audio-samples";
/// Cache for sample paths to avoid repeated directory scans.
static SAMPLE_CACHE: Lazy<RwLock<HashMap<String, Vec<String>>>> = Lazy::new(|| RwLock::new(initialize_sample_cache()));

// user configurable headroom value. defaults to -15Db
pub const DB_HEADROOM: f32 = -8f32;

/// Shared values for all presets in this mod
static contour_resolution: usize = 1200;
static unit_decay: f32 = 9.210340372;

pub trait Conventions<'render> {
  fn get_bp(cps: f32, mel: &'render Melody<Note>, arf: &Arf) -> Bp2;
}

/// A constant microtransient onset to prevent immediate entry.
/// This is the smallest possible for a natural entry onset. 
pub fn amp_microtransient(visibility: Visibility, energy: Energy, presence: Presence) -> KnobPair {
  (
    KnobMacro {
      a: [0.45f32, 0.45f32],
      b: [0f32, 0f32],
      c: [1f32, 1f32],
      ma: MacroMotion::Constant,
      mb: MacroMotion::Constant,
      mc: MacroMotion::Constant,
    },
    ranger::amod_microbreath_4_20,
  )
}

pub fn microtransient() -> KnobPair {
  (
    KnobMacro {
      a: [0.65f32, 0.65f32],
      b: [0f32, 0f32],
      c: [1f32, 1f32],
      ma: MacroMotion::Random,
      mb: MacroMotion::Max,
      mc: MacroMotion::Max,
    },
    ranger::amod_microbreath_4_20,
  )
}

pub fn microtransient2() -> KnobPair {
  (
    KnobMacro {
      a: [0.45f32, 0.45f32],
      b: [0f32, 0f32],
      c: [1f32, 1f32],
      ma: MacroMotion::Random,
      mb: MacroMotion::Max,
      mc: MacroMotion::Max,
    },
    ranger::amod_microbreath_20_100,
  )
}
pub fn grab_variant<T: Copy>(variants: Vec<T>) -> T {
  let mut rng = thread_rng();
  *variants.choose(&mut rng).expect("Vector should not be empty")
}

pub fn get_bp<'render>(cps: f32, mel: &'render Melody<Note>, arf: &Arf, len_cycles: f32) -> Bp2 {
  println!("OVerriding bp");
  return bp2_unit();
  // match arf.presence {
  //   Presence::Staccatto => bp_wah(cps, mel, arf, len_cycles),
  //   Presence::Legato => bp_sighpad(cps, mel, arf, len_cycles),
  //   Presence::Tenuto => bp_cresc(cps, mel, arf, len_cycles),
  // }
}

/// Create bandpass automations with respect to Arf and Melody
fn bp_cresc<'render>(cps: f32, mel: &'render Melody<Note>, arf: &Arf) -> Bp2 {
  let len_cycles = time::count_cycles(&mel[0]);
  let size = (len_cycles.log2() - 1f32).max(1f32); // offset 1 to account for lack of CPC. -1 assumes CPC=2
  let rate_per_size = match arf.energy {
    Energy::Low => 0.5f32,
    Energy::Medium => 1f32,
    Energy::High => 2f32,
  };
  let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);
  let n_samples: usize = ((len_cycles / 2f32) as usize).max(1) * SR;

  let (highpass, lowpass): (Vec<f32>, Vec<f32>) = if let Visibility::Visible = arf.visibility {
    match arf.energy {
      Energy::Low => (
        filter_contour_triangle_shape_highpass(lowest_register, highest_register, n_samples, size * rate_per_size),
        vec![NFf],
      ),
      _ => (
        vec![MFf],
        filter_contour_triangle_shape_lowpass(lowest_register, n_samples, size * rate_per_size),
      ),
    }
  } else {
    (vec![MFf], vec![NFf])
  };

  let levels = Levels::new(0.7f32, 4f32, 0.5f32);
  let odr = ODR {
    onset: 60.0,
    decay: 1330.0,
    release: 110.0,
  };

  (highpass, lowpass, vec![])
}

/// Create bandpass automations with respect to Arf and Melody
fn bp_wah<'render>(cps: f32, mel: &'render Melody<Note>, arf: &Arf) -> Bp2 {
  let len_cycles = time::count_cycles(&mel[0]);
  let size = (len_cycles.log2() - 1f32).max(1f32); // offset 1 to account for lack of CPC. -1 assumes CPC=2
  let rate_per_size = match arf.energy {
    Energy::Low => 0.5f32,
    Energy::Medium => 1f32,
    Energy::High => 2f32,
  };
  let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);
  let n_samples: usize = ((len_cycles / 2f32) as usize).max(1) * SR;

  let levels = Levels::new(0.7f32, 4f32, 0.5f32);

  let level_macro: LevelMacro = LevelMacro {
    stable: match arf.energy {
      Energy::Low => [1f32, 3f32],
      Energy::Medium => [2f32, 3f32],
      Energy::High => [2f32, 3f32],
    },
    peak: match arf.energy {
      Energy::Low => [2.0f32, 3.0f32],
      Energy::Medium => [3.75f32, 5f32],
      Energy::High => [4f32, 8f32],
    },
    sustain: match arf.visibility {
      Visibility::Visible => [0.85f32, 1f32],
      Visibility::Foreground => [0.5f32, 1.0f32],
      Visibility::Background => [0.2f32, 0.5f32],
      Visibility::Hidden => [0.0132, 0.1f32],
    },
  };

  let odr_macro = ODRMacro {
    onset: [60.0, 120f32],
    decay: [230.0, 300f32],
    release: [110.0, 200f32],

    mo: vec![MacroMotion::Constant],
    md: vec![MacroMotion::Constant],
    mr: vec![MacroMotion::Constant],
  };
  let highpass = if let Energy::Low = arf.energy {
    filter_contour_triangle_shape_highpass(lowest_register, highest_register, n_samples, size * rate_per_size)
  } else {
    vec![MFf]
  };
  (
    highpass,
    mask_wah(cps, &mel[low_index], &level_macro, &odr_macro),
    vec![],
  )
}

/// Gets an applied fundamental frequency for a synth by energy
/// and provided powers of 2,
/// given that a high power of two is a higher fundamental is a shorter synth.
pub fn mul_it(arf:&Arf, low:f32, medium:f32, high:f32) -> f32 {
  2f32.powf(match arf.energy {
    Energy::Low => low, Energy::Medium => medium, Energy::High => high
  })
}
/// Create bandpass automations with respect to Arf and Melody
fn bp_sighpad<'render>(cps: f32, mel: &'render Melody<Note>, arf: &Arf) -> Bp2 {
  let len_cycles = time::count_cycles(&mel[0]);
  let size = (len_cycles.log2() - 1f32).max(1f32); // offset 1 to account for lack of CPC. -1 assumes CPC=2
  let rate_per_size = match arf.energy {
    Energy::Low => 0.5f32,
    Energy::Medium => 1f32,
    Energy::High => 2f32,
  };
  let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);
  let n_samples: usize = ((len_cycles / 2f32) as usize).max(1) * SR;
  let levels = Levels::new(0.7f32, 4f32, 0.5f32);
  let level_macro: LevelMacro = LevelMacro {
    stable: [1f32, 1f32],
    peak: [2.25f32, 4f32],
    sustain: [0.4f32, 0.8f32],
  };

  let odr_macro = ODRMacro {
    onset: [260.0, 2120f32],
    decay: [1330.0, 5000f32],
    release: [1510.0, 2000f32],

    mo: vec![MacroMotion::Constant],
    md: vec![MacroMotion::Constant],
    mr: vec![MacroMotion::Constant],
  };

  let highpass = if let Energy::Low = arf.energy {
    filter_contour_triangle_shape_highpass(lowest_register, highest_register, n_samples, size * rate_per_size)
  } else {
    vec![MFf]
  };
  (
    highpass,
    mask_sigh(cps, &mel[low_index], &level_macro, &odr_macro),
    vec![],
  )
}

pub fn get_boost_macros(arf: &Arf) -> Vec<BoostGroupMacro> {
  let gen = || -> BoostGroupMacro {
    let base: i32 = arf.register as i32;
    let bandwidth: (f32, f32) = match arf.visibility {
      Visibility::Hidden => (0.1, 0.2),
      Visibility::Background => (0.2, 0.3),
      Visibility::Visible => (0.3, 0.4),
      Visibility::Foreground => (0.4, 0.6),
    };
    BoostGroupMacro {
      bandpass: [2f32.powi(base), 2f32.powi(base + 1i32)],
      bandwidth: [bandwidth.0, bandwidth.1],
      att: [8f32, 12f32],
      rolloff: [21f32, 2.3f32],
      q: [1f32, 2f32],
      motion: MacroMotion::Random,
    }
  };

  match arf.energy {
    // allpass filter
    Energy::High => vec![],
    // suppress some energy
    Energy::Medium => vec![gen()],
    // suppress a lot of energy
    Energy::Low => vec![gen(), gen()],
  }
}

/// Generate a phrase-length filter contour with a triangle shape, oscillating `k` times per phrase.
/// Peaks `k` times within the phrase and tapers back down to `start_cap` at the end.
pub fn filter_contour_triangle_shape_lowpass<'render>(lowest_register: i8, n_samples: usize, k: f32) -> SampleBuffer {
  let mut lowpass_contour: SampleBuffer = Vec::with_capacity(n_samples);

  let start_cap: f32 = 2.1f32;
  let final_cap: f32 = MAX_REGISTER as f32 - lowest_register as f32 - start_cap;

  let min_f: f32 = 2f32.powf(lowest_register as f32 + start_cap);
  let max_f: f32 = 2f32.powf(lowest_register as f32 + start_cap + final_cap);
  let n: f32 = n_samples as f32;
  let df: f32 = (max_f - min_f).log2();

  for i in 0..n_samples {
    let x: f32 = i as f32 / n;

    // Modulate the frequency of oscillation using k
    let x_adjusted = (k * x).fract();
    let triangle_wave = if x_adjusted <= 0.5 {
      2.0 * x_adjusted
    } else {
      2.0 * (1.0 - x_adjusted)
    };

    // Calculate the lowpass frequency based on the triangle wave
    lowpass_contour.push(min_f + 2f32.powf(df * triangle_wave));
  }

  lowpass_contour
}

/// Generate a phrase-length filter contour with a triangle shape, oscillating `k` times per phrase.
/// Peaks `k` times within the phrase and tapers back down to `start_cap` at the end.
pub fn filter_contour_triangle_shape_highpass<'render>(
  lowest_register: i8, highest_register: i8, n_samples: usize, k: f32,
) -> SampleBuffer {
  let mut highpass_contour: SampleBuffer = Vec::with_capacity(n_samples);

  let start_cap: f32 = (3.0f32).min(MAX_REGISTER as f32 - highest_register as f32);
  let final_cap: f32 = MAX_REGISTER as f32 - highest_register as f32 - start_cap;

  let min_f: f32 = 2f32.powf(lowest_register as f32);
  let max_f: f32 = 2f32.powf(highest_register as f32 + start_cap);
  let n: f32 = n_samples as f32;
  let df: f32 = (max_f - min_f).log2();

  for i in 0..n_samples {
    let x: f32 = i as f32 / n;

    let x_adjusted = (k * x).fract();
    let triangle_wave = if x_adjusted <= 0.5 {
      2.0 * x_adjusted
    } else {
      2.0 * (1.0 - x_adjusted)
    };

    // Calculate the lowpass frequency based on the triangle wave
    highpass_contour.push(max_f - 2f32.powf(df * triangle_wave));
  }

  // highpass_contour;
  vec![MFf]
}

#[derive(Debug)]
pub struct Dressing {
  pub len: usize,
  pub multipliers: Vec<f32>,
  pub amplitudes: Vec<f32>,
  pub offsets: Vec<f32>,
}
pub type Dressor = fn(f32) -> Dressing;

pub struct Instrument {}

use std::fmt;

/// Composition and orchestration details required to generate audio samples.
/// Returns a Renderable: A collection of Stems to pass into the render engine.
type StemSpec<'render> =
  fn(&Conf, &'render Vec<Vec<((i32, i32), (i8, (i8, i8, i8)), f32)>>, &'render Arf) -> Renderable2<'render>;

/// Struct that defines render methods for each role.
pub struct RolePreset<'render> {
  label: &'render str,
  pub kick: StemSpec<'render>,
  pub perc: StemSpec<'render>,
  pub hats: StemSpec<'render>,
  pub chords: StemSpec<'render>,
  pub lead: StemSpec<'render>,
  pub bass: StemSpec<'render>,
}

/// Enum representing the different presets, with support for `Format::display`.
#[derive(Copy, Clone, Debug)]
pub enum Preset {
  Valley,
  Mountain,
  Hop
}

impl fmt::Display for Preset {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", format!("{:?}", self).to_lowercase())
  }
}

impl<'render> Preset {
  /// Returns the `RolePreset` associated with the given `Preset`.
  pub fn get(preset: Preset) -> RolePreset<'render> {
    match preset {
      Preset::Valley => valley::map_role_preset(),
      Preset::Mountain => mountain::map_role_preset(),
      Preset::Hop => hop::map_role_preset(),
    }
  }

  /// Returns the label of the `Preset`.
  pub fn get_preset_label(preset: Preset) -> &'render str {
    Self::get(preset).label
  }

  /// Renders a melody using the specified `Preset` and `Role`.
  pub fn create_stem(
    conf: &Conf, melody: &'render Melody<Note>, arf: &'render Arf, preset: Preset,
  ) -> Renderable2<'render> {
    let preset = Self::get(preset);
    let render_fn = match arf.role {
      Role::Kick => preset.kick,
      Role::Perc => preset.perc,
      Role::Hats => preset.hats,
      Role::Chords => preset.chords,
      Role::Lead => preset.lead,
      Role::Bass => preset.bass,
    };

    render_fn(conf, melody, arf)
  }
}

impl Instrument {
  // pub fn select<'render>(cps:f32, melody:&'render Melody<Note>, arf:&Arf, delays:Vec<DelayParams>) -> Renderable2<'render> {
  //     use Role::*;
  //     use crate::synth::MFf;
  //     use crate::phrasing::ranger::KnobMods;

  //     let renderable = match arf.role {
  //         Kick => hop::kick::renderable(cps, melody, arf),
  //         Perc => hop::perc::renderable(cps, melody, arf),
  //         Hats => hop::hats::renderable(cps, melody, arf),
  //         Lead => hop::lead::renderable(cps, melody, arf),
  //         Bass => hop::bass::renderable(cps, melody, arf),
  //         Chords => valley::chords::renderable(cps, melody, arf),
  //     };

  //     // match arf.role {
  //     //     Kick => ambien::kick::renderable(melody, arf),
  //     //     Perc => ambien::perc::renderable(melody, arf),
  //     //     Hats => ambien::hats::renderable(melody, arf),
  //     //     Lead => ambien::lead::renderable(melody, arf),
  //     //     Bass => ambien::bass::renderable(melody, arf),
  //     //     Chords => ambien::chords::renderable(melody, arf),
  //     // }

  //     match renderable {
  //         Renderable2::Instance(mut stem) => {
  //             stem.5 = delays;
  //             Renderable2::Instance(stem)
  //         },
  //         Renderable2::Group(mut stems) => {
  //             for stem in &mut stems {
  //                 stem.5 = delays.clone()
  //             };
  //             Renderable2::Group(stems)
  //         }
  //     }
  // }
}

fn select_expr(arf: &Arf) -> Expr {
  let mut rng = thread_rng();

  use AmpLifespan::{self, *};
  use Role::{self, *};
  let plucky_lifespans: Vec<AmpLifespan> = vec![Pluck, Snap, Burst];
  let sussy_lifespans: Vec<AmpLifespan> = vec![Spring, Bloom, Pad, Drone];

  let lifespan = match arf.role {
    Kick | Perc | Hats => plucky_lifespans.choose(&mut rng).unwrap(),
    Lead | Chords | Bass => match arf.presence {
      Presence::Legato => sussy_lifespans.choose(&mut rng).unwrap(),
      Presence::Staccatto => plucky_lifespans.choose(&mut rng).unwrap(),
      Presence::Tenuto => {
        if rng.gen_bool(0.33) {
          plucky_lifespans.choose(&mut rng).unwrap()
        } else {
          sussy_lifespans.choose(&mut rng).unwrap()
        }
      }
    },
  };

  let amp_contour: Vec<f32> = crate::phrasing::lifespan::sample_lifespan(crate::synth::SR, lifespan, 1, 1f32);
  (amp_contour, vec![1f32], vec![0f32])
}

/// DEPRECATED the methods below have been replaced by the ranger module,
/// which offers a better interface for dynamic dispatch (using Knob).

/// Microtansient methods probaly should go in micro
pub fn microtransient_chiff(fund: f32, vis: &Visibility, energy: &Energy, presence: &Presence) -> Element {
  let (amps, muls, phss) = micro::set_micro(fund, energy);
  Element {
    mode: Mode::Noise,
    amps,
    muls,
    phss,
    modders: micro::modders_chiff(),
    expr: expr_none(),
    hplp: (vec![MFf], vec![NFf]),
    thresh: (0f32, 1f32),
  }
}

pub fn microtransient_click(fund: f32, vis: &Visibility, energy: &Energy, presence: &Presence) -> Element {
  let (amps, muls, phss) = micro::set_micro(fund, energy);
  Element {
    mode: Mode::Noise,
    amps,
    muls,
    phss,
    modders: micro::modders_chiff(),
    expr: expr_none(),
    hplp: (vec![MFf], vec![NFf]),
    thresh: (0f32, 1f32),
  }
}

pub fn microtransient_pop(fund: f32, vis: &Visibility, energy: &Energy, presence: &Presence) -> Element {
  let (amps, muls, phss) = micro::set_micro(fund, energy);
  Element {
    mode: Mode::Noise,
    amps,
    muls,
    phss,
    modders: micro::modders_chiff(),
    expr: expr_none(),
    hplp: (vec![MFf], vec![NFf]),
    thresh: (0f32, 1f32),
  }
}

/// Four octave freq sweep, responsive to monic and duration.
/// Requires that the input multipliers are truncated by log_2(max_sweep_mul) octaves
/// https://www.desmos.com/calculator/fbzd5wwj2e
static max_sweep_reg: f32 = 4f32;
static min_sweep_reg: f32 = 1f32;
pub fn fmod_sweep(k: usize, x: f32, d: f32) -> f32 {
  let kf = k as f32;
  let growth_const = -unit_decay;
  let sweep_reg: f32 = max_sweep_reg - 1f32;
  2f32.powf(sweep_reg) * (kf * growth_const * x).exp()
}

// values in 25-50 look good. @art-choice could mod in this range
static amod_const: f32 = 50f32;
fn amod_exit(x: f32) -> f32 {
  let y: f32 = (amod_const * x - pi).tanh();
  0.5f32 * (1f32 - y)
}

///A brief one-valued signal with tanh decay to 0.
pub fn amod_impulse(k: usize, x: f32, d: f32) -> f32 {
  let y: f32 = -1f32 + (1f32 / (1f32 - (-x).exp()));
  (0.5f32 * y).tanh() * amod_exit(x)
}

pub fn visibility_gain(v: Visibility) -> f32 {
  match v {
    Visibility::Hidden => db_to_amp(-20f32),
    Visibility::Background => db_to_amp(-16f32),
    Visibility::Foreground => db_to_amp(-10f32),
    Visibility::Visible => db_to_amp(-6f32),
  }
}

pub fn visibility_gain_sample(v: Visibility) -> f32 {
  match v {
    Visibility::Hidden => db_to_amp(-22f32),
    Visibility::Background => db_to_amp(-18f32),
    Visibility::Foreground => db_to_amp(-12f32),
    Visibility::Visible => db_to_amp(-6f32),
  }
}

pub fn amp_scale(cont: &mut Vec<f32>, gain: f32) {
  if gain < 0f32 {
    panic!("Can't scale by less than zero")
  }
  cont.iter_mut().for_each(|val| *val *= gain)
}

use once_cell::sync::Lazy;
use std::sync::RwLock;
/// Retrieves a sample file path based on the given `Arf` configuration.
///
/// # Parameters
/// - `arf`: The amplitude and visibility configuration.
///
/// # Returns
/// A randomly selected file path from the appropriate category.
pub fn get_sample_path(arf: &Arf) -> String {
  let key = match arf.role {
    Role::Hats => match arf.presence {
      Presence::Staccatto | Presence::Legato => format!("{}/hats/short", SAMPLE_SOURCE_DIR),
      Presence::Tenuto => format!("{}/hats/long", SAMPLE_SOURCE_DIR),
    },
    Role::Kick => format!("{}/kick", SAMPLE_SOURCE_DIR),
    Role::Perc => format!("{}/perc", SAMPLE_SOURCE_DIR),
    _ => panic!("No samples provided for role: {}", arf.role),
  };

  // Access the cache
  let cache = SAMPLE_CACHE.read().expect("Failed to read SAMPLE_CACHE");

  // Retrieve the list of paths for the category
  if let Some(paths) = cache.get(&key) {
    paths.choose(&mut rand::thread_rng()).expect("No samples available in category").clone()
  } else {
    panic!("Role not found in cache: {}", arf.role);
  }
}

/// Initializes the sample cache by scanning the audio-sample directories.
///
/// # Returns
/// A `HashMap` where keys are categories (e.g., "kick", "hats-short") and values are vectors of file paths.
fn initialize_sample_cache() -> HashMap<String, Vec<String>> {
  let mut cache = HashMap::new();

  let categories = vec![
    format!("{}/kick", SAMPLE_SOURCE_DIR),
    format!("{}/perc", SAMPLE_SOURCE_DIR),
    format!("{}/hats/long", SAMPLE_SOURCE_DIR),
    format!("{}/hats/short", SAMPLE_SOURCE_DIR),
  ];

  for category in categories {
    let paths = read_dir(&category)
      .expect(&format!("Failed to read directory: {}", category))
      .filter_map(|entry| entry.ok())
      .filter_map(|entry| entry.path().to_str().map(String::from))
      .collect();
    cache.insert(category, paths);
  }

  cache
}

/// Create bandpass automations with respect to Arf and Melody
/// animation_duration_basis determines how long (in cycles) the effect lasts per-note.
fn bp_bark<'render>(cps: f32, mel: &'render Melody<Note>, arf: &Arf, animation_duration_basis:f32) -> Bp2 {
  let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);
  let line_samples = time::samples_of_line(cps, &mel[0]);
  let lsf = line_samples as f32;

  let mut lowpass_contour: SampleBuffer = Vec::with_capacity(line_samples);

  // the basis of the filter; e.g. sustain level 
  let base_cap: f32 = 1.5f32.powi(match arf.energy {
    Energy::Low => 1i32, Energy::Medium => 2i32, Energy::High => 3i32
  });
  // 
  let peak_cap: f32 = MAX_REGISTER as f32 - lowest_register as f32 - base_cap;

  let min_f: f32 = 2f32.powf(lowest_register as f32 + base_cap);
  let max_f: f32 = 2f32.powf(lowest_register as f32 + base_cap + peak_cap);
  let df: f32 = (max_f - min_f).log2(); 

  
  let mut line_p:f32 = 0f32;

  mel[0].iter().enumerate().for_each(|(i, note)| {
    let note_samples = time::samples_of_note(cps, &note);
    let note_dur_cycles = time::duration_to_cycles(note.0);
    let animation_duration_cycles = animation_duration_basis * match arf.presence {
      Presence::Staccatto => 1f32, Presence::Legato => 2f32, Presence::Tenuto => 3f32, 
    };
    let animation_duration_samples = time::samples_of_cycles(cps, animation_duration_cycles * note_dur_cycles);
    let adf = animation_duration_samples as f32;

    for j in 0..note_samples {
      let note_p = j / note_samples;
      let fx_p = j as f32 / adf;
      if j > animation_duration_samples { 
        lowpass_contour.push(min_f);
      } else {
        lowpass_contour.push(min_f + 2f32.powf(df * (1f32 - fx_p)));
      }
    }
    
    line_p = (line_p + note_samples as f32/lsf) .min(1f32);
  });

  let highpass = vec![MFf];
  (
    highpass,
    lowpass_contour,
    vec![],
  )
}