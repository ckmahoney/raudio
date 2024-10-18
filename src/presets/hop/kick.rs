use super::*;


fn knob_amp() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    (Knob { a: 1f32, b: 1f32, c: 0f32 }, ranger::amod_pluck)
}


pub fn stem_resonant_nodes<'render>(arf:&Arf, melody:&'render Melody<Note>) -> Stem<'render> {
    let mut rng = thread_rng();
    let peak1:f32 = 4f32 + rng.gen::<f32>();
    let peak2:f32 = 6f32 + rng.gen::<f32>();
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
    
    
    let expr = (vec![visibility_gain(Visibility::Foreground)], vec![1f32], vec![0f32]);
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
            a: match arf.presence {
                Presence::Staccatto => in_range(&mut rng, 0f32, 0.33f32),
                Presence::Legato => in_range(&mut rng, 0.33f32, 0.66f32),
                Presence::Tenuto => in_range(&mut rng, 0.88f32, 1f32),
            },
            b: 1f32,
            c:0f32
        },
        ranger::amod_pluck
    ));
    knob_mods.0.push((
        Knob {
            a: match arf.presence {
                Presence::Staccatto => in_range(&mut rng, 0f32, 0.33f32),
                Presence::Legato => in_range(&mut rng, 0.33f32, 0.66f32),
                Presence::Tenuto => in_range(&mut rng, 0.88f32, 1f32),
            },
            b: match arf.energy {
                Energy::High => in_range(&mut rng, 0.25f32, 0.5f32),
                Energy::Medium => in_range(&mut rng, 0.5f32, 0.75f32),
                Energy::Low => in_range(&mut rng, 0.85f32, 1f32),
            },
            c:0f32
        },
        ranger::amod_pluck
    ));
    (melody, soids, expr, feel, knob_mods, vec![delay::passthrough])
}

/// Supporting feature
pub fn stem_noise<'render>(arf:&Arf, melody:&'render Melody<Note>) -> Stem<'render> {
    let soids = soid_fx::concat(&vec![
        soid_fx::noise::rank(3, NoiseColor::Pink, 1f32/5f32),
        soid_fx::noise::rank(4, NoiseColor::Equal, 1f32/9f32),
        soid_fx::noise::rank(5, NoiseColor::Violet, 1f32/3f32),
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
    
    knob_mods.0.push((
        Knob {
            a: match arf.presence {
                Presence::Staccatto => in_range(&mut rng, 0.1f32, 0.33f32),
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


pub fn stem_bass<'render>(arf:&Arf, melody:&'render Melody<Note>) -> Stem<'render> {
    let soids = druidic_soids::octave(2f32.powi(10i32));

    let expr = (vec![visibility_gain(Visibility::Visible)], vec![1f32], vec![0f32]);
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
    
    knob_mods.0.push((
        Knob {
            a: match arf.presence {
                Presence::Staccatto => in_range(&mut rng, 0.1f32, 0.33f32),
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


/**
 * 
 * Primary concepts to cover in a synth
 * Can be combined as a list of layers
 * 
 * lifespan
 * centroid
 * height
 * filters
 * spectral distribution
 * 
 * 
 * distortion
 * saturation
 * reverb
 */

 /*
 needs
 to have punch, decay, and body as primary facets
 */
pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
    
    Renderable::Group(vec![
        stem_resonant_nodes(arf, melody),
        stem_noise(arf, melody),
        stem_bass(arf, melody),
    ])
}