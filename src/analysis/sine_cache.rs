use crate::synth::{pi2, SR};

struct SineCache {
  samples: Vec<f32>,
  reso: f32,
}

// benchmark shows this implementation is 2x slower than std implementation
impl SineCache {
  /// Create a sample bank at 1Hertz for cached lookups
  pub fn new(resolution: usize) -> Self {
    let n_samples = SR * resolution;
    let sr = n_samples as f32;
    let period_samples = (0..n_samples).map(|t| (pi2 * t as f32 / sr).sin()).collect();

    SineCache {
      samples: period_samples,
      reso: resolution as f32,
    }
  }

  pub fn get(&self, frequency: f32, phase: f32) -> f32 {
    let period_samples = self.samples.len() as f32;

    // Compute how many samples correspond to a single cycle at the given frequency
    let samples_per_cycle = SR as f32 / frequency;

    // Calculate an effective phase based on the input phase and frequency.
    // This converts the phase to a sample index in the buffer.
    // The phase should be within [0, 1), representing a full cycle.
    let phase_in_samples = phase * samples_per_cycle;

    // Map the phase to the buffer's scale.
    let index = (phase_in_samples * period_samples / samples_per_cycle) % period_samples;
    let index1 = index.floor() as usize;
    let index2 = (index1 + 1) % self.samples.len(); // Ensure it wraps around.
    let mix = index.fract();

    // Linear interpolation between the two nearest samples.
    self.samples[index1] * (1.0 - mix) + self.samples[index2] * mix
  }
}

#[cfg(test)]
mod test_sine_cache {
  use super::*;
  use crate::files;
  use crate::render;
  use crate::time::measure;

  static test_dir: &str = "dev-audio/benchmark";

  static TEST_FREQS: [f32; 6] = [60f32, 245f32, 555f32, 1288f32, 4001f32, 9999f32];

  static TEST_DURS: [usize; 5] = [SR / 4, SR, SR * 3, SR * 12, SR * 60];

  #[test]
  fn test_get_sine() {
    let resolution = 4usize;
    let cache = SineCache::new(resolution);

    let sr = SR as f32;

    for &n_samples in TEST_DURS.iter() {
      let n = n_samples as f32;
      for &freq in TEST_FREQS.iter() {
        fn get_bad(freq: f32, n_samples: usize) -> Vec<f32> {
          let sr = SR as f32;
          (0..n_samples).map(|t| (pi2 * freq * (t as f32 / sr)).sin()).collect()
        }

        fn get_cached(cache: &SineCache, freq: f32, n_samples: usize) -> Vec<f32> {
          let sr = SR as f32;
          (0..n_samples).map(|t| cache.get(freq, freq * (t as f32 / sr))).collect()
        }

        let (samples, duration) = measure(|| get_bad(freq, n_samples));
        println!("Result of get std: {}, Time taken: {:?}", n_samples, duration);

        files::with_dir(test_dir);
        let filename = format!(
          "{}/test-std-n_samples-{}-sine-{}-cost-{:#?}.wav",
          test_dir, n_samples, freq, duration
        );
        match render::pad_and_mix_buffers(vec![samples]) {
          Ok(signal) => {
            // render::samples_f32(44100, &signal, &filename);
          }
          Err(msg) => {
            panic!("Failed to mix and render audio: {}", msg)
          }
        }

        let (samples, duration) = measure(|| get_cached(&cache, freq, n_samples));
        println!("Result of get cached: {}, Time taken: {:?}", n_samples, duration);
        let filename = format!(
          "{}/test-cached-n_samples-{}-sine-{}-cost-{:#?}.wav",
          test_dir, n_samples, freq, duration
        );
        match render::pad_and_mix_buffers(vec![samples]) {
          Ok(signal) => {
            // render::samples_f32(44100, &signal, &filename);
          }
          Err(msg) => {
            panic!("Failed to mix and render audio: {}", msg)
          }
        }
      }
    }
  }
}
