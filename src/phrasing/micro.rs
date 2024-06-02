/// Microtransient methods
/// Accepts parameters k, x, and d where
/// 
/// `x` represents the progression of time over [0, 1]
/// 
/// 
/// Examples in desmos as of Jun 1 2024
/// 
/// https://www.desmos.com/calculator/tbaam3xtsd


use crate::synth::{pi, pi2, e, epi};
use crate::types::timbre::MicroLifespan;
use crate::phrasing::AmpModulation;

pub static micros:[MicroLifespan; 3] = [
    MicroLifespan::Pop,
    MicroLifespan::Chiff,
    MicroLifespan::Click,
];

// Set an approximated limit for maximum monic value, for use in (k/K). Adding/subtracting from K requires a call to .max(1) to prevent negative indexing.
static K:f32 = 200f32; 
static one:f32 = 1f32;
static neg:f32 = -1f32;
static half:f32 = 0.5f32;
static threshold_x_cutoff:f32 = 0.015f32;

#[inline]
fn n_root(n:usize, x:f32) -> f32 {
    x.powf(1f32 / n as f32)
}

#[inline]
/// Expecting a value representing a duration in cycles in the range of [1/32, 32]
/// Returns a value in [0, 1]
fn conform_duration(d:f32) -> f32 {
    (32f32 * d - one) / 1023f32
}

static thousand_pi:f32 = 1000f32*pi;
static offset_x:f32 = 0.0125f32;
#[inline]
/// For unnatural decay, coerces positive values to 0.
fn conform_tail(x:f32) -> f32 {
    neg * half*(thousand_pi*(x - offset_x)).tanh() + half
}


/// Produce amplitude modulation for a short form micro 
/// May have local min/max, but always starts and ends near 0.
/// 
/// ## Arguments
/// `n_samples` The length of buffer to create and fill.
/// `n_cycles` represents the duration in cycles, currently designed for an equal distribution in [1/32, 32]
/// `k` represents the 0 based index of the list
pub fn mod_micro(n_samples:usize, n_cycles:f32, micro:&MicroLifespan, k:usize) -> AmpModulation {
    use MicroLifespan::*;
    let mut modulator:AmpModulation = vec![0f32; n_samples];
    let ns = n_samples as f32;
    let kf = k as f32;
    
    match micro {
        Chiff => {
            for (i, sample) in modulator.iter_mut().enumerate() {
                let x = (i+1) as f32 / ns;
                let d_scale = -200f32 + 100f32*conform_duration(n_cycles);
                let k_scale = n_root(3, kf/16f32);
                let exponent = epi * x;
                let y = (d_scale * k_scale * exponent).exp();
                *sample = y * conform_tail(x)
            }
        },
        Pop => {
            for (i, sample) in modulator.iter_mut().enumerate() {
                let x = (i+1) as f32 / ns;
                let d_scale = 10f32 * conform_duration(100f32*n_cycles);
                let k_scale = n_root(3, kf);
                let exponent = (x - 0.005) - (0.01*kf/K);
                let mut y = neg * (d_scale * k_scale * exponent).exp() + one;
                if y < 0f32 { 
                    y = 0f32
                };
                *sample = y
            }
        },
        Click => {
            for (i, sample) in modulator.iter_mut().enumerate() {
                let x = (i+1) as f32 / ns;
                let d_scale = 10f32 * conform_duration(n_root(3, n_cycles*n_cycles));
                let exponent = 0.001f32 + 0.01f32 * (K - kf).max(1f32).sqrt()/K.sqrt();
                let y = neg * x.powf(d_scale * exponent) + one;

                *sample = y * conform_tail(x)
            }
        }
    };
    modulator
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::{analysis, time, MicroLifespan};

    fn assert_micro_mod(micro:&MicroLifespan, mod_signal:&Vec<f32>) {
        let max_sample_num_greater_than_0:usize = 1 + (time::samples_of_dur(1f32, 1f32) as f32 * threshold_x_cutoff) as usize;
        let max_index = max_sample_num_greater_than_0 + max_sample_num_greater_than_0/2;
        
        for (i, y) in mod_signal.iter().enumerate() {
            assert!(false == y.is_nan(),  "Modulation micro {:#?} must only produce numeric values. Got NAN at index {}", micro, i);
            assert!(*y <= 1f32, "Modulation micro {:#?} must not produce any values above 1. Found {} at index {}", micro, y, i);
            assert!(*y >= 0f32, "Modulation micro {:#?} must not produce any values below 0. Found {} at {}", micro, y, i);

            if i > max_index {
                assert!(*y <= 0.00000001f32, "Must not produce any value after the microtransient {:#?} threshold of {}. Found invalid value {} at index {}", micro, max_sample_num_greater_than_0, y, i)
            }
        }
        
        let rms = analysis::volume::rms(&mod_signal);
        assert!(rms < 1f32, "Modulation micro {:#?} must produce an RMS value less than 1. Got {}", micro, rms);
        assert!(rms > 0f32, "Modulation micro {:#?} must produce an RMS value greater than 0. Got {}", micro, rms);
    }

    #[test]
    /// Show that each modulator has all values in [0, 1]
    /// and that the mean modulation value is in [0, 1]
    fn verify_valid_modulation_range() {
        let n_samples = 48000usize;
        let n_cycles = 30f32;
        for micro in &micros {
            let mod_signal = mod_micro(n_samples, n_cycles, &micro, 1usize);
            assert_micro_mod(&micro, &mod_signal);
        }
    }

    /// Show that the RMS value is consistent over arbitrary sample frequency
    #[test]
    fn verify_constant_over_sample_rate() {
        for index in 1..=10usize {
            let n_samples = index * 4800;
            let n_cycles = 1f32;
           
            for micro in &micros {
                let mod_signal = mod_micro(n_samples, n_cycles, &micro, 1usize);
                assert_micro_mod(&micro, &mod_signal);
            }
        }
    }
}