/// A synth snare from three components
use crate::synth::{MFf, NFf, SampleBuffer, pi2};
use crate::types::timbre::{Mode, Energy, Presence, Visibility};
use crate::druid::{Element, Elementor, modders_none};
use crate::phrasing::ranger::{Modders,Ranger};
use crate::druid::{melodic, bell, noise};
use crate::phrasing::{contour, lifespan};
use crate::timbre::{AmpContour,AmpLifespan};
use super::{microtransient_click, microtransient_chiff, microtransient_pop};

use rand;
use rand::Rng;
static contour_length:usize  = 2000usize;

fn sine_pluck(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = melodic::muls_sine(fund);
    let amps = melodic::amps_sine(fund);
    let phss = vec![0f32; muls.len()];
    let contour = lifespan::mod_lifespan(contour_length, 1f32, &AmpLifespan::Pluck, 1usize, 0f32);
    let expr = (contour, vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(vec![
            (1f32, lifespan::mod_pad),
        ]),
        None,
        None
    ];
    Element {
        mode: Mode::Melodic,
        amps: vec![1f32; muls.len()],
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
    let contour = lifespan::mod_lifespan(contour_length, 1f32, &AmpLifespan::Pluck, 1usize, 0f32);
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

fn triangle_pluck(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let n_partials = 6;
    let muls = melodic::muls_square(fund);
    let amps = bell::coefficients(fund, n_partials);
    let phss = vec![0f32; muls.len()];
    let contour = lifespan::mod_lifespan(1000usize, 1f32, &AmpLifespan::Burst, 1usize, 0f32);
    let expr = (contour,vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(vec![
            (0.5f32, lifespan::mod_snap),
            (0.5f32, lifespan::mod_pluck),
        ]),
        None,
        None
    ];
    Element {
        mode: Mode::Melodic,
        amps: vec![1f32; muls.len()],
        muls,
        phss,
        modders,
        expr,
        hplp: (vec![MFf], vec![NFf]),
        thresh: (0f32, 1f32)
    }
}

fn noise_pluck(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
    let muls = noise::multipliers(fund, energy);
    let mut rng = rand::thread_rng();
    let phss = vec![0f32; muls.len()];
    let contour = lifespan::mod_lifespan(contour_length, 1f32, &AmpLifespan::Pluck, 1usize, 0f32);
    let expr = (contour, vec![1f32], vec![0f32]);
    let modders:Modders = [
        Some(vec![
            (1f32, lifespan::mod_snap),
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
        hplp: (vec![MFf], vec![NFf]),
        thresh: (0f32, 1f32)
    }
}

pub fn synth() -> Elementor {
    vec![
        (0.01f32, triangle_pluck),
        (0.49f32, noise_pluck),
        (0.35f32, sine_pluck),
        (0.15f32, microtransient_pop),
    ]
}