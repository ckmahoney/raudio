use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};

// @art-choice This module would benefit from dynamic selection of knob params
// from the given VEP parameters

fn amp_knob_noise() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let sustain = 0f32;
    let decay_mode = 0.5f32;
    let time_scaling_enabled = 0f32;

    (Knob { a: sustain, b: decay_mode, c: time_scaling_enabled }, ranger::amod_microtransient)

}

fn amp_knob_tonal() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let sustain = 0.5f32;

    (Knob { a: sustain, b: 0.0f32, c: 0.0}, ranger::amod_breath)
}


fn freq_knob_tonal(v:Visibility, e:Energy, p:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();
    let modulation_amount = match e {
        Energy::Low => 0.005f32 + 0.003 * rng.gen::<f32>(),
        Energy::Medium => 0.008f32 + 0.012f32 * rng.gen::<f32>(),
        Energy::High => 0.1f32 + 0.2f32 * rng.gen::<f32>()
    };
    (Knob { a: modulation_amount, b: 0f32, c: 0.0}, ranger::fmod_warble)
}

fn pmod_knob_tonal(v:Visibility, e:Energy, p:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();

    let modulation_depth:f32 = match v {
        Visibility::Hidden => 0.33f32,
        Visibility::Background => 0.5,
        Visibility::Foreground => 0.75,
        Visibility::Visible => 1f32,
    };

    let chorus_visibility:f32 = match v {
        Visibility::Hidden => 0f32,
        Visibility::Background => 0.1f32 + 0.5f32 * rng.gen::<f32>(),
        Visibility::Foreground => 0.6f32 + 0.2f32 * rng.gen::<f32>(),
        Visibility::Visible => 0.8f32 + 0.1f32 * rng.gen::<f32>(),
    };

    (Knob { a: modulation_depth, b: chorus_visibility, c: 0.0}, ranger::pmod_chorus)
}

pub fn expr_noise(arf:&Arf) -> Expr {
    (vec![db_to_amp(-60f32)], vec![1f32], vec![0f32])
}

pub fn expr_tonal(arf:&Arf) -> Expr {
    (vec![1f32], vec![1f32], vec![0f32])
}


/// Defines the constituent stems to create a simple closed hat drum
/// Components include:
///  - an impulse of staccato undertone voicing
///  - a pluck of pink noise 
pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {

    //# microtransient that indicates the note has started
    let soids_noise = druidic_soids::noise(1024f32, druidic_soids::NoiseType::Pink);
    let modifiers_noise:ModifiersHolder = (vec![], vec![], vec![], vec![]);
    let feel_noise:Feel = Feel {
        bp: (vec![4000f32], vec![6000f32]),
        modifiers: modifiers_noise,
        clippers: (0f32, 1f32)
    };
    
    let mut knob_mods_noise:KnobMods = KnobMods::unit();
    knob_mods_noise.0.push(amp_knob_noise());

    // knob_mods_noise.0.push(amp_knob_noise2());
    let stem_noise = (melody, soids_noise, expr_noise(arf), feel_noise, knob_mods_noise, vec![delay::passthrough]);

    //# melodic component

    let soids_tonal = druidic_soids::under_square(2f32.powi(10i32)); 
    let modifiers_tonal:ModifiersHolder = (vec![], vec![], vec![], vec![]);
    let feel_tonal:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: modifiers_tonal,
        clippers: (0f32, 1f32)
    };

    let mut knob_mods_tonal:KnobMods = KnobMods::unit();
    knob_mods_tonal.0.push(amp_knob_tonal());
    // knob_mods_tonal.1.push(freq_knob_tonal(arf.visibility, arf.energy, arf.presence));
    knob_mods_tonal.2.push(pmod_knob_tonal(arf.visibility, arf.energy, arf.presence));
    let stem_tonal = (melody, soids_tonal, expr_tonal(arf), feel_tonal, knob_mods_tonal, vec![delay::passthrough]);

    Renderable::Group(vec![
        stem_noise,
        stem_tonal
    ])
}
