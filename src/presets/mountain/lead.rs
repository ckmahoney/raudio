use super::*;
use crate::analysis::in_range_usize;
use crate::druid::{self, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::types::synthesis::{ModifiersHolder, Soids};
use std::os::unix::thread;

/// Generates a set of delay macros for lead lines in ambient music using VEP parameters.
/// The macros adapt based on `Visibility`, `Energy`, and `Presence`, tailoring delay characteristics
/// to create subtle, floating echoes that emphasize the melody.
///
/// # Arguments
/// - `visibility`: Controls gain level for overall presence.
/// - `energy`: Controls echo density (number of echoes).
/// - `presence`: Adjusts delay cycle lengths and mix.
///
/// # Returns
/// A vector of `DelayParamsMacro` instances for ambient lead delay styles.
fn generate_lead_delay_macros(visibility: Visibility, energy: Energy, presence: Presence) -> Vec<DelayParamsMacro> {
  // Adjust gain level based on visibility for lead presence
  let gain_level = match visibility {
    Visibility::Hidden => db_to_amp(-15.0),
    Visibility::Background => db_to_amp(-12.0),
    Visibility::Foreground => db_to_amp(-9.0),
    Visibility::Visible => db_to_amp(-6.0),
  };

  // Define echo counts based on energy for a supportive ambient effect
  let n_echoes_range = match energy {
    Energy::Low => [4, 5],
    Energy::Medium => [6, 8],
    Energy::High => [8, 10],
  };

  // Define cycle lengths based on presence
  let dtimes_cycles = match presence {
    Presence::Staccatto => vec![0.5, 1.0, 1.5],   // Shorter, rhythmic cycles
    Presence::Legato => vec![1.0, 2.0, 3.0],      // Medium cycles for a flowing feel
    Presence::Tenuto => vec![2.0, 3.0, 4.5, 5.0], // Longer cycles for spacious echoes
  };

  // 1. Sparse Mono Lead Delay (Centered, subtle delay for delicate, floating lead notes)
  let sparse_mono_lead = DelayParamsMacro {
    gain: [gain_level, gain_level * 1.1], // Subtle, supportive gain
    dtimes_cycles: dtimes_cycles.clone(),
    n_echoes: n_echoes_range,
    mix: [0.3, 0.5],              // Lower mix for a background floating effect
    pan: vec![StereoField::Mono], // Mono to keep it centered and minimal
    mecho: vec![MacroMotion::Forward],
    mgain: vec![MacroMotion::Constant],
    mpan: vec![MacroMotion::Constant],
    mmix: vec![MacroMotion::Constant],
  };

  // 2. Wide Stereo Lead Delay (Stereo depth for lead notes, giving a floating, spacious quality)
  let wide_stereo_lead = DelayParamsMacro {
    gain: [gain_level, gain_level * 1.15], // Higher gain for a clearer presence
    dtimes_cycles: dtimes_cycles.clone(),
    n_echoes: n_echoes_range,
    mix: [0.5, 0.7],                             // Stronger mix for spacious stereo presence
    pan: vec![StereoField::LeftRight(0.7, 0.7)], // Wide stereo for floating effect
    mecho: vec![MacroMotion::Forward],
    mgain: vec![MacroMotion::Constant],
    mpan: vec![MacroMotion::Constant],
    mmix: vec![MacroMotion::Constant],
  };

  // 3. Rhythmic Lead Delay (Adds rhythmic, subtle movement to lead notes with varied cycles)
  let rhythmic_lead_delay = DelayParamsMacro {
    gain: [gain_level, gain_level * 1.2], // Higher gain for rhythmic emphasis
    dtimes_cycles,                        // Varied cycles to add rhythm
    n_echoes: [n_echoes_range[0], n_echoes_range[1]], // Range allows flexibility
    mix: [0.6, 0.8],                      // Stronger mix to bring out rhythmic delays
    pan: vec![StereoField::LeftRight(0.5, 0.5)], // Moderate stereo for added interest
    mecho: vec![MacroMotion::Forward],
    mgain: vec![MacroMotion::Constant],
    mpan: vec![MacroMotion::Constant],
    mmix: vec![MacroMotion::Constant],
  };

  vec![sparse_mono_lead, wide_stereo_lead, rhythmic_lead_delay]
}

fn pmod_chorus(v: Visibility, e: Energy, p: Presence) -> KnobPair {
  let mut rng = thread_rng();

  let modulation_depth = match v {
    Visibility::Hidden => [0f32, 0.33f32],
    Visibility::Background => [0.33, 0.5],
    Visibility::Foreground => [0.5, 0.75],
    Visibility::Visible => [0.75f32, 1f32],
  };

  let chorus_visibility = match v {
    Visibility::Hidden => [0f32, 0f32],
    Visibility::Background => [0.1, 0.1 + 0.5 * rng.gen::<f32>()],
    Visibility::Foreground => [0.6, 0.6 + 0.2 * rng.gen::<f32>()],
    Visibility::Visible => [0.8, 0.8 + 0.1 * rng.gen::<f32>()],
  };

  let chorus_visibility = [0.5f32, 1f32];
  let modulation_depth = [0.1f32, 1f32];
  let intensity = [0.5 * 1f32 / 3f32, 0.5 * 1f32 / 3f32];

  (
    KnobMacro {
      a: chorus_visibility,
      b: modulation_depth,
      c: intensity,
      ma: grab_variant(vec![MacroMotion::Forward, MacroMotion::Reverse, MacroMotion::Constant]),
      mb: grab_variant(vec![MacroMotion::Forward, MacroMotion::Reverse, MacroMotion::Constant]),
      mc: grab_variant(vec![MacroMotion::Forward, MacroMotion::Reverse, MacroMotion::Constant]),
    },
    ranger::pmod_chorus2,
  )
}

fn amp_knob_pluck(visibility: Visibility, energy: Energy, presence: Presence) -> KnobPair {
  let mut rng = thread_rng();

  let amp_decay = match energy {
    Energy::Low => [0.5f32, 0.5 + 0.5 * rng.gen::<f32>()],
    Energy::Medium => [0.3f32, 0.3 + 0.5 * rng.gen::<f32>()],
    Energy::High => [0.2f32, 0.2 + rng.gen::<f32>() / 3f32],
  };

  let energy_decay = match energy {
    Energy::Low => [0.5f32, 0.5 + 0.5 * rng.gen::<f32>()],
    Energy::Medium => [0.3f32, 0.3 + 0.5 * rng.gen::<f32>()],
    Energy::High => [0.2f32, 0.2 + rng.gen::<f32>() / 3f32],
  };

  (
    KnobMacro {
      a: amp_decay,
      b: energy_decay,
      c: [0f32, 0f32],
      ma: grab_variant(vec![
        MacroMotion::Forward,
        MacroMotion::Reverse,
        MacroMotion::Constant,
        MacroMotion::Min,
        MacroMotion::Mean,
        MacroMotion::Max,
      ]),
      mb: grab_variant(vec![
        MacroMotion::Forward,
        MacroMotion::Reverse,
        MacroMotion::Constant,
        MacroMotion::Min,
        MacroMotion::Mean,
        MacroMotion::Max,
      ]),
      mc: MacroMotion::Constant,
    },
    ranger::amod_pluck2,
  )
}

fn amp_knob_burp(visibility: Visibility, energy: Energy, presence: Presence) -> KnobPair {
  let mut rng = thread_rng();

  let amp_decay = match visibility {
    Visibility::Visible => [0.5f32, 0.5 + 0.5 * rng.gen::<f32>()],
    Visibility::Foreground => [0.3f32, 0.3 + 0.5 * rng.gen::<f32>()],
    Visibility::Background => [0.2f32, 0.2 + rng.gen::<f32>() / 3f32],
    Visibility::Hidden => [0f32, rng.gen::<f32>() / 3f32],
  };

  let energy_decay = match energy {
    Energy::Low => [0.5f32, 0.5 + 0.5 * rng.gen::<f32>()],
    Energy::Medium => [0.3f32, 0.3 + 0.5 * rng.gen::<f32>()],
    Energy::High => [0.2f32, 0.2 + rng.gen::<f32>() / 3f32],
  };

  (
    KnobMacro {
      a: amp_decay,
      b: energy_decay,
      c: [0f32, 0f32],
      ma: grab_variant(vec![
        MacroMotion::Forward,
        MacroMotion::Reverse,
        MacroMotion::Min,
        MacroMotion::Mean,
        MacroMotion::Max,
      ]),
      mb: grab_variant(vec![
        MacroMotion::Forward,
        MacroMotion::Reverse,
        MacroMotion::Min,
        MacroMotion::Mean,
        MacroMotion::Max,
      ]),
      mc: MacroMotion::Random,
    },
    ranger::amod_burp,
  )
}

fn amp_knob(visibility: Visibility, energy: Energy, presence: Presence) -> KnobPair {
  if let Presence::Staccatto = presence {
    return amp_knob_pluck(visibility, energy, presence);
  }
  return amp_knob_burp(visibility, energy, presence);
}

fn freq_knob_tonal(v: Visibility, e: Energy, p: Presence) -> KnobPair {
  let mut rng = thread_rng();
  let modulation_amount = match e {
    Energy::Low => [0.005f32, 0.005f32 + 0.05 * rng.gen::<f32>()],
    Energy::Medium => [0.08f32, 0.1f32 + 0.12f32 * rng.gen::<f32>()],
    Energy::High => [0.05f32, 0.25f32 * rng.gen::<f32>()],
  };

  (
    KnobMacro {
      a: modulation_amount,
      b: [0f32, 0f32],
      c: [0f32, 0f32],
      ma: grab_variant(vec![MacroMotion::Forward, MacroMotion::Reverse]),
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    ranger::fmod_warble,
  )
}

/// Generates a vector of `DelayParams` tailored for an ambient lead synth, with different settings
/// based on `Visibility`, `Energy`, and `Presence`.
///
/// - `Visibility` affects the number of delays and their prominence.
/// - `Energy` controls gain levels for each delay.
/// - `Presence` influences delay times and spacing to suit the style of articulation.
///
/// # Arguments
/// - `visibility`: Determines the prominence and number of delays.
/// - `energy`: Controls the intensity of delay gain.
/// - `presence`: Sets delay times and spacing for articulation (staccato, legato, tenuto).
/// - `cps`: Cycles per second, used to convert delay ratios to seconds.
///
/// # Returns
/// A vector of `DelayParams` with settings customized for a sparse, evolving ambient lead.
fn gen_delays(visibility: Visibility, energy: Energy, presence: Presence, cps: f32) -> Vec<DelayParams> {
  let mut rng = thread_rng();

  // Determine the number of delays based on visibility
  let n_delays = match visibility {
    Visibility::Visible => in_range_usize(&mut rng, 3, 5),
    Visibility::Foreground => in_range_usize(&mut rng, 2, 4),
    Visibility::Background => in_range_usize(&mut rng, 1, 3),
    _ => 0,
  };

  if n_delays == 0 {
    return vec![];
  }

  // Define gain levels based on visibility, using `db_to_amp` to convert dB values to amplitude
  let gain_level = match visibility {
    Visibility::Hidden => db_to_amp(-15.0),
    Visibility::Background => db_to_amp(-12.0),
    Visibility::Foreground => db_to_amp(-9.0),
    Visibility::Visible => db_to_amp(-6.0),
  };

  // Define echo counts based on energy
  let n_echoes_range = match energy {
    Energy::Low => [4, 6],
    Energy::Medium => [6, 8],
    Energy::High => [8, 10],
  };

  // Define cycle lengths based on presence
  let dtimes_cycles = match presence {
    Presence::Staccatto => vec![0.25, 0.5, 0.75], // Short cycle lengths for quick echoes
    Presence::Legato => vec![1.0, 2.0, 2.5],      // Medium cycle lengths for smooth echoes
    Presence::Tenuto => vec![3.0, 4.0, 5.0],      // Long cycle lengths for extended echoes
  };

  // Define macros for stereo and mono delays using adjusted parameters
  let stereo_macro = DelayParamsMacro {
    gain: [gain_level, gain_level + 0.1], // Slight gain variation
    dtimes_cycles: dtimes_cycles.clone(), // Use defined cycle lengths
    n_echoes: n_echoes_range,
    mix: [0.6, 0.8], // Stronger mix for stereo presence
    pan: vec![StereoField::LeftRight(0.8, 0.8)],
    mecho: vec![MacroMotion::Forward],
    mgain: vec![MacroMotion::Constant],
    mpan: vec![MacroMotion::Constant],
    mmix: vec![MacroMotion::Constant],
  };

  let mono_delay_macro = DelayParamsMacro {
    gain: [gain_level * 0.8, gain_level], // Slightly lower gain for mono
    dtimes_cycles,                        // Use defined cycle lengths
    n_echoes: n_echoes_range,
    mix: [0.4, 0.6], // Moderate mix for subtle mono delays
    pan: vec![StereoField::Mono],
    mecho: vec![MacroMotion::Forward],
    mgain: vec![MacroMotion::Constant],
    mpan: vec![MacroMotion::Constant],
    mmix: vec![MacroMotion::Constant],
  };

  // Generate delays using the macros, selecting randomly for variety
  let mut delays = Vec::new();
  for _ in 0..n_delays {
    let delay = match rng.gen_range(0..2) {
      0 => stereo_macro.gen(&mut rng, cps),
      _ => mono_delay_macro.gen(&mut rng, cps),
    };
    delays.push(delay);
  }

  delays
}

pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  let mullet = match arf.energy {
    Energy::Low => 2f32.powi(11i32),
    Energy::Medium => 2f32.powi(9i32),
    Energy::High => 2f32.powi(7i32),
  };

  let soids = match arf.visibility {
    Visibility::Hidden => druidic_soids::octave(mullet),
    Visibility::Background => druidic_soids::overs_triangle(mullet),
    Visibility::Foreground => druidic_soids::overs_square(mullet),
    Visibility::Visible => druidic_soids::overs_sawtooth(mullet),
  };

  let soids = druidic_soids::octave(mullet);

  let mut knob_mods: KnobMods2 = KnobMods2::unit();

  knob_mods.0.push(amp_onset(arf.visibility, arf.energy, arf.presence));
  knob_mods.0.push(amp_knob(arf.visibility, arf.energy, arf.presence));
  knob_mods.2.push(pmod_chorus(arf.visibility, arf.energy, arf.presence));

  let len_cycles: f32 = time::count_cycles(&melody[0]);
  let n_samples = (SRf * len_cycles / 2f32) as usize;

  let mut dynamics = dynamics::gen_organic_amplitude(10, n_samples, arf.visibility);
  amp_scale(&mut dynamics, visibility_gain(arf.visibility));

  let expr = (dynamics, vec![1f32], vec![0f32]);

  let delays_note = gen_delays(arf.visibility, arf.energy, arf.presence, conf.cps);
  let delays_room = vec![];
  let reverbs_note: Vec<ReverbParams> = vec![];
  let reverbs_room: Vec<ReverbParams> = vec![];

  let stem = (
    melody,
    soids,
    expr,
    MountainCon::get_bp(conf.cps, melody, arf),
    knob_mods,
    delays_note,
    delays_room,
    reverbs_note,
    reverbs_room,
  );

  Renderable2::Instance(stem)
}
