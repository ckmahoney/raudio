use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};

/// Selects a short lived impulse for the pink noise component of a closed hi hat
fn amp_knob_tonal() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let decay_rate = 0.1f32;
    (Knob { a: decay_rate, b: 0.0, c: 0.0}, ranger::amod_impulse)
}

pub fn expr_tonal(arf:&Arf) -> Expr {
    (vec![0f32], vec![1f32], vec![0f32])
}


/// Defines the constituent stems to create a simple closed hat drum
/// Components include:
///  - an impulse of staccato undertone voicing
///  - a pluck of pink noise 
pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
    
    //# noise component
    
    let soids_noise = druidic_soids::noise(1024f32, druidic_soids::NoiseType::Pink);
    let modifiers_noise:ModifiersHolder = (vec![], vec![], vec![], vec![]);
    let feel_noise:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: modifiers_noise,
        clippers: (0f32, 1f32)
    };
    
    let mut knob_mods_noise:KnobMods = KnobMods::unit();
    knob_mods_noise.0.push(amp_knob_noise());
    let stem_noise = (melody, soids_noise, expr_noise(arf), feel_noise, knob_mods_noise, vec![delay::passthrough]);

    //# tonal component

    let soids_tonal = druidic_soids::under_square(2f32.powi(10i32));
    let modifiers_tonal:ModifiersHolder = (vec![], vec![], vec![], vec![]);
    let feel_tonal:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: modifiers_tonal,
        clippers: (0f32, 1f32)
    };

    let mut knob_mods_tonal:KnobMods = KnobMods::unit();
    knob_mods_tonal.0.push(amp_knob_tonal());
    let stem_tonal = (melody, soids_tonal, expr_tonal(arf), feel_tonal, knob_mods_tonal, vec![delay::passthrough]);

    Renderable::Group(vec![
        stem_noise,
    ])
}
