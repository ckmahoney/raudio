use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};

fn amp_knob(visibility:Visibility, energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let sustain = match presence {
        Presence::Staccatto => 0f32,
        Presence::Legato => 0.1f32,
        Presence::Tenuto => 0.3f32
    };
    let decay_rate = match energy {
        Energy::Low => 0.5f32,
        Energy::Medium => 0.75f32,
        Energy::High => 1f32,
    };
    (Knob { a: sustain, b: decay_rate, c: 0.0}, ranger::amod_pluck)
}

pub fn expr(arf:&Arf) -> Expr {
    (vec![1f32, 0.15, 0.9, 0.05, 0.9, 0.5f32, 0.5f32, 0.33f32, 0.1f32, 0f32], vec![1f32], vec![0f32]);
    (vec![1f32], vec![1f32], vec![0f32])
}

/// Selects a color of noise at -4 height
fn driad(arf:&Arf) -> Ely {
    let noise_type = match arf.energy {
        Energy::Low => druidic_soids::NoiseType::Violet,
        Energy::Medium => druidic_soids::NoiseType::Equal,
        Energy::High => druidic_soids::NoiseType::Pink,
    };
    let soids = druidic_soids::noise(16f32, noise_type);
    let modders:ModifiersHolder = (
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let mut knob_mods:KnobMods = KnobMods::unit();
    knob_mods.0.push(amp_knob(arf.visibility, Energy::High, Presence::Legato));
    Ely::new(soids, modders, knob_mods)
}

pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
    let ely = driad(arf);
    let feel:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: ely.modders,
        clippers: (0f32, 1f32)
    };


    let stem = (melody, ely.soids, expr(arf), feel, ely.knob_mods, vec![delay::passthrough]);
    Renderable::Instance(stem)
}