/// A kick drum intended to make your heart pound 
/// 
/// This drum features a flat wall of sound for a transient. 
/// 

use super::*;


/// Four octave freq sweep, responsive to monic and duration. 
/// Requires that the input multipliers are truncated by log_2(max_sweep_mul) octaves
/// https://www.desmos.com/calculator/fbzd5wwj2e
static max_sweep_reg:f32 = 4f32;
static min_sweep_reg:f32 = 1f32;
pub fn fmod_sweep(k:usize, x:f32, d:f32) -> f32 {
    let kf = k as f32;
    let growth_const = -unit_decay;
    let sweep_reg:f32 = max_sweep_reg - 1f32;
    2f32.powf(sweep_reg) * (kf*growth_const*x).exp()
}


// values in 25-50 look good. @art-choice could mod in this range
static amod_const:f32 = 50f32;
fn amod_exit(x:f32) -> f32 {
    let y:f32 = (amod_const * x - pi).tanh();
    0.5f32 * (1f32 - y)
}

/// Intended to represent a finite length one valued signal with tanh decay.
pub fn amod_impulse(k:usize, x:f32, d:f32) -> f32 {
    let y:f32 = -1f32 + (1f32/(1f32-(-x).exp()));
    (0.5f32*y).tanh() * amod_exit(x)
}
    
fn layer_sustain(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_sawtooth(fund);
    let amps = melodic::amps_sawtooth(fund);
    let phss = melodic::phases_sawtooth(fund);
    let expr = (vec![1f32], vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(vec![
            (1f32, lifespan::mod_db_pluck),
        ]),
        Some(vec![
            (1f32, fmod_sweep),
        ]),
        None
    ];
    Element {
        mode: Mode::Melodic,
        amps,
        muls,
        phss,
        modders,
        expr,
        hplp: (vec![MFf], vec![NFf]),
        thresh: (0f32, 1f32)
    }
}

fn layer_impulse(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_max_k(fund);
    let amps = (1..=muls.len()).map(|i| 1f32/(i as f32)).collect();
    let phss = vec![0f32; muls.len()];
    let expr = (vec![  0.5f32],vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(vec![
            (1f32, amod_impulse),
        ]),
        None,
        None
    ];
    Element {
        mode: Mode::Bell,
        amps,
        muls,
        phss,
        modders,
        expr,
        hplp: (vec![MFf], vec![NFf]),
        thresh: (0f32, 1f32)
    }
}
 
pub fn synth(arf:&Arf) -> Elementor {
    vec![
        (0.005f32, layer_impulse),
        (0.995f32, layer_sustain),
    ]
}

pub fn driad(arf:&Arf) -> Ely {
    let impulse:Element = layer_impulse(MFf, &arf.visibility, &arf.energy, &arf.presence);
    let sustain:Element = layer_sustain(MFf, &arf.visibility, &arf.energy, &arf.presence);

    let many_soids = vec![
        impulse.gain(0.005f32),
        sustain.gain(0.995f32),
    ].iter().map(trig::el_to_soid).collect();

    let merged_soids = trig::prepare_soids_input(many_soids);
    let (amps, muls, phis) = trig::process_soids(merged_soids);
    Ely::from_soids(amps, muls, phis)
}

