use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};


pub fn expr_noise(arf:&Arf) -> Expr {
    (vec![db_to_amp(-15f32)], vec![1f32], vec![0f32])
}

pub fn expr_tonal(arf:&Arf) -> Expr {
    (vec![db_to_amp(-15f32)], vec![1f32], vec![0f32])
}


// @art-choice This module would benefit from dynamic selection of knob params
// from the given VEP parameters

/// Selects a short lived impulse for the pink noise component of a closed hi hat
fn amp_knob_noise(visibility:Visibility, energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let sustain = 0.1f32;
    let mut rng = thread_rng();
    let decay_rate = match energy {
        Energy::Low => 0.8 + 0.2f32 * rng.gen::<f32>(),
        Energy::Medium => 0.3 + 0.4f32 * rng.gen::<f32>(),
        Energy::High => 0.05 + 0.2f32 * rng.gen::<f32>(),
    };
    let env_length = match presence {
        Presence::Tenuto => 0.66f32 + 0.2f32 * rng.gen::<f32>(),
        Presence::Legato => 0.2f32 + 0.3f32 * rng.gen::<f32>(),
        Presence::Staccatto => 0.2f32 * rng.gen::<f32>(),
    };
    

    (Knob { a: env_length, b: decay_rate, c: 0.0}, ranger::amod_pluck)
}

/// Selects a short lived impulse for the pink noise component of a closed hi hat
fn amp_knob_tonal() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();
    let decay_rate = 0.2f32 * rng.gen::<f32>();
    (Knob { a: decay_rate, b: 0.0, c: 0.0}, ranger::amod_impulse)
}


/// Defines the constituent stems to create a simple closed hat drum
/// Components include:
///  - a pluck of pink noise 
pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {

    let soids_tonal = druidic_soids::id();
    let modifiers_tonal:ModifiersHolder = (vec![], vec![], vec![], vec![]);
    let feel_tonal:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: modifiers_tonal,
        clippers: (0f32, 1f32)
    };

    let mut knob_mods_tonal:KnobMods = KnobMods::unit();
    let stem_tonal = (melody, soids_tonal, expr_tonal(arf), feel_tonal, knob_mods_tonal, vec![delay::passthrough]);

    Renderable::Group(vec![
        stem_tonal
    ])
}
