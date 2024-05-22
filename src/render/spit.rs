use crate::synth::{SR, pi2, pi, SampleBuffer};
use crate::types::synthesis::{Bp,Range, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::types::timbre::{BandpassFilter, Energy, Presence, BaseOsc, Sound, FilterMode, Timeframe, Phrasing, Ampex};
use crate::types::render::{Span};

use crate::phrasing::contour::{Expr, Position};

pub enum GlideLen {
    None,
    Quarter,
    Eigth,
    Sixteenth
}

/// Context window for a frequency. 
/// Second, Third, and Fourth entries describe the frequencies being navigated.
/// Middle entry is the current frequency to perform.
/// The first and final f32 are the previous/next frequency.
/// First and final entries describe how to glide
///
/// If a C Major chord is spelled as C, E, G and we wanted to arpeggiate the notes,
/// then an analogous FreqSeq looks like (GlideLen::None, None, C, E, GlideLen::None)
/// and then for the second note, (GlideLen::None, C, E, G, GlideLen::None)
pub type FreqSeq = (GlideLen, Freq, Freq, Freq, GlideLen);



pub fn ugen(
    expr: Expr,
    frex: FreqSeq,
    bp: Bp,
    span: Span,
    position: Position,
    d: Range
) -> SampleBuffer {
    vec![0f32]
}