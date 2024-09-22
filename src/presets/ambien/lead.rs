use std::os::unix::thread;

use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};

pub fn expr(arf:&Arf) -> Expr {
    (vec![0f32], vec![1f32], vec![0f32])
}

fn amp_knob_experiement(visibility:Visibility, energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();

    let detune_rate = match energy {
        Energy::Low => rng.gen::<f32>()/6f32,
        Energy::Medium => rng.gen::<f32>()/3f32,
        Energy::High => rng.gen::<f32>(),
    };
    let detune_mix = match visibility {
        Visibility::Visible => 0.33 + 0.47 * rng.gen::<f32>(),
        Visibility::Foreground => 0.2 + 0.13 * rng.gen::<f32>(),
        Visibility::Background => 0.1 * 0.1 * rng.gen::<f32>(),
        Visibility::Hidden => 0.05f32 * rng.gen::<f32>(),
    };

    return (Knob { a: detune_rate, b: detune_mix, c: 0.0 }, ranger::amod_detune);
}


fn amp_knob(visibility:Visibility, energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();
    if let Presence::Legato = presence {
        // let osc_rate = match energy {
        //     Energy::Low => 0.25f32,
        //     Energy::Medium => 0.5f32,
        //     Energy::High => 1f32,
        // };
        let osc_rate = match energy {
            Energy::Low => rng.gen::<f32>()/3f32,
            Energy::Medium => 0.42 + rng.gen::<f32>()/4f32,
            Energy::High => 0.66f32 + 0.33 * rng.gen::<f32>(),
        };
    
        let intensity = match visibility {
            Visibility::Visible => 0.33 + 0.47 * rng.gen::<f32>(),
            Visibility::Foreground => 0.2 + 0.13 * rng.gen::<f32>(),
            Visibility::Background => 0.1 * 0.1 * rng.gen::<f32>(),
            Visibility::Hidden => 0.05f32 * rng.gen::<f32>(),
        };

        return (Knob { a: osc_rate, b: intensity, c: 0.0 }, ranger::amod_oscillation_tri);
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
    // knob_mods.0.push(amp_knob(arf.visibility, Energy::Low, Presence::Legato));
    knob_mods.0.push(amp_knob(arf.visibility, arf.energy, arf.presence));
    knob_mods.0.push(amp_knob_experiement(arf.visibility, arf.energy, arf.presence));
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