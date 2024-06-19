/// A synth snare from three components
use super::*;


fn noise_pluck(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = noise::multipliers(fund, &Energy::Low);
    let mut rng = rand::thread_rng();
    let phss = vec![0f32; muls.len()]; 
    let amps = match energy { 
        Energy::High =>  muls.iter().map(|x| 1f32/x).collect(),
        _ =>  muls.iter().map(|x| 1f32/(x*x)).collect()
    };
    let expr = (vec![1f32], vec![1f32], vec![0f32]);
    let highpass_animation = vec![MFf,MFf, NFf];
    

    let a_modu:Option<WRangers> = Some(
        match presence {
            Presence::Staccatto => vec![(1f32, lifespan::mod_db_pluck)],
            Presence::Legato => vec![
                (0.8f32, lifespan::mod_db_pluck),
                (0.2f32, lifespan::mod_db_fall),
            ],
            Presence::Tenuto => vec![
                (0.66f32, lifespan::mod_db_fall),
                (0.33f32, lifespan::mod_db_pluck),
            ]           
        }
    );
    let modders:Modders = [
        a_modu,
        None,
        None
    ];
    Element {
        mode: Mode::Noise,
        amps,
        muls,
        phss,
        modders,
        expr,
        hplp: (highpass_animation, vec![NFf]),
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
            (0.65f32, lifespan::mod_snap)
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

fn bell_pluck(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let n_partials = 6;
    let muls = bell::multipliers(fund, n_partials);
    let amps = bell::coefficients(fund, n_partials);
    let phss = vec![0f32; muls.len()];
    let contour = lifespan::mod_lifespan(contour_resolution, 1f32, &AmpLifespan::Burst, 1usize, 0f32);
    let expr = (contour, vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(vec![
            (1f32, lifespan::mod_db_pluck)
        ]),
        None,
        None
    ];

    Element {
        mode: Mode::Bell,
        amps: vec![1f32; muls.len()],
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
        (0.35f32, melodic_pluck),
        (0.636f32, noise_pluck),
        (0.002f32, bell_pluck),
        (0.001f32, bell_pluck),
        (0.057f32, microtransient_click),
    ]
}