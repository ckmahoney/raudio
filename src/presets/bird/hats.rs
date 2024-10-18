use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids, soid_fx, noise::NoiseColor};

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
fn amp_knob() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();
    let decay_rate = 0.2f32 * rng.gen::<f32>();
    (Knob { a: decay_rate, b: 0.0, c: 0.0}, ranger::amod_impulse)
}

/// noise component
pub fn stem_visible<'render>(arf:&Arf, melody:&'render Melody<Note>) -> Stem<'render> {
    let feel:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: (vec![], vec![], vec![], vec![]),
        clippers: (0f32, 1f32)
    };

    let soids = soid_fx::concat(&vec![
        soid_fx::noise::rank(0, NoiseColor::Pink, 1f32/3f32),
        soid_fx::noise::rank(1, NoiseColor::Equal, 1f32/5f32),
        soid_fx::noise::rank(2, NoiseColor::Violet, 1f32/11f32),
        soid_fx::noise::rank(3, NoiseColor::Violet, 1f32/9f32),
    ]);

    let expr:Expr = (vec![db_to_amp(-15f32)], vec![1f32], vec![0f32]);

    let mut knob_mods:KnobMods = KnobMods::unit();
    let mut rng = thread_rng();
    // principal layer
    knob_mods.0.push((
        Knob {
            a: 0f32,
            b: match arf.energy {
                Energy::High => in_range(&mut rng, 0f32, 0.1f32),
                Energy::Medium => in_range(&mut rng, 0.1f32, 0.3f32),
                Energy::Low => in_range(&mut rng, 0.2f32, 0.4f32),
            },
            c: 0f32
        },
        ranger::amod_pluck
    ));
    
    // attenuation layer
    knob_mods.0.push((
        Knob {
            a: match arf.presence {
                Presence::Staccatto => in_range(&mut rng, 0f32, 0.15f32),
                Presence::Legato => in_range(&mut rng, 0.23f32, 0.77f32),
                Presence::Tenuto => in_range(&mut rng, 0.88f32, 1f32),
            },
            b: match arf.energy {
                Energy::High => in_range(&mut rng, 0f32, 0.2f32),
                Energy::Medium => in_range(&mut rng, 0.2f32, 0.3f32),
                Energy::Low => in_range(&mut rng, 0.3f32, 0.5f32),
            },
            c:0f32
        },
        ranger::amod_pluck
    ));
    (melody, soids, expr, feel, knob_mods, vec![delay::passthrough])
}


/// tonal component
pub fn stem_foreground<'render>(arf:&Arf, melody:&'render Melody<Note>) -> Stem<'render> {
    let feel:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: (vec![], vec![], vec![], vec![]),
        clippers: (0f32, 1f32)
    };

    let soids_base = soid_fx::concat(&vec![
        druidic_soids::under_sawtooth(2f32.powi(11i32)),
        druidic_soids::overs_square(2f32.powi(11i32)),
    ]);
    let soids = soid_fx::concat(&vec![
        soid_fx::ratio::constant(&soids_base, 0.8f32, 0.15f32),
        soid_fx::ratio::constant(&soids_base, 0.666f32, 0.25f32),
    ]);

    let expr:Expr = (vec![db_to_amp(-15f32)], vec![1f32], vec![0f32]);

    let mut knob_mods:KnobMods = KnobMods::unit();
    let mut rng = thread_rng();
    knob_mods.0.push((Knob {a: rng.gen::<f32>(), b: rng.gen::<f32>()/5f32, c:0f32 }, ranger::amod_pluck));
    (melody, soids, expr, feel, knob_mods, vec![delay::passthrough])
}

/// Defines the constituent stems to create a simple closed hat drum
/// Components include:
///  - a pluck of pink noise 
pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {

    Renderable::Group(vec![
        stem_visible(arf, melody),
        stem_foreground(arf, melody),
    ])
}
