use std::os::unix::thread;

use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids};

fn amp_knob_breath(visibility:Visibility, energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();

    let breath_rate = match energy {
        Energy::Low => rng.gen::<f32>(),
        Energy::Medium => rng.gen::<f32>()/2f32,
        Energy::High => rng.gen::<f32>()/3f32,
    };

    return (Knob { a: breath_rate, b: 0f32, c: 0.0 }, ranger::amod_breath);
}


fn amp_knob_experiement(visibility:Visibility, energy:Energy, presence:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();

    let detune_rate = match energy {
        Energy::Low => rng.gen::<f32>()/6f32,
        Energy::Medium => rng.gen::<f32>()/4f32,
        Energy::High => rng.gen::<f32>()/3f32,
    };
    let detune_mix = match visibility {
        Visibility::Visible => 0.33 + 0.47 * rng.gen::<f32>(),
        Visibility::Foreground => 0.2 + 0.13 * rng.gen::<f32>(),
        Visibility::Background => 0.1 * 0.1 * rng.gen::<f32>(),
        Visibility::Hidden => 0.05f32 * rng.gen::<f32>(),
    };

    return (Knob { a: detune_rate, b: detune_mix, c: 0.0 }, ranger::amod_detune);
}

fn freq_knob_tonal(v:Visibility, e:Energy, p:Presence) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    let mut rng = thread_rng();
    let modulation_amount = match e {
        Energy::Low => 0.005f32 + 0.003 * rng.gen::<f32>(),
        Energy::Medium => 0.008f32 + 0.012f32 * rng.gen::<f32>(),
        Energy::High => 0.25f32 + 0.75f32 * rng.gen::<f32>()
    };
    (Knob { a: modulation_amount, b: 0f32, c: 0.0}, ranger::fmod_warble)
}


/// Create bandpass automations with respsect to Arf and Melody
fn bp<'render>(melody:&'render Melody<Note>, arf:&Arf, len_cycles:f32) -> (SampleBuffer, SampleBuffer) {
    let size = len_cycles.log2()-1f32; // offset 1 to account for lack of CPC. -1 assumes CPC=2
    let rate_per_size = match arf.energy {
        Energy::Low => 0.25f32,
        Energy::Medium => 0.5f32,
        Energy::High => 1f32,
    };

    let mut highest_register:i8 = arf.register;
    let mut lowest_register:i8 = arf.register;
    for line in melody.iter() {
        for (_, (register, _), _) in line.iter() {
            highest_register = (*register).max(highest_register);
            lowest_register = (*register).min(lowest_register);
        }
    };
    let n_samples:usize = (len_cycles/2f32) as usize * SR;

    let (highpass, lowpass):(Vec<f32>, Vec<f32>) = if let Visibility::Visible = arf.visibility {
        match arf.energy {
            Energy::Low => (filter_contour_triangle_shape_highpass(lowest_register, highest_register, n_samples, size*rate_per_size), vec![NFf]),
            Energy::Medium => (vec![MFf],filter_contour_triangle_shape_lowpass(lowest_register+2, n_samples, size*rate_per_size)),
            Energy::High => (vec![MFf],vec![NFf])
        }
    } else {
        (vec![MFf*4f32], vec![NFf/3f32])
    };

    (highpass, lowpass)
} 


pub fn renderable<'render>(melody:&'render Melody<Note>, arf:&Arf) -> Renderable<'render> {
    let mullet = match arf.energy {
        Energy::Low => 256f32,
        Energy::Medium => 64f32,
        Energy::High => 32f32,
    };
    let soids = match arf.visibility {
        Visibility::Hidden => druidic_soids::octave(mullet),
        Visibility::Background => druidic_soids::overs_triangle(mullet),
        Visibility::Foreground => druidic_soids::overs_square(mullet),
        Visibility::Visible => druidic_soids::overs_sawtooth(mullet),
    };
    let mut knob_mods:KnobMods = KnobMods::unit();

    knob_mods.0.push(amp_knob_experiement(arf.visibility, arf.energy, arf.presence));
    knob_mods.0.push(amp_knob_breath(arf.visibility, arf.energy, arf.presence));
    knob_mods.1.push(freq_knob_tonal(arf.visibility, arf.energy, arf.presence));
    let len_cycles:f32 = time::count_cycles(&melody[0]);

    let feel:Feel = Feel {
        bp: bp(melody, arf, len_cycles),
        modifiers: (
            vec![],
            vec![],
            vec![],
            vec![],
        ),
        clippers: (0f32, 1f32)
    };
    let n_samples=(SRf*len_cycles/2f32) as usize; 

    let dynamics = dynamics::gen_organic_amplitude(10, n_samples, arf.visibility);
    let expr = (dynamics, vec![1f32], vec![0f32]);
    let stem = (melody, soids, expr, feel, knob_mods, vec![delay::passthrough]);
    Renderable::Instance(stem)
}