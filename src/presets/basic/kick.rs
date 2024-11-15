/// A synth snare from three components
use super::*;

fn sine_pluck(fund: f32, vis: &Visibility, energy: &Energy, presence: &Presence) -> Element {
  let amps = melodic::amps_sine(fund);
  let muls = melodic::muls_sine(fund);
  let phss = vec![0f32; muls.len()];
  let expr = (vec![1f32], vec![1f32], vec![0f32]);
  let modders: Modders = [Some(vec![(1f32, lifespan::mod_snap)]), None, None];
  Element {
    mode: Mode::Melodic,
    amps,
    muls,
    phss,
    modders,
    expr,
    hplp: (vec![MFf], vec![NFf]),
    thresh: (0f32, 1f32),
  }
}

fn bell_pluck(fund: f32, vis: &Visibility, energy: &Energy, presence: &Presence) -> Element {
  let n_partials = 6;
  let muls = bell::multipliers(fund, n_partials);
  let amps = bell::coefficients(fund, n_partials);
  let phss = vec![0f32; muls.len()];
  let expr = (vec![0.5f32], vec![1f32], vec![0f32]);
  let modders: Modders = [Some(vec![(1f32, lifespan::mod_db_pluck)]), None, None];
  Element {
    mode: Mode::Bell,
    amps: vec![1f32; muls.len()],
    muls,
    phss,
    modders,
    expr,
    hplp: (vec![MFf], vec![NFf]),
    thresh: (0f32, 1f32),
  }
}

fn triangle_pluck(fund: f32, vis: &Visibility, energy: &Energy, presence: &Presence) -> Element {
  let muls = match energy {
    Energy::High => melodic::muls_triangle(fund),
    Energy::Medium => melodic::muls_square(fund),
    Energy::Low => melodic::muls_square(fund),
  };
  let amps = match energy {
    Energy::High => melodic::amps_sawtooth(fund),
    Energy::Medium => melodic::amps_square(fund),
    Energy::Low => melodic::amps_triangle(fund),
  };
  let phss = vec![0f32; muls.len()];
  let expr = (vec![0.5f32], vec![1f32], vec![0f32]);
  let a_modu: Option<WOldRangerDeprecateds> = Some(match presence {
    Presence::Staccatto => vec![(1f32, lifespan::mod_db_pluck)],
    Presence::Legato => vec![(1f32, lifespan::mod_db_fall)],
    Presence::Tenuto => vec![(0.66f32, lifespan::mod_db_pluck), (0.33f32, lifespan::mod_db_bloom)],
  });
  let modders: Modders = [a_modu, None, None];
  Element {
    mode: Mode::Melodic,
    amps,
    muls,
    phss,
    modders,
    expr,
    hplp: (vec![MFf], vec![NFf]),
    thresh: (0f32, 1f32),
  }
}

fn noise_pluck(fund: f32, vis: &Visibility, energy: &Energy, presence: &Presence) -> Element {
  let muls = noise::multipliers(fund, energy);
  let mut rng = rand::thread_rng();
  let phss = vec![0f32; muls.len()];
  let expr = (vec![1f32], vec![1f32], vec![0f32]);
  let modders: Modders = [Some(vec![(1f32, lifespan::mod_db_pluck)]), None, None];
  Element {
    mode: Mode::Noise,
    amps: vec![0.01f32; muls.len()],
    muls,
    phss,
    modders,
    expr,
    hplp: (vec![MFf], vec![NFf]),
    thresh: (0f32, 1f32),
  }
}

pub fn synth(arf: &Arf) -> Elementor {
  vec![
    (0.6f32, sine_pluck),
    (0.3f32, triangle_pluck),
    (0.06f32, noise_pluck),
    (0.003f32, microtransient_pop),
  ]
}
