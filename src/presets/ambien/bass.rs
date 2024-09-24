use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::ranger::{KnobMods};
use crate::druid::{self, soids as druidic_soids};

fn expr() -> Expr {
    (vec![0.125f32], vec![1f32], vec![0f32])
}

fn amp_knob(visibility:Visibility, energy:Energy, presence:Presence) -> Option<(Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32)> {
    if let Visibility::Hidden = visibility {
        return None
    }
    if let Visibility::Background = visibility {
        return None
    }
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
    return Some((Knob { a: osc_rate, b: intensity, c: 0.0 }, ranger::amod_slowest))

}

pub fn driad(arf:&Arf) -> Ely {
    let reference:f32 = 2f32.powi(5i32);
    let mixers:[Soids; 6] = [
        druidic_soids::octave(reference),
        druidic_soids::octave(reference * 1.5f32),
        druidic_soids::octave(reference * 3f32),
        druidic_soids::octave(reference * 9f32),
        druidic_soids::octave(reference * 15f32),
        druidic_soids::octave(reference * 60f32),
    ];
    let mut soids:Soids = (vec![], vec![], vec![]);
    for (amps, muls, offsets) in mixers {
        for i in 0..amps.len() {
            soids.0.push(amps[i]);
            soids.1.push(muls[i]);
            soids.2.push(offsets[i])
        }
    }

    let modders:ModifiersHolder = (
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let mut knob_mods:KnobMods = KnobMods::unit();
    if let Some(knob_mod) = amp_knob(arf.visibility, Energy::High, Presence::Legato) {
        // knob_mods.0.push(knob_mod) 
    }
    Ely::new(soids, modders, knob_mods)
}

pub fn amp_knob_collage() -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    (Knob { a: 0f32, b: 0f32, c: 0.0 }, ranger::amod_collage)
}


pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
    let mut ely = driad(arf);
    let ely_sine = driad(arf);

    let feel_sine:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: ely_sine.modders,
        clippers: (0f32, 1f32)
    };

    let feel:Feel = Feel {
        bp: (vec![MFf], vec![NFf]),
        modifiers: ely.modders,
        clippers: (0f32, 1f32)
    };

    ely.knob_mods.0.push(amp_knob_collage());

    let stem:Stem = (melody, ely.soids, expr(), feel, ely.knob_mods, vec![delay::passthrough]);
    Renderable::Instance(stem)
}