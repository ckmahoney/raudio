/// A synth hihat from three components
use crate::synth::{MFf, NFf, SampleBuffer, pi2};
use crate::types::timbre::{Mode, Energy, Presence, Visibility};
use crate::druid::{Element, Elementor, modders_none};
use crate::druid::{melodic, bell, noise};
use crate::phrasing::lifespan;
use crate::timbre::AmpLifespan;
use super::{microtransient_click, microtransient_chiff, microtransient_pop};

use rand;
use rand::Rng;


fn noise_pluck(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = noise::multipliers(fund, energy);
    let mut rng = rand::thread_rng();
    let phss = (0..muls.len()).map(|_| rng.gen::<f32>() * pi2).collect();
    let contour = lifespan::mod_lifespan(100usize, 1f32, &AmpLifespan::Pluck, 1usize, 0f32);
    let expr = (contour, vec![1f32], vec![0f32]);

    Element {
        mode: Mode::Noise,
        amps: vec![1f32; muls.len()],
        muls,
        phss,
        modders: modders_none(),
        expr,
        hplp: (vec![MFf], vec![NFf]),
        thresh: (0f32, 1f32)
    }
}

fn melodic_pluck(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_sine(fund);
    let amps = melodic::amps_sine(fund);
    let phss = vec![0f32; muls.len()];
    let contour = lifespan::mod_lifespan(100usize, 1f32, &AmpLifespan::Pluck, 1usize, 0f32);
    let expr = (contour, vec![1f32], vec![0f32]);

    Element {
        mode: Mode::Melodic,
        amps: vec![1f32; muls.len()],
        muls,
        phss,
        modders: modders_none(),
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
    let expr = (contour, vec![1f32], vec![0f32]);

    Element {
        mode: Mode::Bell,
        amps: vec![1f32; muls.len()],
        muls,
        phss,
        modders: modders_none(),
        expr,
        hplp: (vec![MFf], vec![NFf]),
        thresh: (0f32, 1f32)
    }
}

pub fn synth() -> Elementor {
    vec![
        (0.3f32, bell_pluck),
        (0.1f32, melodic_pluck),
        (0.5f32, noise_pluck),
        (0.05f32, microtransient_chiff),
        (0.05f32, microtransient_pop),
    ]
}