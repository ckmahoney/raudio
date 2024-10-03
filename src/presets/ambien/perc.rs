use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};


pub fn expr(arf:&Arf) -> Expr {
    (vec![db_to_amp(-10f32)], vec![1f32], vec![0f32])
}

pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
    let modders:ModifiersHolder = (
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let mut knob_mods:KnobMods = KnobMods::unit();
    let feel:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: modders,
        clippers: (0f32, 1f32)
    };

    let soids = druidic_soids::id();

    let stem = (melody, soids, expr(arf), feel, knob_mods, vec![delay::passthrough]);
    Renderable::Instance(stem)
}