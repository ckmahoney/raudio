use std::os::unix::thread;

use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};

pub fn expr(arf:&Arf) -> Expr {
    (vec![1f32, 0.15, 0.9, 0.05, 0.9, 0.5f32, 0.5f32, 0.33f32, 0.1f32, 0f32], vec![1f32], vec![0f32]);
    (vec![db_to_amp(-30f32)], vec![1f32], vec![0f32])
}

fn amp_knob_breath(visibility:Visibility, energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();

    let breath_rate = match energy {
        Energy::Low => rng.gen::<f32>(),
        Energy::Medium => rng.gen::<f32>()/2f32,
        Energy::High => rng.gen::<f32>()/3f32,
    };

    return (Knob { a: breath_rate, b: 0f32, c: 0.0 }, ranger::amod_breath);
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


pub fn driad(arf:&Arf) -> Ely {
    let mullet = match arf.energy {
        Energy::Low => 256f32,
        Energy::Medium => 64f32,
        Energy::High => 32f32,
    };
    let soids = match arf.visibility {
        Visibility::Hidden => druidic_soids::octave(mullet),
        Visibility::Background => druidic_soids::overs_triangle(mullet),
        Visibility::Foreground => druidic_soids::overs_square(mullet),
        Visibility::Visible => druidic_soids::overs_sawtooth(mullet),
    };
    let modders:ModifiersHolder = (
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let mut knob_mods:KnobMods = KnobMods::unit();
    knob_mods.0.push(amp_knob_experiement(arf.visibility, arf.energy, arf.presence));
    knob_mods.0.push(amp_knob_breath(arf.visibility, arf.energy, arf.presence));
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