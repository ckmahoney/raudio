use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};

/// Provides slight pitch bend 
pub fn expr_sub(arf:&Arf) -> Expr {
    (vec![0f32], vec![3.2f32, 2.1f32, 1.0f32], vec![0f32])
}

fn amp_knob_subsine(energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    if let Presence::Legato = presence {
        let osc_rate = match energy {
            Energy::Low => 0.25f32,
            Energy::Medium => 0.5f32,
            Energy::High => 1f32,
        };
        return (Knob { a: osc_rate, b: 0.0, c: 0.0 }, ranger::amod_oscillation_tri);
    }
    let sustain = match presence {
        Presence::Staccatto => 0.0f32,
        Presence::Legato => 0.33f32,
        Presence::Tenuto => 0.66f32
    };
    let monic_stretch = match energy {
        Energy::Low => 0.33f32,
        Energy::Medium => 0.75f32,
        Energy::High => 1f32,
    };
    (Knob { a: sustain, b: monic_stretch, c: 0.0}, ranger::amod_burp)
}



/// Defines the constituent stems to create a complex kick drum
/// Components include:
///  - a sustained subsine
pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
    
    //# subsine component

    let soids_subsine = druidic_soids::octave(64f32);
    let modifiers_subsine:ModifiersHolder = (
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let feel_subsine:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: modifiers_subsine,
        clippers: (0f32, 1f32)
    };
    let mut knob_mods_subsine:KnobMods = KnobMods::unit();
    knob_mods_subsine.0.push(amp_knob_subsine(arf.energy, arf.presence));

    let stem_subsine = (melody, soids_subsine, expr_sub(arf), feel_subsine, knob_mods_subsine, vec![delay::passthrough]);

    Renderable::Group(vec![
        stem_subsine
    ])
}