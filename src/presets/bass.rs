use super::*;

fn melodic_el(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_square(fund);
    let amps = melodic::amps_square(fund);
    let mut rng = rand::thread_rng();
    let phss = match energy { 
        Energy::High =>  (0..muls.len()).map(|_| (rng.gen::<f32>() - 0.5f32) * pi2/16f32).collect(),
        _ =>  vec![0f32; muls.len()]
    };
    let expr = (vec![1f32], vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(vec![
            (0.65f32, lifespan::mod_drone)
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