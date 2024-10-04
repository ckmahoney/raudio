use super::super::*;

fn knob_amp() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    (Knob { a: 1f32, b: 1f32, c: 0f32 }, ranger::amod_pluck)
}

/// Featured component 
pub fn stem_visible<'render>(arf:&Arf, melody:&'render Melody<Note>) -> Stem<'render> {
    let soids = druidic_soids::id();
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
    knob_mods.0.push((
        Knob {
            a: 0.3f32,
            b: 0.3f32,
            c:0f32
        },
        ranger::amod_pluck
    ));
    (melody, soids, expr, feel, knob_mods, vec![delay::passthrough])
}

/// Supporting feature
pub fn stem_foreground<'render>(arf:&Arf, melody:&'render Melody<Note>) -> Stem<'render> {
    let soids = soid_fx::concat(&vec![
        soid_fx::noise::rank(3, NoiseColor::Pink, 1f32/3f32),
        soid_fx::noise::rank(4, NoiseColor::Pink, 1f32/3f32),
        soid_fx::noise::rank(5, NoiseColor::Pink, 1f32/3f32),
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
    knob_mods.0.push((
        Knob {
            a: 0f32,
            b: 1f32,
            c:0f32
        },
        ranger::amod_pluck
    ));
    let mut rng = thread_rng();
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
    for v in [Visibility::Hidden, Visibility::Background, Visibility::Visible, Visibility::Foreground] {
        println!("vis {:?} val {}", v, visibility_gain(v))
    }
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