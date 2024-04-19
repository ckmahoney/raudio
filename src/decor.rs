use crate::types::{Range, Radian};
use crate::types::synthesis::{Freq, Note, Direction};
use crate::types::timbre;
use crate::types::timbre::{Sound, Energy, Presence, Phrasing};
use crate::envelope::db_to_amp;
use crate::time;


pub struct Modulators {
    pub amp: AmpMod,
    pub freq: FreqMod,
    pub phase: PhaseMod,
}

pub struct Ctx {
    pub dur_seconds: f32,
    pub root: f32
}

pub struct Coords {
    pub cps: Freq,
    pub k: usize,
    pub i: usize
}

pub type Modulator<T> = fn (xyz:&Coords, ctx:&Ctx, snd:&Sound, dir:&Direction, phr:&Phrasing) -> T;

pub type AmpMod = Modulator<Range>;
pub type PhaseMod = Modulator<Radian>;
pub type FreqMod = Modulator<Freq>;

/// Generate a monic frequency modulation curve by Presence and Energy
/// Recommended that values stay in bounds of \[0.5, 1\]
/// or do manual validation that result (when muliplied by ctx.r) is below Nyquist freq
fn fmod(xyz:&Coords, ctx:&Ctx, snd:&Sound, dir:&Direction, phr:&Phrasing) -> f32 {
    1.00f32.powi(xyz.k as i32)
}

/// Generate a monic amplitude modulation curve by Presence and Energy
fn amod(xyz:&Coords, ctx:&Ctx, snd:&Sound, dir:&Direction, phr:&Phrasing) -> Range {
    let dur_scale_factor = match &snd.presence {
        timbre::Presence::Staccatto => {
            0.2f32
        },
        timbre::Presence::Legato => {
            0.9f32
        },
        timbre::Presence::Tenuto => {
            1.0f32
        }
    };
    //@art-choice currently applied as global min/max values; can scale with cps
    let min_max_dur_seconds = match snd.presence {
        timbre::Presence::Staccatto => {
            (0.1f32, 0.5f32)
        },
        timbre::Presence::Legato => {
            (0.25f32, 100f32)
        },
        timbre::Presence::Tenuto => {
            (0.3f32, 120f32)
        }
    };

    let dur_seconds = (ctx.dur_seconds * dur_scale_factor)
        .min(min_max_dur_seconds.1)
        .max(min_max_dur_seconds.0);

    let final_sample = time::samples_from_dur(xyz.cps, dur_seconds);
    if xyz.i > final_sample {
        return 0f32
    }

    // @art-choice this model leaves headroom between each level 
    let min_max_db = match &snd.energy {
        timbre::Energy::Low => {
            (-60f32, -50f32)
        },
        timbre::Energy::Medium => {
            (-40f32, -30f32)
        },
        timbre::Energy::High => {   
            (-20f32, -10f32)
        }
    };

    // @art-choice amplitude scaling based on the monic
    let amp_k = match &snd.energy {
        timbre::Energy::Low => {
            let k = xyz.k as f32;
            if xyz.k > 3 { 
                1f32/(k *k);
            } else {
                1.0/k;
            }
        },
        timbre::Energy::Medium => {
            1.0/xyz.k as f32;
        },
        timbre::Energy::High => {   
            1.0f32;
        }
    };

    let dDecibel = (min_max_db.1 - min_max_db.0)/final_sample as f32;
    let decibel = dDecibel * xyz.i as f32;
    db_to_amp(decibel)
}

fn pmod(xyz:&Coords, ctx:&Ctx, snd:&Sound, dir:&Direction, phr:&Phrasing) -> f32 {
    0f32
}

pub fn gen(cps:f32, note:&Note)->Modulators {
    let (dur, tone, ampl) = note;
    Modulators {
        amp:amod,
        freq:fmod,
        phase:pmod
    }
}


#[test]
fn test_Min_max() {
    let x = 10f32.min(1000f32).max(1f32);
    assert_eq!(x, 10f32)
}