use std::os::unix::thread;

use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};

fn amp_knob(visibility:Visibility, energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();
    let sustain = match presence {
        Presence::Staccatto => in_range(&mut rng, 0.1f32, 0.2f32),
        Presence::Legato => in_range(&mut rng, 0.25f32, 0.5f32),
        Presence::Tenuto => in_range(&mut rng, 0.44f32, 0.66f32),
    };
    let decay_rate = match energy {
        Energy::Low => in_range(&mut rng, 0.33f32,0.55f32),
        Energy::Medium => in_range(&mut rng, 0.44f32, 0.77f32),
        Energy::High => in_range(&mut rng, 0.77f32, 1f32),
    };
    (Knob { a: sustain, b: decay_rate, c: 0.0}, ranger::amod_pluck)
}


pub fn expr(arf:&Arf) -> Expr {
    (vec![visibility_gain(arf.visibility) * visibility_gain(Visibility::Background)], vec![1f32], vec![0f32])
}

pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
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
    knob_mods.0.push(amp_knob(arf.visibility, arf.energy, arf.presence));

    let mut rng = thread_rng();
    let soids = soid_fx::concat(&vec![
        soid_fx::noise::resof(rng.gen::<f32>() +2f32),
        soid_fx::noise::resof(rng.gen::<f32>() +2f32),
        soid_fx::noise::resof(rng.gen::<f32>() +5f32),
    ]);

    let stem = (melody, soids, expr(arf), feel, knob_mods, vec![delay::passthrough]);
    Renderable::Instance(stem)
}