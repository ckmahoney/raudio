use std::os::unix::thread;

use super::super::*;

fn knob_amp() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    (Knob { a: 1f32, b: 1f32, c: 0f32 }, ranger::amod_pluck)
}

/// Featured component 
pub fn stem_visible<'render>(arf:&Arf, melody:&'render Melody<Note>) -> Stem<'render> {
    let mut rng = thread_rng();
    let peak1:f32 = 1f32 + rng.gen::<f32>();
    let peak2:f32 = 2f32 + rng.gen::<f32>();
    let peak3:f32 = (peak1.sqrt() + peak2.sqrt()).powi(2i32);

    let mut compound_ratios:Vec<(f32, f32)> = vec![
        (peak1, 0.6f32),
        (peak2, 0.9f32),
        (peak3, 0.3f32),
    ];

    let soids:Soids = soid_fx::concat(&vec![
        compound_ratios.iter().fold(druidic_soids::id(), |soids, (k, gain)| soid_fx::ratio::constant(&soids, *k, *gain)),
        soid_fx::noise::rank(0, NoiseColor::Violet, 1f32/7f32),
        soid_fx::noise::rank(1, NoiseColor::Pink, 1f32/11f32),
    ]);
    let soids:Soids = soid_fx::fmod::sawtooth(&soids, 5);
    let soids:Soids = soid_fx::fmod::square(&soids,3);
    // let soids:Soids = soid_fx::fmod::square(&soids,3);
    
    let expr = (vec![visibility_gain(arf.visibility) * visibility_gain(Visibility::Foreground)], vec![1f32], vec![0f32]);
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
    knob_mods.0.push((
        Knob {
            a: 0.05f32,
            b: 1f32,
            c:0f32
        },
        ranger::amod_pluck
    ));
    knob_mods.0.push((
        Knob {
            a: 0.5f32,
            b: 1f32,
            c:0f32
        },
        ranger::amod_pluck
    ));
    knob_mods.0.push((
        Knob {
            a: 0.15f32,
            b: 1f32,
            c:0f32
        },
        ranger::amod_pluck
    ));
    (melody, soids, expr, feel, knob_mods, vec![delay::passthrough])
}

/// Supporting feature
pub fn stem_foreground<'render>(arf:&Arf, melody:&'render Melody<Note>) -> Stem<'render> {
    let soids = soid_fx::concat(&vec![
        soid_fx::noise::rank(1, NoiseColor::Pink, 1f32/5f32),
        soid_fx::noise::rank(2, NoiseColor::Equal, 1f32/9f32),
        soid_fx::noise::rank(3, NoiseColor::Violet, 1f32/3f32),
    ]);
    let expr = (vec![visibility_gain(Visibility::Background)], vec![1f32], vec![0f32]);
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
    let mut rng:ThreadRng = thread_rng();
    // principal layer
    knob_mods.0.push((
        Knob {
            a: 0.2f32,
            b: 0.3f32,
            c: 0f32
        },
        ranger::amod_pluck
    ));
    
    // attenuation layer
    knob_mods.0.push((
        Knob {
            a: match arf.presence {
                Presence::Staccatto => in_range(&mut rng, 0f32, 0.33f32),
                Presence::Legato => in_range(&mut rng, 0.33f32, 0.66f32),
                Presence::Tenuto => in_range(&mut rng, 0.88f32, 1f32),
            },
            b: match arf.energy {
                Energy::High => in_range(&mut rng, 0f32, 0.33f32),
                Energy::Medium => in_range(&mut rng, 0.33f32, 0.5f32),
                Energy::Low => in_range(&mut rng, 0.5f32, 0.66f32),
            },
            c:0f32
        },
        ranger::amod_pluck
    ));
    (melody, soids, expr, feel, knob_mods, vec![delay::passthrough])
}

/// Secondary component
pub fn stem_background() {

}

/// Background component
pub fn stem_hidden<'render>(arf:&Arf, melody:&'render Melody<Note>) -> Stem<'render> {
    let soids = druidic_soids::octave(500f32);
    let expr = (vec![visibility_gain(Visibility::Hidden)], vec![1f32], vec![0f32]);
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
    // knob_mods.0.push((
    //     Knob {
    //         a: 0.3f32,
    //         b: 0.8f32,
    //         c: 0f32
    //     },
    //     ranger::amod_pluck
    // ));
    let mut rng = thread_rng();
    (melody, soids, expr, feel, knob_mods, vec![delay::passthrough])
}

/// Defines the constituent stems to create a complex kick drum
/// Components include:
///  - a transient id element
pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
    
    Renderable::Group(vec![
        stem_visible(&arf, &melody),
        stem_foreground(&arf, &melody),
        stem_hidden(&arf, &melody)
    ])
}