use super::*;



// values in 25-50 look good. @art-choice could mod in this range
fn amod_exit(x:f32, amod_const:f32) -> f32 {
    let y:f32 = (amod_const * x - pi).tanh();
    0.5f32 * (1f32 - y)
}

pub fn amod_impulse_short(k:usize, x:f32, d:f32) -> f32 {
    static amod_const:f32 = 10f32;
    let y:f32 = -1f32 + (1f32/(1f32-(-x).exp()));
    (0.5f32*y).tanh() * amod_exit(x,amod_const)
}

pub fn amod_impulse_long(k:usize, x:f32, d:f32) -> f32 {
    static amod_const:f32 = 20f32;
    let y:f32 = -1f32 + (1f32/(1f32-(-x).exp()));
    (0.5f32*y).tanh() * amod_exit(x,amod_const)
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

fn layer_bell(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let n_partials = 16;
    let muls = bell::multipliers(fund, n_partials);
    let amps = bell::coefficients(fund, n_partials);
    let phss = vec![0f32; muls.len()];
    let expr = (vec![1f32], vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(vec![
            (1f32, lifespan::mod_db_pluck),
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

fn pmod_noise(k:usize, x:f32, d:f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen::<f32>() * pi2
}

fn noise_pluck(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_max_k(fund);
    let phss = vec![0f32; muls.len()];
    let amps:Vec<f32> = (1..=muls.len()).map(|i| 1f32/(i as f32)).collect();
    let amps = vec![0.5f32; muls.len()];
    let expr = (vec![1f32], vec![1f32], vec![0f32]);
    let highpass_animation = vec![2000f32];
    
    let a_modu:WOldRangerDeprecateds = match presence {
        Presence::Staccatto => vec![(1f32, lifespan::mod_db_pluck)],
        Presence::Legato => vec![
            (0.8f32, lifespan::mod_db_pluck),
            (0.2f32, lifespan::mod_db_fall),
        ],
        Presence::Tenuto => vec![
            (0.66f32, lifespan::mod_db_fall),
            (0.33f32, lifespan::mod_db_pluck),
        ]           
    };
    let modders:Modders = [
        Some(a_modu),
        None,
        Some(vec![(1f32, pmod_noise)])
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


fn layer_impulse(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_max_k(fund);
    let amps = (1..=muls.len()).map(|i| 1f32).collect();
    let phss = vec![0f32; muls.len()];
    let expr = (vec![  0.5f32],vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(vec![
            (1f32, amod_impulse_short),
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
        (0.12f32, layer_impulse),
        (0.12f32, layer_bell),
        (0.76f32, noise_pluck),
    ]
}

pub fn driad(arf:&Arf) -> Ely {
    let bell:Element = layer_bell(MFf, &arf.visibility, &arf.energy, &arf.presence);
    let impulse:Element = layer_impulse(MFf, &arf.visibility, &arf.energy, &arf.presence);
    let pluck:Element = noise_pluck(MFf, &arf.visibility, &arf.energy, &arf.presence);

    let all_soids = vec![
        bell.gain(0.12f32),
        impulse.gain(0.12f32),
        pluck.gain(0.76f32)
    ].iter().map(trig::el_to_soid).collect();

    let merged_soids = trig::prepare_soids_input(all_soids);
    let (amps, muls, phis) = trig::process_soids(merged_soids);
    Ely::from_soids(amps, muls, phis)
}

