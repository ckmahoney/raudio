use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids, soid_fx, noise::NoiseColor};


/// noise component
pub fn stem_visible<'render>(cps:f32, arf:&Arf, melody:&'render Melody<Note>) -> Stem2<'render> {
    let expr:Expr = (vec![db_to_amp(-4.5f32)*visibility_gain(Visibility::Foreground)], vec![1f32], vec![0f32]);

    let soids = soid_fx::concat(&vec![
        soid_fx::noise::rank(0, NoiseColor::Pink, 1f32/3f32),
        soid_fx::noise::rank(2, NoiseColor::Violet, 1f32/11f32),
        soid_fx::noise::rank(3, NoiseColor::Violet, 1f32/9f32),
    ]);


    let mut knob_mods:KnobMods = KnobMods::unit();
    let mut rng = thread_rng();
    
    // principal layer
    knob_mods.0.push((
        Knob {
            a: match arf.presence {
                Presence::Staccatto => in_range(&mut rng, 0f32, 0.5f32),
                Presence::Legato => in_range(&mut rng, 0.33f32, 0.66f32),
                Presence::Tenuto => in_range(&mut rng, 0.95f32, 1f32),
            },
            b: match arf.energy {
                Energy::High => 0f32,
                Energy::Medium => in_range(&mut rng, 0.1f32, 0.2f32),
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
                // takes pluck for staccato/legato
                Presence::Staccatto => in_range(&mut rng, 0.22f32, 0.33f32),
                Presence::Legato => in_range(&mut rng, 0.33f32, 0.75f32),
                // takes burp for tenuto
                Presence::Tenuto => in_range(&mut rng, 0.95f32, 1f32),
            },
            b: match arf.energy {
                Energy::High => in_range(&mut rng, 0f32, 0.2f32),
                Energy::Medium => in_range(&mut rng, 0.2f32, 0.3f32),
                Energy::Low => in_range(&mut rng, 0.3f32, 0.5f32),
            },
            c:0f32
        },
        if let Presence::Tenuto = arf.presence { ranger::amod_burp } else { ranger::amod_pluck }
    ));
    let len_cycles:f32 = time::count_cycles(&melody[0]);
    (melody, soids, expr, get_bp(cps, melody, arf, len_cycles), knob_mods, vec![delay::passthrough])
}


/// tonal component
pub fn stem_tonal<'render>(cps:f32, arf:&Arf, melody:&'render Melody<Note>) -> Stem2<'render> {
    

    let soids_base = soid_fx::concat(&vec![
        druidic_soids::under_sawtooth(2f32.powi(11i32)),
        druidic_soids::overs_square(2f32.powi(11i32)),
    ]);
    let soids = soid_fx::concat(&vec![
        soid_fx::ratio::constant(&soids_base, 0.8f32, 0.15f32),
        soid_fx::ratio::constant(&soids_base, 0.666f32, 0.25f32),
    ]);

    let expr:Expr = (vec![visibility_gain(Visibility::Hidden)], vec![1f32], vec![0f32]);

    let mut knob_mods:KnobMods = KnobMods::unit();
    let mut rng = thread_rng();
    knob_mods.0.push((Knob {a: rng.gen::<f32>(), b: rng.gen::<f32>()/5f32, c:0f32 }, ranger::amod_pluck));
    (melody, soids, expr, (vec![MFf], vec![NFf], vec![]), knob_mods, vec![delay::passthrough])
}

/// Defines the constituent stems to create a simple closed hat drum
/// Components include:
///  - a pluck of pink noise 
pub fn renderable<'render>(cps:f32, melody:&'render Melody<Note>, arf:&Arf) -> Renderable2<'render> {

    Renderable2::Group(vec![
        stem_visible(cps, arf, melody),
        stem_tonal(cps,arf, melody),
    ])
}
