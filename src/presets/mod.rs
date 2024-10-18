pub mod basic;
pub mod hard;
pub mod ambien;
pub mod bird;
pub mod hop;
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
use crate::synth::{MFf, NFf, SampleBuffer, pi, pi2};
use crate::phrasing::older_ranger::{Modders,OldRangerDeprecated,WOldRangerDeprecateds};
use crate::phrasing::lifespan;
use crate::phrasing::micro;
use crate::{render, AmpLifespan};
use crate::analysis::{trig,volume::db_to_amp};

use crate::types::render::{Feel, Melody, Stem};
use crate::types::synthesis::{Freq, Note, Direction, Ely, PhaseModParams, ModulationEffect};
use crate::types::timbre::{Arf, Role, Mode, Visibility, Sound, Sound2, Energy, Presence, Phrasing};
use crate::types::{Range, Radian};
use crate::druid::{Element, Elementor, melodic, bell, noise};
use crate::phrasing::contour::expr_none;
use crate::phrasing::ranger::{self, Knob,KnobMods};
use crate::render::Renderable;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::contour::Expr;
use crate::druid::{self, soids as druidic_soids, soid_fx, noise::NoiseColor};
use rand::thread_rng;



use crate::analysis::delay::DelayParams;


pub struct Armoir;
impl Armoir {
    pub fn select_melodic(energy:Energy) -> Dressor {
        match energy {
            Energy::Low => melodic::dress_triangle as fn(f32) -> Dressing,
            Energy::Medium => melodic::dress_square as fn(f32) -> Dressing,
            Energy::High => melodic::dress_sawtooth as fn(f32) -> Dressing 
        }
    }
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
    pub fn unit<'render>(melody:&'render Melody<Note>, energy:Energy, delays:Vec<DelayParams>) -> Stem<'render> {
        let dressor = Armoir::select_melodic(energy);
        let dressing:Dressing = dressor(crate::synth::MFf);

        // overly verbose code to demonstrate the pattern 
        let dressing_as_vecs = vec![(dressing.amplitudes, dressing.multipliers, dressing.offsets)];
        let tmp = trig::prepare_soids_input(dressing_as_vecs);
        let soids = trig::process_soids(tmp);
        (
            melody,
            soids,
            (vec![1f32],vec![1f32],vec![0f32]),
            Feel::unit(),
            KnobMods::unit(),
            delays
        )
    }

    pub fn select<'render>(melody:&'render Melody<Note>, arf:&Arf, delays:Vec<DelayParams>) -> Renderable<'render> {
        use Role::*;
        use crate::synth::MFf;
        use crate::phrasing::ranger::KnobMods;
        use crate::presets::hard::*;
        use crate::presets::basic::*;
        
        let renderable = if true {
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
            match arf.role {
                Kick => bird::kick::renderable(melody, arf),
                Perc => bird::perc::renderable(melody, arf),
                Hats => bird::hats::renderable(melody, arf),
                Lead => bird::lead::renderable(melody, arf),
                Bass => bird::bass::renderable(melody, arf),
                Chords => bird::chords::renderable(melody, arf),
            }
        } else {
            match arf.role {
                Kick => urbuntu::kick::renderable(melody, arf),
                Perc => urbuntu::perc::renderable(melody, arf),
                Hats => urbuntu::hats::renderable(melody, arf),
                Lead => urbuntu::lead::renderable(melody, arf),
                Bass => urbuntu::bass::renderable(melody, arf),
                Chords => urbuntu::chords::renderable(melody, arf),
            }
        };

        println!("Overwriting delays from preset with those specified in score");
        match renderable {
            Renderable::Instance(mut stem) => {
                stem.5 = delays;
                Renderable::Instance(stem)
            },
            Renderable::Group(mut stems) => {
                for stem in &mut stems {
                    stem.5 = delays.clone()
                };
                Renderable::Group(stems)
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
        Visibility::Hidden => db_to_amp(-50f32),
        Visibility::Background => db_to_amp(-35f32),
        Visibility::Foreground => db_to_amp(-20f32),
        Visibility::Visible => db_to_amp(-5f32)
    }
}

pub fn in_range(rng:&mut ThreadRng, min:f32,max:f32) ->  f32{ 
    min + (max - min) * rng.gen::<f32>()
}