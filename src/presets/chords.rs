use super::*;

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
            (0.65f32, lifespan::mod_pad)
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


fn layer_sustain(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_sawtooth(fund);
    let amps = melodic::amps_sawtooth(fund);
    let phss = melodic::phases_sawtooth(fund);
    let expr = (vec![1f32], vec![1f32], vec![0f32]);
    let modders:Modders = [
        None,
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

    
 

pub fn driad(arf:&Arf) -> Ely {
    let sustain:Element = layer_sustain(MFf, &arf.visibility, &arf.energy, &arf.presence);

    let all_soids = vec![
        sustain.gain(1f32),
    ].iter().map(trig::el_to_soid).collect();

    let knob_mods:KnobMods = KnobMods (
        vec![(Knob { a:1f32, b:0f32, c:0f32}, ranger::amod_impulse)],
        vec![(Knob { a:1f32, b:0.1f32, c:0f32}, ranger::fmod_chorus)],
        vec![]
    );

    let merged_soids = trig::prepare_soids_input(all_soids);
    let (amps, muls, phis) = trig::process_soids(merged_soids);
    Ely {
        knob_mods,
        ..Ely::from_soids(amps, muls, phis)
    }
}