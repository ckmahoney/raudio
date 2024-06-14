/// Focal point of this module, the presets for your score 
pub mod kick;
pub mod snare;
pub mod hats;
pub mod bass;
pub mod chords;
pub mod lead;


/// Shared imports for all presets in this mod
static contour_resolution:usize = 1200;
use rand;
use rand::Rng;

use crate::synth::{MFf, NFf, SampleBuffer, pi2};
use crate::phrasing::ranger::{Modders,Ranger,Cocktail};
use crate::phrasing::lifespan;
use crate::phrasing::micro;     
use crate::timbre::AmpLifespan;

use crate::types::synthesis::{Freq, Note, Direction};
use crate::types::timbre::{Arf, Role, Mode,  Visibility, Sound, Sound2, Energy, Presence, Phrasing};
use crate::types::{Range, Radian};
use crate::druid::{Element, Elementor, melodic, bell, noise};
use crate::phrasing::contour::expr_none;


pub type SynthGen = fn (arf:&Arf) -> Elementor;

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

pub fn select(arf:&Arf) -> SynthGen {
    match arf.role {
        Role::Kick => kick::synth,
        Role::Perc => snare::synth,
        Role::Hats => hats::synth,
        Role::Bass => bass::synth,
        Role::Chords => chords::synth,
        Role::Lead => lead::synth,
    }
}
