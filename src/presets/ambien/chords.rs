use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};

// @art-choice This module would benefit from dynamic selection of knob params
// from the given VEP parameters

fn amp_knob_overs(a:f32) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    (Knob { a: 0f32, b: 1f32, c: 0f32 }, ranger::amod_lfo_sine)
}

fn amp_knob_unders(a:f32) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    (Knob { a: 0f32, b: 1f32, c: 0f32 }, ranger::amod_lfo_sine)
}

pub fn expr_overs(arf:&Arf) -> Expr {
    (vec![1f32], vec![1f32], vec![0f32])
}

pub fn expr_unders(arf:&Arf) -> Expr {
    (vec![1f32], vec![1f32], vec![0f32])
}


/// Defines the constituent stems to create a simple closed hat drum
/// Components include:
///  - an impulse of staccato undertone voicing
///  - a pluck of pink overs 
pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {

    let soids_unders1 = druidic_soids::overs_triangle(2f32.powi(8i32)); 
    let modifiers_unders1:ModifiersHolder = (vec![], vec![], vec![], vec![]);
    let feel_unders1:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: modifiers_unders1,
        clippers: (0f32, 1f32)
    };
    
    let mut knob_mods_overs:KnobMods = KnobMods::unit();
    knob_mods_overs.0.push(amp_knob_overs(0f32));

    let stem_overs = (melody, soids_unders1, expr_overs(arf), feel_unders1, knob_mods_overs, vec![delay::passthrough]);

    //# melodic component

    let soids_unders = druidic_soids::under_square(2f32.powi(8i32)); 
    let modifiers_unders:ModifiersHolder = (vec![], vec![], vec![], vec![]);
    let feel_unders:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: modifiers_unders,
        clippers: (0f32, 1f32)
    };

    let mut knob_mods_unders:KnobMods = KnobMods::unit();
    knob_mods_unders.0.push(amp_knob_unders(1f32));
    let stem_unders = (melody, soids_unders, expr_unders(arf), feel_unders, knob_mods_unders, vec![delay::passthrough]);

    Renderable::Group(vec![
        stem_overs,
        // stem_unders
    ])
}
