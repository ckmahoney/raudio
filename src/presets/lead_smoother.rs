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

fn choose_pmod(e:&Energy) -> WRangers {
    match e {
        Energy::Low => vec![ (0.33f32, pmod_shimmer) ],
        Energy::Medium => vec![ (0.33f32, pmod_chorus) ],
        Energy::High => vec![ (0.33f32, pmod_detune) ]
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
        None,
        Some(choose_pmod(&energy)),
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