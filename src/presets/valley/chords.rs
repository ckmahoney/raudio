use super::*;
use hound::Sample;

use crate::analysis::melody::{find_reach, mask_sigh, mask_wah, LevelMacro, Levels, ODRMacro, ODR};
use crate::druid::{self, soids as druidic_soids};
use crate::monic_theory::note_to_freq;
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::time;
use crate::types::synthesis::{BoostGroup, BoostGroupMacro, ModifiersHolder, Soids};

/// Generates a set of delay macros for chord textures in ambient music, utilizing VEP parameters.
/// Each macro represents a different chord delay style, designed to create wide, immersive textures
/// that enhance the ambient mix's depth and spatial quality.
///
/// # Arguments
/// - `visibility`: Controls gain level, adjusting the overall prominence of the chords.
/// - `energy`: Influences echo density, layering more echoes for higher energy levels.
/// - `presence`: Adjusts delay cycle lengths to achieve various spatial effects.
///
/// # Returns
/// A vector of `DelayParamsMacro` instances for ambient chord delay styles.
fn generate_chord_delay_macros(visibility: Visibility, energy: Energy, presence: Presence) -> Vec<DelayParamsMacro> {
  // Determine gain level based on visibility to set chord presence in the mix
  let gain_level = match visibility {
    Visibility::Hidden => db_to_amp(-12.0),
    Visibility::Background => db_to_amp(-9.0),
    Visibility::Foreground => db_to_amp(-6.0),
    Visibility::Visible => db_to_amp(-3.0),
  };

  // Adjust echo density based on energy level for layering control
  let n_echoes_range = match energy {
    Energy::Low => [3, 4],
    Energy::Medium => [4, 6],
    Energy::High => [6, 8],
  };

  // Set delay cycle lengths based on presence, adding variety to the spatial effect
  let dtimes_cycles = match presence {
    Presence::Staccatto => vec![1f32/8f32, 1f32/4f32, 1f32/2f32],
    Presence::Legato => vec![1f32/8f32, 1f32/4f32, 1f32/3f32, 2f32/3f32, 1f32, 3f32/2f32, 2f32], // Medium cycles for smooth, sustained echoes
    Presence::Tenuto => vec![1.333, 1.5, 2.0, 3.0], // Longer cycles for a more spacious feel
  };

  // 1. Wide Stereo Pad Delay
  // Creates a wide, lush stereo spread for ambient chord textures.
  let wide_stereo_pad = DelayParamsMacro {
    gain: [gain_level, gain_level + 0.1], // Slight gain increase for depth
    dtimes_cycles: dtimes_cycles.clone(),
    n_echoes: n_echoes_range,
    mix: [0.5, 0.7],                             // Stronger mix for an immersive stereo effect
    pan: vec![StereoField::LeftRight(0.7, 0.7)], // Wide stereo spread for spatial depth
    mecho: vec![MacroMotion::Forward],
    mgain: vec![MacroMotion::Constant],
    mpan: vec![MacroMotion::Constant],
    mmix: vec![MacroMotion::Constant],
  };

  // 2. Smooth Mono Pad Delay
  // Provides a centered, smooth delay that blends chords for a cohesive background layer.
  let smooth_mono_pad = DelayParamsMacro {
    gain: [gain_level * 0.8, gain_level], // Slightly lower gain for blending
    dtimes_cycles: dtimes_cycles.clone(),
    n_echoes: n_echoes_range,
    mix: [0.4, 0.6],              // Balanced mix to maintain smoothness
    pan: vec![StereoField::Mono], // Mono to keep it centered and cohesive
    mecho: vec![MacroMotion::Forward],
    mgain: vec![MacroMotion::Constant],
    mpan: vec![MacroMotion::Constant],
    mmix: vec![MacroMotion::Constant],
  };

  // 3. Evolving Pad Delay
  // Adds long, varied delay cycles to create a slowly evolving, immersive chord texture.
  let evolving_pad_delay = DelayParamsMacro {
    gain: [gain_level, gain_level + 0.2], // Slightly increased gain for evolving textures
    dtimes_cycles: match presence {
      Presence::Staccatto => vec![1.0, 1.5, 2.0], // Moderate cycles for subtle movement
      Presence::Legato => vec![2.0, 3.0, 4.0],    // Longer cycles for smooth flow
      Presence::Tenuto => vec![3.0, 4.0, 5.0, 6.0], // Longest cycles for gradual evolution
    },
    n_echoes: [n_echoes_range[0], n_echoes_range[1] + 1], // Slightly more echoes for layering
    mix: [0.5, 0.8],                                      // Higher mix for a prominent ambient presence
    pan: vec![StereoField::LeftRight(0.6, 0.6)],          // Moderate stereo spread for depth
    mecho: vec![MacroMotion::Forward],
    mgain: vec![MacroMotion::Constant],
    mpan: vec![MacroMotion::Constant],
    mmix: vec![MacroMotion::Constant],
  };

  vec![wide_stereo_pad, smooth_mono_pad, evolving_pad_delay]
}

fn amp_knob_presence(visibility: Visibility, energy: Energy, presence: Presence) -> KnobPair {
  let mut rng = thread_rng();

  let decay_length = match visibility {
    Visibility::Visible => [0.33f32, 0.33 + 0.5 * rng.gen::<f32>()],
    Visibility::Foreground => [0.33f32, 0.33 + 0.33 * rng.gen::<f32>()],
    Visibility::Background => [0.2, 0.2 + 0.2 * rng.gen::<f32>()],
    Visibility::Hidden => [0.1f32, 0.2f32],
  };
  let detune_rate = match energy {
    Energy::Low => [0.5f32, 0.5 + rng.gen::<f32>() / 2f32],
    Energy::Medium => [0.33f32, 0.33 + 2f32 * rng.gen::<f32>() / 3f32],
    Energy::High => [0f32, rng.gen::<f32>() / 6f32],
  };

  return (
    KnobMacro {
      a: decay_length,
      b: [0f32, 0f32],
      c: [0.2f32, 1f32],
      ma: grab_variant(vec![MacroMotion::Forward, MacroMotion::Reverse, MacroMotion::Constant]),
      mb: MacroMotion::Constant, // unused
      mc: grab_variant(vec![MacroMotion::Forward, MacroMotion::Reverse, MacroMotion::Constant]),
    },
    ranger::amod_fall,
  );
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
      mc: grab_variant(vec![
        MacroMotion::Forward,
        MacroMotion::Reverse,
        MacroMotion::Constant,
        MacroMotion::Min,
        MacroMotion::Mean,
        MacroMotion::Max,
      ]),
    },
    ranger::pmod_chorus2,
  )
}

/// Generate a phrase length filter contour for the given melody and arf.
pub fn filter_contour_linear_rise<'render>(
  melody: &'render Melody<Note>, arf: &Arf, n_samples: usize,
) -> (SampleBuffer, SampleBuffer) {
  let len_cycles: f32 = time::count_cycles(&melody[0]);

  let mut highpass_contour: SampleBuffer = vec![MFf; n_samples];
  let mut lowpass_contour: SampleBuffer = Vec::with_capacity(n_samples);

  // the default position of the lowpass filter.
  let start_cap: f32 = 2.1f32;
  let final_cap: f32 = MAX_REGISTER as f32 - arf.register as f32 - start_cap;

  let min_f: f32 = 2f32.powf(arf.register as f32 + start_cap);
  let max_f: f32 = 2f32.powf(arf.register as f32 + start_cap + final_cap);
  let n: f32 = n_samples as f32;
  let df: f32 = (max_f - min_f).log2();

  for i in 0..n_samples {
    let x: f32 = i as f32 / n;
    lowpass_contour.push(min_f + 2f32.powf(df * x));
  }
  (highpass_contour, lowpass_contour)
}

fn dynamics(arf: &Arf, n_samples: usize, k: f32) -> SampleBuffer {
  let min_db = -30f32;
  let max_db = 0f32;
  let gain: f32 = visibility_gain(arf.visibility);

  let n = n_samples as f32;

  let mut dynamp_contour: Vec<f32> = Vec::with_capacity(n_samples);
  for i in 0..n_samples {
    let x: f32 = i as f32 / n;

    let x_adjusted = (k * x).fract();
    let triangle_wave = if x_adjusted <= 0.5 {
      2.0 * x_adjusted
    } else {
      2.0 * (1.0 - x_adjusted)
    };

    let y = db_to_amp(min_db + (max_db - min_db) * triangle_wave);

    // Calculate the lowpass frequency based on the triangle wave
    dynamp_contour.push(y);
  }

  dynamp_contour
}

pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  
  let mullet = get_mullet(&arf);

  let len_cycles: f32 = time::count_cycles(&melody[0]);

  let soids = &druidic_soids::overs_sawtooth(mullet);
  let soids = soid_fx::fmod::reece(&soids, 1.5f32, 7);
  let soids = soid_fx::fmod::reece(&soids, 1.5f32, 6);

  let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(melody);
  let level_macro: LevelMacro = LevelMacro {
    stable: [1f32, 1f32],
    peak: [3f32, 6f32],
    sustain: [0.4f32, 0.8f32],
  };
  let odr_macro = ODRMacro {
    onset: [160.0, 1200f32],
    decay: [233f32, 1650f32],
    release: [151f32, 200f32],
    mo: vec![MacroMotion::Constant],
    md: vec![MacroMotion::Constant],
    mr: vec![MacroMotion::Constant],
  };
  
  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  knob_mods.0.push(amp_onset(arf.visibility, arf.energy, arf.presence));
  knob_mods.0.push(amp_knob_presence(arf.visibility, arf.energy, arf.presence));
  knob_mods.2.push(pmod_chorus(arf.visibility, arf.energy, arf.presence));
  let n_samples = (SRf * len_cycles / 2f32) as usize;

  let mut dynamics = dynamics::gen_organic_amplitude(10, n_samples, arf.visibility);
  amp_scale(&mut dynamics, visibility_gain(arf.visibility));

  let expr = (dynamics, vec![1f32], vec![0f32]);
  let mut rng = thread_rng();
  let delays_note = generate_chord_delay_macros(arf.visibility, arf.energy, arf.presence)
    .iter()
    .map(|mac| mac.gen(&mut rng, conf.cps))
    .collect();

  let delays_room = vec![];

  let reverbs_note: Vec<ReverbParams> = vec![];
  let reverbs_room: Vec<ReverbParams> = vec![];

  let stem = (
    melody,
    soids,
    expr,
    ValleyCon::get_bp(conf.cps, melody, arf),
    knob_mods,
    delays_note,
    delays_room,
    reverbs_note,
    reverbs_room,
  );

  Renderable2::Instance(stem)
}
