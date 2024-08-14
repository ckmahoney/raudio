use super::*;


fn pmod_noise(k:usize, x:f32, d:f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen::<f32>() * 1.5f32 * pi
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

fn layer_sustain(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_sawtooth(fund);
    let amps = melodic::amps_sawtooth(fund);
    let phss = melodic::phases_sawtooth(fund);
    let expr = (vec![1f32], vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(vec![
            (1f32, lifespan::mod_db_pluck),
        ]),
        Some(vec![
            (1f32, fmod_sweep),
        ]),
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

fn layer_impulse(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_max_k(fund);
    let amps = (1..=muls.len()).map(|i| 1f32/(i as f32)).collect();
    let phss = vec![0f32; muls.len()];
    let expr = (vec![0.5f32],vec![1f32], vec![0f32]);
    let modders:Modders = [ 
        Some(vec![
            (1f32, amod_impulse),
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

pub fn driad(arf:&Arf) -> Ely {
    let impulse:Element = layer_impulse(MFf, &arf.visibility, &arf.energy, &arf.presence);
    let sustain:Element = layer_sustain(MFf, &arf.visibility, &arf.energy, &arf.presence);

    let all_soids = vec![
        impulse.gain(1f32),
        sustain.gain(0.05f32),
    ].iter().map(trig::el_to_soid).collect();

    let knob_mods:KnobMods = KnobMods (
        vec![(Knob { a:0.2f32, b:0f32, c:0f32}, ranger::amod_impulse)],
        vec![(Knob { a:0.2f32, b:0.1f32, c:0f32}, ranger::fmod_sweepdown)],
        vec![]
    );

    let merged_soids = trig::prepare_soids_input(all_soids);
    let (amps, muls, phis) = trig::process_soids(merged_soids);
    Ely {
        knob_mods,
        ..Ely::from_soids(amps, muls, phis)
    }
}
