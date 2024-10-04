use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids, soid_fx};

/// Softens the overall amplitude
pub fn expr_id(arf:&Arf) -> Expr {
    (vec![db_to_amp(-10f32)], vec![1f32], vec![0f32])
}

fn knob_amp() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    (Knob { a: 0.11f32, b: 1f32, c: 0f32 }, ranger::amod_pluck)
}

fn stem_visible<'render>(arf:&Arf, melody:&'render Melody<Note>) -> Stem<'render> {
    let soids = druidic_soids::id();
    let expr = (vec![visibility_gain(Visibility::Background)], vec![1f32], vec![0f32]);

    let feel:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: (
            vec![],
            vec![],
            vec![],
            vec![],
        ),
        clippers: (0f32, 1f32)
    };
    let mut knob_mods:KnobMods = KnobMods::unit();
    knob_mods.0.push((
        Knob {
            a: 0f32,
            b: 0.9f32,
            c:0f32
        },
        ranger::amod_pluck
    ));
    let noises:Vec<Soids> = (0..10).map(|register| soid_fx::noise::rank(register, NoiseColor::Violet, 0.1f32)).collect();
    let soids = soid_fx::concat(&noises);

    (melody, soids, expr, feel, knob_mods, vec![delay::passthrough])
}

fn stem_foreground<'render>(arf:&Arf, melody:&'render Melody<Note>) -> Stem<'render> {
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
    knob_mods.0.push((
        Knob {
            a: 1f32,
            b: 0.1f32,
            c:0f32
        },
        ranger::amod_pluck
    ));
    let expr = (vec![visibility_gain(Visibility::Foreground)], vec![1f32], vec![0f32]);
    let soids = soid_fx::detune::reece(&druidic_soids::id(), 12, 0.5f32);
    (melody, soids, expr, feel_id, knob_mods, vec![delay::passthrough])
}

/// Defines the constituent stems to create a complex kick drum
/// Components include:
///  - a transient id element
pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
    Renderable::Group(vec![
        stem_visible(arf, melody),
        stem_foreground(arf, melody),
    ])
}