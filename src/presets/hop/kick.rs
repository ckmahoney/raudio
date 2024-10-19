use super::*;


fn knob_amp() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    (Knob { a: 1f32, b: 1f32, c: 0f32 }, ranger::amod_pluck)
}


fn gain(arf:&Arf) -> f32 { 
    let x = match arf.presence {
        Presence::Tenuto => 3f32,
        Presence::Legato => 2f32,
        Presence::Staccatto => 1f32,
    } ;
    
    let mul = match arf.energy {
        Energy::High => 3f32,
        Energy::Medium => 2f32,
        Energy::Low => 1f32,
    };

    x * mul
}


/// Supporting feature
pub fn stem_noise<'render>(arf:&Arf, melody:&'render Melody<Note>) -> Stem<'render> {
    let soids = soid_fx::concat(&vec![
        soid_fx::noise::rank(1, NoiseColor::Pink, 1f32/5f32),
        soid_fx::noise::rank(2, NoiseColor::Equal, 1f32/9f32),
    ]);
    let expr = (vec![gain(arf)*visibility_gain(Visibility::Hidden)], vec![1f32], vec![0f32]);
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

    // attenuation layer
    knob_mods.0.push((
        Knob {
            a: in_range(&mut rng, 0.95f32, 1f32),
            b: 1f32,
            c:0f32
        }, 
        ranger::amod_pluck
    ));
    (melody, soids, expr, feel, knob_mods, vec![delay::passthrough])
}


pub fn stem_bass<'render>(arf:&Arf, melody:&'render Melody<Note>) -> Stem<'render> {
    let soids = druidic_soids::upto(2);

    let expr = (vec![db_to_amp(-4.5f32)*gain(arf)], vec![1f32], vec![0f32]); 
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
                Presence::Legato => in_range(&mut rng, 0.7f32, 0.94f32),
                Presence::Tenuto => in_range(&mut rng, 0.9f32, 1f32),
            },
            b: match arf.energy {   
                Energy::High => in_range(&mut rng, 0f32, 0.33f32),
                Energy::Medium => in_range(&mut rng, 0.33f32, 0.5f32),
                Energy::Low => in_range(&mut rng, 0.5f32, 0.66f32),
            },
            c:0f32
        },
        ranger::amod_burp
    ));

    // attenuation layer
    knob_mods.0.push((
        Knob {
            a: in_range(&mut rng, 0.95f32, 1f32),
            b: match arf.energy {   
                Energy::High => in_range(&mut rng, 0f32, 0.33f32),
                Energy::Medium => in_range(&mut rng, 0.33f32, 0.5f32),
                Energy::Low => in_range(&mut rng, 0.5f32, 0.66f32),
            },
            c:0f32
        }, 
        if let Energy::High = arf.energy { ranger::amod_burp } else { ranger::amod_pluck } 
    ));
    
    knob_mods.1.push((
        Knob {
            a: match arf.energy {   
                Energy::High => in_range(&mut rng, 0.33f32,0.5f32),
                Energy::Medium => in_range(&mut rng, 0.1f32, 0.3f32),
                Energy::Low => in_range(&mut rng, 0.01f32, 0.1f32), 
            },
            b: match arf.presence {
                Presence::Staccatto => in_range(&mut rng, 0.4f32, 0.7f32),
                Presence::Legato => in_range(&mut rng, 0.3f32, 0.3f32),
                Presence::Tenuto => in_range(&mut rng, 0.4f32, 0.4f32),
            },
            c:0f32
        },
        ranger::fmod_sweepdown
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
        stem_noise(arf, melody),
        stem_bass(arf, melody),
    ])
}