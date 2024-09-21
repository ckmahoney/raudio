use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::ranger::{KnobMods};
use crate::druid::{self, soids as druidic_soids};

fn expr() -> Expr {
    (vec![1f32], vec![1f32], vec![0f32])
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
    return Some((Knob { a: osc_rate, b: intensity, c: 0.0 }, ranger::amod_oscillation_sine))

}

pub fn driad(arf:&Arf) -> Ely {
    let soids:Soids = match arf.energy {
        Energy::Low => druidic_soids::octave(16f32),
        Energy::Medium => druidic_soids::octave(8f32),
        Energy::High => druidic_soids::octave(4f32),
    };
    let modders:ModifiersHolder = (
        vec![],
        vec![],
        vec![],
        vec![],
    );
    let mut knob_mods:KnobMods = KnobMods::unit();
    if let Some(knob_mod) = amp_knob(arf.visibility, Energy::High, Presence::Legato) {
        knob_mods.0.push(knob_mod)
    }
    Ely::new(soids, modders, knob_mods)
}


pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
    let ely = driad(arf);
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

    let stem:Stem = (melody, ely.soids, expr(), feel, ely.knob_mods, vec![delay::passthrough]);
    Renderable::Instance(stem)
}