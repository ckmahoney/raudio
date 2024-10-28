use super::super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::druid::{self, soids as druidic_soids, soid_fx};

pub fn expr(arf:&Arf, n_samples:usize) -> Expr {
    let dynamics = dynamics::gen_organic_amplitude(10, n_samples, arf.visibility);
    (dynamics, vec![1f32], vec![0f32])
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
            Energy::Low => (filter_contour_triangle_shape_highpass(lowest_register-2, highest_register-2, n_samples, size*rate_per_size), vec![NFf]),
            _ => (vec![MFf],filter_contour_triangle_shape_lowpass(lowest_register, n_samples, size*rate_per_size))
        }
    } else {
        (vec![MFf], vec![NFf/8f32])
    };

    (highpass, lowpass)
} 

fn amp_knob_principal(rng:&mut ThreadRng, arf:&Arf)  -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
    return (
        Knob { 
            a: match arf.presence {
                Presence::Staccatto => in_range(rng, 0.11f32, 0.3f32),
                Presence::Legato => in_range(rng, 0.33f32, 0.5f32),
                Presence::Tenuto => in_range(rng, 0.7f32, 0.9f32),
            },
            b: match arf.visibility {
                Visibility::Visible => in_range(rng, 0f32, 0.2f32),
                Visibility::Foreground =>in_range(rng, 0.2f32, 0.3f32),
                _ => in_range(rng, 0.3f32, 0.5f32),
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
                Energy::High => in_range(rng, 0.34f32, 0.5f32),
                Energy::Medium => in_range(rng, 0.23f32, 0.34f32),
                Energy::Low => in_range(rng, 0.0f32, 0.21f32),
            },
            b: 1f32,
            c: 0.0 
        }, 
        ranger::amod_detune
    )
}



pub fn renderable<'render>(cps:f32, melody:&'render Melody<Note>, arf:&Arf) -> Renderable2<'render> {
    let mut rng = thread_rng();
    let len_cycles:f32 = time::count_cycles(&melody[0]);
    let n_samples=(SRf*len_cycles/2f32) as usize; 

    
    let mut knob_mods:KnobMods = KnobMods::unit();
    knob_mods.0.push(amp_microtransient(arf.visibility, arf.energy, arf.presence));
    knob_mods.0.push(amp_knob_principal(&mut rng, &arf));
    knob_mods.0.push(amp_knob_attenuation(&mut rng, &arf));
    let soids = druidic_soids::overs_square(2f32.powi(7i32));

    // let soids = soid_fx::amod::reece(&soids, 2);
    let stem = (melody, soids, expr(arf, n_samples), get_bp(cps, melody, arf, len_cycles), knob_mods, vec![delay::passthrough]);

    Renderable2::Group(vec![
        stem,
    ])
}