use super::*;

fn amod_tremelo(k:usize, x:f32, d:f32) -> f32 {
    let width = 1f32/3f32;
    let cycle = x * d / width;
    (pi2*cycle).sin()
}

fn amod_chorus(k:usize, x:f32, d:f32) -> f32 {
    let width = k as f32 /3f32;
    let cycle = x * d / width;
    (pi2*cycle).sin()
}

fn amod_detune(k:usize, x:f32, d:f32) -> f32 {
    let width = 1f32/k as f32;
    let cycle = x * d / width;

    (pi2*cycle).sin()
}

fn choose_amod(e:&Energy) -> WRangers {
    match e {
        Energy::Low => vec![(1f32, amod_detune)],
        Energy::Medium => vec![(1f32, amod_chorus)],
        Energy::High => vec![(1f32, amod_tremelo)],
    }
}

fn melodic_el(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_triangle(fund);
    let amps = melodic::amps_triangle(fund);
    let mut rng = rand::thread_rng();
    let phss = match energy { 
        Energy::High =>  (0..muls.len()).map(|_| (rng.gen::<f32>() - 0.5f32) * pi2/16f32).collect(),
        _ =>  vec![0f32; muls.len()]
    };
    let expr = (vec![1f32], vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(vec![
            (1f32, amod_chorus)
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
        hplp: (vec![MFf], vec![NFf]),
        thresh: (0f32, 1f32)
    }
}

pub fn synth(arf:&Arf) -> Elementor {
    vec![
        (1f32, melodic_el),
    ]
}