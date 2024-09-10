/// A synth hihat from three components
use super::*;

fn noise_pluck(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = noise::multipliers(fund, energy);
    let mut rng = rand::thread_rng();
    let phss = (0..muls.len()).map(|_| rng.gen::<f32>() * pi2).collect();
    let contour = lifespan::mod_lifespan(100usize, 1f32, &AmpLifespan::Pluck, 1usize, 0f32);
    let expr = (contour, vec![1f32], vec![0f32]);
    let a_modu:Option<WOldRangerDeprecateds> = Some(
        match presence {
            Presence::Staccatto => vec![(1f32, lifespan::mod_db_pluck)],
            Presence::Legato => vec![(1f32, lifespan::mod_db_fall)],
            Presence::Tenuto => vec![
                (0.66f32, lifespan::mod_db_pluck),
                (0.33f32, lifespan::mod_spring),
            ]           
        }
    );
    let amps = match energy { 
        Energy::High =>  vec![1f32; muls.len()],
        _ =>  vec![0.5f32; muls.len()]
    };
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
        hplp: (vec![MFf], vec![NFf]),
        thresh: (0f32, 1f32)
    }
}

fn bell_pluck(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let n_partials = 6;
    let muls = bell::multipliers(fund, n_partials);
    let amps = bell::coefficients(fund, n_partials);
    let phss = vec![0f32; muls.len()];
    let contour = lifespan::mod_lifespan(100usize, 1f32, &AmpLifespan::Pluck, 1usize, 0f32);
    let expr = (vec![1f32], vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(vec![
            (1f32, lifespan::mod_db_pluck),
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
        (0.875f32, noise_pluck),
        (0.005f32, bell_pluck),
        (0.003f32, bell_pluck),
        (0.002f32, bell_pluck),
        (0.09f32, microtransient_chiff),
    ]
}