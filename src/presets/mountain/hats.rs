use super::*;

/// Returns a `Stem3` for the hats preset.
///
/// # Parameters
/// - `conf`: Configuration object for additional context.
/// - `melody`: Melody structure specifying note events for the stem.
/// - `arf`: Configuration for amplitude and visibility adjustments.
///
/// # Returns
/// A `Stem3` with configured sample buffers, amplitude expressions, and effect parameters.
pub fn stemmy<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  let sample_path = get_sample_path(arf);

  let (ref_samples, sample_rate) = read_audio_file(&sample_path).expect("Failed to read hats sample");
  let gain = visibility_gain_sample(arf.visibility);
  let amp_expr = dynamics::gen_organic_amplitude(10, 2000, arf.visibility).iter().map(|v| v * gain).collect();

  let mut delays_note = vec![];
  let mut reverbs_note = vec![];

  if let Visibility::Foreground = arf.visibility {
    let delay_macros = generate_delay_macros(arf.visibility, arf.energy, arf.presence);
    let mut rng = rand::thread_rng();
    delays_note = delay_macros.iter().map(|mac| mac.gen(&mut rng, conf.cps)).collect();

    reverbs_note = vec![];
  }
  let mut rng = thread_rng();

  let lowpass_cutoff = match arf.energy {
    Energy::Low => NFf / 10f32,
    Energy::Medium => NFf / 8f32,
    Energy::High => NFf / 6f32,
  };
  let ref_sample = ref_samples[0].to_owned();

  Renderable2::Sample((
    melody,
    ref_sample,
    amp_expr,
    lowpass_cutoff,
    delays_note,
    vec![],
    reverbs_note,
    vec![],
  ))
}

pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  // Return the renderable sample
  Renderable2::Mix(vec![
    (0.3, stemmy(conf, melody, arf)),
    (0.3, crate::presets::hill::hats::renderable(conf, melody, arf)),
    (0.2, stemmy(conf, melody, arf)),
  ])
}

/// Generates a set of delay macros for hats in house music.
///
/// # Parameters
/// - `visibility`: Controls gain level for delay feedback.
/// - `energy`: Influences delay density and feedback time.
/// - `presence`: Adjusts delay timing and spatialization.
///
/// # Returns
/// A vector of `DelayParamsMacro` instances.
fn generate_delay_macros(visibility: Visibility, energy: Energy, presence: Presence) -> Vec<DelayParamsMacro> {
  let delay_gain = match visibility {
    Visibility::Hidden => db_to_amp(-24.0),
    Visibility::Background => db_to_amp(-18.0),
    Visibility::Foreground => db_to_amp(-12.0),
    Visibility::Visible => db_to_amp(-6.0),
  };

  let delay_time = match energy {
    Energy::Low => vec![0.125, 0.25],
    Energy::Medium => vec![0.25, 0.5],
    Energy::High => vec![0.5, 1.0],
  };

  let pan_spread = match presence {
    Presence::Staccatto => vec![StereoField::LeftRight(0.9, 0.1)],
    Presence::Legato => vec![StereoField::Mono],
    Presence::Tenuto => vec![StereoField::LeftRight(0.7, 0.3)],
  };

  vec![DelayParamsMacro {
    gain: [delay_gain, delay_gain + 0.05],
    dtimes_cycles: delay_time,
    n_echoes: [2, 4],
    mix: [0.3, 0.5],
    pan: pan_spread,
    mecho: vec![MacroMotion::Forward],
    mgain: vec![MacroMotion::Constant],
    mpan: vec![MacroMotion::Constant],
    mmix: vec![MacroMotion::Constant],
  }]
}
