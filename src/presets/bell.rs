use crate::synth::{pi, pi2, NF, SampleBuffer};
use crate::types::synthesis::{Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::types::render::*;
use crate::types::timbre::{BandpassFilter, Energy, Presence, BaseOsc, Sound, FilterMode, Timeframe, Phrasing, Ampex};
use crate::monic_theory::tone_to_freq;
use crate::time;

use rand;
use rand::Rng;

/// activation function for bandpass filter. True indicates frequency is OK; false says to filter it out.
fn bandpass_filter(filter:&BandpassFilter, phr:&Phrasing, freq:f32, i:usize, n:usize) -> bool {
    let min_frequency = filter.2.0;
    let max_frequency = filter.2.1;
    match filter.0 {
        FilterMode::Linear => {
            match filter.1 {
                FilterPoint::Constant => {
                    return freq > min_frequency && freq < max_frequency;
                },
                FilterPoint::Mid => {
                    true
                },
                FilterPoint::Tail => {
                    true
                }
            }
        },
        FilterMode::Logarithmic => {
            panic!("No implementation for a logarithmic mixer yet")
        }
    }
}


type BellPartial = (f32, f32);


fn gen_float(min:f32, max:f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}

/// A soft but present sub x2 octave
fn gen_sub(fund:f32) -> BellPartial {
    let weight = gen_float(0.0001, 0.001);
    let fmod = fund/4f32;
    (weight, fmod)
}

/// A soft but present sub octave
fn gen_bass(fund:f32) -> BellPartial {
    let weight = gen_float(0.001, 0.01);
    let fmod = fund/2f32;
    (weight, fmod)
}

/// wide variety of amplitude presence
fn gen_fundamental(fund:f32) -> BellPartial {
    let weight = gen_float(0.000001, 0.001);
    (weight, fund)
}

fn gen_strike(fund:f32) -> BellPartial {
    let weight = gen_float(0.005, 0.02);
    let fmod = gen_float(1.98, 2.10);
    (weight, fmod)
}

fn gen_tierce(fund:f32) -> BellPartial {
    let weight = gen_float(0.001, 0.01);
    let fmod = gen_float(2.5, 2.8);
    (weight, fmod)
}

fn gen_quint(fund:f32) -> BellPartial {
    let weight = gen_float(0.001, 0.005);
    let fmod = gen_float(3.95, 4.56);
    (weight, fmod)
}

fn gen_nominal(fund:f32) -> BellPartial {
    let weight = gen_float(0.0001, 0.001);
    let fmod = gen_float(5f32, 12f32);
    (weight, fmod)
}

fn gen_coefficients(fund:f32,) -> Vec<BellPartial> {
    vec![
        gen_sub(fund),
        gen_bass(fund),
        gen_strike(fund),
        gen_tierce(fund),
        gen_quint(fund),
        gen_nominal(fund),
        gen_nominal(fund),
    ]
}

pub struct Mgen {
    osc: BaseOsc,
    sound: Sound
}

impl Mgen {

    /// Given a note and an opinionated synthesizer,
    /// And vec of (weight, coefficient) pairs,
    /// Render the note at time (cps, &phr)
    /// 
    /// Distinct from a melodic mgen, which applies the fundamental as its most present harmonic,
    /// A bellgen has a centroid higher than the fundamental in the frequency domain.
    /// It also takes a list of partials, whose union is the fundamental for the synthesizer. 
    pub fn inflect_bell(&self, coeffs:&Vec<(f32, f32)>, note:&Note, phr:&mut Phrasing) -> SampleBuffer {
        let frequency = tone_to_freq(&note.1);
        let ampl = &note.2;
        let n_samples = time::samples_per_cycle(phr.cps) as usize;
        let max_partial = coeffs.iter().fold(0f32, |max, (_, f)| if *f > max { *f } else { max } ).ceil();
        /* Each enharmonic partial has harmonics. This is the maximum allowed partial multiplier */
        let max_coeff_k = NF / (max_partial * frequency) as usize;
        let mut sig = vec![0f32; n_samples];

        let max_monic:usize = match self.sound.energy {
            Energy::Low => {
                13
            },
            Energy::Medium => {
                51
            },
            Energy::High => {   
                NF
            }
        }.min(max_coeff_k);

        // modulators for the three distinct components
        fn amp_fundamental(p:f32) -> f32 {
            // preset that looked good in desmos
            (-1f32 * (p - 1f32).powi(6)) + 1f32
        }

        fn amp_strike(p:f32) -> f32 {
            // preset that looked good in desmos
            (p - 1f32).powi(8)

        }

        fn amp_partial(p:f32)-> f32 {
            1f32 - p
        }

        println!("Using {} for energy {:#?}", max_monic, self.sound.energy);
        for (index, (weight, fmod)) in coeffs.iter().enumerate() {
            for k in 1..=max_monic {

                for j in 0..n_samples {
                    phr.note.p = j as f32 / n_samples as f32;
                    
                    let f = frequency * fmod;
                    if bandpass_filter(&self.sound.bandpass, phr, f, j, f as usize) {

                        let t = j as f32 / NF as f32;
                        phr.note.p = j as f32 / n_samples as f32;

                        let amp_k = if index == 0 || index == 1 {
                            amp_fundamental(phr.note.p) / (2 - index) as f32
                        } else if index == 2 {
                            amp_strike(phr.note.p)
                        } else {
                            amp_partial(phr.note.p) / (k*k) as f32
                        };
    

                        let freq = pi2 * f; 
                        let amp = ampl * amp_k * weight;
                        let phase = f * pi2 * t;
                        sig[j] += amp * phase.sin();
                    } else {
                        continue
                    }
                }
            };
        }

        sig
    }   
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::files;
    use crate::render;

    #[test]
    fn test_inflect_bell() {
        let n_cycles = 4f32;
        let cps = 1f32;
        let coeffs_cool_kick:Vec<(f32,f32)> = vec![
            (1f32, 0.33),
            (2.5, 0.5),
            (3.5, 0.3),
            (4.2, 0.2),
            (5.1, 0.1)
        ];

        let coeffs1:Vec<(f32,f32)> = vec![
            (0.0125, 0.25),
            (0.033, 0.5),
            (0.05, 2.13),
            (0.03, 2.7),
            (0.02, 4.2),
            (0.01, 5.1)
        ];

        let coeffs:Vec<(f32,f32)> = vec![
            (0.00055, 0.25),
            (0.0013, 0.5),
            (0.005, 1.0),
            (0.02, 2.11),
            (0.023, 2.7),
            (0.012, 4.2),
            (0.0071, 5.1)
        ];

        let mut phr = Phrasing {
            cps, 
            form: Timeframe {
                cycles: n_cycles,
                p: 0f32,
                instance: 0
            },
            arc: Timeframe {
                cycles: n_cycles,
                p: 0f32,
                instance: 0
            },
            line: Timeframe {
                cycles: n_cycles,
                p: 0f32,
                instance: 0
            },
            note: Timeframe {
                cycles:n_cycles,
                p: 0f32,
                instance: 0
            }
        };

        let sound = Sound {
            bandpass: (FilterMode::Linear, FilterPoint::Tail, (1f32, 24000f32)),
            energy: Energy::High,
            presence : Presence::Legato,
            pan: 0f32,
        };

        let note:Note = ( (n_cycles as i32, 1), (7, (0i8, 0i8, 1i8)), 1f32);
        let mgen = Mgen {
            osc:BaseOsc::Bell,
            sound
        };
        let samples = mgen.inflect_bell(&coeffs, &note, &mut phr);

        println!("Created samples {:#?}", &samples[0..10]);

        files::with_dir("dev-audio/presets");

        render::samples_f32(44100, &samples, "dev-audio/presets/test_bell.wav");
    }

    #[test]
    fn test_some_generated_bells() {
        let n_cycles = 4f32;
        let n_bells = 16;
        let cps = 1f32;
        let mut buff:SampleBuffer = Vec::new();

        let mut phr = Phrasing {
            cps, 
            form: Timeframe {
                cycles: n_cycles,
                p: 0f32,
                instance: 0
            },
            arc: Timeframe {
                cycles: n_cycles,
                p: 0f32,
                instance: 0
            },
            line: Timeframe {
                cycles: n_cycles,
                p: 0f32,
                instance: 0
            },
            note: Timeframe {
                cycles:n_cycles,
                p: 0f32,
                instance: 0
            }
        };

        let sound = Sound {
            bandpass: (FilterMode::Linear, FilterPoint::Tail, (1f32, 24000f32)),
            energy: Energy::High,
            presence : Presence::Legato,
            pan: 0f32,
        };

        let note:Note = ( (n_cycles as i32, 1), (7, (0i8, 0i8, 1i8)), 1f32);
        let mgen = Mgen {
            osc:BaseOsc::Bell,
            sound
        };

        for i in 0..n_bells {
            let coeffs = gen_coefficients(1.0);
            buff.append(&mut mgen.inflect_bell(&coeffs, &note, &mut phr))
        }


        files::with_dir("dev-audio/presets");

        render::samples_f32(44100, &buff, "dev-audio/presets/test_gen_bells.wav");
    }
}