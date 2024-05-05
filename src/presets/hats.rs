use crate::preset::{pi, pi2, Ctx, Coords, AmpMod, PhaseMod,FreqMod, none};
use crate::types::synthesis::{Freq, Note, Direction};

use crate::envelope::db_to_amp;
use crate::time;
use rand;
use rand::Rng;

use crate::types::{Range, Radian};
use crate::types::timbre;
use crate::types::timbre::{Sound, Energy, Presence, Phrasing};


fn default_db(energy:&Energy) -> (f32, f32) {
    match energy {
        timbre::Energy::Low => {
            (-9f32, -6f32)
        },
        timbre::Energy::Medium => {
            (-6f32, -3f32)
        },
        timbre::Energy::High => {   
            (-3f32, -0f32)
        }
    }
}


/// for function f(x) with range in [a, b]
/// returns g(x) for a given value y representing f(x).
fn map_range_lin(f_a:f32, f_b:f32, g_a:f32, g_b:f32, y:f32) -> f32 {
    let mean_g:f32 = (g_b + g_a) / 2f32;
    let range_f = (f_b - f_a).abs();
    let range_g:f32 = (g_b - g_a).abs();

    let linear_interp = range_g / range_f;
    mean_g + (linear_interp * y)
}


pub use none::fmod;

/// Generate a monic amplitude modulation curve by Presence and Energy
pub fn amod(xyz:&Coords, ctx:&Ctx, snd:&Sound, dir:&Direction, phr:&Phrasing) -> Range {
    // needs to get sample rate from args
    let sample_rate = 48000usize;
    let max_monic = (sample_rate / xyz.k) as f32;

    let dur_scale_factor = match &snd.presence {
        timbre::Presence::Staccatto => {
            0.1f32
        },
        timbre::Presence::Legato => {
            0.5f32
        },
        timbre::Presence::Tenuto => {
            1.0f32
        }
    };
    //@art-choice currently applied as global min/max values; can scale with cps
    let min_max_dur_seconds = match snd.presence {
        timbre::Presence::Staccatto => {
            (0.05f32, 15f32)
        },
        timbre::Presence::Legato => {
            (0.1f32, 100f32)
        },
        timbre::Presence::Tenuto => {
            (0.2f32, 120f32)
        }
    };

    let dur_seconds = (ctx.dur_seconds * dur_scale_factor)
        .min(min_max_dur_seconds.1)
        .max(min_max_dur_seconds.0);

    let final_sample = time::samples_from_dur(xyz.cps, dur_seconds);
    if xyz.i > final_sample {
        return 0f32
    }

    // @art-curr Uses inverse monic amplitude. E.g. fundamental is silent, highest monic is dominant.
    let k = xyz.k as f32;
    let amp_k:f32 = 
        match &snd.energy {
        timbre::Energy::Low => {
            if xyz.k > 3 {  
                0f32
            } else {
                let mul = 1.0;
                mul/k
            }
        },
        timbre::Energy::Medium => {
            let k = xyz.k as f32;
            if xyz.k > 9 { 
                0f32 
            } else {
                let mul:f32  = 1.0;
                mul/k
            }
        },
        timbre::Energy::High => {   
            let k = xyz.k as f32;
            if xyz.k > 23 { 
                let mul = 1.0;
                mul * 1f32/(k *k) 
            } else {
                let mul:f32  = 1.0f32;
                1.0
            }
        }
    };


    let j = time::samples_from_dur(xyz.cps, dur_seconds) as f32;
    let p:f32 = xyz.i as f32/j;
    let one:f32 = 1f32;
    let amp_t = 0.5 - ((pi2*p - 3f32).tanh() / 2f32);

    // let amp_t:f32 = if xyz.i == 0 { 1f32 } else {
    //     let x = p;
    //     let constant = -0.5f32;
    //     let coeff = 0.5f32;
    //     let fx = pi2 * x - 3f32;



    //     let coeff = 2f32/3f32;
    //     coeff * fx.tanh() + constant
    // }.min(1f32);

    amp_t * amp_k
}

// skip phase modulation
pub use none::pmod;
