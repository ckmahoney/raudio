use crate::synth::{SRf, SR};
use crate::types::synthesis::{Note, Ratio};
use std::collections::HashMap;
use std::time::Instant;

/// Given dynamic playback rate and constant sample rate,
/// determines the number of samples required to recreate
/// one second of audio signal.
pub fn samples_per_cycle(cps: f32) -> usize {
  (SRf / cps) as usize
}

pub fn cycles_from_n(cps: f32, n: usize) -> f32 {
  let one = samples_per_cycle(cps) as f32;
  n as f32 / one
}

/// Given a duration in seconds, return the number of samples representing this length
pub fn samples_of_dur(cps: f32, dur: f32) -> usize {
  samples_of_seconds(cps, dur)
}

/// Given a duration in seconds, return the number of samples representing this length
pub fn samples_of_seconds(cps: f32, dur: f32) -> usize {
  samples_from_dur(cps, dur)
}

/// Given a duration in milliseconds, return the number of samples representing this length
pub fn samples_of_milliseconds(cps: f32, dur_ms: f32) -> usize {
  samples_from_dur(cps, dur_ms / 1000f32)
}

/// Given a duration in seconds, return the number of samples representing this length
#[inline]
pub fn samples_from_dur(cps: f32, dur: f32) -> usize {
  (SRf * dur.abs() / cps) as usize
}

/// Provides the number of samples required to span k cycles at cps.
#[inline]
pub fn samples_of_cycles(cps: f32, k: f32) -> usize {
  (samples_per_cycle(cps) as f32 * k.abs()) as usize
}

/// Provides the time in seconds for a given duration ratio.
pub fn dur(cps: f32, ratio: &Ratio) -> f32 {
  step_to_seconds(cps, ratio)
}

/// Provides the time in seconds for a given duration ratio.
pub fn step_to_samples(cps: f32, ratio: &Ratio) -> usize {
  samples_of_seconds(cps, step_to_seconds(cps, ratio))
}

/// Provides the time in seconds for a given duration ratio.
pub fn step_to_seconds(cps: f32, ratio: &Ratio) -> f32 {
  duration_to_cycles((ratio.0.abs(), ratio.1.abs())) / cps
}

pub fn duration_to_cycles((numerator, denominator): Ratio) -> f32 {
  (numerator as f32 / denominator as f32).abs()
}

/// Given a line (vec of notes), determine the total number of cycles it requests.
pub fn count_cycles(line: &Vec<Note>) -> f32 {
  line.iter().fold(0f32, |acc, (duration, _, _)| acc + duration_to_cycles(*duration))
}

/// Given a note, determine the total number of samples it requests.
pub fn samples_of_note(cps: f32, (duration, _, _): &Note) -> usize {
  samples_of_cycles(cps, duration_to_cycles(*duration))
}

pub fn samples_of_line(cps: f32, line: &Vec<Note>) -> usize {
  samples_of_cycles(cps, count_cycles(line))
}

/// Measures the execution time of a function.
///
/// # Arguments
///
/// * `f` - A closure to execute for which the execution time is measured.
///
/// # Returns
///
/// A tuple containing the result of the function and the duration it took to execute.
pub fn measure<T, F: FnOnce() -> T>(f: F) -> (T, std::time::Duration) {
  let start = Instant::now(); // Start timing before the function is called.
  let result = f(); // Call the function and store the result.
  let duration = start.elapsed(); // Calculate how long it took to call the function.
  (result, duration) // Return the result and the duration.
}

/// Given a duration in seconds and select constraints,
/// Return the cps and size required to produce a track of length n_seconds
type Timing = (f64, usize);
fn get_timing(n_seconds: f64, min_cps: f64, base: f64, cpc: f64, min_size: usize) -> Timing {
  let n_cycles = base.powi(min_size as i32) * cpc;
  let cps = n_cycles / n_seconds;
  if cps < min_cps {
    get_timing(n_seconds, min_cps, base, cpc, min_size + 1)
  } else {
    (cps, min_size)
  }
}
#[derive(Hash, Eq, PartialEq)]
struct Duration {
  whole_seconds: i32,
  divisor: i32,
}

/// Given a length in seconds, get an approximate size (assuming base 2, cpc 4) for that duration.
/// Supports up to 5 minutes with the internal default map.
pub fn get_approx_size(seconds: f64) -> usize {
  let durations = [
    (15.0, 3),
    (30.0, 4),
    (60.0, 5),
    (90.0, 5),
    (120.0, 6),
    (200.0, 6),
    (300.0, 7),
  ];

  durations
    .iter()
    .filter(|(duration, _)| *duration >= seconds)
    .min_by(|(d1, _), (d2, _)| d1.partial_cmp(d2).unwrap())
    .map_or(0, |(_, size)| *size) // Return 0 if no match is found
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_get_timing() {
    let n_seconds: f64 = 15.0;
    let min_cps: f64 = 0.5;
    let min_size: usize = 3;
    let base: f64 = 2.0;
    let cpc: f64 = 4.0;

    let expected: Timing = (0f64, 0usize);
    let actual: Timing = get_timing(n_seconds, min_cps, base, cpc, min_size);
    println!("got timing {:?}", actual)
  }
  #[test]
  fn test_with_size_map() {
    let test_cases = vec![
      (7.0, 0.5, 3, 2.0, 4.0), // These tuples are (seconds, min_cps, min_size, base, cpc)
      (12.0, 0.5, 3, 2.0, 4.0),
      (17.0, 0.5, 4, 2.0, 4.0),
      (60.0, 0.5, 5, 2.0, 4.0),
      (61.0, 0.5, 6, 2.0, 4.0),
      (89.9, 0.5, 6, 2.0, 4.0),
      (94.0, 0.5, 5, 2.0, 4.0),
      (123.0, 0.5, 5, 2.0, 4.0),
      (245.0, 0.5, 6, 2.0, 4.0),
      (299.0, 0.5, 5, 2.0, 4.0),
    ];

    for (seconds, min_cps, min_size, base, cpc) in test_cases {
      let size = get_approx_size(seconds);
      let actual: Timing = get_timing(seconds, min_cps, base, cpc, size);
      println!(
        "Test for {} seconds -> Expected size: {}, Got a timing: {:?}",
        seconds, size, actual
      );
    }
  }
}
