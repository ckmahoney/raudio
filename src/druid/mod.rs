/// Synthesizers
/// aka Druid
/// 
/// All synths contain four elementary components:
/// Melodic, Enharmonic, Bell, and Noise
/// 
/// Synthesizers are likely to feature one or two components primarly 
/// and attenuate or deactivate the remaining components.
/// 
/// Wild synths may feature all components, shifting form from one to the next!
/// 
/// This model should be able to provide 95% of the sounds we want to use in music :)

mod melodic;
mod enharmonic;
mod bell;
mod noise;

use crate::phrasing::ranger::{Weight, Modders};
use crate::phrasing::contour::Expr;
use crate::types::synthesis::{Range,Freq,Bp,Muls};
use crate::types::timbre::{Mode, Contrib};
use crate::types::render::{Span};
use crate::render::blend::Frex;

/// # Element
/// 
/// `muls` A list of multipliers for a fundamental frequency. Minimum length 1.
/// `modders` Gentime dynamic modulation. A major contributor for defining this Element's sound.
/// `expr` Carrier signal for amplitude, frequency, and phase for a Note. For example, a pluck has an amplitude envelope rapidly falling and a none phase.
pub struct Element {
    mode:Mode,
    muls: Muls,
    modders:Modders,
    expr:Expr,
    hplp: Bp,
    thresh: (Range, Range)
}

/// # Druid
/// 
/// A collection of weighted contributors for a syntehsizer.
/// 
/// Weights of all elements must equal 1. 
pub type Druid = Vec<(Weight, Element)>;

pub struct Apply {
    span:Span,
    frex:Frex
}

// needs an expr, bandpass, multipliers, modders, and noise_clip

pub fn create(contrib:&Contrib) {

}