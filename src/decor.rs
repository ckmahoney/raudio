use crate::types::{Range, Radian};
use crate::types::synthesis::{Freq, Note, Direction};
use crate::types::timbre;
use crate::types::timbre::{Sound, Energy, Presence, Phrasing};
use crate::envelope::db_to_amp;
use crate::time;
use rand;
use rand::Rng;

static pi:f32 = std::f32::consts::PI;
static pi2:f32 = pi*2f32;

pub struct Modulators {
    pub amp: AmpMod,
    pub freq: FreqMod,
    pub phase: PhaseMod,
}

pub struct Ctx {
    pub dur_seconds: f32,
    pub root: f32,
    pub extension: usize
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

/// for function f(x) with range in [a, b]
/// returns g(x) for a given value y representing f(x).
fn map_range_lin(f_a:f32, f_b:f32, g_a:f32, g_b:f32, y:f32) -> f32 {
    let mean_g:f32 = (g_b + g_a) / 2f32;
    let range_f = (f_b - f_a).abs();
    let range_g:f32 = (g_b - g_a).abs();

    let linear_interp = range_g / range_f;
    mean_g + (linear_interp * y)
}

#[test]
fn test_map_range_lin() {
    let min_f = -1f32;
    let max_f = 1f32;
    let min_g = 2f32; 
    let max_g = 3f32;

    let mut y = 0f32.sin();
    let mut expected = 2.5f32;
    let mut actual = map_range_lin(min_f, max_f, min_g, max_g, y);
    assert_eq!(expected, actual, "Expected to find {} but actually got {}", expected, actual);

    y = (pi/2f32).sin();
    expected = 3.0f32;
    actual = map_range_lin(min_f, max_f, min_g, max_g, y);
    assert_eq!(expected, actual, "Expected to find {} but actually got {}", expected, actual);

    y = pi.sin();
    expected = 2.5f32;
    actual = map_range_lin(min_f, max_f, min_g, max_g, y);
    assert_eq!(expected, actual, "Expected to find {} but actually got {}", expected, actual);


    y = (3f32 * pi/2f32).sin();
    expected = 2.0f32;
    actual = map_range_lin(min_f, max_f, min_g, max_g, y);
    assert_eq!(expected, actual, "Expected to find {} but actually got {}", expected, actual);
}

/// @art-choice Frequency modualtor providing time, glide, and sweep effects parametrized by extension
/// @art-curr Has a vibrato effect at the entry portion of the note. 
/// @art-choice Create a glide by going from highest available octave to ctx.root
/// returns a value in (0, 2.pow(ctx.extension))
fn fmod(xyz:&Coords, ctx:&Ctx, snd:&Sound, dir:&Direction, phr:&Phrasing) -> f32 {
    if ctx.extension == 0 {
        return 1f32;
    }

    let dur_seconds = match snd.presence {
        timbre::Presence::Staccatto => {
            ctx.dur_seconds * 0f32
        },
        timbre::Presence::Legato => {
            ctx.dur_seconds * 0.33
        },
        timbre::Presence::Tenuto => {
            ctx.dur_seconds * 0.66
        }
    };

    // last sample to apply frequency modulation 
    let final_sample = time::samples_from_dur(xyz.cps, dur_seconds);
    if xyz.i > final_sample {
        return 1.0f32
    }

    let glide_mix = (final_sample - xyz.i) as f32 / final_sample as f32;
    let glide_rate_cycles = xyz.cps / 0.25;
    let j = time::samples_from_dur(xyz.cps, 0.5) as f32;
    let p = (xyz.i as f32%j)/j;
    let x = glide_rate_cycles * p;

    let min_factor = 2f32.powf(-0.66f32/12f32);
    let max_factor = 2f32.powf(0.33f32/12f32);

    let min_f = -1f32;
    let max_f = 1f32;
    let mul_glide:f32 = map_range_lin(min_f, max_f, min_factor, max_factor, x.sin());
    glide_mix * mul_glide * match &snd.energy {
        timbre::Energy::Low => {
            0.001f32
        },
        timbre::Energy::Medium => {
            0.01f32
        },
        timbre::Energy::High => {   
            0.02f32
        }
    };
    // DISABLING frequency modulation; don't need it right now and it is hard to control
    1f32
}

/// Generate a monic amplitude modulation curve by Presence and Energy
fn amod(xyz:&Coords, ctx:&Ctx, snd:&Sound, dir:&Direction, phr:&Phrasing) -> Range {
    let dur_scale_factor = match &snd.presence {
        timbre::Presence::Staccatto => {
            0.1f32
        },
        timbre::Presence::Legato => {
            0.8f32
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

    // @art-choice this model leaves headroom between each level 
    let min_max_db = match &snd.energy {
        timbre::Energy::Low => {
            (-50f32, -40f32)
        },
        timbre::Energy::Medium => {
            (-40f32, -30f32)
        },
        timbre::Energy::High => {   
            (-20f32, -10f32)
        }
    };

    // @art-choice amplitude scaling based on the monic
    // @art-curr uses linearly fading monics with gain when under threshold, else exponentially fading 
    let amp_k:f32 = match &snd.energy {
        timbre::Energy::Low => {
            let k = xyz.k as f32;
            if xyz.k > 7 { 
                let mul = 1.0;
                mul * 1f32/(k *k) - (2f32*k)
            } else {
                let mul = 0.75;
                mul * 1.0/k
            }
        },
        timbre::Energy::Medium => {
            let k = xyz.k as f32;
            if xyz.k > 15 { 
                let mul = 1.0;
                mul * 1f32/(k *k) - (2f32*k)
            } else {
                let mul:f32  = 1.0;
                (mul * 1.0/k).max(1.0)
            }
        },
        timbre::Energy::High => {   
            let k = xyz.k as f32;
            if xyz.k > 23 { 
                let mul = 1.0;
                mul * 1f32/(k *k) - (2f32*k)
            } else {
                let mul:f32  = 1.33;
                (mul * 1.0/k).max(1.0)
            }
        }
    };
    let dDecibel = (min_max_db.1 - min_max_db.0)/final_sample as f32;
    let decibel = min_max_db.0 + dDecibel * xyz.i as f32;
    amp_k * db_to_amp(decibel)
}

/// returns a value in [-pi, pi]
fn pmod(xyz:&Coords, ctx:&Ctx, snd:&Sound, dir:&Direction, phr:&Phrasing) -> f32 {
    let dur_seconds = match snd.presence {
        timbre::Presence::Staccatto => {
            ctx.dur_seconds * 0.2f32
        },
        timbre::Presence::Legato => {
            ctx.dur_seconds * 0.33
        },
        timbre::Presence::Tenuto => {
            ctx.dur_seconds * 0.9
        }
    };

    // last sample to apply phase modulation 
    let final_sample = time::samples_from_dur(xyz.cps, dur_seconds);
    if xyz.i > final_sample {
        return 0.0f32
    }

    //@art-choice Select a modulation mix function 
    //@art-curr linear fade out for vibrato, linear fade in for noise
    let vibrato_mix = (final_sample - xyz.i) as f32 / final_sample as f32;
    let noise_mix = xyz.i as f32 / final_sample as f32;
    let vibrato_rate_cycles = xyz.cps / 0.66;

    let mut rng = rand::thread_rng();
    let j = time::samples_from_dur(xyz.cps, 0.5) as f32;
    let p = (xyz.i as f32%j)/j;
    let x = vibrato_rate_cycles * p;

    let add_vibrato:f32 = map_range_lin(-1f32, 1f32, -1f32*pi, pi, x.sin());
    let add_noise = match &snd.energy {
        timbre::Energy::Low => {
            0f32
        },
        timbre::Energy::Medium => {
            0.05 * rng.gen::<f32>() * pi
        },
        timbre::Energy::High => {   
            0.2 * rng.gen::<f32>() * pi
        }
    };
    vibrato_mix * add_vibrato + noise_mix * add_noise
}

pub fn gen(cps:f32, note:&Note)->Modulators {
    let (dur, tone, ampl) = note;
    Modulators {
        amp:amod,
        freq:fmod,
        phase:pmod
    }
}

