use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::ranger::{KnobMods};
use crate::druid::{self, soids as druidic_soids};

pub fn driad(arf:&Arf) -> Ely {
    let soids:Soids = druidic_soids::octave(8f32);
    let modders:ModifiersHolder = (
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let mut knob_mods:KnobMods = KnobMods::unit();
    Ely::new(soids, modders, knob_mods)
}