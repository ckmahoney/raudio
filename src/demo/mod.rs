mod ambien;
mod beat;
mod chill_beat;
mod effects;
mod kuwuku;
mod trio;
mod urbuntu;
mod vagrant;
use crate::{presets, render};

use crate::phrasing::contour::Expr;
// static out_dir:&str = "audio/demo";
static out_dir:&str = "/media/naltroc/engraver 2/music-gen/demo";
use crate::types::timbre::{Visibility, Mode, Role, Arf, FilterMode, Sound, Sound2, Energy, Presence, Timeframe, Phrasing,AmpLifespan, AmpContour};
use crate::types::synthesis::{Ely, Soids, Ampl,Frex, GlideLen, Register, Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::analysis::delay::{self, DelayParams};
use crate::synth::{MFf,NFf, SR};
use crate::render::Renderable;

pub fn location(name:&str) -> String {
    format!("{}/{}", out_dir, name)
}

/// given a length, duration, ampltidue, and space selection, 
/// create a note in the register.
fn test_note(duration:Duration, register:i8, amp:f32, overs:bool) -> Note {
    let monic:i8 = 1;
    let rotation:i8 = 0;
    
    let q:i8 = if overs { 0 } else { 1 };
    let monic = 1;
    let monae:Monae = (rotation,q, monic);
    (duration, (register, monae), amp)
}

/// helper for making a test line of specific length with arbitrary pitch.
pub fn make_line(durations:Vec<Duration>, registers:Vec<i8>, amps:Vec<Ampl>, muls:bool) -> Vec<Note> {
    let len = durations.len();
    if len != registers.len() || len != amps.len() {
        panic!("Must provide the same number of components per arfutor. Got actual lengths for duration {} register {} amp {}", len, registers.len(), amps.len());
    }

    let mut line:Vec<Note> = Vec::with_capacity(len);
    for (i, duration) in durations.iter().enumerate() {
        let register = registers[i];
        let amp = amps[i];
        line.push(test_note(*duration, register, amp, muls))
    }
    line
}

pub fn zip_line(tala:Vec<Duration>, tones:Vec<Tone>, amps:Vec<Ampl>) -> Vec<Note> {
    let len = tala.len();
    if len != amps.len() || tones.len() != len {
        panic!("Must provide the same number of components per contributor. Got actual lengths for durations {}, amps {} and tones {}", len, amps.len(), tones.len());
    }

    let mut line:Vec<Note> = Vec::with_capacity(len);
    for (i, step) in tala.iter().enumerate() {
        line.push((*step, tones[i], amps[i]))
    }
    line
}