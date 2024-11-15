use super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids, soid_fx, noise::NoiseColor};

/// Softens the overall amplitude
pub fn expr_id(arf:&Arf) -> Expr {
    (vec![db_to_amp(-10f32)], vec![1f32], vec![0f32])
}

fn knob_amp() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    (Knob { a: 1f32, b: 1f32, c: 0f32 }, ranger::amod_pluck)
}

/// Defines the constituent stems to create a complex kick drum
/// Components include:
///  - a transient id element
pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
    
    //# id component
    let soids_id = druidic_soids::id();
    let modifiers_id:ModifiersHolder = (
        vec![],
        vec![],
        vec![],
        vec![],
    );

    let feel_id:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: modifiers_id,
        clippers: (0f32, 1f32)
    };
    
    let mut knob_mods:KnobMods = KnobMods::unit();
    // knob_mods.0.push(knob_amp());

    let soids = soid_fx::detune::reece(&druidic_soids::id(), 12, 0.25f32);
    let soids = soid_fx::map(&soids, 3, vec![
        (soid_fx::fmod::triangle, 0.11f32),
    ]);

    let soids = soid_fx::map(&soids, 3, vec![
        (soid_fx::fmod::sawtooth, 0.05f32),
    ]);
    let stem_id = (melody, soids, expr_id(arf), feel_id, knob_mods, vec![delay::passthrough]);

    Renderable::Group(vec![
        stem_id,
    ])
}