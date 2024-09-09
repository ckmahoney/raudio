use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};

pub fn expr(arf:&Arf) -> Expr {
    (vec![1f32, 0.15, 0.9, 0.05, 0.9, 0.5f32, 0.5f32, 0.33f32, 0.1f32, 0f32], vec![1f32], vec![0f32]);
    (vec![1f32], vec![1f32], vec![0f32])
}

fn amp_knob(energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    if let Presence::Legato = presence {
        let osc_rate = match energy {
            Energy::Low => 0.25f32,
            Energy::Medium => 0.5f32,
            Energy::High => 1f32,
        };
        return (Knob { a: osc_rate, b: 0.0, c: 0.0 }, ranger::amod_oscillation_tri);
    }

    let sustain = match presence {
        Presence::Staccatto => 0f32,
        Presence::Legato => 0.66f32,
        Presence::Tenuto => 1f32
    };

    let decay_rate = match energy {
        Energy::Low => 0.2f32,
        Energy::Medium => 0.5f32,
        Energy::High => 0.9f32,
    };
    
    (Knob { a: sustain, b: decay_rate, c: 0.0}, ranger::amod_pluck)
}

pub fn driad(arf:&Arf) -> Ely {
    let soids:Soids = match arf.energy {
        Energy::Low => druidic_soids::octave(16f32),
        Energy::Medium => druidic_soids::octave(8f32),
        Energy::High => druidic_soids::octave(4f32),
    };
    let modders:ModifiersHolder = (
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let mut knob_mods:KnobMods = KnobMods::unit();
    knob_mods.0.push(amp_knob(Energy::High, Presence::Legato));
    knob_mods.0.push(amp_knob(Energy::Medium, Presence::Tenuto));
    Ely::new(soids, modders, knob_mods)
}

pub fn stem<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
    let ely = driad(arf);
    let feel:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: ely.modders,
        clippers: (0f32, 1f32)
    };

    let stem = (melody, ely.soids, expr(arf), feel, ely.knob_mods, vec![delay::passthrough]);
    Renderable::Instance(stem)
}