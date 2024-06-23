use super::*;

fn pmod_smooth(k:usize, x:f32, d:f32) -> f32 {
    let mut rng = rand::thread_rng();
    let p = d % 1f32;
    let base_rate = 3f32; // three hertz phase modulation rate
    let applied_rate = base_rate*d;
    
    (x * pi2  * applied_rate).sin()
}

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
        Some(vec![
            (1f32, pmod_smooth)
        ]),
    ];
    // let lowpass_animation = vec![1000f32, 4000f32, 400f32];
    let lowpass_animation = vec![NFf];
    let mut rng = rand::thread_rng();
    let filterpoints:Vec<f32> = (0..100).map(|x| x as f32 * 10000f32 * rng.gen::<f32>()).collect();
    
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
        (1f32, melodic_el),
    ]
}