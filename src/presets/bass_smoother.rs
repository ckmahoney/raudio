use super::*;
use rand::seq::SliceRandom;

fn pmod_smooth(k:usize, x:f32, d:f32) -> f32 {
    let mut rng = rand::thread_rng();
    let p = d % 1f32;
    let base_rate = 2f32/3f32; // three hertz phase modulation rate
    let applied_rate = base_rate*d;
    
    (x * pi2  * applied_rate).sin()
}

fn amod(presence:&Presence) -> WOldRangerDeprecateds {
    match presence {
        Presence::Staccatto => vec![(1f32, lifespan::mod_db_pluck)],
        Presence::Tenuto => vec![
            (1f32, lifespan::mod_db_fall),
        ],        
        Presence::Legato => vec![(1f32, lifespan::mod_drone)],
    }
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
        Some(amod(&presence)),
        None,
        None,
    ];
    
    let lowpass_animation = vec![NFf];
    let mut rng = rand::thread_rng();
    let opts_bubbles:Vec<i32> = vec![2,3,4,5, 6,7,8];
    let n_points = opts_bubbles.choose(&mut rng).unwrap_or(&4i32);
    let lowpass_filterpoints:Vec<f32> = (0..*n_points)
        .map(|x| 
            200f32+1200f32 * rng.gen::<f32>()
        ).collect();
    
    Element {
        mode: Mode::Melodic,
        amps,
        muls,
        phss,
        modders,
        expr, 
        hplp: (vec![MFf], lowpass_filterpoints),
        thresh: (0f32, 1f32)
    }
}

fn edgy_el(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_square(fund);
    let amps = melodic::amps_square(fund);
    let mut rng = rand::thread_rng();
    let phss = match energy { 
        Energy::High =>  (0..muls.len()).map(|_| (rng.gen::<f32>() - 0.5f32) * pi2/16f32).collect(),
        _ =>  vec![0f32; muls.len()]
    };
    let expr = (vec![1f32], vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(amod(&presence)),
        None,
        Some(vec![
            (1f32, pmod_smooth)
        ]),
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
        (0.45f32, melodic_el),
        (0.05f32, edgy_el),//@art-choice only include this when high energy
    ]
}