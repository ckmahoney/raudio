/// Methods for producing absoultely positioned noise
/// Remarkably different than relative noise; Absolute noise uses general conventionally defined algorithms for producing fields of values for space greater than 0.
/// Overall a fourier series representation for noise is overkill and very expensive.
/// Though it is good to see implementations here analog to the mgen for melodic and enharmonic synths.
/// 
/// Some methods applied below to improve performance:
/// 
/// ## Cache
/// Cache 1 period of each noise value. Takes 42 seconds to fill the "Red" cache (1/5 entries).
/// Copies from cach are very slow and expensive
/// Bandlimited copies much faster, scaling O(n) 
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
/// Therefore, for higher power noise signals, we can use this degredation technique to increase runtime performance.
/// This is a better selection typically for percussive instruments which are more concerned about covering a wide range in a short time.

extern crate rand;
use rand::Rng;
use rand::seq::SliceRandom;
use std::f32::consts::PI;

use crate::synth::{pi, pi2, SR, MF, NF, SampleBuffer};
use crate::types::synthesis::{Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::types::render::*;
use crate::types::timbre::{AmpContour,AmpLifespan,BandpassFilter, Energy, Presence, BaseOsc, Sound, FilterMode, Timeframe, Phrasing, Ampex};
use crate::monic_theory::tone_to_freq;
use crate::time;
use crate::phrasing::contour;
use crate::phrasing::lifespan;

use crate::phrasing::bandpass_filter;


pub struct Mgen {
    osc: BaseOsc,
    sound: Sound
}


#[derive(Debug)]
pub enum NoiseColor {
    Violet,
    Blue,
    White,
    Pink,
    Red,
}

impl NoiseColor {
    pub fn variants() -> Vec<NoiseColor> {
        vec![
            NoiseColor::White,
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
            NoiseColor::White => 1.0,
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

    pub fn inflect_noise_shortened(&self, color:&NoiseColor, note:&Note, phr:&mut Phrasing) -> SampleBuffer {
        let dur = time::duration_to_cycles(note.0);
        let frequency = tone_to_freq(&note.1);
        let ampl = &note.2;
        let n_samples = (dur  * time::samples_per_cycle(phr.cps) as f32) as usize;
        let max_added_freq = NF - frequency as usize;
        
        
        let n_freqs:usize = frequency as usize + match self.sound.energy {
            Energy::Low => {
                100
            },
            Energy::Medium => {
                400
            },
            Energy::High => {   
                max_added_freq
            }
        }.min(1000).max(max_added_freq);

        let mut phases = vec![0.0; n_freqs];
        let mut amplitudes = vec![0.0; n_freqs];
        let mut sig = vec![0f32; n_samples];
        let mut rng = rand::thread_rng();

        phr.note.cycles = dur;

        // Pre-calculate phases and amplitudes
        for k in 0..n_freqs {
            let freq = frequency as usize + k;
            phases[k] = rng.gen::<f32>() * 2.0 * PI;
            amplitudes[k] = NoiseColor::get_amp_mod(color, freq);
        }

        for k in 0..n_freqs {
            let f = (frequency as usize + k) as f32;
            for j in 0..n_samples {
                phr.note.p = j as f32 / n_samples as f32;
                let p_extra = phr.note.p * phr.note.cycles / phr.line.cycles;
                if amplitudes[k] > 0f32 && bandpass_filter(&self.sound.bandpass, f, phr.line.p + p_extra) {
                    let t = j as f32 / SR as f32;
                    let amp = amplitudes[k];

                    sig[j] += amp * (f * pi2 * t + phases[k]).sin();
                } else {
                    continue
                }
            }
        };

        sig
    }   


    /// Keep a small portion of the noise signal
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
            let f = *freq as f32+ offset;
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

/// Given a list of notes representing a complete phrase, 
/// Mutate phrasing to create an expressive SampleBuffer
fn render_line(mgen:&Mgen, line:&Vec<Note>, color:&NoiseColor, phr:&mut Phrasing) -> SampleBuffer {
    let mut samples:SampleBuffer = Vec::new();

    let len = line.len() as f32;
    let n_cycles = line.iter().fold(0f32, |acc, note| acc + time::duration_to_cycles(note.0));

    phr.line.cycles = n_cycles;
    for (index, note) in line.iter().enumerate() {
        //@bug not correct implementation of p. needs to be decided by accumulative position not index
        phr.line.p = index as f32 / len;
        samples.append(&mut mgen.inflect_noise_degraded_modulated(&color, &note, phr))
    }
    samples
}

fn note_by_register(n_cycles:i32, register:i8) -> Note {
    ( (n_cycles, 1), (register, (0i8, 0i8, 1i8)), 1f32)
}


mod test {
    static test_dir:&str = "dev-audio/presets/noise";
    use super::*;

    #[test]
    /// This shows that it is way too expensive to keep a cache of fourier series values (even 1 second) 
    /// just 1 period of 1hertz red noise takes 41 seconds to render.
    /// It enabled making copies at 12 seconds shorter copies, and halted with SIGTERM 9 when attempting to do three in one functionc all.
    fn test_cache() {
        let freq = 500f32;
        let (cache, duration )= crate::time::measure(|| NoiseCache::new(SR));
        println!("Time to create single entry cache: {:#?}", duration);
        let (_, duration) = crate::time::measure(|| cache.get(freq, &NoiseColor::Red, SR/2));
        println!("Time to render shorter segment: {:#?}", duration);
        let (_, duration) = crate::time::measure(|| cache.get(freq, &NoiseColor::Red, SR));
        println!("Time to render copy segment: {:#?}", duration);
        let (_, duration) = crate::time::measure(|| cache.get(freq, &NoiseColor::Red, SR*4));
        println!("Time to render longer segment: {:#?}", duration);
    }

    #[test]
    fn test_all() {
        use crate::files;
        use crate::render;

        // for color in NoiseColor::variants() {
        for color in vec![NoiseColor::Pink] {
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
                bandpass: (FilterMode::Linear, FilterPoint::Constant, (MF as f32, NF as f32)),
                energy: Energy::High,
                presence : Presence::Legato,
                pan: 0f32,
            };

            let line:Vec<Note> = vec![
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
                note_by_register(1, 12),
            ];

            let mgen = Mgen {
                osc:BaseOsc::Noise,
                sound
            };

            let samples = render_line(&mgen, &line, &color, &mut phr);

            files::with_dir(test_dir);
            let test_name = format!("noise-degraded-1000-{:#?}",color);
            let filename = format!("{}/{}.wav",test_dir,test_name);

            render::samples_f32(44100, &samples, &filename);
            println!("Rendered audio file{:#?} ", filename);
        }
    }
}












struct NoiseCache {
    // pink: Vec<Vec<f32>>,
    // white: Vec<Vec<f32>>,
    // blue: Vec<Vec<f32>>,
    // violet: Vec<Vec<f32>>,
    red: Vec<Vec<f32>>,
}

/// Given a frequency value, 
/// Get the highest supported noise offset value 
/// helps make noise computation cheaper
fn get_max_freq(freq:f32) -> usize {
    1000usize.min(NF - freq as usize)
}

/// Generate a sampled fourier series of the specified noise type 
impl NoiseCache {
    fn looper(source: &Vec<f32>, repeat_times: f32) -> Vec<f32> {
        let total_elements = (source.len() as f32 * repeat_times).round() as usize;
    
        let result = source.iter()
            .cycle()  
            .take(total_elements)  
            .copied() 
            .collect::<Vec<f32>>();
    
        result
    }

    fn to_size(freq:f32, group:&Vec<Vec<f32>>, n_samples:usize) -> Vec<Vec<f32>> {
        let len = group[0].len();
        if len == n_samples {
            return group.clone()
        }
        let start = freq.floor() as usize;
        let end = get_max_freq(freq);
        group[start..end].iter().map(|row| 
            NoiseCache::looper(row, n_samples as f32/ len as f32)
        ).collect()
    }

    pub fn new(n_samples:usize) -> Self {
        NoiseCache {
            // violet: Mgen::buffs_by_duration(n_samples, &NoiseColor::Violet),
            // blue: Mgen::buffs_by_duration(n_samples, &NoiseColor::Blue),
            // white: Mgen::buffs_by_duration(n_samples, &NoiseColor::White),
            // pink: Mgen::buffs_by_duration(n_samples, &NoiseColor::Pink),
            red: Mgen::buffs_by_duration(n_samples, &NoiseColor::Red),
        }
    }

    pub fn get(&self, freq:f32, color:&NoiseColor, n_samples:usize) -> Vec<Vec<f32>> {
        let group = match color {
            // NoiseColor::Violet => &self.violet,
            // NoiseColor::Blue => &self.blue,
            // NoiseColor::White => &self.white,
            // NoiseColor::Pink => &self.pink,
            NoiseColor::Red => &self.red,
            _ => { &self.red} ,
        };

        NoiseCache::to_size(freq, group, n_samples)
    }
}