use crate::analysis::delay::{passthrough, DelayParams, StereoField};
use crate::analysis::in_range;
use crate::analysis::volume::db_to_amp;
use crate::reverb::convolution::ReverbParams;
use crate::types::render::DruidicScoreEntry;
use crate::types::timbre::{
  AmpContour, Arf, Distance, Echo, Enclosure, Energy, Positioning, Presence, SpaceEffects, Visibility,
};
use rand::{self, rngs::ThreadRng, Rng};
// todo
// 1 separate stems by beat vs inst
// 2 apply render process
// 3 put insts in bigger reverb

/// Given a client request for positioning and echoing a melody,
/// produce application parameters to create the effect.
///
/// `enclosure` contributes to reverb and delay time
/// `distance` contributes to gain and reverb
/// `echoes` contributes to delay n artifacts
/// `complexity` contributes to reverb, reverb as saturation, and delay times
pub fn positioning(
  cps: f32, enclosure: &Enclosure, Positioning {
    complexity,
    distance,
    echo,
  }: &Positioning,
) -> SpaceEffects {
  let mut rng = rand::thread_rng();
  let gain: f32 = match distance {
    Distance::Adjacent => 1f32,
    Distance::Near => db_to_amp(-6f32),
    Distance::Far => db_to_amp(-12f32),
  };

  SpaceEffects {
    delays: gen_delays(&mut rng, cps, echo, *complexity),
    reverbs: gen_reverbs(&mut rng, cps, distance, enclosure, *complexity),
    gain,
  }
}

/// Given a client request for positioning and echoing a melody,
/// produce application parameters to create the effect.
///
/// `enclosure` contributes to reverb and delay time
/// `distance` contributes to gain and reverb
/// `echoes` contributes to delay n artifacts
/// `complexity` contributes to reverb, reverb as saturation, and delay times
pub fn create_space_effects(
  cps: f32, enclosure: &Enclosure, complexity: f32, distance: &Distance, echo: &Echo,
) -> SpaceEffects {
  let mut rng = rand::thread_rng();
  let gain: f32 = match distance {
    Distance::Adjacent => 1f32,
    Distance::Near => db_to_amp(-6f32),
    Distance::Far => db_to_amp(-12f32),
  };

  SpaceEffects {
    delays: gen_delays(&mut rng, cps, echo, complexity),
    reverbs: gen_reverbs(&mut rng, cps, distance, enclosure, complexity),
    gain,
  }
}

pub fn gen_delays(rng: &mut ThreadRng, cps: f32, echo: &Echo, complexity: f32) -> Vec<DelayParams> {
  match *echo {
    Echo::None => vec![passthrough],
    Echo::Slapback => vec![gen_slapback(cps, rng, complexity), gen_slapback(cps, rng, complexity)],
    Echo::Trailing => vec![gen_trailing(cps, rng, complexity), gen_trailing(cps, rng, complexity)],
    Echo::Bouncy => {
      let n_copies = 2 + (complexity * 10f32).max(2f32) as usize;
      let mix: f32 = 1f32 / n_copies as f32;
      (0..n_copies)
        .map(|i| {
          if i % 2 == 0 {
            let mut dp = gen_trailing(cps, rng, complexity);
            dp.mix = mix;
            dp
          } else {
            let mut dp = gen_slapback(cps, rng, complexity);
            dp.mix = mix;
            dp
          }
        })
        .collect()
    }
  }
}

/// positioning params applied as convolution parameters to blur or distort the signal
pub fn gen_convolution_stem(
  rng: &mut ThreadRng, arf: &Arf, len_seconds: f32, cps: f32, distance: &Distance, enclosure: &Enclosure,
) -> ReverbParams {
  let v = match arf.visibility {
    Visibility::Visible => 1f32,
    Visibility::Foreground => 0.75f32,
    Visibility::Background => 0.5f32,
    Visibility::Hidden => 0.25f32,
  } - 0.25f32;
  let e = match arf.energy {
    Energy::High => 1f32,
    Energy::Medium => 0.66f32,
    Energy::Low => 0.33f32,
  } - 0.33f32;
  let p = match arf.presence {
    Presence::Tenuto => 1f32,
    Presence::Legato => 0.66f32,
    Presence::Staccatto => 0.33f32,
  } - 0.33f32;

  let complexity = ((v + e + p) / 3f32).powf(3f32 / 2f32);
  reverb_params_stem(rng, len_seconds, cps, distance, enclosure, complexity)
}

/// reverb_params
pub fn reverb_params(
  rng: &mut ThreadRng, total_len_seconds: f32, cps: f32, distance: &Distance, enclosure: &Enclosure, complexity: f32,
) -> ReverbParams {
  // amp correlates to size of reverb/space. bigger means more distortion, scales exponentially
  // Appears that values up to 1/12 offer a mild effect,
  // whereas values above 1/6 clearly distort the signal.
  let mut amp: f32 = if complexity < 0.25f32 {
    in_range(rng, 0f32, 1f32 / 24f32)
  } else if complexity < 0.5f32 {
    in_range(rng, 1f32 / 24f32, 1f32 / 12f32)
  } else if complexity < 0.75f32 {
    in_range(rng, 1f32 / 12f32, 1f32 / 6f32)
  } else {
    in_range(rng, 1f32 / 6f32, 1f32 / 3f32)
  };

  // decay correlates to transient preservation (blur)
  // for dur=8, the decay becomes very blury when decay >= 0.9
  let decay = match distance {
    Distance::Far => in_range(rng, 0.8f32, 0.95f32),
    Distance::Adjacent => in_range(rng, 0.4f32, 0.7f32),
    Distance::Near => in_range(rng, 0.05f32, 0.05f32),
  };

  // duration translate to intensity of effect
  // this also scales with the signal! Must be with respect to total_len_seconds
  // currently I know this is going to be applied twice... once at the line layer, and once at the mix layer.
  // hence the division by four factor
  let reverb_signal_base_length = total_len_seconds / (4f32 * cps);
  let dur: f32 = reverb_signal_base_length
    * match enclosure {
      Enclosure::Spring => in_range(rng, 0.1f32, 0.5f32),
      Enclosure::Room => in_range(rng, 0.5f32, 1f32),
      Enclosure::Hall => in_range(rng, 1f32, 2f32),
      Enclosure::Vast => in_range(rng, 2f32, 3f32),
    };

  ReverbParams {
    mix: in_range(rng, 0.08, 0.12),
    amp,
    dur,
    rate: decay,
  }
}

/// reverb_params
pub fn reverb_params_stem(
  rng: &mut ThreadRng, total_len_seconds: f32, cps: f32, distance: &Distance, enclosure: &Enclosure, complexity: f32,
) -> ReverbParams {
  // amp correlates to size of reverb/space. bigger means more distortion, scales exponentially
  // Appears that values up to 1/12 offer a mild effect,
  // whereas values above 1/6 clearly distort the signal.
  let mut amp: f32 = if complexity < 0.25f32 {
    in_range(rng, 0f32, 1f32 / 24f32)
  } else if complexity < 0.5f32 {
    in_range(rng, 1f32 / 24f32, 1f32 / 12f32)
  } else if complexity < 0.75f32 {
    in_range(rng, 1f32 / 12f32, 1f32 / 6f32)
  } else {
    in_range(rng, 1f32 / 6f32, 1f32 / 3f32)
  };

  // decay correlates to transient preservation (blur)
  // for dur=8, the decay becomes very blury when decay >= 0.9
  let decay = match distance {
    Distance::Far => in_range(rng, 0.9f32, 1f32),
    Distance::Adjacent => in_range(rng, 0.5f32, 0.9f32),
    Distance::Near => in_range(rng, 0.05f32, 0.2f32),
  };

  // duration translate to intensity of effect
  // this also scales with the signal! Must be with respect to total_len_seconds
  // currently I know this is going to be applied twice... once at the line layer, and once at the mix layer.
  // hence the division by four factor
  let reverb_signal_base_length = total_len_seconds / (8f32 * cps);
  let dur: f32 = reverb_signal_base_length
    * match enclosure {
      Enclosure::Spring => in_range(rng, 0.1f32, 1f32),
      Enclosure::Room => in_range(rng, 1f32, 2f32),
      Enclosure::Hall => in_range(rng, 2f32, 4f32),
      Enclosure::Vast => in_range(rng, 4f32, 8f32),
    };

  ReverbParams {
    mix: complexity.powf(0.5f32) / 9f32,
    amp,
    dur,
    rate: decay,
  }
}

/// Create a saturation layer and room layer
pub fn gen_reverbs(
  rng: &mut ThreadRng, cps: f32, distance: &Distance, enclosure: &Enclosure, complexity: f32,
) -> Vec<ReverbParams> {
  println!("Called the old reverb method. returning empty.");

  vec![]
}

fn gen_saturation(cps: f32, complexity: f32) -> ReverbParams {
  ReverbParams {
    mix: 0.5f32,
    rate: 0.005 * complexity / cps,
    dur: complexity / cps,
    amp: complexity.powi(3),
  }
}

/// Very transparent and live sounding room effect
fn gen_spring(distance: &Distance) -> ReverbParams {
  ReverbParams {
    mix: 0.05,
    amp: 1f32,
    dur: 1f32,
    rate: 1f32,
  }
}

/// short delay with loud echo
/// works best with percussive or plucky sounds
fn gen_slapback(cps: f32, rng: &mut ThreadRng, complexity: f32) -> DelayParams {
  let n_echoes = if complexity < 0.5f32 { 2 } else { 3 };
  let rate = 2f32.powi(-rng.gen_range(0..4) as i32);
  let len_seconds: f32 = rate / cps;
  let gain: f32 = db_to_amp(-3f32) + rng.gen::<f32>() * db_to_amp(-1f32);
  let pan = StereoField::Mono;
  DelayParams {
    mix: 0.5f32,
    len_seconds,
    n_echoes,
    gain,
    pan,
  }
}

/// longer delay with fading echoes
fn gen_trailing(cps: f32, rng: &mut ThreadRng, complexity: f32) -> DelayParams {
  let n_echoes = if complexity < 0.33f32 {
    rng.gen_range(4..7)
  } else if complexity < 0.66 {
    rng.gen_range(5..9)
  } else {
    rng.gen_range(6..11)
  };

  // choose delay lengths that are probably more than one cycle,
  // and likely to be syncopated.
  let factor = 1.5f32 * rng.gen_range(1..4) as f32;
  let rate = factor / rng.gen_range(1..9) as f32;
  let len_seconds: f32 = rate / cps;
  let gain: f32 = db_to_amp(-6f32) + (db_to_amp(-6f32) * rng.gen::<f32>() / 3f32);
  let mix: f32 = 0.5f32;
  DelayParams {
    mix,
    len_seconds,
    n_echoes,
    gain,
    pan: StereoField::Mono,
  }
}

/// simple contour generator intended to be used in a bandpass frequency context.
pub fn gen_bp_contour(n: usize, freq1: f32, freq2: f32, n_samples: usize) -> Vec<f32> {
  let mut rng = rand::thread_rng();

  let mut checkpoints: Vec<(f32, f32, f32)> = (0..n)
    .map(|_| {
      let p = rng.gen_range(0.0..=1.0);
      let v = rng.gen_range(0.5..=1.0); // Randomized v in [0, 1]
      let contour = 2f32 * rng.gen_range(-0.5..=0.5); // Randomized contour in [-1, 1]
      (p, v, contour)
    })
    .collect();

  crate::analysis::freq::render_checkpoints(&checkpoints, freq1, freq2, n_samples)
}
