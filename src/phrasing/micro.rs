/// Microtransient methods
/// Accepts parameters k, x, and d where
/// 
/// `x` represents the progression of time over [0, 1]
/// 
/// 
/// Examples in desmos as of Jun 1 2024
/// 
/// https://www.desmos.com/calculator/tbaam3xtsd

use crate::synth::{pi, pi2, e, epi, NFf, MFf, SR};
use crate::phrasing::AmpModulation;
use crate::types::synthesis::SampleBuffer;
use crate::types::timbre::{MicroLifespan, Mode, Arf, Energy, Presence, Visibility};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::cell::RefCell;
use std::thread_local;

const MAX_MICRO_HEIGHT: usize = 21;

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(rand::thread_rng());
}

fn gen_rng() -> f32 {
    RNG.with(|rng| rng.borrow_mut().gen::<f32>())
}

pub static micros: [MicroLifespan; 3] = [
    MicroLifespan::Pop,
    MicroLifespan::Chiff,
    MicroLifespan::Click,
];

static K: f32 = 200f32;
static one: f32 = 1f32;
static neg: f32 = -1f32;
static half: f32 = 0.5f32;
static threshold_x_cutoff: f32 = 0.015f32;

#[inline]
fn n_root(n: usize, x: f32) -> f32 {
    x.powf(1f32 / n as f32)
}

#[inline]
fn conform_duration(d: f32) -> f32 {
    (32f32 * d - one) / 1023f32
}

static thousand_pi: f32 = 1000f32 * pi;
static offset_x: f32 = 0.0125f32;

#[inline]
fn conform_tail(x: f32) -> f32 {
    neg * half * (thousand_pi * (x - offset_x)).tanh() + half
}

/// Amplitude modulation for chiff
pub fn amp_chiff(k: usize, x: f32, d: f32) -> f32 {
    let d_scale = -200f32 + 100f32 * conform_duration(d);
    let k_scale = n_root(3, k as f32 / 16f32);
    let exponent = epi * x;
    let y = (d_scale * k_scale * exponent).exp();
    y * conform_tail(x)
}

/// Amplitude modulation for pop
pub fn amp_pop(k: usize, x: f32, d: f32) -> f32 {
    let d_scale = 10f32 * conform_duration(100f32 * d);
    let k_scale = n_root(3, k as f32);
    let exponent = (x - 0.005) - (0.01 * k as f32 / K);
    let y = neg * (d_scale * k_scale * exponent).exp() + one;
    y.max(0f32)
}

/// Amplitude modulation for click
pub fn amp_click(k: usize, x: f32, d: f32) -> f32 {
    let d_scale = 10f32 * conform_duration(n_root(3, d * d));
    let exponent = 0.001f32 + 0.01f32 * (K - k as f32).max(1f32).sqrt() / K.sqrt();
    let y = neg * x.powf(d_scale * exponent) + one;
    y * conform_tail(x)
}

/// Produce amplitude modulation for a short form micro
/// May have local min/max, but always starts and ends near 0.
/// 
/// ## Arguments
/// `n_samples` The length of buffer to create and fill.
/// `n_cycles` represents the duration in cycles, currently designed for an equal distribution in [1/32, 32]
/// `k` represents the 0 based index of the list
pub fn mod_micro(n_samples: usize, n_cycles: f32, micro: &MicroLifespan, k: usize) -> AmpModulation {
    let mut modulator: AmpModulation = vec![0f32; n_samples];
    let ns = n_samples as f32;

    for (i, sample) in modulator.iter_mut().enumerate() {
        let x = (i + 1) as f32 / ns;
        *sample = match micro {
            MicroLifespan::Chiff => amp_chiff(k, x, n_cycles),
            MicroLifespan::Pop => amp_pop(k, x, n_cycles),
            MicroLifespan::Click => amp_click(k, x, n_cycles),
        };
    }

    modulator
}

fn micro_height(energy:&Energy) -> usize {
    match energy {
        Energy::Low => 8,
        Energy::Medium => 31,
        Energy::High => 131,
    }
}

pub fn muls_micro(freq: f32, energy:&Energy) -> Vec<f32> {
    let n = ((NFf / freq) as usize).min(micro_height(energy));
    (1..=n).map(|x| x as f32).collect()
}

pub fn amps_micro(freq: f32, energy:&Energy) -> Vec<f32> {
    let n = ((NFf / freq) as usize).min(micro_height(energy));
    vec![1f32; n]
}

pub fn phases_micro(freq: f32, energy:&Energy) -> Vec<f32> {
    let n = ((NFf / freq) as usize).min(micro_height(energy));
    vec![0f32; n]
}

pub fn set_micro(freq: f32, energy:&Energy) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let muls = muls_micro(freq, energy);
    let amps = muls_micro(freq, energy);
    let phases = muls_micro(freq, energy);
    (
        amps,
        muls,
        phases
    )
}

fn mod_phase_micro_noisy(k: usize, x: f32, d: f32) -> f32 {
    let max_distortion = pi / 2f32;
    (1f32 - x) * gen_rng() * max_distortion
}

fn mod_phase_micro(k: usize, x: f32, d: f32) -> f32 {
    let max_distortion = pi / 64f32;
    (1f32 - x) * gen_rng() * max_distortion
}

pub fn modders_chiff() -> crate::phrasing::ranger::Modders {
    [
        None,
        Some(vec![(1f32, amp_chiff)]),
        Some(vec![(1f32, mod_phase_micro_noisy)])
    ]
}

pub fn modders_click() -> crate::phrasing::ranger::Modders {
    [
        None,
        Some(vec![(1f32, amp_click)]),
        Some(vec![(1f32, mod_phase_micro)])
    ]
}

pub fn modders_pop() -> crate::phrasing::ranger::Modders {
    let mut s = rand::thread_rng();

    [
        None,
        Some(vec![(1f32, amp_pop)]),
        None,
    ]
}



#[cfg(test)]
mod test {
    use super::*;

    use crate::{analysis, time, MicroLifespan};

    fn assert_microtransient(micro:&MicroLifespan, mod_signal:&Vec<f32>) {
        let max_sample_num_greater_than_0:usize = 1 + (time::samples_of_dur(1f32, 1f32) as f32 * threshold_x_cutoff) as usize;
        let max_index = max_sample_num_greater_than_0 + max_sample_num_greater_than_0/2;
        
        for (i, y) in mod_signal.iter().enumerate() {
            assert!(false == y.is_nan(),  "Modulation micro {:#?} must only produce numeric values. Got NAN at index {}", micro, i);
            assert!(*y <= 1f32, "Modulation micro {:#?} must not produce any values above 1. Found {} at index {}", micro, y, i);
            assert!(*y >= -1f32, "Modulation micro {:#?} must not produce any values below -1. Found {} at index {}", micro, y, i);
            
            if i > max_index {
                // assert!(*y <= 0.00000001f32, "Must not produce any value after the microtransient {:#?} threshold of {}. Found invalid value {} at index {}", micro, max_sample_num_greater_than_0, y, i)
            }
        }

        let early_rms = analysis::volume::rms(&mod_signal[0..100]);
        assert!(early_rms > 0.00001f32, "Early RMS must be easily audible for {:#?}. Got actual value of {}", micro, early_rms);
        let rms = analysis::volume::rms(&mod_signal);
        assert!(rms < 1f32, "Modulation micro {:#?} must produce an RMS value less than 1. Got {}", micro, rms);
        assert!(rms > 0f32, "Modulation micro {:#?} must produce an RMS value greater than 0. Got {}", micro, rms);
    }
    

    #[cfg(test)]
    mod test_spec {
        use super::*;
        static K:usize = 200;

        #[test]
        /// Show that each modulator has all values in [0, 1]
        /// and that the mean modulation value is in [0, 1]
        fn verify_valid_modulation_range() {
            let n_samples = 48000usize;
            for micro in &micros {
                for k in 1..=K {
                    for n_cycles in [1f32/32f32] {
                        let mod_signal = mod_micro(n_samples, n_cycles, &micro, k);
                        assert_microtransient(&micro, &mod_signal);
                    }
                }
            }
        }
    }
    

    #[cfg(test)]
    mod test_integration {
        use super::*;
        use crate::{analysis, time, MicroLifespan};
        static test_dir:&str = "dev-audio/druid";
        use crate::files;
        use crate::render::engrave;
        use crate::types::timbre::Mode;
        use crate::phrasing::contour::expr_none;
        use crate::druid::{ApplyAt, Element, Elementor, modders_none, test_vep, test_frex, inflect};

        static cps:f32 = 1.0f32;

        fn nearly_none_chiff(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
            let (amps, muls, phss) = set_micro(fund, energy);
            Element {
                mode: Mode::Noise,
                amps,
                muls,
                phss,
                modders: modders_chiff(),
                expr: expr_none(),
                hplp: (vec![MFf], vec![NFf]),
                thresh: (0f32, 1f32)
            }
        }

        fn nearly_none_click(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
            let (amps, muls, phss) = set_micro(fund, energy);
            Element {
                mode: Mode::Noise,
                amps,
                muls,
                phss,
                modders: modders_click(),
                expr: expr_none(),
                hplp: (vec![MFf], vec![NFf]),
                thresh: (0f32, 1f32)
            }
        }

        fn nearly_none_pop(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
            let (amps, muls, phss) = set_micro(fund, energy);
            Element {
                mode: Mode::Noise,
                amps,
                muls,
                phss,
                modders: modders_pop(),
                expr: expr_none(),
                hplp: (vec![MFf], vec![NFf]),
                thresh: (0f32, 1f32)
            }
        }

        #[test]
        fn test_blend_single_element_chiff() {
            let test_name:&str = "micro-chiff";
            let (freqs, durs, frexs) = test_frex();
            let mut signal:SampleBuffer = Vec::new();

            let (vis, en, pre) = test_vep();
            let elementor:Elementor = vec![
                (1f32, nearly_none_chiff)
            ];

            for (index, frex) in frexs.iter().enumerate() {
                let dur = durs[index];
                let at = ApplyAt { frex: *frex, span: (cps, dur) };
                signal.append(&mut inflect(&frex, &at, &elementor, &vis, &en, &pre));
            }
            assert_microtransient(&MicroLifespan::Chiff, &signal);
            files::with_dir(test_dir);
            let filename:String = format!("{}/{}.wav", test_dir, test_name);
            engrave::samples(SR, &signal, &filename);
        }

        #[test]
        fn test_blend_single_element_click() {
            let test_name:&str = "micro-click";
            let (freqs, durs, frexs) = test_frex();
            let mut signal:SampleBuffer = Vec::new();

            let (vis, en, pre) = test_vep();
            let elementor:Elementor = vec![
                (1f32, nearly_none_click)
            ];

            for (index, frex) in frexs.iter().enumerate() {
                let dur = durs[index];
                let at = ApplyAt { frex: *frex, span: (cps, dur) };
                signal.append(&mut inflect(&frex, &at, &elementor, &vis, &en, &pre));
            }
            assert_microtransient(&MicroLifespan::Click, &signal);
            files::with_dir(test_dir);
            let filename:String = format!("{}/{}.wav", test_dir, test_name);
            engrave::samples(SR, &signal, &filename);
        }

        #[test]
        fn test_blend_single_element_pop() {
            let test_name:&str = "micro-pop";
            let (freqs, durs, frexs) = test_frex();
            let mut signal:SampleBuffer = Vec::new();

            let (vis, en, pre) = test_vep();
            let elementor:Elementor = vec![
                (1f32, nearly_none_pop)
            ];

            for (index, frex) in frexs.iter().enumerate() {
                let dur = durs[index];
                let at = ApplyAt { frex: *frex, span: (cps, dur) };
                signal.append(&mut inflect(&frex, &at, &elementor, &vis, &en, &pre));
            }

            files::with_dir(test_dir);
            assert_microtransient(&MicroLifespan::Pop, &signal);
            let filename:String = format!("{}/{}.wav", test_dir, test_name);
            engrave::samples(SR, &signal, &filename);
        }
    }
}
