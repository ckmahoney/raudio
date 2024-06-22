use super::*;


/// Four octave freq sweep, responsive to monic and duration. 
/// Requires that the input multipliers are truncated by log_2(max_sweep_mul) octaves
/// https://www.desmos.com/calculator/fbzd5wwj2e
static max_sweep_reg:f32 = 4f32;
static min_sweep_reg:f32 = 1f32;
pub fn fmod_sweep(k:usize, x:f32, d:f32) -> f32 {
    let kf = k as f32;
    let growth_const = -unit_decay;
    let sweep_reg:f32 = max_sweep_reg - 3f32;
    2f32.powf(sweep_reg) * (kf*growth_const*x).exp()
}


// values in 25-50 look good. @art-choice could mod in this range
static amod_const:f32 = 10f32;
fn amod_exit(x:f32) -> f32 {
    let y:f32 = (amod_const * x - pi).tanh();
    0.5f32 * (1f32 - y)
}

/// Intended to represent a finite length one valued signal with tanh decay.
pub fn amod_impulse(k:usize, x:f32, d:f32) -> f32 {
    let y:f32 = -1f32 + (1f32/(1f32-(-x).exp()));
    (0.5f32*y).tanh() * amod_exit(x)
}
    
fn pmod_noise(k:usize, x:f32, d:f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen::<f32>() * 1.5f32 * pi
}

fn layer_impulse(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_max_k(fund);
    let phss = vec![0f32; muls.len()];
    let amps = vec![1f32; muls.len()];
    let expr = (vec![1f32],vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(vec![
            (1f32, amod_impulse),
        ]),
        None,
        Some(vec![(1f32, pmod_noise)])
    ];
    let highpass_animation = vec![1200f32];
    let low_animation = vec![2400f32];
    Element {
        mode: Mode::Noise,
        amps,
        muls,
        phss,
        modders,
        expr,
        hplp: (vec![MFf], vec![NFf]),
        thresh: (0f32, 1f32)
    }
}
 
fn melodic_pluck(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_sine(fund);
    let amps = melodic::amps_sine(fund);
    let mut rng = rand::thread_rng();
    let phss = match energy { 
        Energy::High =>  (0..muls.len()).map(|_| rng.gen::<f32>() * pi2).collect(),
        _ =>  vec![0f32; muls.len()]
    };
    let contour = lifespan::mod_lifespan(contour_resolution, 1f32, &AmpLifespan::Pluck, 1usize, 0f32);
    let expr = (contour, vec![1f32], vec![0f32]);
    let lowpass_animation = vec![NFf, MFf];
    let modders:Modders = [
        Some(vec![
            (1f32, lifespan::mod_snap)
        ]),
        None,
        None
    ];

    Element {
        mode: Mode::Melodic,
        amps,
        muls,
        phss,
        modders,
        expr,
        hplp: (vec![MFf], lowpass_animation),
        thresh: (0f32, 1f32)
    }
}

pub fn synth(arf:&Arf) -> Elementor {
    vec![
        (0.33f32, melodic_pluck),
        (0.66f32, layer_impulse),
    ]
}