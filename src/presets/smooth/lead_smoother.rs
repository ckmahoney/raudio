use super::*;

fn pmod_shimmer(k:usize, x:f32, d:f32) -> f32 {
    let mut rng = rand::thread_rng();
    let p = d % 1f32;
    let base_rate = 2f32/k as f32;
    let applied_rate = base_rate*d; 
    
    (x * pi2  * applied_rate).sin()
}

fn pmod_chorus(k:usize, x:f32, d:f32) -> f32 {
    let mut rng = rand::thread_rng();
    let p = d % 1f32;
    let base_rate = k as f32*4f32/3f32;
    let applied_rate = base_rate*d; 
    
    (x * pi2  * applied_rate).sin()
}
fn pmod_detune(k:usize, x:f32, d:f32) -> f32 {
    let mut rng = rand::thread_rng();
    let p = d % 1f32;
    let base_rate = k as f32*7f32;
    let applied_rate = base_rate*d; 
    
    (x * pi2  * applied_rate).sin()
}

// offset_above_zero is computed from vib range 
static vib_a:f32 = 30f32;
static vib_b:f32 = 12f32;
static vib_c:f32 = pi/3f32;
fn fmod_vibrato(k:usize, x:f32, d:f32) -> f32 {
    let vib_range:f32 = 2f32.powf(2f32/12f32) / 32f32;
    let offset_center_1:f32 = 1f32 + (vib_range/2f32);
    let y =((vib_a/(vib_b*x+vib_c).powf(3f32))).sin();
    vib_range * y+offset_center_1
}


// offset_above_zero is computed from vib range 
static vib_d:f32 = 50f32;
static vib_e:f32 = 12f32;
static vib_f:f32 = pi/2f32;
fn fmod_vibrato2(k:usize, x:f32, d:f32) -> f32 {
    let vib_range:f32 = 2f32 / 3f32;
    let offset_center_1:f32 = 1f32 + (vib_range/2f32);
    let y =((vib_d/(vib_e*x+vib_f).powf(3f32))).sin();
    vib_range * y+offset_center_1
}


fn choose_pmod(e:&Energy) -> WOldRangerDeprecateds {
    match e {
        Energy::Low => vec![ (0.33f32, pmod_shimmer) ],
        Energy::Medium => vec![ (0.33f32, pmod_chorus) ],
        Energy::High => vec![ (0.33f32, pmod_detune) ]
    }
}

fn choose_fmod(v:&Visibility) -> Option<WOldRangerDeprecateds> {
    match v {
        Visibility::Visible => Some(vec![ (0.33f32, fmod_vibrato) ]),
        Visibility::Foreground => Some(vec![ (0.33f32, fmod_vibrato2) ]),
        _ => None
    }
}

fn melodic_el(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_sawtooth(fund);
    let amps = melodic::amps_sawtooth(fund);
    let mut rng = rand::thread_rng();
    let phss = match energy { 
        Energy::High =>  (0..muls.len()).map(|_| (rng.gen::<f32>() - 0.5f32) * pi2/16f32).collect(),
        _ =>  vec![0f32; muls.len()]
    };
    let phss = melodic::phases_sawtooth(fund);
    let expr = (vec![1f32], vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(vec![
            (0.65f32, lifespan::mod_pluck)
        ]),
        Some(vec![(1f32, fmod_vibrato)]),
        choose_fmod(&vis),
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

pub fn synth(arf:&Arf) -> Elementor {
    vec![
        (1f32, melodic_el),
    ]
}