use crate::preset;
use crate::types::synthesis::{Freq, Note, Direction};

use crate::envelope::db_to_amp;
use crate::time;
use rand;
use rand::Rng;


pub fn gen(cps:f32, note:&Note)-> preset::Modulators {
    let (dur, tone, ampl) = note;
    preset::Modulators {
        amp:preset::pluck::amod_one_over_x,
        freq:preset::drone::fmod,
        phase:preset::none::pmod,
    }
}

pub fn gen_from(cps:f32, note:&Note, mbs: &preset::SomeModulators)-> preset::Modulators {
    let amod = match mbs.amp {
        Some(amod) => {
            amod
        },
        None => {
            preset::none::amod
        }
    };
    let fmod = match mbs.freq {
        Some(fmod) => {
            fmod
        },
        None => {
            preset::none::fmod
        }
    };
    let pmod = match mbs.phase {
        Some(pmod) => {
            pmod
        },
        None => {
            preset::none::pmod
        }
    };
    
    preset::Modulators {
        amp: amod,
        freq: fmod,
        phase: pmod,
    }
}

