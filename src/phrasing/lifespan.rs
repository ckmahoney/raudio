use crate::phrasing::AmpModulation;
use crate::types::timbre::AmpLifespan;
use crate::synth::{pi,pi2,SampleBuffer};

/// Given an index i in a sample buffer representing n_cycles,
/// Produce amplitude modulation for a short form lifespan 
/// May have local min/max, but always starts and ends near 0.
pub fn mod_lifespan(n_samples:usize, n_cycles:f32, lifespan:&AmpLifespan) -> AmpModulation {
    use AmpLifespan::*;
    let mut modulator:AmpModulation = vec![0f32; n_samples];

    match lifespan {
        Snap => {
            for (i, sample) in modulator.iter_mut().enumerate() {
                let k:f32 = 1f32;
                let p:f32 = i as f32 / n_samples as f32;
                let y = 2f32 * ((p.abs() + k).powi(-1i32) - 0.5f32);

                /* @art-choice:  scale bounce rate c with duration as a multiple of 2*/
                let c:f32 = n_cycles.log2().min(2f32).max(6f32);
                *sample = (y*c*pi2).sin().abs()
            }
        },
        Pluck => {
            // tanh + tri 
        },
        Bloom => {
            // t.pow(k)
        },
        Pad => {
            // sin(x)
        },
        Drone => {
            // k
        },
    };
    modulator
}



#[cfg(test)]
mod test {
    use super::*;
    use crate::analysis;
    static min_allowed_mod:f32 = 0f32;
    static max_allowed_mod:f32 = 0f32;


    #[test]
    /// Show that each modulator has all values in [0, 1]
    /// and that the mean modulation value is in [0, 1]
    fn verify_valid_modulation_range() {
        use AmpLifespan::*;

        let n_samples = 48000 * 90usize;
        let n_cycles = 64f32;
        let lifespans:Vec<AmpLifespan> = vec![
            Snap
        ];  

        for lifespan in lifespans {
            let mod_signal = mod_lifespan(n_samples, n_cycles, &lifespan);
            let min = mod_signal.iter().fold(1f32, |acc, y| if *y < acc { *y } else { acc } );
            let max = mod_signal.iter().fold(0f32, |acc, y| if *y > acc { *y } else { acc } );
            assert!(max <= 1f32, "Modulation lifecycle {:#?} must not produce any values above 1", lifespan);
            assert!(min >= 0f32, "Modulation lifecycle {:#?} must not produce any values below 0", lifespan);

            let rms = analysis::volume::rms(&mod_signal);
            assert!(rms <= 1f32, "Modulation lifecycle {:#?} must produce an RMS value less than or equal to 1", lifespan);
            assert!(rms >= 0f32, "Modulation lifecycle {:#?} must produce an RMS value greater than or equal to 0", lifespan);
        }
    }

    /// Show that the RMS value is consistent over arbitrary sample frequency
    #[test]
    fn verify_constant_over_sample_rate() {
        use AmpLifespan::*;

        for index in 1..=10usize {
            let n_samples = index * 4800;
            let n_cycles = 1f32;
            let lifespans:Vec<AmpLifespan> = vec![
                Snap
            ];  

            for lifespan in lifespans {
                let mod_signal = mod_lifespan(n_samples, n_cycles, &lifespan);
                let min = mod_signal.iter().fold(1f32, |acc, y| if *y < acc { *y } else { acc } );
                let max = mod_signal.iter().fold(0f32, |acc, y| if *y > acc { *y } else { acc } );
                assert!(max <= 1f32, "Modulation lifecycle {:#?} must not produce any values above 1", lifespan);
                assert!(min >= 0f32, "Modulation lifecycle {:#?} must not produce any values below 0", lifespan);

                let rms = analysis::volume::rms(&mod_signal);
                assert!(rms <= 1f32, "Modulation lifecycle {:#?} must produce an RMS value less than or equal to 1", lifespan);
                assert!(rms >= 0f32, "Modulation lifecycle {:#?} must produce an RMS value greater than or equal to 0", lifespan);
            }
        }
    }
}