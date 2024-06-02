use rustfft::num_complex::Complex;
use crate::phrasing::AmpModulation;
use crate::types::timbre::AmpLifespan;
use crate::synth::{pi, pi2, epi, SampleBuffer};
pub static lifespans: [AmpLifespan; 5] = [
    AmpLifespan::Pad,
    AmpLifespan::Spring,
    AmpLifespan::Pluck,
    AmpLifespan::Bloom,
    AmpLifespan::Drone,
];

static one: f32 = 1f32;
static neg: f32 = -1f32;
static six: f32 = 6f32;
static three: f32 = 3f32;
static K: f32 = 200f32;

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
    let y: f32 = (t / three) + (t.powi(3i32) / three) + (one / six) + (one / six) * ((kf / 16f32) * pi2 * t + (pi2 * d)).sin();
    let c = (one + d).powf(0.33333333);
    let a: f32 = (c * pi2 * t.powf(1.5f32)).tanh();
    let base = Complex::new(t - one, 0f32);
    let b: f32 = -(c * pi2 * base.powf(0.6).re).tanh();
    a * y * b
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

pub fn mod_lifespan(n_samples: usize, n_cycles: f32, lifespan: &AmpLifespan, k: usize, d: f32) -> AmpModulation {
    let mut modulator: AmpModulation = vec![0f32; n_samples];
    let ns = n_samples as f32;

    for (i, sample) in modulator.iter_mut().enumerate() {
        let x = (i + 1) as f32 / ns;
        *sample = match lifespan {
            AmpLifespan::Burst => mod_burst(k, x, n_cycles),
            AmpLifespan::Snap => mod_snap(k, x, n_cycles),
            AmpLifespan::Spring => mod_spring(k, x, n_cycles),
            AmpLifespan::Pluck => mod_pluck(k, x, n_cycles),
            AmpLifespan::Bloom => mod_bloom(k, x, n_cycles),
            AmpLifespan::Pad => mod_pad(k, x, n_cycles),
            AmpLifespan::Drone => mod_drone(k, x, n_cycles),
        };
    }

    modulator
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::analysis;

    fn assert_lifespan_mod(lifespan:&AmpLifespan,mod_signal:&Vec<f32>) {
        for (i, y) in mod_signal.iter().enumerate() {
            assert!(false == y.is_nan(),  "Modulation lifecycle {:#?} must only produce numeric values. Got NAN at index {}", lifespan, i);
            assert!(*y <= 1f32, "Modulation lifecycle {:#?} must not produce any values above 1. Found {} at {}", lifespan, y, i);
            assert!(*y >= 0f32, "Modulation lifecycle {:#?} must not produce any values below 0. Found {} at {}", lifespan, y, i);
        }
        
        let rms = analysis::volume::rms(&mod_signal);
        assert!(rms < 1f32, "Modulation lifecycle {:#?} must produce an RMS value less than 1. Got {}", lifespan, rms);
        assert!(rms > 0f32, "Modulation lifecycle {:#?} must produce an RMS value greater than 0. Got {}", lifespan, rms);
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