use super::*;
pub mod bass;
pub mod chords;
pub mod hats;
pub mod kick;
pub mod lead;
pub mod perc;

/// ease in and out
pub fn amp_expr(n_seconds: f32) -> SampleBuffer {
  let n_samples: usize = (crate::synth::SRf * n_seconds) as usize;
  let r = 1f32 / 2f32;

  (0..n_samples).map(|x| (pi * (x as f32 / n_samples as f32) / n_seconds).sin().powf(r)).collect()
}
