use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};


pub fn expr(arf:&Arf) -> Expr {
    (vec![visibility_gain(arf.visibility)], vec![1f32], vec![0f32])
}


fn amp_knob(visibility:Visibility, energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let sustain = match presence {
        Presence::Staccatto => 0f32,
        Presence::Legato => 0.1f32,
        Presence::Tenuto => 0.3f32
    };
    let decay_rate = match energy {
        Energy::Low => 0.5f32,
        Energy::Medium => 0.75f32,
        Energy::High => 1f32,
    };
    (Knob { a: sustain, b: decay_rate, c: 0.0}, ranger::amod_pluck)
}


/// Selects the number of resonant nodes to add 
fn amp_reso_gen(modders:&mut KnobMods, visibility:Visibility, energy:Energy, presence:Presence) {
    let n = match energy {
        Energy::Low => 2,
        Energy::Medium => 3,
        Energy::High => 5,
    };
    let mut rng = thread_rng();
    for i in 0..n {
        let a:f32 = match visibility {
            Visibility::Visible => rng.gen::<f32>()/5f32,
            Visibility::Foreground => rng.gen::<f32>()/3f32,
            Visibility::Background => rng.gen::<f32>()/2f32,
            Visibility::Hidden => 0.5f32 + rng.gen::<f32>()/2f32,
        }; 
        let b:f32 = match energy {
            Energy::Low => rng.gen::<f32>()/2f32,
            Energy::Medium => rng.gen::<f32>()/3f32,
            Energy::High => rng.gen::<f32>()/5f32,
        };
        modders.0.push((Knob { a: a, b: b, c: 0.0}, ranger::amod_peak))
    }
}


pub fn renderable<'render>(cps:f32, melody:&'render Melody<Note>, arf:&Arf) -> Renderable2<'render> {
    let mut knob_mods:KnobMods = KnobMods::unit();
    let mut rng:ThreadRng = thread_rng();
    // principal layer
    knob_mods.0.push((
        Knob {
            a: in_range(&mut rng, 0.2f32, 0.8f32),
            b: in_range(&mut rng, 0.2f32, 0.8f32),
            c: 0f32
        },
        ranger::amod_pluck
    ));
    
    // attenuation layer
    knob_mods.0.push((
        Knob {
            a: match arf.presence {
                Presence::Staccatto => in_range(&mut rng, 0.33f32, 0.66f32),
                Presence::Legato => in_range(&mut rng, 0.53f32, 0.76f32),
                Presence::Tenuto => in_range(&mut rng, 0.88f32, 1f32),
            },
            b: match arf.energy {
                Energy::High => in_range(&mut rng, 0f32, 0.33f32),
                Energy::Medium => in_range(&mut rng, 0.33f32, 0.5f32),
                Energy::Low => in_range(&mut rng, 0.5f32, 1f32),
            },
            c:0f32
        },
        ranger::amod_oscillation_sine
    ));

    let len_cycles:f32 = time::count_cycles(&melody[0]);
    let soids = soid_fx::concat(&vec![
        soid_fx::noise::rank(1, NoiseColor::Equal, 1f32),
        soid_fx::noise::rank(0, NoiseColor::Blue, 1f32),
        soid_fx::noise::rank(1, NoiseColor::Blue, 0.5f32),
    ]);
    let stem = (melody, soids, expr(arf), get_bp(cps, melody, arf, len_cycles), knob_mods, vec![delay::passthrough]);
    Renderable2::Instance(stem)
}