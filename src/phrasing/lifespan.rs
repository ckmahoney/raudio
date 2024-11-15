use crate::phrasing::AmpModulation;
use crate::synth::{epi, pi, pi2, SampleBuffer};
use crate::types::timbre::{AmpContour, AmpLifespan};
/// # Component Amplitude Envelopes
///
/// ## Description
///
/// Provides methods for computing a value in [0, 1] given common synthesis parameters.
/// The pub methods accept three parameters: `k`, `x`, and `d`
///
/// Where:
/// `k` typically represents a harmoinc or index (usize)
/// `x` represents the progression of time from [0, 1]
/// `d` is a free parameter representing duration in cycles.
///
/// ## Guarantees
///
/// Functions are guaranteed to be defined for all x in (0,1], and if not defined at 0, returns 1.
/// Functions are guaranteed to be responsive to both x and k and return a value in [0, 1] for all x in [0,1].
/// Functions may optionally respond to d parameter.
///
/// ## Placement
///
/// Currently the methods describe their 2D placement with respect to a spectrogram.
///
///
/// ### Horizontal Placement
/// Horiztonal placement refers to where the body of energy is activated.
/// Left means the signal starts strong and ends weaker,
/// Center means it starts weaker, grows in strength, and falls back down
/// Right means the signal starts weak and ends strong.
///
/// ### Vertical Placement
/// Vertical placement refers to the spectral centroid.
/// Bottom means values near the fundamental are emphasized.
/// Center means values "farther away" (e.g. k > 7) are emphasized.
/// Top means values "way up" (e.g. k > 15) are emphasized.
///
/// Between the two dimensions, horizontal placement is more straightforward to generalize.
/// Vertical placement, when properly implemented, provides a magnificient array of tingly ear candy.

/// This module currently contains a lot of ad hoc functions.
/// Most of them have been predesigned in Desmos and then ported here.
/// Static variables for number names (eg one, six, twenty) are used to help identify the meaningful parts (x, y) of the expressions more easily. It also helps since these are universal constants that are read thousands of times per sample.
use rustfft::num_complex::Complex;
pub static lifespans: [AmpLifespan; 8] = [
  AmpLifespan::Fall,
  AmpLifespan::Burst,
  AmpLifespan::Snap,
  AmpLifespan::Spring,
  AmpLifespan::Pluck,
  AmpLifespan::Bloom,
  AmpLifespan::Pad,
  AmpLifespan::Drone,
];

static neg: f32 = -1f32;
static one: f32 = 1f32;
static two: f32 = 2f32;
static three: f32 = 3f32;
static six: f32 = 6f32;
static twenty: f32 = 20f32;
static K: f32 = 2000f32;
static epsilon: f32 = 0.000001f32;

static min_db: f32 = 180f32;
fn view_db(y: f32) -> f32 {
  (y + min_db) / min_db
}
/// the below three decibel cast and render methods are useful for plotting the
/// amplitudes in a range that make sense to our eyes.
/// All _db_ functions are designed in desmos to expected relative contours of our ear.
fn into_db(y: f32) -> f32 {
  twenty * (y + epsilon).log10()
}

fn from_decibel(x: f32) -> f32 {
  20.0 * (x + epsilon).log10()
}

fn render(y: f32) -> f32 {
  view_db(into_db(y))
}

static relu_coeff: f32 = 0.38;
/// C
fn relu(y: f32) -> f32 {
  let p_a = three * (relu_coeff - y);
  let p_b = one - (two * relu_coeff);
  let denom: f32 = one + (p_a / p_b).exp();
  y / denom
}

#[test]
fn test_render() {
  for x in vec![0f32, 0.001f32, 0.01f32, 0.5f32, 0.99f32, 0.999f32, 1f32] {
    let val = render(x);
    println!("x {} val {}", x, val)
  }
}

fn pluck_a(k: f32, x: f32) -> f32 {
  let scaled_x = pi * x - (pi / k);
  let b: f32 = k - one - (six * (K - k) * scaled_x).tanh();
  b / k
}

pub fn mod_burst(k: usize, x: f32, d: f32) -> f32 {
  let kf = k as f32;
  let t = x;
  let k_scale = neg * six * kf.powf(1f32 / 3f32);
  let x_offset = -2f32;
  let y = (k_scale * t - x_offset).tanh();
  (y / 2f32) + 0.5f32
}

pub fn mod_snap(k: usize, x: f32, d: f32) -> f32 {
  let kf = k as f32;
  let t = x;
  let k_scale = kf.powf(1f32 / 3f32) * epi;
  let exponent = neg * t * k_scale;
  exponent.exp()
}

pub fn mod_spring(k: usize, x: f32, d: f32) -> f32 {
  let t = x;
  let k = 1f32;
  let y = 2f32 * ((t + k).powi(-1i32) - 0.5f32);
  let c: f32 = d.log2().min(2f32).max(6f32);
  (y * c * pi2).sin().abs()
}

pub fn mod_pluck(k: usize, x: f32, d: f32) -> f32 {
  let kf = k as f32;
  let t = x;
  let y1: f32 = 0.5f32 - (24f32 * (t - 0.5f32)).tanh() / 2f32;
  let y2: f32 = one / (kf.powf(one / d) * std::f32::consts::E.powf(pi2 * t));
  let y: f32 = (y1 * y2).sqrt();
  let b: f32 = 2f32.powf(-1f32 * t * (kf * t / d.log2().max(1f32)).sqrt()) * -1f32 * (d * pi2 * (t - 1f32)).tanh();
  pluck_a(kf, t) * y * b
}

pub fn mod_bloom(k: usize, x: f32, d: f32) -> f32 {
  let kf = k as f32;
  let t = x;
  let y: f32 =
    (t / three) + (t.powi(3i32) / three) + (one / six) + (one / six) * ((kf / 16f32) * pi2 * t + (pi2 * d)).sin();
  let c = (one + d).powf(0.33333333);
  let a: f32 = (c * pi2 * t.powf(1.5f32)).tanh();
  let base = Complex::new(t - one, 0f32);
  let b: f32 = -(c * pi2 * base.powf(0.6).re).tanh();
  a * y * b
}

fn entry_bloom(x: f32, k: usize) -> f32 {
  one + (neg * two / (one + (10f32 * pi * x * (k as f32).sqrt()).exp()))
}

///View on desmos:
///https://www.desmos.com/calculator/hreww2fasr
pub fn mod_db_bloom(k: usize, x: f32, d: f32) -> f32 {
  let a = (two * k as f32).sqrt();
  let y = (x * a - a).exp() / k as f32;
  entry_bloom(x, k) * y
}

pub fn mod_pad(k: usize, x: f32, d: f32) -> f32 {
  let t = x;
  let stable_amp = 0.9;
  let g = d.max(0.001) * (k as f32).powf(1.5f32);
  let adds: Vec<f32> = vec![
    t,
    t.powf(one / 3f32),
    t.powf(one / 7f32),
    t.powf(one / 11f32),
    t.powf(one / 13f32),
  ];
  let v: f32 = (one / adds.len() as f32) * adds.iter().map(|x| (pi2 * g * x).sin()).sum::<f32>();
  let y = stable_amp + (one - stable_amp) * v;
  let a = (d.powi(2i32) * pi2 * t).tanh();
  let b = -(d * pi2 * (t - one)).tanh();
  a * y * b
}

pub fn mod_drone(k: usize, x: f32, d: f32) -> f32 {
  let t = x;
  let y: f32 = (4f32 * (d + one) * t).tanh();
  let a: f32 = one;
  let b: f32 = -(pi2 * (t - one) * (2f32 + d).sqrt()).tanh();
  a * y * b
}

fn h_entry(y: f32) -> f32 {
  let p = 24f32 * pi * y;
  one + neg * 2f32 / (one + p.exp())
}

/// Depends on the fact that this h_entry is a steep tanh contour whose integral is near 1.
fn h_exit(y: f32) -> f32 {
  neg * (-0.35 * y).exp() * h_entry(y - one)
}

use crate::synth::SR;
static e_at_selected_x: f32 = 1.718;
pub fn mod_db_fall(k: usize, x: f32, d: f32) -> f32 {
  let a: f32 = (x * (one - (one / (k as f32 + one).sqrt()))).exp();
  let b: f32 = one - (a - one) / e_at_selected_x;
  let d: f32 = ((K - k as f32) as f32 / K as f32) * (b * b) / ((k as f32).powi(2i32));
  let y: f32 = d * h_exit(x);

  y.max(0f32)
}

fn translate_pluck(k: usize) -> f32 {
  let kf = k as f32;
  0.5 - 0.5 * (kf / K)
}

fn scale_pluck(k: usize) -> f32 {
  let kf = k as f32;
  (neg * pi * kf / K) + pi
}

fn pluck_exit(k: usize, x: f32) -> f32 {
  let kf = k as f32;
  one - x * (kf * kf - one).powf(one / three)
}

pub fn mod_db_pluck(k: usize, x: f32, d: f32) -> f32 {
  let m_x = scale_pluck(k) * (x - translate_pluck(k));
  let a = (m_x).tanh();
  let y = neg * (a / 2f32) + 0.5f32;
  relu(y * pluck_exit(k, x))
}

#[test]
fn test_shaper_entry() {
  let start = 0f32;
  let end = 1f32;
  assert!(
    h_entry(start) == 0f32,
    "Entry shaper must a zeroing effect at the start of the signal"
  );
  assert!(
    h_entry(end) == 1f32,
    "Entry shaper must have no effect at the end of the signal"
  )
}

#[test]
fn test_shaper_exit() {
  let start = 0f32;
  let end = 1f32;
  assert!(
    h_exit(start) == 1f32,
    "Exit shaper must no effect at the start of the signal"
  );
  assert!(
    h_exit(end) == 0f32,
    "Exit shaper must have a zeroing effect at the end of the signal"
  )
}

pub fn mod_lifespan(n_samples: usize, n_cycles: f32, lifespan: &AmpLifespan, k: usize, d: f32) -> AmpModulation {
  let mut modulator: AmpModulation = vec![0f32; n_samples];
  let ns = n_samples as f32;

  for (i, sample) in modulator.iter_mut().enumerate() {
    let x = (i + 1) as f32 / ns;
    *sample = match lifespan {
      AmpLifespan::Fall => mod_db_fall(k, x, n_cycles),
      AmpLifespan::Burst => mod_burst(k, x, n_cycles),
      AmpLifespan::Snap => mod_snap(k, x, n_cycles),
      AmpLifespan::Spring => mod_spring(k, x, n_cycles),
      AmpLifespan::Pluck => mod_db_pluck(k, x, n_cycles),
      AmpLifespan::Bloom => mod_bloom(k, x, n_cycles),
      AmpLifespan::Pad => mod_pad(k, x, n_cycles),
      AmpLifespan::Drone => mod_drone(k, x, n_cycles),
    };
  }

  modulator
}

/// Create an amplitude modulation buffer sampled from the provided variant.
/// actual duration is determined by n_samples; n_cycles may or not be applied based on the selected modulator.
pub fn sample_lifespan(n_samples: usize, lifespan: &AmpLifespan, k: usize, n_cycles: f32) -> AmpModulation {
  let mut modulator: AmpModulation = vec![0f32; n_samples];
  let ns = n_samples as f32;

  for (i, sample) in modulator.iter_mut().enumerate() {
    let x = (i + 1) as f32 / ns;
    *sample = match lifespan {
      AmpLifespan::Fall => mod_db_fall(k, x, n_cycles),
      AmpLifespan::Burst => mod_burst(k, x, n_cycles),
      AmpLifespan::Snap => mod_snap(k, x, n_cycles),
      AmpLifespan::Spring => mod_spring(k, x, n_cycles),
      AmpLifespan::Pluck => mod_db_pluck(k, x, n_cycles),
      AmpLifespan::Bloom => mod_bloom(k, x, n_cycles),
      AmpLifespan::Pad => mod_pad(k, x, n_cycles),
      AmpLifespan::Drone => mod_drone(k, x, n_cycles),
    };
  }

  modulator
}

pub fn select_lifespan(contour: &AmpContour) -> AmpLifespan {
  match contour {
    AmpContour::Chops => AmpLifespan::Spring,
    AmpContour::Fade => AmpLifespan::Fall,
    AmpContour::Surge => AmpLifespan::Bloom,
    AmpContour::Throb => AmpLifespan::Pad,
    AmpContour::Flutter => AmpLifespan::Spring,
  }
}
#[cfg(test)]
mod test {
  use super::*;
  use crate::analysis;

  fn assert_lifespan_mod(lifespan: &AmpLifespan, mod_signal: &Vec<f32>) {
    for (i, y) in mod_signal.iter().enumerate() {
      assert!(
        false == y.is_nan(),
        "Modulation lifecycle {:#?} must only produce numeric values. Got NAN at index {}",
        lifespan,
        i
      );
      assert!(
        *y <= 1f32,
        "Modulation lifecycle {:#?} must not produce any values above 1. Found {} at {}",
        lifespan,
        y,
        i
      );
      assert!(
        *y >= 0f32,
        "Modulation lifecycle {:#?} must not produce any values below 0. Found {} at {}",
        lifespan,
        y,
        i
      );
    }

    let rms = analysis::volume::rms(&mod_signal);
    assert!(
      rms < 1f32,
      "Modulation lifecycle {:#?} must produce an RMS value less than 1. Got {}",
      lifespan,
      rms
    );
    assert!(
      rms > 0f32,
      "Modulation lifecycle {:#?} must produce an RMS value greater than 0. Got {}",
      lifespan,
      rms
    );
  }

  #[test]
  /// Show that each modulator has all values in [0, 1]
  /// and that the mean modulation value is in [0, 1]
  fn verify_valid_modulation_range() {
    let n_samples = 48000 * 90usize;
    let n_cycles = 64f32;
    for lifespan in &lifespans {
      let mod_signal = mod_lifespan(n_samples, n_cycles, &lifespan, 1usize, 0f32);
      assert_lifespan_mod(&lifespan, &mod_signal);
      println!("Passes lifespan mod test {:#?}", &lifespan)
    }
  }

  /// Show that the RMS value is consistent over arbitrary sample frequency
  #[test]
  fn verify_constant_over_sample_rate() {
    for index in 1..=10usize {
      let n_samples = index * 4800;
      let n_cycles = 1f32;

      for lifespan in &lifespans {
        let mod_signal = mod_lifespan(n_samples, n_cycles, &lifespan, 1usize, 0f32);
        assert_lifespan_mod(&lifespan, &mod_signal);
      }
    }
  }
}
