
extern crate rand;
use rand::Rng;
use rand::seq::SliceRandom;
use std::f32::consts::PI;

use crate::synth::{pi, pi2, SR, MF, NF, SampleBuffer};
use crate::types::synthesis::{Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::types::render::*;
use crate::types::timbre::{AmpContour,AmpLifespan,BandpassFilter, Energy, Presence, BaseOsc, Sound, FilterMode, Timeframe, Phrasing, Ampex};
use crate::monic_theory::tone_to_freq;
use crate::{render, time};
use crate::phrasing::contour;
use crate::phrasing::lifespan;

use crate::phrasing::bandpass_filter;


pub struct Mgen {
    osc: BaseOsc,
    sound: Sound
}


#[derive(Debug)]
/* Power spectrum functions described by conventional noise counterparts. */
pub enum PowerColor {
    Violet,
    Blue,
    White,
    Pink,
    Red,
}

pub struct OscShells {
    sine:Vec<f32>,
    triangle:Vec<f32>,
    square:Vec<f32>,
    sawtooth:Vec<f32>,
}

pub struct HarmShells {
    o1m3:Vec<f32>,
    o1m5:Vec<f32>,
    o1m7:Vec<f32>,
    // u1m3:Vec<f32>,
    // u1m5:Vec<f32>,
    // u1m7:Vec<f32>,
}

use BaseOsc::*;

pub static shapes:[BaseOsc; 4] = [
    Sine,
    Square,
    Triangle,
    Sawtooth
];

impl BaseOsc {

    #[inline]
    pub fn get_amp_mod(color: &PowerColor, f:f32, k:usize) -> f32 {
        use PowerColor::*;
        let freq = f * k as f32;
        let fu = freq as usize;

        if (fu > NF) || (fu < MF) {
            return 0f32
        };
        let max_k = (NF / fu) as f32;
        let kf = k as f32;

        match color {
            Violet => 1.0 / (max_k - kf).powi(2),
            Blue => 1.0 / (max_k - kf).sqrt(),
            White => 1.0,
            Pink => 1.0 / (kf).sqrt(),
            Red => 1.0 / (kf).powi(2),
        }
    }

    /// Uses application minimum and manimum frequencies to generate amplitude coefficients for man available harmonic buffers for conventional waveforms
    /// as of may 14 2024 has interesting reuslts. eg the "sine" pattern sounded like a triangle, and the "triangle" is just a sine
    pub fn melodic_shells() -> OscShells { 
        let k = NF / MF;
        let basis = 1..=k;

        let one:f32 = 1f32;
        let two:f32 = 2f32;
        let coef_tri = 8f32/(pi*pi);
        let coef_square = 4f32 / pi;
        let coef_saw = two / pi;

        let sine:Vec<f32> = basis.clone()
            .map(|n| 
                if n == 1 || n % 2 == 0 { 1f32 / (k*k) as f32 } else { 0f32 }
                .abs()
            ).collect();
        let square:Vec<f32> = basis.clone()
            .map(|n| 
                if n % 2 == 0 { 0f32 } 
                else { 
                    coef_square * (two * n as f32 - one).sin() / (two * n as f32)
                } 
                .abs()
            ).collect();
        let triangle:Vec<f32> = basis.clone()
            .map(|n| 
                if n % 2 == 0 { 0f32 } 
                else { 
                    coef_tri * (two*n as f32).cos() / (two*n as f32 - one).powi(2i32) 
                }
                .abs()
            ).collect();
        let sawtooth:Vec<f32> = basis.clone()
            .map(|n|
                (coef_saw * (-one).powi(n as i32 - 1) * (n as f32).sin() / n as f32)
                .abs()
            ).collect();

        
        OscShells {
            sine,
            triangle,
            square,
            sawtooth
        }
    }

    pub fn harmonic_shells() -> HarmShells { 
        HarmShells {
            o1m3: harmonic_shell(true, 3),
            o1m5: harmonic_shell(true, 5),
            o1m7: harmonic_shell(true, 7),
        }
    }
}

pub fn harmonic_shell(overtones: bool, max_monic:usize) -> SampleBuffer {
    let k = NF / MF;
    let basis = 1..=k;
    let n:usize = 1 + ((max_monic - 1) / 2);
    let fundamental_monics:Vec<usize> = (0..n).map(|i| 2*i + 1).collect();

    let mut amps = vec![0f32; k];


    for m in fundamental_monics {
        let mf = m as f32;
        let coef_m = 1f32 / mf;

        for octave in 0..8 {
            let h = 2f32.powi(octave as i32) * m as f32;
            if h as usize > NF { continue }

            let coef_h = 1f32 / (h as f32).powi(2i32);

            let coef_extra = 1f32 / (mf * h as f32);
            let index = h as usize - 1;
            amps[index] += coef_m * coef_h;
        }
    }
    render::normalize(&mut amps);
    amps
}
fn gen_samples(fund:f32, shell:&Vec<f32>, n_samples:usize) -> SampleBuffer {
    let max_k = (NF as f32 / fund) as usize;
    let harmonics = &shell[0..max_k];

    for (index, a) in shell[0..100].iter().enumerate() {
        if *a > 0f32 {
            println!("monic {} amp {} ", index+1,a)
        }
    }
    let sr = SR as f32;
    (0..n_samples).map(|j| {  
        (1..=max_k).map(|k| {  
            let amplitude = harmonics[k-1];  
            if amplitude == 0.0 { 0.0 } 
            else { amplitude * (pi2 * fund * k as f32 * j as f32/ sr as f32).sin() }
        }).sum()
    }).collect()
}


#[cfg(test)]
mod test {
    use super::*;
    static test_dir:&str = "dev-audio/presets/melodic";
    use crate::files;
    use crate::render;

    #[test]
    fn test_melodic_shells() {
        let stash = BaseOsc::melodic_shells();
    
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
        files::with_dir(test_dir);
        let fundamental = 415f32;

        let filename = format!("{}/test-shell-sine.wav", test_dir);
        let mut samples = gen_samples(fundamental, &stash.sine, SR * 2);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);


        let filename = format!("{}/test-shell-square.wav", test_dir);
        let mut samples = gen_samples(fundamental, &stash.square, SR * 2);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);


        let filename = format!("{}/test-shell-triangle.wav", test_dir);
        let mut samples = gen_samples(fundamental, &stash.triangle, SR * 2);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);


        let filename = format!("{}/test-shell-sawtooth.wav", test_dir);
        let mut samples = gen_samples(fundamental, &stash.sawtooth, SR * 2);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);
    }



    #[test]
    fn test_harmonic_shells() {
        let stash = BaseOsc::harmonic_shells();
    
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
        files::with_dir(test_dir);
        let fundamental = 415f32;

        let filename = format!("{}/test-shell-o1m3.wav", test_dir);
        let mut samples = gen_samples(fundamental, &stash.o1m3, SR * 4);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);


        let filename = format!("{}/test-shell-o1m5.wav", test_dir);
        let mut samples = gen_samples(fundamental, &stash.o1m5, SR * 4);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);


        let filename = format!("{}/test-shell-o1m7.wav", test_dir);
        let mut samples = gen_samples(fundamental, &stash.o1m7, SR * 4);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);

        let ss:Vec<SampleBuffer> = vec![
            gen_samples(fundamental*1f32, &stash.o1m3, SR * 2),
            gen_samples(fundamental*3f32, &stash.o1m5, SR * 4),        
            gen_samples(fundamental*5f32, &stash.o1m7, SR * 4)        
        ];
        
        let filename = format!("{}/test-shell-mix-major.wav", test_dir);
        match render::pad_and_mix_buffers(ss) {
            Ok(signal) => render::samples_f32(SR, &signal, &filename),
            _ => {} 
        }
    }
}

