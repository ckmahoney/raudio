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

/// Create bandpass automations with respect to Arf and Melody
fn bp<'render>(melody: &'render Melody<Note>, arf: &Arf, len_cycles: f32) -> (SampleBuffer, SampleBuffer) {
  let size = len_cycles.log2() - 1f32; // offset 1 to account for lack of CPC. -1 assumes CPC=2
  let rate_per_size = match arf.energy {
    Energy::Low => 0.25f32,
    Energy::Medium => 0.5f32,
    Energy::High => 1f32,
  };

  let mut highest_register: i8 = arf.register;
  let mut lowest_register: i8 = arf.register;
  for line in melody.iter() {
    for (_, (register, _), _) in line.iter() {
      highest_register = (*register).max(highest_register);
      lowest_register = (*register).min(lowest_register);
    }
  }
  let n_samples: usize = (len_cycles / 2f32) as usize * SR;

  let (highpass, lowpass): (Vec<f32>, Vec<f32>) = if let Visibility::Visible = arf.visibility {
    match arf.energy {
      Energy::Low => (
        filter_contour_triangle_shape_highpass(
          lowest_register - 2,
          highest_register - 2,
          n_samples,
          size * rate_per_size,
        ),
        vec![NFf],
      ),
      _ => (
        vec![MFf],
        filter_contour_triangle_shape_lowpass(lowest_register, n_samples, size * rate_per_size),
      ),
    }
  } else {
    (vec![MFf], vec![NFf / 8f32])
  };

  (highpass, lowpass)
}

fn amp_knob_principal(rng: &mut ThreadRng, arf: &Arf) -> KnobPair {
  (
    KnobMacro {
      a: match arf.presence {
        Presence::Staccatto => [0.11f32, 0.3f32],
        Presence::Legato => [0.33f32, 0.5f32],
        Presence::Tenuto => [0.7f32, 0.9f32],
      },
      b: match arf.visibility {
        Visibility::Visible => [0f32, 0.2f32],
        Visibility::Foreground => [0.2f32, 0.3f32],
        _ => [0.3f32, 0.5f32],
      },
      c: [0.0, 0.0],
      ma: grab_variant(vec![MacroMotion::Forward, MacroMotion::Reverse, MacroMotion::Constant]),
      mb: grab_variant(vec![MacroMotion::Forward, MacroMotion::Reverse, MacroMotion::Constant]),
      mc: MacroMotion::Constant,
    },
    ranger::amod_burp,
  )
}

fn amp_knob_attenuation(rng: &mut ThreadRng, arf: &Arf) -> KnobPair {
  (
    KnobMacro {
      a: match arf.energy {
        Energy::High => [0.34f32, 0.5f32],
        Energy::Medium => [0.23f32, 0.34f32],
        Energy::Low => [0.0f32, 0.21f32],
      },
      b: [1f32, 1f32], // Static value as [1, 1]
      c: [0.0, 0.0],   // Static value as [0, 0]
      ma: grab_variant(vec![MacroMotion::Forward, MacroMotion::Reverse, MacroMotion::Constant]),
      mb: MacroMotion::Constant,
      mc: MacroMotion::Constant,
    },
    ranger::amod_detune,
  )
}

pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  let mut rng = thread_rng();
  let len_cycles: f32 = time::count_cycles(&melody[0]);
  let n_samples = (SRf * len_cycles / 2f32) as usize;

  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  knob_mods.0.push(amp_microtransient(arf.visibility, arf.energy, arf.presence));
  knob_mods.0.push(amp_knob_principal(&mut rng, &arf));
  knob_mods.0.push(amp_knob_attenuation(&mut rng, &arf));

  let soids = druidic_soids::overs_square(2f32.powi(7i32));

  let delays_note = vec![delay::passthrough];
  let delays_room = vec![];
  let reverbs_note: Vec<ReverbParams> = vec![];
  let reverbs_room: Vec<ReverbParams> = vec![];

  let stem = (
    melody,
    soids,
    expr(arf, n_samples),
    get_bp(conf.cps, melody, arf, len_cycles),
    knob_mods,
    delays_note,
    delays_room,
    reverbs_note,
    reverbs_room,
  );

  Renderable2::Group(vec![stem])
}
