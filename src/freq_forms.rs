//! Generation of conventional waveforms using frequency-domain
use std::f32::consts::PI;
use crate::synth_config::SynthConfig;

use std::collections::HashMap;

struct PhaseTracker {
    phases: HashMap<i32, f32>, // Rounded frequency as key
    precision: f32,            // Rounding precision
}

impl PhaseTracker {
    pub fn new(precision: f32) -> Self {
        PhaseTracker {
            phases: HashMap::new(),
            precision,
        }
    }

    fn round_frequency(&self, freq: f32) -> i32 {
        (freq / self.precision).round() as i32
    }

    pub fn get_phase(&self, freq: f32) -> f32 {
        let key = self.round_frequency(freq);
        *self.phases.get(&key).unwrap_or(&0.0)
    }

    pub fn update_phase(&mut self, freq: f32, phase: f32) {
        let key = self.round_frequency(freq);
        self.phases.insert(key, phase);
    }
}


pub fn normalize_waveform(samples: &mut [f32]) {
    let (min, max) = samples.iter().fold((f32::MAX, f32::MIN), |(min, max), &val| {
        (min.min(val), max.max(val))
    });
    let amplitude_range = max - min;
    samples.iter_mut().for_each(|sample| {
        *sample = (*sample - min) / amplitude_range * 2.0 - 1.0;
    });
}

pub fn sine(config: &SynthConfig, t: u32, freq: f32, bias: Option<f32>) -> f32 {
    let phase_offset = 0.0;
    let t = t as f32 / config.sample_rate as f32;
    (2.0 * PI * freq * t as f32 + phase_offset).sin() * config.amplitude_scaling
}

pub fn square(config: &SynthConfig, t: u32, freq: f32, bias: Option<f32>) -> f32 {
    let phase_offset = 0.0;
    let nyquist = config.sample_rate as f32 / 2.0;
    let max_harmonic = (nyquist / freq).floor() as i32;
    let t = t as f32 / config.sample_rate as f32;
    (1..=max_harmonic).step_by(2).fold(0.0, |acc, n| {
        acc + ((2.0 * PI * freq * n as f32 * t + phase_offset).sin() / n as f32)
    })
}

pub fn sawtooth(config: &SynthConfig, t: u32, freq: f32, bias: Option<f32>) -> f32 {
    let adjusted_freq = freq + config.tuning_offset_hz;
    let nyquist = config.sample_rate as f32 / 2.0;
    let max_harmonic = (nyquist / adjusted_freq).floor() as i32;
    let t = t as f32 / config.sample_rate as f32;
    let mut sum = 0.0;
    for n in 1..=max_harmonic {
        let harmonic_bias = (n as f32 * bias.unwrap_or(0.5)).rem_euclid(1.0);
        sum += (2.0 * std::f32::consts::PI * adjusted_freq * n as f32 * t + config.phase_offset + harmonic_bias).sin() / n as f32;
    }
    // Normalize the sum to keep it within -1.0 to 1.0
    (sum / max_harmonic as f32) * config.amplitude_scaling
}

pub fn triangle(config: &SynthConfig, t: u32, freq: f32, bias: Option<f32>) -> f32 {
    let adjusted_freq = freq + config.tuning_offset_hz;
    let nyquist = config.sample_rate as f32 / 2.0;
    let max_harmonic = (nyquist / adjusted_freq).floor() as i32;
    let t = t as f32 / config.sample_rate as f32;
    let mut sum = 0.0;
    for n in (1..=max_harmonic).step_by(2) {
        let harmonic_bias = (n as f32 * bias.unwrap_or(0.5)).rem_euclid(1.0);
        sum += (2.0 * std::f32::consts::PI * adjusted_freq * n as f32 * t + config.phase_offset + harmonic_bias).sin() / (n as f32).powi(2);
    }
    sum * config.amplitude_scaling
}


#[cfg(test)]
mod tests {
    use super::*;

    // Define a basic SynthConfig for testing
    fn test_config() -> SynthConfig {
        SynthConfig {
            sample_rate: 44100,
            min_frequency: 20.0,
            max_frequency: 20000.0,
            amplitude_scaling: 1.0,
            phase_offset: 0.0,
            tuning_offset_hz: 0.0,
            cps: 1.0
        }
    }

    #[test]
    fn test_square_wave_basic() {
        let config = test_config();
        let sample = square(&config, 0, 440.0, Some(0.0));
        assert!(sample >= -1.0 && sample <= 1.0, "Square wave sample is not within expected range.");
    }

    #[test]
    fn test_sawtooth_wave_basic() {
        let config = test_config();
        let sample = sawtooth(&config, 0, 440.0, Some(0.5));
        assert!(sample >= -1.0 && sample <= 1.0, "Sawtooth wave sample is not within expected range.");
    }
}