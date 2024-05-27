/// Methods for producing absoultely positioned noise
/// Remarkably different than relative noise; Absolute noise uses general conventionally defined algorithms for producing fields of values for space greater than 0.
/// Overall a fourier series representation for noise is overkill and very expensive.
/// Though it is good to see implementations here analog to the mgen for melodic and enharmonic synths.
/// 
/// Some methods applied below to improve performance:
/// 
/// ## Shortening
/// Much better success is seen reducing the harmonic reach of the noise signal.
/// This also adds a lot more character to the result. Testing with a max of +1000 noise 
/// produces a pretty cool distortion effect. Will be great for percussion!
/// 
/// ## Degrading
/// Maybe similiar in concept to bitcrushing, degrading is a more frequency-fair version of shortening.
/// A relative portion of harmonics (say 20%) are kepth and sampled from the total available space.
/// For low frequencies this means a higher quantity of samples; and higher frequencies will have a less powerful signal overall.
/// The results of degrading by dropping frequencies is distinct on the power decreasing spectrum vs power increasing.
/// For red and pink noises, the result sounds like a distortion on the fundamental 
/// For blue and violet noises, the result sounds like typical noise.
/// 
/// Therefore, for higher power noise signals (blue and violet), we can use this degredation technique to increase runtime performance.
/// This can be used for instrumentsthat cover a wide spectral range in a short time like hats and cymbals.

extern crate rand;
use rand::Rng;
use rand::seq::SliceRandom;
use std::f32::consts::PI;
use rand::rngs::ThreadRng;

use crate::synth::{pi, pi2, SR, MF, NF, SampleBuffer};
use crate::types::synthesis::{Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::types::render::*;
use crate::types::timbre::{AmpContour,AmpLifespan,BandpassFilter, Energy, Presence, BaseOsc, Sound, FilterMode, Timeframe, Phrasing};
use crate::monic_theory::tone_to_freq;
use crate::time;
use crate::render;
use crate::render::realize;
use crate::phrasing::contour;
use crate::phrasing::{lifespan, AmpModulation};

use crate::phrasing::bandpass_filter;


pub struct Mgen {
    pub osc: BaseOsc,
    pub     sound: Sound
}


#[derive(Debug)]
pub enum NoiseColor {
    Violet,
    Blue,
    Equal,
    Pink,
    Red,
}

impl NoiseColor {
    pub fn variants() -> Vec<NoiseColor> {
        vec![
            NoiseColor::Equal,
            NoiseColor::Pink,
            NoiseColor::Blue,
            NoiseColor::Red,
            NoiseColor::Violet,
        ]
    }

    #[inline]
    pub fn get_amp_mod(color: &NoiseColor, f:usize) -> f32 {
        match color {
            NoiseColor::Violet => (f as f32).powi(2),
            NoiseColor::Blue => (f as f32).sqrt(),
            NoiseColor::Equal => 1.0,
            NoiseColor::Pink => 1.0 / (f as f32).sqrt(),
            NoiseColor::Red => 1.0 / (f as f32).powi(2),
        }
    }
}

impl Mgen {
    pub fn new(osc:BaseOsc, sound:Sound) -> Self {
        Mgen { osc, sound }
    }

    pub fn buffs_by_duration(n_samples:usize, color:&NoiseColor) -> Vec<SampleBuffer> {
        let frequency = 1f32;
        let n_freqs = NF - frequency as usize;

        // update n_monics to reflect requested sound
        let mut phases = vec![0.0; n_freqs];
        let mut amplitudes = vec![0.0; n_freqs];
        let mut sig = vec![0f32; n_samples];
        let mut rng = rand::thread_rng();

        let mut buffs:Vec<SampleBuffer> = Vec::with_capacity(n_freqs);

        // Pre-calculate phases and amplitudes
        for k in 1..=n_freqs {
            phases.push(rng.gen::<f32>() * 2.0 * PI);
            amplitudes.push(NoiseColor::get_amp_mod(color, k));
            buffs.push(Vec::with_capacity(n_samples))
        }

        for k in 0..n_freqs {
            let dest = &mut buffs[k];
            let f = frequency + k as f32;
            for j in 0..n_samples {
                let t = j as f32 / SR as f32;
                let amp = amplitudes[k];
                dest.push(amp * (f * pi2 * t + phases[k]).sin());
            }
        };

        buffs
    }   

    pub fn inflect_noise_shortened(&self, rng:&mut ThreadRng, color:&NoiseColor, note:&Note, phr:&mut Phrasing) -> SampleBuffer {
        let dur = time::duration_to_cycles(note.0);
        let frequency = tone_to_freq(&note.1);
        let ampl = &note.2;
        let n_samples = time::samples_of_duration(phr.cps, &note.0);
        let max_added_freq = NF - frequency as usize;
        
        let s:f32 = if frequency.log2() > 10f32 {
            0f32
        } else if frequency.log2() > 7f32 {
            0.5f32
        } else {
            1f32
        };
        let n_freqs:usize = (frequency as usize * match self.sound.energy {
            Energy::Low => {
                2f32.powf(s) as usize
            },
            Energy::Medium => {
                2f32.powf(s+0.5) as usize
            },
            Energy::High => {   
                2f32.powf(s+1f32) as usize
            }
        }).min(200);



        let mut phases = vec![0.0; n_freqs];
        let mut amplitudes = vec![0.0; n_freqs];
        let mut sig = vec![0f32; n_samples];

        phr.note.cycles = dur;
        // Pre-calculate phases and amplitudes
        for k in 0..n_freqs {
            let freq = frequency as usize + k;
            phases[k] = rng.gen::<f32>() * 2.0 * PI;
            amplitudes[k] = NoiseColor::get_amp_mod(color, freq);
        }

        for j in 0..n_samples {
            phr.note.p = j as f32 / n_samples as f32;
            let line_p_extra = phr.note.p * phr.note.cycles / phr.line.cycles;
            for i in 0..n_freqs {
                let f = (frequency as usize + i) as f32;
                if amplitudes[i] > 0f32 && bandpass_filter(&self.sound.bandpass, f, phr.line.p + line_p_extra) {
                    let t = j as f32 / SR as f32;
                    let amp = amplitudes[i];

                    sig[j] += amp * (f * pi2 * t + phases[i]).sin();
                } else {
                    continue
                }
            }
        };

        sig
    }


    /// Higher block_size values improve render time by skipping samples
    /// But introduces high frequency artifacts. a constant very annoying high pitched tone.
    pub fn inflect_noise_shortened_blocked(&self, rng:&mut ThreadRng, color:&NoiseColor, note:&Note, phr:&mut Phrasing) -> SampleBuffer {
        let dur = time::duration_to_cycles(note.0);
        let frequency = tone_to_freq(&note.1);
        let ampl = &note.2;
        let n_samples = time::samples_of_duration(phr.cps, &note.0);
        let max_added_freq = NF - frequency as usize;
        
        let s:f32 = if frequency.log2() > 10f32 {
            0f32
        } else if frequency.log2() > 7f32 {
            0.5f32
        } else {
            1f32
        };
        let n_freqs:usize = (frequency as usize * match self.sound.energy {
            Energy::Low => {
                2f32.powf(s) as usize
            },
            Energy::Medium => {
                2f32.powf(s+0.5) as usize
            },
            Energy::High => {   
                2f32.powf(s+1f32) as usize
            }
        }).min(200);

        let block_size:usize = 2;

        let mut phases = vec![0.0; n_freqs];
        let mut amplitudes = vec![0.0; n_freqs];
        let mut sig = vec![0f32; n_samples];

        phr.note.cycles = dur;
        // Pre-calculate phases and amplitudes
        for k in 0..n_freqs {
            let freq = frequency as usize + k;
            phases[k] = rng.gen::<f32>() * 2.0 * PI;
            amplitudes[k] = NoiseColor::get_amp_mod(color, freq);
        }

        for j in (0..n_samples).step_by(block_size) {
            phr.note.p = j as f32 / n_samples as f32;
            let line_p_extra = phr.note.p * phr.note.cycles / phr.line.cycles;
            for i in 0..n_freqs {
                let f = (frequency as usize + i) as f32;
                if amplitudes[i] > 0f32 && bandpass_filter(&self.sound.bandpass, f, phr.line.p + line_p_extra) {
                    let t = j as f32 / SR as f32;
                    let amp = amplitudes[i];

                    sig[j] += amp * (f * pi2 * t + phases[i]).sin();
                } else {
                    continue
                }
            }
        };

        sig
    }


    pub fn inflect_noise_shortened_ifft(&self, rng:&mut ThreadRng, color:&NoiseColor, note:&Note, phr:&mut Phrasing) -> SampleBuffer {
        let dur = time::duration_to_cycles(note.0);
        let frequency = tone_to_freq(&note.1);
        let ampl = &note.2;
        let n_samples = time::samples_of_duration(phr.cps, &note.0);
        let max_added_freq = NF - frequency as usize;
        
        let s:f32 = if frequency.log2() > 10f32 {
            0f32
        } else if frequency.log2() > 7f32 {
            0.5f32
        } else {
            1f32
        };

        let n_freqs:usize = (frequency as usize * match self.sound.energy {
            Energy::Low => {
                2f32.powf(s) as usize
            },
            Energy::Medium => {
                2f32.powf(s+0.5) as usize
            },
            Energy::High => {   
                2f32.powf(s+1f32) as usize
            }
        }).min(200);

        let mut freqs = vec![0.0f32; n_freqs];
        let mut phases = vec![0.0f32; n_freqs];
        let mut amplitudes = vec![0.0f32; n_freqs];
        let mut sig = vec![0f32; n_samples];

        phr.note.cycles = dur;
        // Pre-calculate phases and amplitudes
        for k in 0..n_freqs {
            let freq = frequency as usize + k;
            freqs[k] = freq as f32;
            phases[k] = rng.gen::<f32>() * pi2;
            amplitudes[k] = NoiseColor::get_amp_mod(color, freq);
        }

        let step_size = n_samples;

        for j in (0..n_samples).step_by(step_size) {
            let mut fs = freqs.clone();
            let mut ps = phases.clone();
            let mut aas = amplitudes.clone();
            phr.note.p = j as f32 / n_samples as f32;
            let line_p_extra = phr.note.p * phr.note.cycles / phr.line.cycles;
            for i in 0..fs.len() {
                
                if bandpass_filter(&self.sound.bandpass, fs[i], phr.line.p + line_p_extra) {
                    let t = j as f32 / SR as f32;
                    let amp = aas[i];
                } else {
                    aas[i] = 0f32
                }
            }
            let mut sinus:Vec<(f32,f32,f32)> = (&fs).iter().enumerate().map(|(index, f)| { (*f, ps[index],aas[index]) }).collect();
            let results = render::ifft::ifft(&mut sinus, SR/1024, step_size);
            for idx in 0..step_size {
                if j + idx < sig.len() {
                    sig[j + idx] = results[idx];
                }
            }
        };

        sig
    }


    /// Keep a portion of the noise signal
    /// Has a lot of visible harmonic components (does not sound like "random" noise)
    /// However the result does not sound related to the input note, which supports a goal of noise
    pub fn inflect_noise_degraded(&self, color:&NoiseColor, note:&Note, phr:&mut Phrasing) -> SampleBuffer {
        let dur = time::duration_to_cycles(note.0);
        let frequency = tone_to_freq(&note.1);
        let ampl = &note.2;
        let n_samples = (dur  * time::samples_per_cycle(phr.cps) as f32) as usize;
        let max_monic = NF / frequency as usize;

        let offset = frequency.fract();
        
        let r = frequency.log2() as i32;
        const MAX_REGISTER:i32 = 15i32;
        let n_keepers:usize = match self.sound.energy {
            // @art-choice Choose how many frequencies to drop 
            Energy::Low => {
                let x:i32= (MAX_REGISTER-4i32 - r).max(1);
                2f32.powi(x)
            },
            Energy::Medium => {
                let x:i32= (MAX_REGISTER-2i32 - r).max(1);
                2f32.powi(x)
            },
            Energy::High => {   
                let x:i32= (MAX_REGISTER -1i32 - r).max(1);
                2f32.powi(x)
            }
        } as usize;

        println!("Using {} noise partials for {:#?} energy ", n_keepers, self.sound.energy);

        let mut opts:Vec<usize> = (frequency as usize..NF).collect();
        let mut rng = rand::thread_rng();
        opts.shuffle(&mut rng);


        let mut phases = vec![0.0; n_keepers];
        let mut amplitudes = vec![0.0; n_keepers];
        let mut sig = vec![0f32; n_samples];
        let mut rng = rand::thread_rng();

        // Pre-calculate phases and amplitudes
        for (index, freq) in opts[0..n_keepers].iter().enumerate() {
            let f = *freq as f32+ offset;
            
            phases[index] = rng.gen::<f32>() * pi2;
            amplitudes[index] = NoiseColor::get_amp_mod(color, f as usize);
        }

        phr.note.cycles = dur;
        for (index, freq) in opts[0..n_keepers].iter().enumerate() {
            let f = *freq as f32+ offset;
            for j in 0..n_samples {
                phr.note.p = j as f32 / n_samples as f32;
                
                let p_extra = phr.note.p * phr.note.cycles / phr.line.cycles;
                
                if amplitudes[index] > 0f32 && bandpass_filter(&self.sound.bandpass, f, phr.line.p + p_extra) {
                    let t = j as f32 / SR as f32;
                    let amp = amplitudes[index];

                    sig[j] += amp * (f * pi2 * t + phases[index]).sin();
                } else {
                    continue
                }
            }
        };

        sig
    }


    /// Keep a small portion of the noise signal
    /// and give each remaining portion a unique modulator
    pub fn inflect_noise_degraded_modulated(&self, color:&NoiseColor, note:&Note, phr:&mut Phrasing) -> SampleBuffer {
        phr.note.cycles = time::duration_to_cycles(note.0);
        let frequency = tone_to_freq(&note.1);
        let ampl = &note.2;
        let n_samples = (phr.note.cycles * time::samples_per_cycle(phr.cps) as f32) as usize;
        let max_monic = NF / frequency as usize;

        let offset = frequency.fract();
        
        let r = frequency.log2() as i32;
        const MAX_REGISTER:i32 = 15i32;
        let n_keepers:usize = match self.sound.energy {
            // @art-choice Choose how many frequencies to drop 
            Energy::Low => {
                let x:i32= (MAX_REGISTER-5i32 - r).max(1);
                2f32.powi(x)
            },
            Energy::Medium => {
                let x:i32= (MAX_REGISTER-3i32 - r).max(1);
                2f32.powi(x)
            },
            Energy::High => {   
                let x:i32= (MAX_REGISTER -2i32 - r).max(1);
                2f32.powi(x)
            }
        } as usize;

        let mut opts:Vec<usize> = (frequency as usize..NF).collect();
        let mut rng = rand::thread_rng();
        opts.shuffle(&mut rng);


        let mut phases = vec![0.0; n_keepers];
        let mut amplitudes = vec![0.0; n_keepers];
        let mut contours:Vec<Vec<f32>> = vec![vec![]; n_keepers];
        let mut sig = vec![0f32; n_samples];
        let mut rng = rand::thread_rng();

        let d:f32 = 1f32;

        let mut ls = &lifespan::lifespans;
        let mut lifespan_opts:Vec<usize> = (0..ls.len()).collect();
        lifespan_opts.shuffle(&mut rng);

        let n_buckets = 2usize;
        let lifespans:Vec<&AmpLifespan> = lifespan_opts.iter().take(n_buckets).map(|i| &ls[*i]).collect();

        // Pre-calculate phases and amplitudes
        for (index, freq) in opts[0..n_keepers].iter().enumerate() {
            let f = *freq as f32 + offset;
            phases[index] = rng.gen::<f32>() * pi2;
            amplitudes[index] = NoiseColor::get_amp_mod(color, f as usize);
            contours[index] = if index  % 8 < 4 {
                let n_cycles = rng.gen::<f32>() * 4f32;
                lifespan::mod_lifespan(n_samples, n_cycles, lifespans[0], *freq, d)
            } else  {
                let n_cycles = 20f32 + rng.gen::<f32>() * 80f32;
                lifespan::mod_lifespan(n_samples, n_cycles, lifespans[1], *freq, d)
            };
            // if index % 2 == 0 { contours[index].reverse() }
        }
        

        for (index, freq) in opts[0..n_keepers].iter().enumerate() {
            let f = *freq as f32+ offset;

            for j in 0..n_samples {
                let amp = amplitudes[index];
                phr.note.p = j as f32 / n_samples as f32;
                let p_extra = phr.note.p * phr.note.cycles / phr.line.cycles;
                if amp > 0f32 && bandpass_filter(&self.sound.bandpass, f, phr.line.p + p_extra) {
                    let t = j as f32 / SR as f32;
                    let amod = crate::phrasing::contour::sample(&contours[index], phr.note.p);
                    
                    sig[j] += amod * amp * (f * pi2 * t + phases[index]).sin();
                } else {
                    continue
                }
            }
        };

        sig
    }   
}

fn render_line_degraded(mgen:&Mgen, line:&Vec<Note>, color:&NoiseColor, phr:&mut Phrasing) -> SampleBuffer {
    let mut samples:SampleBuffer = Vec::new();

    let len = line.len() as f32;
    let n_cycles = line.iter().fold(0f32, |acc, note| acc + time::duration_to_cycles(note.0));

    phr.line.cycles = n_cycles;
    for (index, note) in line.iter().enumerate() {
        phr.line.p = realize::dur_to(&line, index) / n_cycles;
        samples.append(&mut mgen.inflect_noise_degraded(&color, &note, phr))
    }
    samples
}


/// Given a list of notes representing a complete phrase, 
/// Mutate phrasing to create an expressive SampleBuffer
fn render_degraded_modulated(mgen:&Mgen, line:&Vec<Note>, color:&NoiseColor, phr:&mut Phrasing) -> SampleBuffer {
    let mut samples:SampleBuffer = Vec::new();

    let len = line.len() as f32;
    let n_cycles = line.iter().fold(0f32, |acc, note| acc + time::duration_to_cycles(note.0));

    phr.line.cycles = n_cycles;
    for (index, note) in line.iter().enumerate() {
        phr.line.p = realize::dur_to(&line, index) / n_cycles;
        samples.append(&mut mgen.inflect_noise_degraded_modulated(&color, &note, phr))
    }
    samples
}


/// Given a list of notes representing a complete phrase, 
/// Mutate phrasing to create an expressive SampleBuffer
fn render_line_shortened_ifft(mgen:&Mgen, line:&Vec<Note>, color:&NoiseColor, phr:&mut Phrasing) -> SampleBuffer {
    let mut samples:SampleBuffer = Vec::new();

    let len = line.len() as f32;
    let n_cycles = line.iter().fold(0f32, |acc, note| acc + time::duration_to_cycles(note.0));
    let mut rng = rand::thread_rng();

    phr.line.cycles = n_cycles;
    for (index, note) in line.iter().enumerate() {
        phr.line.p = realize::dur_to(&line, index) / n_cycles;

        samples.append(&mut mgen.inflect_noise_shortened_ifft(&mut rng, color, &note, phr))
    }
    samples
}

/// Given a list of notes representing a complete phrase, 
/// Mutate phrasing to create an expressive SampleBuffer
fn render_line_shortened(mgen:&Mgen, line:&Vec<Note>, color:&NoiseColor, phr:&mut Phrasing) -> SampleBuffer {
    let mut samples:SampleBuffer = Vec::new();

    let len = line.len() as f32;
    let n_cycles = line.iter().fold(0f32, |acc, note| acc + time::duration_to_cycles(note.0));
    let mut rng = rand::thread_rng();

    phr.line.cycles = n_cycles;
    for (index, note) in line.iter().enumerate() {
        phr.line.p = realize::dur_to(&line, index) / n_cycles;

        samples.append(&mut mgen.inflect_noise_shortened(&mut rng, color, &note, phr))
    }
    samples
}

fn render_line_shortened_blocked(mgen:&Mgen, line:&Vec<Note>, color:&NoiseColor, phr:&mut Phrasing) -> SampleBuffer {
    
    let mut samples:SampleBuffer = Vec::new();

    let len = line.len() as f32;
    let n_cycles = line.iter().fold(0f32, |acc, note| acc + time::duration_to_cycles(note.0));
    let mut rng = rand::thread_rng();

    phr.line.cycles = n_cycles;
    for (index, note) in line.iter().enumerate() {
        phr.line.p = realize::dur_to(&line, index) / n_cycles;

        samples.append(&mut mgen.inflect_noise_shortened_blocked(&mut rng, color, &note, phr))
    }
    samples

}

fn note_by_register(n_cycles:i32, register:i8) -> Note {
    ( (n_cycles, 1), (register, (0i8, 0i8, 1i8)), 1f32)
}


mod test {
    use super::*;
    use crate::files;
    use crate::render;

    static test_dir:&str = "dev-audio/presets/noise";

    static presence:Presence = Presence::Legato; // assume "worst case" for test
    static bandpass:(FilterMode, FilterPoint, (f32, f32)) = (FilterMode::Linear, FilterPoint::Constant, (MF as f32, NF as f32));
    static color:NoiseColor = NoiseColor::Pink;

    fn test_line() -> Vec<Note> {
        vec![
            note_by_register(1, 5),
            note_by_register(1, 5),
            note_by_register(1, 5),
            note_by_register(1, 5),
            note_by_register(1, 6),
            note_by_register(1, 6),
            note_by_register(1, 6),
            note_by_register(1, 6),
            note_by_register(1, 8),
            note_by_register(1, 8),
            note_by_register(1, 8),
            note_by_register(1, 8),
            note_by_register(1, 10),
            note_by_register(1, 10),
            note_by_register(1, 10),
            note_by_register(1, 10),
            note_by_register(1, 12),
            note_by_register(1, 12),
            note_by_register(1, 12),
            note_by_register(1, 12)
        ]
    }

    #[test]
    fn test_all() {
        test_degraded();
        test_inflect_noise_shortened();
    }

    #[test]
    fn test_inflect_noise_shortened() {
        // runtimes
        // low -> 2.030511241s 
        // medium -> 2.006620384s 
        // high -> 2.341231525s
        
        for energy in [Energy::Low, Energy::Medium, Energy::High] {
            let (test_name, duration) = crate::time::measure(|| {
                let mut phr = Phrasing {
                    cps:1f32, 
                    form: Timeframe {
                        cycles: 5f32,
                        p: 0f32,
                        instance: 0
                    },
                    arc: Timeframe {
                        cycles: 5f32,
                        p: 0f32,
                        instance: 0
                    },
                    line: Timeframe {
                        cycles: 5f32,
                        p: 0f32,
                        instance: 0
                    },
                    note: Timeframe {
                        cycles:0f32,
                        p: 0f32,
                        instance: 0
                    }
                };

                let sound = Sound {
                    bandpass,
                    energy,
                    presence,
                    pan: 0f32,
                };

                let mgen = Mgen {
                    osc:BaseOsc::Noise,
                    sound
                };

                let samples = render_line_shortened(&mgen, &test_line(), &color, &mut phr);

                files::with_dir(test_dir);
                let test_name = format!("noise-shortened-{:#?}-{:#?}",energy, color);
                let filename = format!("{}/{}.wav",test_dir,test_name);

                render::samples_f32(SR, &samples, &filename);
                test_name
            });
            println!("Completed test {:#?} in {:#?} ", test_name, duration);    
        }
    }

    #[test]
    fn test_shortened_blocked() {
        // runtimes
        // low -> 75.922789ms
        // medium -> 73.356566ms
        // high -> 83.386089ms
        
        for energy in [Energy::Low, Energy::Medium, Energy::High] {
            let (test_name, duration)= crate::time::measure(|| {
                let mut phr = Phrasing {
                    cps:1f32, 
                    form: Timeframe {
                        cycles: 5f32,
                        p: 0f32,
                        instance: 0
                    },
                    arc: Timeframe {
                        cycles: 5f32,
                        p: 0f32,
                        instance: 0
                    },
                    line: Timeframe {
                        cycles: 5f32,
                        p: 0f32,
                        instance: 0
                    },
                    note: Timeframe {
                        cycles:0f32,
                        p: 0f32,
                        instance: 0
                    }
                };

                let sound = Sound {
                    bandpass,
                    energy,
                    presence,
                    pan: 0f32,
                };

                let mgen = Mgen {
                    osc:BaseOsc::Noise,
                    sound
                };

                let samples = render_line_shortened_blocked(&mgen, &test_line(), &color, &mut phr);

                files::with_dir(test_dir);
                let test_name = format!("noise-shortened-blocked-{:#?}-{:#?}",energy, color);
                let filename = format!("{}/{}.wav",test_dir,test_name);

                render::samples_f32(SR, &samples, &filename);
                test_name
            });
            println!("Completed test {:#?} in {:#?} ", test_name, duration);    
        }
    }

    #[test]
    fn test_shortened_ifft() {
        // runtimes
        // low -> 9.249487142s
        // medium -> 9.496798079s
        // high -> 10.914292173s
        
        for energy in [Energy::Low, Energy::Medium, Energy::High] {
            let (test_name, duration)= crate::time::measure(|| {
                let mut phr = Phrasing {
                    cps:1f32, 
                    form: Timeframe {
                        cycles: 5f32,
                        p: 0f32,
                        instance: 0
                    },
                    arc: Timeframe {
                        cycles: 5f32,
                        p: 0f32,
                        instance: 0
                    },
                    line: Timeframe {
                        cycles: 5f32,
                        p: 0f32,
                        instance: 0
                    },
                    note: Timeframe {
                        cycles:0f32,
                        p: 0f32,
                        instance: 0
                    }
                };

                let sound = Sound {
                    bandpass,
                    energy,
                    presence,
                    pan: 0f32,
                };

                let mgen = Mgen {
                    osc:BaseOsc::Noise,
                    sound
                };

                let samples = render_line_shortened_ifft(&mgen, &test_line(), &color, &mut phr);

                files::with_dir(test_dir);
                let test_name = format!("noise-shortened-{:#?}-{:#?}",energy, color);
                let filename = format!("{}/{}.wav",test_dir,test_name);

                render::samples_f32(SR, &samples, &filename);
                test_name
            });
            println!("Completed test {:#?} in {:#?} ", test_name, duration);    
        }
    }

    #[test]
    fn test_degraded() {
        // runtimes
        // low -> 943.783023ms
        // medium -> 2.906214868s 
        // high -> 5.638831656s
        
        for energy in [Energy::Low, Energy::Medium, Energy::High] {
            let (test_name, duration)= crate::time::measure(|| {
                
                let mut phr = Phrasing {
                    cps:1f32, 
                    form: Timeframe {
                        cycles: 5f32,
                        p: 0f32,
                        instance: 0
                    },
                    arc: Timeframe {
                        cycles: 5f32,
                        p: 0f32,
                        instance: 0
                    },
                    line: Timeframe {
                        cycles: 5f32,
                        p: 0f32,
                        instance: 0
                    },
                    note: Timeframe {
                        cycles:0f32,
                        p: 0f32,
                        instance: 0
                    }
                };

                let sound = Sound {
                    bandpass,
                    energy,
                    presence : Presence::Legato,
                    pan: 0f32,
                };

                let mgen = Mgen {
                    osc:BaseOsc::Noise,
                    sound
                };

                let samples = render_line_degraded(&mgen, &test_line(), &color, &mut phr);

                files::with_dir(test_dir);
                let test_name = format!("noise-degraded-{:#?}-{:#?}",energy, color);
                let filename = format!("{}/{}.wav",test_dir,test_name);

                render::samples_f32(SR, &samples, &filename);
                test_name
            });
            println!("Completed test {:#?} in {:#?} ", test_name, duration);    
        }
    }
}