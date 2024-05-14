use rustfft::num_complex::Complex;

use crate::phrasing::AmpModulation;
use crate::types::timbre::AmpLifespan;
use crate::synth::{pi,pi2,SampleBuffer};

/// Given an index i in a sample buffer representing n_cycles,
/// Produce amplitude modulation for a short form lifespan 
/// May have local min/max, but always starts and ends near 0.
/// Parameter "k" represents harmonic distance. Use "1" when fixing this to constant value.
/// Parameter "d" is free. Use it to select presets. Accepts values in [0, 1] with 0 as its constant value.
pub fn mod_lifespan(n_samples:usize, n_cycles:f32, lifespan:&AmpLifespan, k:usize, d:f32) -> AmpModulation {
    use AmpLifespan::*;
    let mut modulator:AmpModulation = vec![0f32; n_samples];
    let kf = k as f32;

    // Set an approximated limit for maximum monic value, for use in (dk/K)
    let K = 80f32; 
    let one = 1f32;
    match lifespan {
        Spring => {
            for (i, sample) in modulator.iter_mut().enumerate() {
                // @art-broke: current implementation uses a fixed constant k of 1. should scale with harmonics.
                let k:f32 = 1f32;
                let t:f32 = i as f32 / n_samples as f32;
                let y = 2f32 * ((t + k).powi(-1i32) - 0.5f32);

                /* @art-choice:  scale bounce rate c with duration as a multiple of 2*/
                let c:f32 = n_cycles.log2().min(2f32).max(6f32);
                *sample = (y*c*pi2).sin().abs()
            }
        },
        Pluck => {
            for (i, sample) in modulator.iter_mut().enumerate() {
                let t:f32 = i as f32 / n_samples as f32;
                let y:f32 = 0.5f32 - (3f32 * (t - 0.5f32)).tanh()/2f32;
                let a:f32 = (1f32/kf) * (kf - 1f32 - (6f32 * (K-kf) * pi * (t - (1f32/kf)) )).tanh();
                let b:f32 = 2f32.powf(-1f32 * t * (kf * t / n_cycles.log2().max(1f32)).sqrt()) * -1f32 * (n_cycles * pi2 * (t-1f32)).tanh();

                *sample = a * y * b
            }
        },
        Bloom => {
            let six = 6f32;
            let three = 3f32;
            for (i, sample) in modulator.iter_mut().enumerate() {
                let t:f32 = i as f32 / n_samples as f32;
                let y:f32 = (t / three) + (t.powi(3i32) /three) + (one/six) + (one/six)*((kf/16f32)*pi2*t+(pi2*d)).sin();
                let c = (one+n_cycles).powf(0.33333333);
                let a:f32 =  (c * pi2 * t.powf(1.5f32)).tanh();

                let base = Complex::new(t-one, 0f32);

                let b:f32 = -(c * pi2 * base.powf(0.6).re).tanh();

                *sample = a * y * b
            }
        },
        Pad => {
            let stable_amp = 0.9;
            let g = d.max(0.001) * kf.powf(1.5f32);

            for (i, sample) in modulator.iter_mut().enumerate() {
                let t:f32 = i as f32 / n_samples as f32;

                // @art-choice: Use the first five primes as basis waves
                let adds:Vec<f32> = vec![
                    t,
                    t.powf(one/3f32),
                    t.powf(one/7f32),
                    t.powf(one/11f32),
                    t.powf(one/13f32)
                ];

                let v:f32 = (one/adds.len() as f32)*adds.iter().map(|x| (pi2 * g * x).sin()).sum::<f32>();
                let y = stable_amp + (one-stable_amp) * v;
                let a= (n_cycles.powi(2i32) * pi2 * t).tanh();
                let b = -(n_cycles * pi2 * (t-one)).tanh();
                *sample = a * y * b
            }
        },
        Drone => {
            for (i, sample) in modulator.iter_mut().enumerate() {
                let t:f32 = i as f32 / n_samples as f32;

                let y:f32 =(4f32*(n_cycles+one)* t).tanh();
                let a:f32 = one;
                let b:f32 = -(pi2*(t-one)*(2f32+n_cycles).sqrt()).tanh();
                *sample = a * y * b
            }
        },
    };
    modulator
}

#[test]
fn do_test() {
    println!("(-0.5f32).powf(0.6) {}", (-2f32).powf(2.3f32))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::analysis;
    static min_allowed_mod:f32 = 0f32;
    static max_allowed_mod:f32 = 0f32;

    static lifespans:[AmpLifespan; 1] = [
        // AmpLifespan::Pad,
        // AmpLifespan::Spring,
        // AmpLifespan::Pluck,
        // AmpLifespan::Bloom,
        AmpLifespan::Drone,
    ];

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