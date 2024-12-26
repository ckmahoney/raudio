use super::*;
use crate::druid::{self, soid_fx, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::types::synthesis::{ModifiersHolder, Soids};

pub fn expr(arf: &Arf, n_samples: usize) -> Expr {
  let gain = visibility_gain(arf.visibility);
  let mut dynamics = dynamics::gen_organic_amplitude(10, n_samples, arf.visibility);
  amp_scale(&mut dynamics, gain);
  (dynamics, vec![1f32], vec![0f32])
}

/// Generates a set of distinct delay macros for bass in ambient music, with VEP-based parameters
/// for each style. Each macro represents a different bass delay trope, suitable for creating depth,
/// grounding, or rhythmic texture in the low end.
///
/// # Arguments
/// - `visibility`: Controls gain level, affecting how prominent or subtle the delay is.
/// - `energy`: Determines the number of echoes, adding density for higher energy levels.
/// - `presence`: Adjusts delay cycle lengths to match the bass character (shorter for more defined, rhythmic delays).
///
/// # Returns
/// A vector of `DelayParamsMacro` instances, each representing a different bass delay style.
fn generate_bass_delay_macros(visibility: Visibility, energy: Energy, presence: Presence) -> Vec<DelayParamsMacro> {
  // Adjust gain level based on visibility
  let gain_level = match visibility {
    Visibility::Hidden => db_to_amp(-15.0),
    Visibility::Background => db_to_amp(-12.0),
    Visibility::Foreground => db_to_amp(-9.0),
    Visibility::Visible => db_to_amp(-6.0),
  };

  // Determine echo counts based on energy level for density control
  let n_echoes_range = match energy {
    Energy::Low => [2, 3],
    Energy::Medium => [3, 5],
    Energy::High => [5, 7],
  };

  // Set delay cycle lengths based on presence, making it shorter for clearer bass delay textures
  let dtimes_cycles = match presence {
    Presence::Staccatto => vec![0.25, 0.5, 0.75, 1.0], // Short cycles for rhythmic texture
    Presence::Legato => vec![0.5, 1.0, 1.5],           // Medium cycles for smooth flow
    Presence::Tenuto => vec![1.0, 2.0, 3.0],           // Longer cycles for spacious, evolving bass
  };

  // 1. Classic Mono Bass Delay
  // Provides a grounded, centered bass presence with shorter delay times.
  let classic_mono_bass = DelayParamsMacro {
    gain: [gain_level * 0.8, gain_level], // Lower gain for subtle presence
    dtimes_cycles: dtimes_cycles.clone(),
    n_echoes: n_echoes_range,
    mix: [0.2, 0.4],              // Low mix to keep it supportive and centered
    pan: vec![StereoField::Mono], // Centered, mono delay
    mecho: vec![MacroMotion::Forward],
    mgain: vec![MacroMotion::Constant],
    mpan: vec![MacroMotion::Constant],
    mmix: vec![MacroMotion::Constant],
  };

  // 2. Wide Stereo Bass Delay
  // Adds width to the bass, creating a subtle stereo spread for a supportive low-end layer.
  let wide_stereo_bass = DelayParamsMacro {
    gain: [gain_level, gain_level + 0.1], // Slightly higher gain for stereo presence
    dtimes_cycles: dtimes_cycles.clone(),
    n_echoes: n_echoes_range,
    mix: [0.3, 0.5],                             // Moderate mix to provide noticeable width
    pan: vec![StereoField::LeftRight(0.6, 0.6)], // Subtle stereo panning for depth
    mecho: vec![MacroMotion::Forward],
    mgain: vec![MacroMotion::Constant],
    mpan: vec![MacroMotion::Constant],
    mmix: vec![MacroMotion::Constant],
  };

  // 3. Rhythmic Bass Delay
  // Adds rhythmic texture with varied delay cycles and slightly higher mix, ideal for dynamic bass.
  let rhythmic_bass_delay = DelayParamsMacro {
    gain: [gain_level, gain_level * 1.1], // Slightly boosted gain for rhythmic emphasis
    dtimes_cycles: match presence {
      Presence::Staccatto => vec![0.25, 0.5, 1.0], // Short cycles for rhythmic pulsing
      Presence::Legato => vec![0.5, 1.0, 1.5, 2.0], // Medium cycles for smooth rhythm
      Presence::Tenuto => vec![1.0, 2.0, 3.0, 4.0], // Longer cycles for evolving, spacious rhythm
    },
    n_echoes: n_echoes_range,
    mix: [0.4, 0.6],              // Higher mix to enhance rhythmic movement
    pan: vec![StereoField::Mono], // Mono for a centered, rhythmic bass
    mecho: vec![MacroMotion::Forward],
    mgain: vec![MacroMotion::Constant],
    mpan: vec![MacroMotion::Constant],
    mmix: vec![MacroMotion::Constant],
  };

  vec![classic_mono_bass, wide_stereo_bass, rhythmic_bass_delay]
}

fn amp_knob_principal(rng: &mut ThreadRng, arf: &Arf) -> KnobPair {
  (
    KnobMacro {
      a: match arf.presence {
        Presence::Staccatto => [0.2f32, 0.4f32],
        Presence::Legato => [0.33f32, 0.66f32],
        Presence::Tenuto => [0.7f32, 1f32],
      },
      b: match arf.visibility {
        Visibility::Visible => [0f32, 0.2f32],
        Visibility::Foreground => [0.2f32, 0.3f32],
        _ => [0.3f32, 0.5f32],
      },
      c: [0.0, 0.0],
      ma: grab_variant(vec![
        MacroMotion::Forward,
        MacroMotion::Reverse,
        MacroMotion::Random,
        MacroMotion::Constant,
      ]),
      mb: grab_variant(vec![MacroMotion::Forward, MacroMotion::Reverse, MacroMotion::Constant]),
      mc: MacroMotion::Constant,
    },
    ranger::amod_pluck2,
  )
}

pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  let mut rng = thread_rng();
  let len_cycles: f32 = time::count_cycles(&melody[0]);
  let n_samples = (SRf * len_cycles / 2f32) as usize;

  let soids = druidic_soids::overs_square(get_mullet(&arf));

  let delays_note = vec![];
  let delays_room = vec![];
  let reverbs_note: Vec<ReverbParams> = vec![];
  let reverbs_room: Vec<ReverbParams> = vec![];

  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  knob_mods.0.push(amp_onset(arf.visibility, arf.energy, arf.presence));
  knob_mods.0.push(amp_knob_principal(&mut rng, &arf));

  let stem = (
    melody,
    soids,
    expr(arf, n_samples),
    ValleyCon::get_bp(conf.cps, melody, arf),
    knob_mods,
    delays_note,
    delays_room,
    reverbs_note,
    reverbs_room,
  );

  Renderable2::Instance(stem)
}
