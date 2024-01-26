//! Generation of conventional waveforms using time-domain

use crate::synth_config::SynthConfig;
use std::f32::consts::PI;
use crate::render::Ugen;


pub fn sine(config: &SynthConfig, t: u32, freq: f32, _bias: Option<f32>) -> f32 {
    let adjusted_freq = freq + config.tuning_offset_hz;
    let phase = t as f32 * adjusted_freq * 2.0 * PI / config.sample_rate as f32;
    (phase + config.phase_offset).sin() * config.amplitude_scaling
}

pub fn sawtooth(config: &SynthConfig, t: u32, freq: f32, bias: Option<f32>) -> f32 {
    let adjusted_freq = freq + config.tuning_offset_hz;
    let pos = (t as f32 * adjusted_freq % config.sample_rate as f32) / config.sample_rate as f32;
    let bias_val = bias.unwrap_or(0.5);
    2.0 * (pos - bias_val) * config.amplitude_scaling
}

pub fn triangle(config: &SynthConfig, t: u32, freq: f32, _bias: Option<f32>) -> f32 {
    let adjusted_freq = freq + config.tuning_offset_hz;
    let phase = t as f32 * adjusted_freq / config.sample_rate as f32;
    2.0 * phase.abs().rem_euclid(2.0) - 1.0
}

pub fn render_test(config: &SynthConfig, ts: Vec<u32>, sr:u32, ugen: &Ugen) -> Vec<f32> {
    let mut samples: Vec<f32> = Vec::new();
    let freq: f32 = 400.0;
    let amp = 0.1;
    for t in ts {
        let sample = amp * ugen(config, t, freq, Some(0.5));
        samples.push(sample);
    }
    samples
}


#[cfg(test)]
mod tests {
    use super::*;

    #[macro_export]
    macro_rules! assert_approx_eq {
        ($a:expr, $b:expr, $epsilon:expr) => {
            assert!(($a as f32).abs() - ($b as f32).abs() < $epsilon, "assertion failed: `(left ≈ right)`\n  left: `{:?}`,\n right: `{:?}`", $a, $b);
        };
    }


    #[test]
    fn test_sine() {
        let config = SynthConfig::new(96000, 20.0, 20000.0, 1.0, 0.0, 0.0, 1.0);
        let epsilon = 1e-4;

        // Test at various points in the sine wave cycle
        assert_approx_eq!(0.0, sine(&config, 0, 1.0, None), epsilon);
        assert_approx_eq!(1.0, sine(&config, 24000, 1.0, None), epsilon);
        assert_approx_eq!(0.0, sine(&config, 48000, 1.0, None), epsilon);
        assert_approx_eq!(-1.0, sine(&config, 72000, 1.0, None), epsilon);
        assert_approx_eq!(0.0, sine(&config, 96000, 1.0, None), epsilon);
    }

    #[test]
    fn test_sawtooth() {
        let config = SynthConfig::new(96000, 20.0, 20000.0, 1.0, 0.0, 0.0, 1.0);
        let epsilon = 1e-4;
    
        assert_eq!(-1.0, sawtooth(&config, 0, 1.0, None));
        assert_eq!(0.0, sawtooth(&config, 48000, 1.0, None));
        assert_approx_eq!(1.0, sawtooth(&config, 95999, 1.0, None), epsilon);
        assert_eq!(-1.0, sawtooth(&config, 96000, 1.0, None));
    
        assert_eq!(-1.0, sawtooth(&config, 0, 2.0, None));
        assert_eq!(0.0, sawtooth(&config, 24000, 2.0, None));
    }
}