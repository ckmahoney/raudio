use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids, soid_fx};

pub fn expr(arf:&Arf) -> Expr {
    (vec![visibility_gain(arf.visibility)], vec![1f32], vec![0f32])
}



fn amp_knob_hidden(visibility:Visibility, energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();
    let osc_rate = match energy {
        Energy::Low => 0.1 * rng.gen::<f32>(),
        Energy::Medium => 0.33f32/4f32 + 0.2 * rng.gen::<f32>(),
        Energy::High => 0.33f32 + 0.4 * rng.gen::<f32>(),
    };

    let intensity = match visibility {
        Visibility::Visible => 0.33 + 0.47 * rng.gen::<f32>(),
        Visibility::Foreground => 0.2 + 0.13 * rng.gen::<f32>(),
        Visibility::Background => 0.1 * 0.1 * rng.gen::<f32>(),
        Visibility::Hidden => 0.05f32 * rng.gen::<f32>(),
    };
    return (Knob { a: osc_rate, b: intensity, c: 0.0 }, ranger::amod_slowest)
}

fn amp_knob_principal(rng:&mut ThreadRng, arf:&Arf)  -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    return (
        Knob { 
            a: match arf.presence {
                Presence::Staccatto => in_range(rng, 0f32, 0.5f32),
                Presence::Legato => in_range(rng, 0.33f32, 0.88f32),
                Presence::Tenuto => in_range(rng, 0.88f32, 1f32),
            },
            b: match arf.energy {
                Energy::High => in_range(rng, 0f32, 0.2f32),
                Energy::Medium => in_range(rng, 0.2f32, 0.3f32),
                Energy::Low => in_range(rng, 0.3f32, 0.5f32),
            },
            c: 0.0 
        },  
        ranger::amod_burp
    )
}

fn amp_knob_attenuation(rng:&mut ThreadRng, arf:&Arf)  -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    return (
        Knob { 
            a: match arf.energy {
                Energy::High => in_range(rng, 0.5f32, 0.8f32),
                Energy::Medium => in_range(rng, 0.3f32, 0.4f32),
                Energy::Low => in_range(rng, 0.0f32, 0.31f32),
            },
            b: match arf.visibility {
                Visibility::Visible => in_range(rng, 0.8f32, 1f32),
                Visibility::Foreground => in_range(rng, 0.5f32, 0.8f32),
                _ => in_range(rng, 0f32, 0.3f32),
            },
            c: 0.0 
        }, 
        ranger::amod_detune
    )
}



pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
    let mut rng = thread_rng();
    //# id component

    let feel:Feel = Feel {
        bp: (vec![MFf], vec![NFf/32f32]),
        modifiers: (
            vec![],
            vec![],
            vec![],
            vec![],
        ),
        clippers: (0f32, 1f32)
    };
    
    let mut knob_mods:KnobMods = KnobMods::unit();
    knob_mods.0.push(amp_knob_principal(&mut rng, &arf));
    knob_mods.0.push(amp_knob_attenuation(&mut rng, &arf));
    let soids = druidic_soids::id();

    let soids = soid_fx::map(&soids, 1, vec![
        (soid_fx::fmod::square, 0.33f32),
    ]);

    let soids = soid_fx::amod::reece(&soids, 2);
    let stem = (melody, soids, expr(arf), feel, knob_mods, vec![delay::passthrough]);

    Renderable::Group(vec![
        stem,
    ])
}