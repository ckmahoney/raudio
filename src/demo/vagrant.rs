use super::{location, out_dir};
use crate::files;
use crate::synth::{pi2, SRf, SR};
use crate::{presets, render};
static demo_name: &str = "vagrant";

/// Generate a signal representing exactly one period of each component from start Hertz to stop Hertz
fn gen_signal(start: usize, stop: usize) -> Vec<f32> {
  let mut sig: Vec<f32> = vec![];
  (start..stop).for_each(|k| {
    let n_samples = SR / k;
    for j in 0..n_samples {
      let t = j as f32 % SRf;
      let v = (k as f32 * pi2 * t).sin();
      sig.push(v);
    }
  });

  sig
}

/// Generate a signal representing exactly one period of each component from start Hertz to stop Hertz
/// resulting signal is 50 minutes of a "mad scientest" synth
/// no apparent patterns initially. a sweep begins to emerge in 10minutes-15minutes. by 20 minutes the sweep is fully developed over 7 hertz and is a perfect triangle wave.
fn gen_signal_k_length(start: usize, stop: usize) -> Vec<f32> {
  let mut sig: Vec<f32> = vec![];
  (start..stop).for_each(|k| {
    let n_samples = SR / 8;
    for j in 0..n_samples {
      let t = j as f32 % SRf;
      let v = (k as f32 * pi2 * t).sin();
      sig.push(v);
    }
  });

  sig
}

/// Generate a signal representing exactly one period of each component from start Hertz to stop Hertz
/// resulting signal has a clear frequency sweep up. can hear discrete steps at midrange and higher freqs.
fn gen_signal_m_length(start: usize, stop: usize, m: usize) -> Vec<f32> {
  let mut sig: Vec<f32> = vec![];

  (start..stop).for_each(|k| {
    // Calculate the period for the current frequency in samples
    let period = SRf / k as f32;
    let n_samples = (period * m as f32).round() as usize; // m periods

    // Generate samples for m periods of this frequency
    for j in 0..n_samples {
      let t = j as f32 / SRf; // time in seconds
      let v = (k as f32 * 2.0 * std::f32::consts::PI * t).sin();
      sig.push(v);
    }
  });

  sig
}

const MAX_DURATION: f32 = 0.2; // Maximum duration in seconds for the lowest frequency
const MIN_LN_VALUE: f32 = 0.1; // Minimum ln(k) value to avoid large durations

/// Generate a signal representing one period of each component from start Hertz to stop Hertz,
/// Resulting signal has a continuous frequency response.
fn gen_signal_log_time(start: usize, stop: usize, m: usize) -> Vec<f32> {
  let mut sig: Vec<f32> = vec![];

  (start..stop).for_each(|k| {
    // Logarithmic scaling: add a small value to avoid division by very small ln(k) values
    let ln_k = (k as f32).ln().max(MIN_LN_VALUE); // Clamp ln(k) to avoid excessively large durations
    let duration = MAX_DURATION / ln_k;
    let n_samples = (SRf * duration).round() as usize; // number of samples based on scaled duration

    // Generate samples for this frequency
    for j in 0..n_samples {
      let t = j as f32 / SRf; // time in seconds
      let v = (k as f32 * 2.0 * std::f32::consts::PI * t).sin();
      sig.push(v);
    }
  });

  sig
}

#[test]
fn test_gen_signal() {
  let path: String = location(demo_name);
  files::with_dir(&path);
  let signal = gen_signal(24, (SR / 2) - 1);
  let filename = format!("{}/vagrant.wav", path);

  render::engrave::samples(crate::synth::SR, &signal, &filename);
  println!("Completed rendering test signal to {}", filename)
}

#[test]
fn test_gen_signal_k_length() {
  let path: String = location(demo_name);
  files::with_dir(&path);
  let signal = gen_signal_k_length(24, (SR / 2) - 1);
  let filename = format!("{}/vagrant_k_length.wav", path);

  render::engrave::samples(crate::synth::SR, &signal, &filename);
  println!("Completed rendering test signal to {}", filename)
}

#[test]
fn test_gen_signal_m_length() {
  let m = 12;
  for m in 1..15 {
    let path: String = location(demo_name);
    files::with_dir(&path);
    let signal = gen_signal_m_length(24, (SR / 2) - 1, m);
    let filename = format!("{}/vagrant_m_length_m={}.wav", path, m);

    render::engrave::samples(crate::synth::SR, &signal, &filename);
    println!("Completed rendering test signal to {}", filename)
  }
}

#[test]
fn test_gen_signal_log_time() {
  let m = 12;
  for m in 1..15 {
    let path: String = location(demo_name);
    files::with_dir(&path);
    let signal = gen_signal_log_time(24, (SR / 2) - 1, m);
    let filename = format!("{}/vagrant_gen_signal_log_time={}.wav", path, m);

    render::engrave::samples(crate::synth::SR, &signal, &filename);
    println!("Completed rendering test signal to {}", filename)
  }
}
