/// Focal point of this module, the presets for your score 
pub mod kick;
pub mod kick_hard;
pub mod snare;
pub mod snare_hard;
pub mod hats;
pub mod hats_hard;
pub mod bass;
pub mod bass_smoother;
pub mod chords;
pub mod chords_smoother;
pub mod lead;
pub mod lead_smoother;


/// Shared imports for all presets in this mod
static contour_resolution:usize = 1200;
static unit_decay:f32 = 9.210340372; 
use rand;
use rand::Rng;

use crate::synth::{MFf, NFf, SampleBuffer, pi, pi2};
use crate::phrasing::older_ranger::{Modders,OldRangerDeprecated,WOldRangerDeprecateds};
use crate::phrasing::lifespan;
use crate::phrasing::micro;     
use crate::timbre::AmpLifespan;
use crate::analysis::trig;

use crate::types::synthesis::{Freq, Note, Direction, Ely};
use crate::types::timbre::{Arf, Role, Mode,  Visibility, Sound, Sound2, Energy, Presence, Phrasing};
use crate::types::{Range, Radian};
use crate::druid::{Element, Elementor, melodic, bell, noise};
use crate::phrasing::contour::expr_none;
use crate::phrasing::ranger::{self, Knob,KnobMods};

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
        Role::Kick => kick_hard::synth,
        Role::Perc => snare_hard::synth,
        Role::Hats => hats_hard::synth,
        Role::Bass => bass_smoother::synth,
        Role::Chords => chords_smoother::synth,
        Role::Lead => lead_smoother::synth,
    }
}

