use std::os::unix::thread;

use super::*;
use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};

fn expr() -> Expr {
    let ampenv = amp_expr(4f32);
    (ampenv, vec![1f32], vec![0f32])
}

fn select_overtones(freq:f32, arf:&Arf) -> Vec<f32> {
    let n = match arf.energy {
        Energy::Low => 3,
        Energy::Medium => 4,
        Energy::High => 5,
    };

    let r = match arf.visibility {
        Visibility::Hidden => 0,
        Visibility::Background => 1,
        Visibility::Foreground => 2,
        Visibility::Visible => 3,
    };

    let limit = match arf.energy {
        Energy::Low => 3,
        Energy::Medium => 4,
        Energy::High => 5,
    };
    let mut rng = thread_rng();
    let options:Vec<f32> =( 1..=limit).into_iter().step_by(2).flat_map(|x| 
        if r == 0 {vec![x as f32]} 
        else {
            (-r..r).into_iter().map(|i| 1.5f32.powi(i as i32)).collect() 
        } 
    ).collect();
    let muls:Vec<f32> = (0..n).into_iter().map(|a| *options.choose(&mut rng).unwrap()).collect();
    muls
}

/// create a harmonic pallette texture like a house stab
fn generate_rich_texture(arf:&Arf) -> Soids {
    let mut amps:Vec<f32> = vec![];
    let mut muls:Vec<f32> = vec![];
    let mut offsets:Vec<f32> = vec![];

    let reference_freq:f32 = match arf.visibility {
        Visibility::Hidden => 2f32.powi(9i32),
        Visibility::Background => 2f32.powi(8i32),
        Visibility::Foreground => 2f32.powi(6i32),
        Visibility::Visible => 2f32.powi(5i32),
    };

    let overs = select_overtones(reference_freq, arf);
    let shade:Soids = match arf.energy {
        Energy::Low => druidic_soids::octave(2f32.powi(9i32)),
        Energy::Medium => druidic_soids::overs_triangle(2f32.powi(9i32)),
        Energy::High => druidic_soids::overs_sawtooth(2f32.powi(9i32)),
    };
    let mut rng = thread_rng();

    for i in 0..overs.len() {
        let mult = overs[i];
        let ampl:f32 = rng.gen::<f32>() * 0.5 + 0.5;
        let offset:f32 = rng.gen::<f32>() * pi - (pi/2f32);
        shade.0.iter().for_each(|amp| amps.push(ampl * amp));
        shade.1.iter().for_each(|mul| muls.push(mult * mul));
        shade.2.iter().for_each(|offset| offsets.push(offset + *offset));
    }
    (amps, muls, offsets)
}


fn amp_knob(visibility:Visibility, energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();
    if let Presence::Staccatto = presence {

        let sustain = match presence {
            Presence::Staccatto => 0f32,
            Presence::Legato => 0.66f32,
            Presence::Tenuto => 1f32
        };
    
        let decay_rate = match energy {
            Energy::Low => rng.gen::<f32>()/5f32,
            Energy::Medium => 0.25 + rng.gen::<f32>()/2f32,
            Energy::High => 0.66f32 + 0.33 * rng.gen::<f32>(),
        };
        
        return (Knob { a: sustain, b: decay_rate, c: 0.0}, ranger::amod_pluck)
    };

    let osc_rate = match presence {
        Presence::Staccatto => 025f32,
        Presence::Legato => 0.66f32,
        Presence::Tenuto => 1f32
    };

    let time_scale = match energy {
        Energy::Low => rng.gen::<f32>()/3f32,
        Energy::Medium => 0.42 + rng.gen::<f32>()/4f32,
        Energy::High => 0.66f32 + 0.33 * rng.gen::<f32>(),
    };

    let dilation = match visibility {
        Visibility::Visible => 0.5 + 0.5 * rng.gen::<f32>(),
        Visibility::Foreground => 0.3 + 0.2 * rng.gen::<f32>(),
        Visibility::Background => 0.1 * 0.2 * rng.gen::<f32>(),
        Visibility::Hidden => 0.5f32,
    };

    return (Knob { a: osc_rate, b: time_scale, c: dilation }, ranger::amod_oscillation_sin_mul);

}


pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
    let soids:Soids = generate_rich_texture(arf);
    let mut knob_mods:KnobMods = KnobMods::unit();
    knob_mods.0.push(amp_knob(arf.visibility, arf.energy, arf.presence));

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

    let stem = (melody, soids, expr(), feel, knob_mods, vec![delay::passthrough]);
    Renderable::Instance(stem)
}