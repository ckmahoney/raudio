pub mod basic;
pub mod hard;
pub mod ambien;
pub mod hop;
pub mod bird;
pub mod kuwuku;
pub mod smooth;
pub mod urbuntu;


/// Shared values for all presets in this mod
static contour_resolution:usize = 1200;
static unit_decay:f32 = 9.210340372; 

use rand;
use rand::{rngs::ThreadRng,Rng,prelude::SliceRandom};

/// Shared imports for all presets in this mod
use crate::analysis::delay;
use crate::synth::{MIN_REGISTER,MAX_REGISTER, MFf, NFf, SampleBuffer, pi, pi2, SR, SRf};
use crate::phrasing::older_ranger::{Modders,OldRangerDeprecated,WOldRangerDeprecateds};
use crate::phrasing::{micro,lifespan, dynamics};
use crate::{render, AmpLifespan};
use crate::analysis::{trig,volume::db_to_amp, in_range, melody::{mask_wah,mask_sigh,find_reach,ODR, ODRMacro, Levels,LevelMacro}};

use crate::time;
use crate::types::render::{Feel, Melody, Stem, Stem2};
use crate::types::synthesis::{Bp2,BoostGroup,Freq, Note, Direction, Ely, PhaseModParams, ModulationEffect};
use crate::types::timbre::{Arf, Role, Mode, Visibility, Sound, Sound2, Energy, Presence, Phrasing};
use crate::types::{Range, Radian};
use crate::druid::{Element, Elementor, melodic, bell, noise};
use crate::phrasing::contour::expr_none;
use crate::phrasing::ranger::{self, Knob,KnobMods};
use crate::render::{Renderable,Renderable2,};
use crate::types::synthesis::{BoostGroupMacro,ModifiersHolder,Soids};
use crate::phrasing::contour::Expr;
use crate::druid::{self, soids as druidic_soids, soid_fx, noise::NoiseColor};
use rand::thread_rng;

// user configurable headroom value. defaults to -15Db
pub const DB_HEADROOM:f32 = -15f32;


use crate::analysis::delay::DelayParams;


pub fn amp_microtransient(visibility:Visibility, energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    (Knob { a: 0.45f32, b: 0f32, c: 1.0}, ranger::amod_microbreath_4_20)
}

pub fn get_bp<'render>(cps:f32,  mel:&'render Melody<Note>, arf:&Arf, len_cycles:f32) -> Bp2 {
    match arf.presence {
        Presence::Staccatto => bp_wah(cps, mel, arf, len_cycles),
        Presence::Legato => bp_sighpad(cps, mel, arf, len_cycles),
        Presence::Tenuto => bp_cresc(cps, mel, arf, len_cycles),
    }
}

/// Create bandpass automations with respsect to Arf and Melody
fn bp_cresc<'render>(cps:f32, mel:&'render Melody<Note>, arf:&Arf, len_cycles:f32) -> Bp2 {
    let size = (len_cycles.log2()-1f32).max(1f32); // offset 1 to account for lack of CPC. -1 assumes CPC=2
    let rate_per_size = match arf.energy {
        Energy::Low => 0.5f32,
        Energy::Medium => 1f32,
        Energy::High => 2f32,
    };
    let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);
    let n_samples:usize = ((len_cycles/2f32) as usize).max(1) * SR;

    let (highpass, lowpass):(Vec<f32>, Vec<f32>) = if let Visibility::Visible = arf.visibility {
        match arf.energy {
            Energy::Low => (filter_contour_triangle_shape_highpass(lowest_register, highest_register, n_samples, size*rate_per_size), vec![NFf]),
            _ => (vec![MFf],filter_contour_triangle_shape_lowpass(lowest_register, n_samples, size*rate_per_size))
        }
    } else {
        (vec![MFf], vec![NFf])
    };

    let levels = Levels::new(0.7f32, 4f32, 0.5f32);
    let odr = ODR {
        onset: 60.0,
        decay: 1330.0,    
        release: 110.0,  
    };

    (highpass, lowpass, vec![])
}

/// Create bandpass automations with respsect to Arf and Melody
fn bp_wah<'render>(cps:f32, mel:&'render Melody<Note>, arf:&Arf, len_cycles:f32) -> Bp2  {
    let size = (len_cycles.log2()-1f32).max(1f32); // offset 1 to account for lack of CPC. -1 assumes CPC=2
    let rate_per_size = match arf.energy {
        Energy::Low => 0.5f32,
        Energy::Medium => 1f32,
        Energy::High => 2f32,
    };
    let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);
    let n_samples:usize = ((len_cycles/2f32) as usize).max(1) * SR;

    let levels = Levels::new(0.7f32, 4f32, 0.5f32);
   

    let level_macro:LevelMacro = LevelMacro {
        stable: [1f32, 2f32],
        peak: [3.5f32, 4f32],
        sustain: [0.2f32, 0.4f32],
    };
    
    let odr_macro = ODRMacro {
        onset: [60.0, 120f32],
        decay: [230.0, 300f32],    
        release: [110.0, 200f32],  
    };
    let highpass = if let Energy::Low = arf.energy { 
        filter_contour_triangle_shape_highpass(lowest_register, highest_register, n_samples, size*rate_per_size)
    } else { vec![MFf]  };
    (
        highpass,
        mask_wah(cps, &mel[low_index], &level_macro, &odr_macro),
        vec![]
    )
}

/// Create bandpass automations with respsect to Arf and Melody
fn bp_sighpad<'render>(cps:f32, mel:&'render Melody<Note>, arf:&Arf, len_cycles:f32) -> Bp2 {
    let size = (len_cycles.log2()-1f32).max(1f32); // offset 1 to account for lack of CPC. -1 assumes CPC=2
    let rate_per_size = match arf.energy {
        Energy::Low => 0.5f32,
        Energy::Medium => 1f32,
        Energy::High => 2f32,
    };
    let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);
    let n_samples:usize = ((len_cycles/2f32) as usize).max(1) * SR;
    let levels = Levels::new(0.7f32, 4f32, 0.5f32);
    let level_macro:LevelMacro = LevelMacro {
        stable: [1f32, 1f32],
        peak: [2.25f32, 4f32],
        sustain: [0.4f32, 0.8f32],
    };
    
    let odr_macro = ODRMacro {
        onset: [1260.0, 2120f32],
        decay: [2330.0, 8500f32],    
        release: [1510.0, 2000f32],  
    };


    let highpass = if let Energy::Low = arf.energy { 
        filter_contour_triangle_shape_highpass(lowest_register, highest_register, n_samples, size*rate_per_size)
    } else { vec![MFf] };
    (
        highpass,
        mask_sigh(cps, &mel[low_index], &level_macro, &odr_macro),
        vec![]
    )
}


pub fn get_boost_macros(arf:&Arf) -> Vec<BoostGroupMacro> {

    let gen = || -> BoostGroupMacro {
        let base:i32 = arf.register as i32;
        let bandwidth:(f32, f32) = match arf.visibility {
            Visibility::Hidden => (0.1, 0.2),
            Visibility::Background => (0.2, 0.3),
            Visibility::Visible => (0.3, 0.4),
            Visibility::Foreground => (0.4, 0.6),
        };
        BoostGroupMacro {
            bandpass: [2f32.powi(base), 2f32.powi(base+1i32)],
            bandwidth: [bandwidth.0, bandwidth.1],
            att: [8f32, 12f32],
            rolloff: [21f32, 2.3f32],
            q: [1f32, 2f32]
        }
    };

    match arf.energy {
        // allpass filter 
        Energy::High => vec![],
        // suppress some energy
        Energy::Medium => vec![gen() ],
        // suppress a lot of energy 
        Energy::Low => vec![gen(), gen()],
    }
}

/// Generate a phrase-length filter contour with a triangle shape, oscillating `k` times per phrase.
/// Peaks `k` times within the phrase and tapers back down to `start_cap` at the end.
pub fn filter_contour_triangle_shape_lowpass<'render>(
    lowest_register: i8,
    n_samples: usize,
    k: f32
) -> SampleBuffer {
    let mut highpass_contour: SampleBuffer = vec![MFf; n_samples];
    let mut lowpass_contour: SampleBuffer = Vec::with_capacity(n_samples);

    let start_cap: f32 = 2.1f32;
    let final_cap: f32 = MAX_REGISTER as f32 - lowest_register as f32 - start_cap;

    let min_f: f32 = 2f32.powf(lowest_register as f32 + start_cap);
    let max_f: f32 = 2f32.powf(lowest_register as f32 + start_cap + final_cap);
    let n: f32 = n_samples as f32;
    let df: f32 = (max_f - min_f).log2();

    for i in 0..n_samples {
        let x: f32 = i as f32 / n;

        // Modulate the frequency of oscillation using k
        let x_adjusted = (k * x).fract();
        let triangle_wave = if x_adjusted <= 0.5 {
            2.0 * x_adjusted
        } else {
            2.0 * (1.0 - x_adjusted)
        };

        // Calculate the lowpass frequency based on the triangle wave
        lowpass_contour.push(min_f + 2f32.powf(df * triangle_wave));
    }

    lowpass_contour
}


/// Generate a phrase-length filter contour with a triangle shape, oscillating `k` times per phrase.
/// Peaks `k` times within the phrase and tapers back down to `start_cap` at the end.
pub fn filter_contour_triangle_shape_highpass<'render>(
    lowest_register: i8,
    highest_register: i8,
    n_samples: usize,
    k: f32
) -> SampleBuffer {
    let mut highpass_contour: SampleBuffer = Vec::with_capacity(n_samples);

    let start_cap: f32 = (3.0f32).min(MAX_REGISTER as f32 - highest_register as f32);
    let final_cap: f32 = MAX_REGISTER as f32 - highest_register as f32 - start_cap;

    let min_f: f32 = 2f32.powf(lowest_register as f32);
    let max_f: f32 = 2f32.powf(highest_register as f32 + start_cap);
    let n: f32 = n_samples as f32;
    let df: f32 = (max_f - min_f).log2();

    for i in 0..n_samples {
        let x: f32 = i as f32 / n;

        let x_adjusted = (k * x).fract();
        let triangle_wave = if x_adjusted <= 0.5 {
            2.0 * x_adjusted
        } else {
            2.0 * (1.0 - x_adjusted)
        };

        // Calculate the lowpass frequency based on the triangle wave
        highpass_contour.push(max_f - 2f32.powf(df * triangle_wave));
    }

    // highpass_contour;
    vec![MFf]
}

#[derive(Debug)]
pub struct Dressing {
    pub len: usize,
    pub multipliers: Vec<f32>,
    pub amplitudes: Vec<f32>,
    pub offsets: Vec<f32>,
}
pub type Dressor = fn (f32) -> Dressing;

pub struct Instrument;

impl Instrument {
    

    pub fn select<'render>(cps:f32, melody:&'render Melody<Note>, arf:&Arf, delays:Vec<DelayParams>) -> Renderable2<'render> {
        use Role::*;
        use crate::synth::MFf;
        use crate::phrasing::ranger::KnobMods;
        use crate::presets::hard::*;
        use crate::presets::basic::*;
        
        let renderable = match arf.role {
            Kick => hop::kick::renderable(cps, melody, arf),
            Perc => hop::perc::renderable(cps, melody, arf),
            Hats => hop::hats::renderable(cps, melody, arf),
            Lead => hop::lead::renderable(cps, melody, arf),
            Bass => hop::bass::renderable(cps, melody, arf),
            Chords => hop::chords::renderable(cps, melody, arf),
        };
    
        // match arf.role {
        //     Kick => kuwuku::kick::renderable(melody, arf),
        //     Perc => kuwuku::perc::renderable(melody, arf),
        //     Hats => kuwuku::hats::renderable(melody, arf),
        //     Lead => urbuntu::lead::renderable(melody, arf),
        //     Bass => kuwuku::bass::renderable(melody, arf),
        //     Chords => kuwuku::chords::renderable(melody, arf),
        // };

        // match arf.role {
        //     Kick => ambien::kick::renderable(melody, arf),
        //     Perc => ambien::perc::renderable(melody, arf),
        //     Hats => ambien::hats::renderable(melody, arf),
        //     Lead => ambien::lead::renderable(melody, arf),
        //     Bass => ambien::bass::renderable(melody, arf),
        //     Chords => ambien::chords::renderable(melody, arf),
        // }
        // match arf.role {
        //     Kick => bird::kick::renderable(melody, arf),
        //     Perc => bird::perc::renderable(melody, arf),
        //     Hats => bird::hats::renderable(melody, arf),
        //     Lead => bird::lead::renderable(melody, arf),
        //     Bass => bird::bass::renderable(melody, arf),
        //     Chords => bird::chords::renderable(melody, arf),
        // }

        // match arf.role {
        //     Kick => urbuntu::kick::renderable(melody, arf),
        //     Perc => urbuntu::perc::renderable(melody, arf),
        //     Hats => urbuntu::hats::renderable(melody, arf),
        //     Lead => urbuntu::lead::renderable(melody, arf),
        //     Bass => urbuntu::bass::renderable(melody, arf),
        //     Chords => urbuntu::chords::renderable(melody, arf),
        // }
            
            
        match renderable {
            Renderable2::Instance(mut stem) => {
                stem.5 = delays;
                Renderable2::Instance(stem)
            },
            Renderable2::Group(mut stems) => {
                for stem in &mut stems {
                    stem.5 = delays.clone()
                };
                Renderable2::Group(stems)
            }
        }
    }
}

fn select_expr(arf:&Arf) -> Expr {
    let mut rng  = thread_rng();

    use AmpLifespan::{self,*};
    use Role::{self,*};
    let plucky_lifespans:Vec<AmpLifespan> = vec![Pluck, Snap, Burst];
    let sussy_lifespans:Vec<AmpLifespan> = vec![Spring, Bloom, Pad, Drone];

    let lifespan = match arf.role {
        Kick | Perc | Hats => plucky_lifespans.choose(&mut rng).unwrap(),
        Lead | Chords | Bass => match arf.presence {
            Presence::Legato => sussy_lifespans.choose(&mut rng).unwrap(),
            Presence::Staccatto => plucky_lifespans.choose(&mut rng).unwrap(),
            Presence::Tenuto => {
                if rng.gen_bool(0.33) {
                    plucky_lifespans.choose(&mut rng).unwrap()
                } else {
                    sussy_lifespans.choose(&mut rng).unwrap()
                }
            },
        }
    };


    let amp_contour: Vec<f32> = crate::phrasing::lifespan::sample_lifespan(crate::synth::SR, lifespan, 1, 1f32);
    (amp_contour, vec![1f32], vec![0f32])
}

/// DEPRECATED the methods below have been replaced by the ranger module, 
/// which offers a better interface for dynamic dispatch (using Knob).


/// Microtansient methods probaly should go in micro
pub fn microtransient_chiff(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let (amps, muls, phss) = micro::set_micro(fund, energy);
    Element {
        mode: Mode::Noise,
        amps,
        muls,
        phss,
        modders: micro::modders_chiff(),
        expr: expr_none(),
        hplp: (vec![MFf], vec![NFf]),
        thresh: (0f32, 1f32)
    }
}

pub fn microtransient_click(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let (amps, muls, phss) = micro::set_micro(fund, energy);
    Element {
        mode: Mode::Noise,
        amps,
        muls,
        phss,
        modders: micro::modders_chiff(),
        expr: expr_none(),
        hplp: (vec![MFf], vec![NFf]),
        thresh: (0f32, 1f32)
    }
}

pub fn microtransient_pop(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let (amps, muls, phss) = micro::set_micro(fund, energy);
    Element {
        mode: Mode::Noise,
        amps,
        muls,
        phss,
        modders: micro::modders_chiff(),
        expr: expr_none(),
        hplp: (vec![MFf], vec![NFf]),
        thresh: (0f32, 1f32)
    }
}



/// Four octave freq sweep, responsive to monic and duration. 
/// Requires that the input multipliers are truncated by log_2(max_sweep_mul) octaves
/// https://www.desmos.com/calculator/fbzd5wwj2e
static max_sweep_reg:f32 = 4f32;
static min_sweep_reg:f32 = 1f32;
pub fn fmod_sweep(k:usize, x:f32, d:f32) -> f32 {
    let kf = k as f32;
    let growth_const = -unit_decay;
    let sweep_reg:f32 = max_sweep_reg - 1f32;
    2f32.powf(sweep_reg) * (kf*growth_const*x).exp()
}


// values in 25-50 look good. @art-choice could mod in this range
static amod_const:f32 = 50f32;
fn amod_exit(x:f32) -> f32 {
    let y:f32 = (amod_const * x - pi).tanh();
    0.5f32 * (1f32 - y)
}

///A brief one-valued signal with tanh decay to 0.
pub fn amod_impulse(k:usize, x:f32, d:f32) -> f32 {
    let y:f32 = -1f32 + (1f32/(1f32-(-x).exp()));
    (0.5f32*y).tanh() * amod_exit(x)
}

pub fn visibility_gain(v:Visibility) -> f32 {
    match v {
        Visibility::Hidden => db_to_amp(-18f32),
        Visibility::Background => db_to_amp(-12f32),
        Visibility::Foreground => db_to_amp(-9f32),
        Visibility::Visible => db_to_amp(-6f32)
    }
}
