
extern crate rand;
use rand::Rng;
use rand::seq::SliceRandom;
use std::f32::consts::PI;

use crate::synth::{pi, pi2, SR, MF, NF, SampleBuffer};
use crate::types::synthesis::{Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::types::render::*;
use crate::types::timbre::{AmpContour,AmpLifespan,BandpassFilter, Energy, Presence, BaseOsc, Sound, FilterMode, Timeframe, Phrasing};
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
    Equal,
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
    o1m1:Vec<f32>,
    o1m3:Vec<f32>,
    o1m5:Vec<f32>,
    o1m7:Vec<f32>,
    u1m1:Vec<f32>,
    u1m3:Vec<f32>,
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
            Equal => 1.0,
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
            o1m1: harmonic_shell(true, 1),
            u1m1: harmonic_shell(false, 1),
            o1m3: harmonic_shell(true, 3),
            u1m3: harmonic_shell(false, 3),
            o1m5: harmonic_shell(true, 5),
            o1m7: harmonic_shell(true, 7),
        }
    }
}

/// Construct an amplitude coefficient vector applicable as either overtones or undertones
/// Representing the activation of all octaves of monic in (1..max_monic)
pub fn harmonic_shell(overtones: bool, max_monic:usize) -> SampleBuffer {
    let k = NF / MF;
    let basis = 1..=k;
    let n:usize = 1 + ((max_monic - 1) / 2);
    let fundamental_monics:Vec<usize> = (0..n).map(|i| 2*i + 1).collect();
    let mut amps = vec![0f32; k];
    let octave_range:usize = 8;

    if overtones { 
        for m in fundamental_monics {
            let mf = m as f32;
            let coef_m = 1f32;

            for octave in 0..octave_range {
                let h = 2f32.powi(octave as i32) * m as f32;
                if h as usize > NF { continue } // out of amps bounds

                /* This version of the coefficient is better suited for lead instruments because it emphasizes the fundamental more */
                // let coef_h = 1f32 / (h as f32);

                /* This version of the coefficient is better for supporting/background instruments because it plainly outlines the space */
                let coef_h = 2f32.powi(-1i32 * octave as i32);

                let index = h as usize - 1;

                amps[index] += coef_m * coef_h;
            }
        }
    } else {
        for m in fundamental_monics {
            let mf = m as f32;
            let coef_m = if m == 1 { 0.5f32 } else { 1f32 };


            // for octave in 0..octave_range {
            //     let h = 2f32.powi(octave as i32) * m as f32;
            //     if h as usize > NF { continue } // out of amps bounds
            //     let o = octave_range - octave;

            //     /* This version of the coefficient is better suited for lead instruments because it emphasizes the fundamental more */
            //     // let coef_h = 1f32 / (h as f32);

            //     /* This version of the coefficient is better for supporting/background instruments because it plainly outlines the space */
            //     let coef_h = 1f32;

            //     let index = h as usize - 1;

            //     amps[index] += coef_m * coef_h;
            // }

            // for amp(k) = 0.5 * tanh(1 + 3(x-c)/c.pow(4/3))
            for octave in 0..octave_range {
                let h = 2f32.powi(octave as i32) * m as f32;
                if h as usize > NF { continue } // out of amps bounds
                let o = octave_range - octave;

                /* 
                Constant "c" is generally best at 1 (no centroid offset) to support both low and high fundamentals. 
                If it is known that most fundamentals will be high, offcet c =4 or c =16 also sound great. 
                */
                let c = 1f32;
                /* This version of the coefficient is better for supporting/background instruments because it plainly outlines the space */
                let coef_h = 0.5f32 * (1f32 + 3f32*(h - c)/c.powf(4f32/3f32)).tanh();

                let index = h as usize - 1;

                amps[index] += coef_m * coef_h;
            }

        }
    }
    render::normalize(&mut amps);
    amps
}

/* Mix component for a monic coefficient vector. Applied as (weight, rotation) */
type Neighbor = (f32, i32);
type Space<'amps> = (Vec<Neighbor>, &'amps Vec<f32>);

fn gen_samples_poly(basis:f32, fund:f32, overs:Space, unders:Space, n_samples:usize) -> SampleBuffer {
    let mut buffs:Vec<SampleBuffer> = Vec::with_capacity(overs.0.len() + unders.0.len());    
    for (weight, rot) in overs.0 {
        let empty = vec![0f32; overs.1.len()];
        let f = fund * basis.powi(rot);
        buffs.push(gen_samples(weight, f, &overs.1, &empty, n_samples))

    }

    for (weight, rot) in unders.0 {
        let empty = vec![0f32; unders.1.len()];
        let f = fund * basis.powi(rot);
        buffs.push(gen_samples(weight, f, &empty, &unders.1, n_samples))

    }

    match render::mix_and_normalize_buffers(buffs) {
        Ok(sig) => sig,
        Err(msg) => panic!("render error {}",msg)
    }
}

fn gen_samples(amp:f32, fund:f32, shell_overs:&Vec<f32>, shell_unders:&Vec<f32>, n_samples:usize) -> SampleBuffer {
    let max_k = (NF as f32 / fund) as usize;
    let overtones = &shell_overs[0..max_k];
    let undertones = &shell_unders[0..max_k];

    let sr = SR as f32;
    (0..n_samples).map(|j| {  
        (1..=max_k).map(|k| {  

            let coef_over = shell_overs[k-1];  
            let coef_under = shell_unders[k-1];  

            if coef_under == 0.0 && coef_over == 0.0 { 0.0 } 
            else { 
                let a = if coef_over > 0f32 { 
                    let f = fund * k as f32;
                    let fu = f as usize;
                    if fu > NF || fu < MF { 0f32 } else {
                        coef_over * (pi2 * f * j as f32 / sr as f32).sin() 
                    }
                } else { 0f32 };

                let b = if coef_under > 0f32 { 
                    let f = fund / k as f32;
                    let fu = f as usize;
                    if fu > NF || fu < MF { 0f32 } else {
                        coef_under * (pi2 * f * j as f32 / sr as f32).sin()
                    }
                } else { 0f32 };
                amp * (a + b)
            }
        }).sum()
    }).collect()
}

pub fn gen_pad(cps:f32, amp:f32, harmonic_basis:f32, freq:f32, depth:usize, m:usize, n_cycles:f32) -> SampleBuffer { 
    let neighbors:Vec<Neighbor> = (0..depth).map(|i|(1f32, i as i32)).collect();
    let over_shells = harmonic_shell(true,m);
    let under_shells = harmonic_shell(false,m);

    let unit_o:Space =  (neighbors.to_owned(), &over_shells);
    let unit_u:Space = (neighbors.to_owned(), &under_shells);
    let mut samples = gen_samples_poly(harmonic_basis, freq, unit_o, unit_u, time::samples_of_cycles(cps, n_cycles));        
    render::norm_scale(&mut samples, amp);
    samples
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
        let l =&stash.sine.len();
        let empty = vec![0f32; *l];
        let mut samples = gen_samples(1.0f32, fundamental, &stash.sine, &empty, SR * 2);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);


        let filename = format!("{}/test-shell-square.wav", test_dir);
        let mut samples = gen_samples(1.0f32, fundamental, &stash.square, &empty, SR * 2);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);


        let filename = format!("{}/test-shell-triangle.wav", test_dir);
        let mut samples = gen_samples(1.0f32, fundamental, &stash.triangle, &empty, SR * 2);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);


        let filename = format!("{}/test-shell-sawtooth.wav", test_dir);
        let mut samples = gen_samples(1.0f32, fundamental, &stash.sawtooth, &empty, SR * 2);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);
    }


    #[test]
    fn test_harmonic_shells() {
        let stash = BaseOsc::harmonic_shells();
        files::with_dir(test_dir);

        let l =&stash.u1m1.len();
        let empty = vec![0f32; *l];

        let fundamental = 54.687554f32;
        let filename = format!("{}/test-shell-o1m1.wav", test_dir);
        let mut samples = gen_samples(1.0f32, fundamental, &stash.o1m1, &empty, SR * 4);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);

        let filename = format!("{}/test-shell-o1m3.wav", test_dir);
        let mut samples = gen_samples(1.0f32, fundamental, &stash.o1m3, &empty, SR * 4);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);


        let filename = format!("{}/test-shell-o1m5.wav", test_dir);
        let mut samples = gen_samples(1.0f32, fundamental, &stash.o1m5, &empty, SR * 4);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);


        let filename = format!("{}/test-shell-o1m7.wav", test_dir);
        let mut samples = gen_samples(1.0f32, fundamental, &stash.o1m7, &empty, SR * 4);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);


        let fundamental = 7000 as f32;
        let fundamental = 0.25f32 * 2000 as f32;
        let filename = format!("{}/test-shell-u1m1.wav", test_dir);
        let under_shells = harmonic_shell(false, 1);
        let mut samples = gen_samples(1.0f32, fundamental, &empty, &under_shells, SR * 4);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);

        let filename = format!("{}/test-shell-u1m3.wav", test_dir);
        let under_shells = harmonic_shell(false, 3);
        let mut samples = gen_samples(1.0f32, fundamental, &empty,  &under_shells, SR * 4);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);

        let filename = format!("{}/test-shell-u1m5.wav", test_dir);
        let under_shells = harmonic_shell(false, 5);
        let mut samples = gen_samples(1.0f32, fundamental, &empty,  &under_shells, SR * 4);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);

    }

    fn test_bi_q(){ 
        let stash = BaseOsc::harmonic_shells();
    
        files::with_dir(test_dir);

        let l =&stash.u1m1.len();
        let empty = vec![0f32; *l];


        let fundamental = 64f32;
        let filename = format!("{}/test-shell-u1m7.wav", test_dir);
        let under_shells = harmonic_shell(false, 7);
        let mut samples = gen_samples(1.0f32, fundamental, &empty,  &under_shells, SR * 4);        
        render::normalize(&mut samples);
        render::samples_f32(SR, &samples, &filename);


        let over_shells = harmonic_shell(true, 5);
        let under_shells = harmonic_shell(false, 5);
        for freq in (0i32..8i32).map(|x| 2f32.powi(x) * fundamental) {
            let filename = format!("{}/test-shell-o1m5-u1m5-{}.wav", test_dir, freq);
            
            let mut samples = gen_samples(1.0f32, freq, &over_shells,  &under_shells, SR * 4);        
            render::normalize(&mut samples);
            render::samples_f32(SR, &samples, &filename);
        }


    }

    #[test]
    fn test_poly_q(){ 
        let stash = BaseOsc::harmonic_shells();
        files::with_dir(test_dir);

        let l =&stash.u1m1.len();
        let empty = vec![0f32; *l];

        let fundamental = 64f32;
        let harmonic_basis = 1.5f32;

        for freq in (0i32..8i32).map(|x| 2f32.powi(x) * fundamental) {
            for n in [2,3] {
                let neighbors:Vec<Neighbor> = (0..n).map(|i|(1f32, i as i32)).collect();
                for m in [1,3,5] {
                    let over_shells = harmonic_shell(true,m);
                    let under_shells = harmonic_shell(false,m);

                    // let filename = format!("{}/test-shell-freq-{}-poly-o{}m{}.wav", test_dir, freq, n, m);
                    // let unit_o:Space = (neighbors.to_owned(), &over_shells);
                    // let unit_u:Space = (vec![], &empty);
                    // let mut samples = gen_samples_poly(harmonic_basis, freq, unit_o, unit_u, SR * 4);        
                    // render::normalize(&mut samples);
                    // render::samples_f32(SR, &samples, &filename);


                    // let filename = format!("{}/test-shell-freq-{}-poly-u{}m{}.wav", test_dir, freq, n, m);
                    // let unit_o:Space =  (vec![], &empty);
                    // let unit_u:Space = (neighbors.to_owned(), &under_shells);
                    // let mut samples = gen_samples_poly(harmonic_basis, freq, unit_o, unit_u, SR * 4);        
                    // render::normalize(&mut samples);
                    // render::samples_f32(SR, &samples, &filename);


                    let filename = format!("{}/test-shell-freq-{}-poly-o{}m{}-u{}m{}.wav", test_dir, freq, n, m, n, m);
                    let unit_o:Space =  (neighbors.to_owned(), &over_shells);
                    let unit_u:Space = (neighbors.to_owned(), &under_shells);
                    let mut samples = gen_samples_poly(harmonic_basis, freq, unit_o, unit_u, SR * 4);        
                    render::normalize(&mut samples);
                    render::samples_f32(SR, &samples, &filename);
                }
            }
        }
    }
}

