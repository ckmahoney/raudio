/// A synth snare from three components
use crate::synth::{MFf, NFf, SampleBuffer, pi2};
use crate::types::timbre::{Mode, Energy, Presence, Visibility};
use crate::druid::{Element, Elementor, modders_none, melodic, bell, noise};
use crate::phrasing::ranger::{Modders,Ranger};
use crate::phrasing::lifespan;
use crate::phrasing::lifespan::mod_snap;
use crate::phrasing::contour::expr_none;
use crate::phrasing::micro;
use crate::timbre::AmpLifespan;
use super::{microtransient_click, microtransient_pop};

use rand;
use rand::Rng;

static contour_resolution:usize = 1200;


fn noise_pluck(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = noise::multipliers(fund, energy);
    let mut rng = rand::thread_rng();
    let phss = (0..muls.len()).map(|_| rng.gen::<f32>() * pi2).collect();
    let contour = lifespan::mod_lifespan(contour_resolution, 1f32, &AmpLifespan::Burst, 1usize, 0f32);

    let expr = (contour, vec![1f32], vec![0f32]);
    let highpass_animation = vec![MFf,MFf, NFf];
    let modders:Modders = [
        Some(vec![
            (0.3f32, lifespan::mod_snap),
        ]),
        None,
        None
    ];

    Element {
        mode: Mode::Noise,
        amps: vec![1f32; muls.len()],
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
    let phss = vec![0f32; muls.len()];
    let contour = lifespan::mod_lifespan(contour_resolution, 1f32, &AmpLifespan::Pluck, 1usize, 0f32);
    let expr = (contour, vec![1f32], vec![0f32]);
    let lowpass_animation = vec![NFf, MFf];
    let modders:Modders = [
        Some(vec![
            (0.65f32, lifespan::mod_snap),
            (0.35f32, lifespan::mod_spring),
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
            (1f32, lifespan::mod_snap)
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

pub fn synth() -> Elementor {
    vec![
        (0.05f32, bell_pluck),
        (0.65f32, melodic_pluck),
        (0.32f32, noise_pluck),
        (0.05f32, microtransient_click),
        (0.03f32, microtransient_pop),
    ]
}