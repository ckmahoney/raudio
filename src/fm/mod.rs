use crate::render::engrave;
use crate::synth::{pi, pi2, NFf, SRf, SR, NF};
use rand::{self, thread_rng, Rng};
use crate::phrasing::ranger::{self, Knob};

mod operator;
mod dex;
mod testhelp;
use testhelp::*;
use operator::*;
use crate::render::get_knob;
use crate::phrasing::ranger::KnobMacro;
use crate::types::synthesis::{MacroMotion};
use crate::analysis::freq::slice_signal;
use crate::analysis::melody::{eval_odr_level, LevelMacro, Levels, ODR, ODRMacro};

pub fn mul_envelopes(a:Vec<f32>, b:Vec<f32>, compress:bool) -> Vec<f32> {
  
  let target_size = if compress { 
    a.len().min(b.len())
  } else {
    a.len().max(b.len())
  };

  let resized_a = slice_signal(&a, 0f32, 1f32, target_size);
  let resized_b = slice_signal(&b, 0f32, 1f32, target_size);
  resized_a.iter().enumerate().map(|(i, x)| x * b[i]).collect()
}

/// Generates a canonical FM signal over `n_cycles` of the carrier's fundamental period,
/// without any input checks or clamping.
///
/// - `cps`: cycles per second for the *carrier* (used to compute total samples),
/// - `gain`: amplitude scale factor,
/// - `carrier_freq`: carrier frequency in Hz,
/// - `mod_freq`: modulator frequency in Hz,f
/// - `mod_index`: modulation index (dimensionless),
/// - `n_cycles`: how many cycles of the carrier to render.
///
/// Returns a `Vec<f32>` of samples at 48kHz sample rate.
pub fn generate_signal(
  cps: f32, gain: f32, carrier_freq: f32, mod_freq: f32, mod_index: f32, n_cycles: f32,
) -> Vec<f32> {
  // Hardcode sample rate (48kHz) for illustration; adapt to your needs.
  let sample_rate = 48_000.0;

  // Number of samples for `n_cycles` of the carrier's fundamental period
  let n_samples = (n_cycles * sample_rate / cps) as usize;
  let dt = 1.0 / sample_rate;

  let mut signal = Vec::with_capacity(n_samples);
  let mut t = 0.0;

  for _ in 0..n_samples {
    // Canonical FM phase: 2π f_c t + I sin(2π f_m t)
    let phase = pi2 * carrier_freq * t + mod_index * (pi2 * mod_freq * t).sin();
    let sample = gain * phase.sin();

    signal.push(sample);
    t += dt;
  }

  signal
}

/// Generates a compound FM signal over `n_cycles` of the carrier's fundamental period,
/// using a sequence of modulation weights and frequencies applied hierarchically.
///
/// - `cps`: cycles per second for the *carrier* (used to compute total samples),
/// - `gain`: amplitude scale factor,
/// - `carrier_freq`: carrier frequency in Hz,
/// - `mod_chain`: a vector of `(mod_index, mod_freq)` tuples defining the modulation chain,
/// - `n_cycles`: how many cycles of the carrier to render.
///
/// Returns a `Vec<f32>` of samples at 48kHz sample rate.
pub fn generate_compound_signal(
  cps: f32,
  gain: f32,
  carrier_freq: f32,
  mod_chain: Vec<(f32, f32)>, // (modulation index, modulation frequency)
  n_cycles: f32,
) -> Vec<f32> {
  let sample_rate = 48_000.0; // Sample rate (hardcoded at 48kHz)
  let dt = 1.0 / sample_rate; // Time step
  let n_samples = (n_cycles * sample_rate / cps) as usize; // Total samples for the given cycles

  let mut signal = Vec::with_capacity(n_samples);

  for n in 0..n_samples {
    let t = n as f32 * dt; // Current time
    let mut mod_signal = 0.0;

    // Compute hierarchical modulation signal
    for (mod_index, mod_freq) in mod_chain.iter().rev() {
      mod_signal = mod_index * (2.0 * std::f32::consts::PI * mod_freq * t + mod_signal).sin();
    }

    // Generate the final signal using the carrier frequency and modulation chain
    let sample = gain * (2.0 * std::f32::consts::PI * carrier_freq * t + mod_signal).sin();
    signal.push(sample);
  }

  signal
}

pub fn compute_max_mod_freq(max_carrier_freq: f32, max_mod_index: f32) -> f32 {
  // Solve for f_m:
  //   f_m < (sample_rate/2 - max_carrier_freq) / (max_mod_index + 1)
  let numerator = (SRf * 0.5) - max_carrier_freq;
  let denominator = max_mod_index + 1.0;
  let raw_max_mod_freq = numerator / denominator;

  // Clamp if negative (i.e. no valid mod freq that won't alias).
  if raw_max_mod_freq <= 0.0 {
    0.0
  } else {
    raw_max_mod_freq
  }
}

fn compound_sine(weights: Vec<f32>, x: f32) -> f32 {
  // Start from the innermost sine (final entry in weights)
  let mut result = x;
  for &weight in weights.iter().rev() {
    result = (weight * result).sin();
  }
  result
}

fn nested_sine_up_to_n(n: u32, x: f32) -> f32 {
  let mut result = x; // Start with the innermost term
  for i in (1..=n).rev() {
    // Reverse iterate from n to 1
    result = if i % 2 == 0 {
      (result / i as f32).sin() // Reciprocal weights for even terms
    } else {
      (i as f32 * result).sin() // Large weights for odd terms
    };
  }
  result
}

fn nested_sine_reverse_n(n: u32, x: f32) -> f32 {
  let mut result = x; // Start with the innermost term
  for i in 1..=n {
    // Iterate normally from 1 to n
    result = if i % 2 == 0 {
      (result / i as f32).sin() // Reciprocal weights for even terms
    } else {
      (i as f32 * result).sin() // Large weights for odd terms
    };
  }
  result
}

fn render_many(cps: f32, freq: f32, n_cycles: f32, depth: u32) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
  let sample_rate = SRf; // Sample rate (f32)
  let dt = 1.0 / sample_rate; // Time step
  let n_samples = (n_cycles * sample_rate / cps) as usize; // Total samples for the given duration

  let mut nested_signal = Vec::with_capacity(n_samples);
  let mut reverse_signal = Vec::with_capacity(n_samples);
  let mut combined_signal = Vec::with_capacity(n_samples);

  for i in 0..n_samples {
    let t = i as f32 * dt; // Current time
    let x = freq * t * cps; // Argument for sine functions

    // Generate the signals
    let nested = nested_sine_up_to_n(depth, x);
    let reverse = nested_sine_reverse_n(depth, x);
    let combined = (nested + reverse) / 2.0; // Simple average of both signals

    nested_signal.push(nested);
    reverse_signal.push(reverse);
    combined_signal.push(combined);
  }

  (nested_signal, reverse_signal, combined_signal)
}
fn remaining_bandwidth(
  nf: f32,                     // Nyquist frequency
  carrier: f32,                // Carrier frequency
  modulators: Vec<(f32, f32)>, // Vec of (mod_index, mod_freq)
) -> f32 {
  let mut bandwidth = nf - carrier; // Initial available bandwidth
  for (mod_index, mod_freq) in modulators {
    let mod_bandwidth = 2.0 * mod_index * mod_freq; // Bandwidth contribution of this modulator
    bandwidth -= mod_bandwidth;
    if bandwidth < 0.0 {
      return 0.0; // Exceeded Nyquist, no remaining bandwidth
    }
  }
  bandwidth
}

fn step_range(a: i32, b: i32) -> Vec<i32> {
  if a <= b {
    (a..=b).collect() // Forward
  } else {
    (b..=a).rev().collect() // Reverse
  }
}

#[test]
fn the_fm_song() {
  let nf = NFf;
  let car_f = 250f32;

  let mut song: Vec<f32> = vec![];
  for j in vec![1f32, 2f32, 4f32, 8f32, 12f32, 16f32].iter() {
    let min = 1;
    let max = 4;
    let r = min..=max;
    let carrier = car_f * *j as f32;

    let r = if j % 2f32 != 0f32 {
      step_range(min, max)
    } else {
      step_range(max, min)
    };
    for n in r {
      // attempt to keep the same 'groove' as frequency increases.
      let modulator_playback_rate = 1f32;

      let w = n as f32;
      let n_cycles = 2f32.powi((n - 4) as i32);
      let n_cycles = 2f32.powi(4);
      let modulators: Vec<(f32, f32)> = vec![(w, 15.0), (w, 3.0), (w, 9.0), (w, 2.0), (w, 1.0)];

      // audible frequency values for modualtion rate
      // (and also here very intionally harmonic and zero sum)
      // produce warm rich tonal space
      let modulators: Vec<(f32, f32)> = vec![(w, 300.0), (w, 100.0), (w, 150.0), (w, 50.0), (w, 25.0)]
        .iter()
        .map(|(w, m)| (*w, modulator_playback_rate * *m))
        .collect();

      // hmm, rearranging lowest to highest has a similar character
      // but feels brighter
      let modulators: Vec<(f32, f32)> = vec![(w, 25.0), (w, 50.0), (w, 100.0), (w, 150.0), (w, 300.0)]
        .iter()
        .map(|(w, m)| (*w, modulator_playback_rate * *m))
        .collect();

      // wow! a mixed up / random sort order is in between.
      let modulators: Vec<(f32, f32)> = vec![(w, 150.0), (w, 25.0), (w, 50.0), (w, 300.0), (w, 100.0)]
        .iter()
        .map(|(w, m)| (*w, modulator_playback_rate * *m))
        .collect();

      println!("Interesting! It seems that for a set of operators, they all describe a central timbre. Applying them from highest to lowest is the 'darkest' version of the sound, and applying them lowest to highest the 'brightest'. Any other mix will produce an intermediary verison of the same spectral contents.");

      let remaining = remaining_bandwidth(nf, carrier, modulators.clone());
      println!("Remaining Bandwidth: {}", remaining);

      let sig = generate_compound_signal(1f32, 0.1f32, carrier, modulators.clone(), n_cycles);
      let filename = format!("dev-audio/test-mod-chain-{}-n-{}", carrier, n);
      engrave::samples(SR, &sig, &filename);
      song.extend(sig)
    }
  }
  let filename = format!("dev-audio/mod-song");
  engrave::samples(SR, &song, &filename);
}

#[test]
fn m3ain() {
  let nf = NFf;
  let car_f = 333.3 / 3f32;

  let mut song: Vec<f32> = vec![];
  for j in vec![1.5f32.powi(2i32), 4f32, 1.5f32, 1f32, 2f32 / 3f32, 1f32 / 2f32].iter() {
    let min = 8;
    let max = 2;
    let r = min..=max;
    let carrier = car_f * *j as f32;

    let modulator_playback_rate = 1f32 / 4f32;

    let r = if j % 2f32 != 0f32 {
      step_range(min, max)
    } else {
      step_range(max, min)
    };
    for n in r {
      let w = n as f32;
      let n_cycles = 2f32.powi((n - 4) as i32);
      let modulators: Vec<(f32, f32)> = vec![
        (w, 2.0), // (mod_index, mod_freq)
        (w, 2.0),
        (w, 2.0),
        (w, 2.0),
        (w, 2.0),
      ];

      let modulators: Vec<(f32, f32)> = vec![
        (w, 1f32), // (mod_index, mod_freq)
        (w, 1f32 / 2f32.powi(3i32)),
        (w, 2f32),
        (w, 1f32 / 2f32.powi(5i32)),
      ];

      let www = modulator_playback_rate;
      let modulators: Vec<(f32, f32)> = vec![
        (w, www * 1f32), // (mod_index, mod_freq)
        (w, www * 1f32 / 2f32.powi(3i32)),
        (w, www * 2f32),
        (w, www * 1f32 / 2f32.powi(5i32)),
      ];

      let www = 1f32;
      let modulators: Vec<(f32, f32)> = vec![
        (w, www * 3f32), // (mod_index, mod_freq)
        (w, www * 2f32 / 2f32.powi(3i32)),
        (w, www * 5f32),
        (w, www * 2f32 / 2f32.powi(5i32)),
      ];

      // .iter().enumerate().map(|(j, (w, mul))| {
      //     // let weight = w * (j+1) as f32;
      //     (w, *mul)
      // }).collect();

      let remaining = remaining_bandwidth(nf, carrier, modulators.clone());
      println!("Remaining Bandwidth: {}", remaining);

      if remaining > 0.0 {
        println!("Safe for more modulation.");
      } else {
        println!("Exceeded Nyquist. Adjust parameters.");
      }

      let sig = generate_compound_signal(1f32, 0.1f32, carrier, modulators.clone(), n_cycles);
      let filename = format!("dev-audio/test-mod-chain-{}-n-{}", carrier, n);
      engrave::samples(SR, &sig, &filename);
      song.extend(sig)
    }
  }
  let filename = format!("dev-audio/mod-song");
  engrave::samples(SR, &song, &filename);
}

#[test]
fn ma2in() {
  let n = 5; // Depth of the nested sine
  let x = std::f32::consts::PI / 4.0; // Example input value

  // Largest term as outermost
  let result_outermost = nested_sine_up_to_n(n, x);
  println!("Result (largest as outermost): {}", result_outermost);

  // Largest term as innermost
  let result_innermost = nested_sine_reverse_n(n, x);
  println!("Result (largest as innermost): {}", result_innermost);
  let car_freq = 330f32;

  for n in (114..118) {
    let (one, two, three) = render_many(1f32, car_freq, 9f32, n);

    let filename = format!("dev-audio/test-recur-forward-freq-{}-n-{}", car_freq, n);

    // engrave::samples(SR, &one, &filename);
    let filename = format!("dev-audio/test-recur-reverse-freq-{}-n-{}", car_freq, n);
    engrave::samples(SR, &two, &filename);
    // let filename = format!("dev-audio/test-recur-combined-freq-{}-n-{}", car_freq,n);
    //     engrave::samples(SR, &three, &filename);
  }
}

#[test]
fn test_fm() {
  let cps: f32 = 1.5f32;
  let gain: f32 = 0.1f32;
  let carrier: f32 = 330f32;
  let mod_freq: f32 = 100f32;
  let mod_index: f32 = 3f32;
  let n_cycles = 9f32;

  for i in (1..20) {
    let car_freq = i as f32 * 10f32 * 6f32;
    let mod_freq = compute_max_mod_freq(car_freq, 12f32);
    println!("For a carrier of {} the max modulator value is {}", car_freq, mod_freq);

    let result = generate_signal(cps, gain, car_freq, mod_freq, mod_index, n_cycles);
    let filename = format!(
      "dev-audio/test-fm-carrier-{}-modulator-{}-mod_index-{}",
      carrier, mod_freq, mod_index
    );

    engrave::samples(SR, &result, &filename);
  }
}
