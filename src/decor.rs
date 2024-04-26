use crate::preset;
use crate::types::synthesis::{Freq, Note, Direction};

use crate::envelope::db_to_amp;
use crate::time;
use rand;
use rand::Rng;


pub fn gen(cps:f32, note:&Note)-> preset::Modulators {
    let (dur, tone, ampl) = note;
    preset::Modulators {
        amp:preset::plain::amod,
        freq:preset::plain::fmod,
        phase:preset::plain::pmod,
    }
}

