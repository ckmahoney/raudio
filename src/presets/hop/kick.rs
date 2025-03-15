use super::*;

/// Returns a `DrumSample` for the kick preset.
///
/// # Parameters
/// - `conf`: Configuration object for additional context.
/// - `melody`: Melody structure specifying note events for the stem.
/// - `arf`: Configuration for amplitude and visibility adjustments.
///
/// # Returns
/// A `DrumSample` with configured sample buffers, amplitude expressions, and effect parameters.
pub fn stemmy<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  // Dynamically retrieve a kick sample file path
  let sample_path = get_sample_path(arf);

  // Read the audio sample from the retrieved path
  let (ref_samples, sample_rate) = read_audio_file(&sample_path).expect("Failed to read kick sample");

  let gain = visibility_gain_sample(arf.visibility);
  let amp_expr = dynamics::gen_organic_amplitude(10, 2000, arf.visibility).iter().map(|v| v * gain).collect();

  let mut rng = thread_rng();
  let delays_note = vec![];
  let delays_room = vec![];
  let reverbs_note = vec![];
  let reverbs_room = vec![];

  let lowpass_cutoff = NFf;
  let ref_sample = ref_samples[0].to_owned();

  Renderable2::Sample((
    melody,
    ref_sample,
    amp_expr,
    lowpass_cutoff,
    delays_note,
    delays_room,
    reverbs_note,
    reverbs_room,
  ))
}

pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  Renderable2::Mix(vec![(0.8, stemmy(conf, melody, arf)), (0.2, synthy(conf, melody, arf))])
}

fn gain(arf: &Arf) -> f32 {
  let x = match arf.presence {
    Presence::Tenuto => 2f32,
    Presence::Legato => 1.5f32,
    Presence::Staccatto => 1f32,
  };

  let mul = match arf.energy {
    Energy::High => 2f32,
    Energy::Medium => 1.5f32,
    Energy::Low => 1f32,
  };

  x * mul
}

fn synthy<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  let soids = druidic_soids::upto(2);

  let expr = (vec![db_to_amp(-4.5f32) * gain(arf)], vec![1f32], vec![0f32]);

  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  let mut rng: ThreadRng = thread_rng();

  // Principal layer
  knob_mods.0.push((
    KnobMacro {
      a: match arf.presence {
        Presence::Staccatto => [0.3f32, 0.5f32],
        Presence::Legato => [0.5f32, 0.9f32],
        Presence::Tenuto => [0.9f32, 1f32],
      },
      b: match arf.energy {
        Energy::High => [0f32, 0.33f32],
        Energy::Medium => [0.33f32, 0.5f32],
        Energy::Low => [0.5f32, 0.66f32],
      },
      c: [0f32, 0f32],
      ma: grab_variant(vec![MacroMotion::Constant]),
      mb: grab_variant(vec![
        MacroMotion::Forward,
        MacroMotion::Reverse,
        MacroMotion::Random,
        MacroMotion::Constant,
      ]),
      mc: MacroMotion::Random,
    },
    if let Presence::Tenuto = arf.presence {
      ranger::amod_burp
    } else {
      ranger::amod_pluck2
    },
  ));

  // Attenuation layer
  knob_mods.0.push((
    KnobMacro {
      a: match arf.presence {
        Presence::Staccatto => [0.5f32, 0.7f32],
        Presence::Legato => [0.7f32, 0.85f32],
        Presence::Tenuto => [0.85f32, 1f32],
      },
      b: match arf.energy {
        Energy::High => [0f32, 0.11f32],
        Energy::Medium => [0.11f32, 0.33f32],
        Energy::Low => [0.33f32, 0.5f32],
      },
      c: [0f32, 0f32],
      ma: grab_variant(vec![MacroMotion::Constant]),
      mb: grab_variant(vec![
        MacroMotion::Forward,
        MacroMotion::Reverse,
        MacroMotion::Random,
        MacroMotion::Constant,
      ]),
      mc: MacroMotion::Random,
    },
    ranger::amod_burp,
  ));

  // Secondary layer
  knob_mods.1.push((
    KnobMacro {
      a: match arf.energy {
        Energy::High => [0f32, 0.75f32],
        Energy::Medium => [0f32, 0.5f32],
        Energy::Low => [0.01f32, 0.1f32],
      },
      b: match arf.presence {
        Presence::Staccatto => [0.1f32, 0.2f32],
        Presence::Legato => [0.1f32, 0.2f32],
        Presence::Tenuto => [0.1f32, 0.4f32],
      },
      c: [0f32, 0f32],
      ma: grab_variant(vec![MacroMotion::Constant]),
      mb: grab_variant(vec![
        MacroMotion::Forward,
        MacroMotion::Reverse,
        MacroMotion::Random,
        MacroMotion::Constant,
      ]),
      mc: MacroMotion::Random,
    },
    ranger::fmod_sweepdown,
  ));

  let delays_note = vec![delay::passthrough];
  let delays_room = vec![];
  let reverbs_note: Vec<ReverbParams> = vec![];
  let reverbs_room: Vec<ReverbParams> = vec![];

  Renderable2::Instance((
    melody,
    soids,
    expr,
    bp2_unit(),
    knob_mods,
    delays_note,
    delays_room,
    reverbs_note,
    reverbs_room,
  ))
}
