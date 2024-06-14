pub mod kick;
pub mod snare;
pub mod hats;
pub mod bass;
pub mod chords;
pub mod lead;

static contour_resolution:usize = 1200;
use rand;
use rand::Rng;

use crate::synth::{MFf, NFf, SampleBuffer, pi2};
use crate::phrasing::ranger::{Modders,Ranger,Cocktail};
use crate::phrasing::lifespan;
use crate::phrasing::micro;
use crate::timbre::AmpLifespan;

use crate::types::synthesis::{Freq, Note, Direction};
use crate::types::timbre::{Mode,  Visibility, Sound, Sound2, Energy, Presence, Phrasing};
use crate::types::{Range, Radian};
use crate::druid::{Element, Elementor, melodic, bell, noise};
use crate::phrasing::contour::expr_none;

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