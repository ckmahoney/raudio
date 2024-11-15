use hound::Sample;

use super::*;
use crate::analysis::melody::{find_reach, mask_sigh, mask_wah, LevelMacro, Levels, ODRMacro, ODR};
use crate::druid::{self, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::time;
use crate::types::synthesis::{BoostGroup, BoostGroupMacro, ModifiersHolder, Soids};

fn amp_knob_detune(visibility: Visibility, energy: Energy, presence: Presence) -> KnobPair {
  let mut rng = thread_rng();

  let detune_rate = match energy {
    Energy::Low => [0f32, rng.gen::<f32>() / 6f32],
    Energy::Medium => [0f32, rng.gen::<f32>() / 3f32],
    Energy::High => [0f32, rng.gen::<f32>()],
  };
  let detune_mix = match visibility {
    Visibility::Visible => [0.33, 0.33 + 0.47 * rng.gen::<f32>()],
    Visibility::Foreground => [0.2, 0.2 + 0.13 * rng.gen::<f32>()],
    Visibility::Background => [0.1, 0.1 + 0.1 * rng.gen::<f32>()],
    Visibility::Hidden => [0f32, 0.05 * rng.gen::<f32>()],
  };

  (
    KnobMacro {
      a: detune_rate,
      b: detune_mix,
      c: [0.0, 0.0], // Static value as [0, 0]
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    ranger::amod_detune,
  )
}

fn freq_knob_tonal(v: Visibility, e: Energy, p: Presence) -> KnobPair {
  let mut rng = thread_rng();
  let modulation_amount = match e {
    Energy::Low => [0.005f32, 0.005f32 + 0.003 * rng.gen::<f32>()],
    Energy::Medium => [0.008f32, 0.008f32 + 0.012f32 * rng.gen::<f32>()],
    Energy::High => [0.1f32, 0.1f32 + 0.2f32 * rng.gen::<f32>()],
  };

  (
    KnobMacro {
      a: modulation_amount,
      b: [0f32, 0f32], // Static value as [0, 0]
      c: [0f32, 0f32], // Static value as [0, 0]
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    ranger::fmod_warble,
  )
}

fn pmod_chorus(v: Visibility, e: Energy, p: Presence) -> KnobPair {
  let mut rng = thread_rng();

  let modulation_depth = match v {
    Visibility::Hidden => [0.33f32, 0.33f32],
    Visibility::Background => [0.5, 0.5],
    Visibility::Foreground => [0.75, 0.75],
    Visibility::Visible => [1f32, 1f32],
  };

  let chorus_visibility = match v {
    Visibility::Hidden => [0f32, 0f32],
    Visibility::Background => [0.1, 0.1 + 0.5 * rng.gen::<f32>()],
    Visibility::Foreground => [0.6, 0.6 + 0.2 * rng.gen::<f32>()],
    Visibility::Visible => [0.8, 0.8 + 0.1 * rng.gen::<f32>()],
  };

  (
    KnobMacro {
      a: modulation_depth,
      b: chorus_visibility,
      c: [0.0, 0.0], // Static value as [0, 0]
      ma: MacroMotion::Random,
      mb: MacroMotion::Random,
      mc: MacroMotion::Random,
    },
    ranger::pmod_chorus,
  )
}

/// Generate a phrase length filter contour for the given melody and arf.
pub fn filter_contour_linear_rise<'render>(
  melody: &'render Melody<Note>, arf: &Arf, n_samples: usize,
) -> (SampleBuffer, SampleBuffer) {
  let len_cycles: f32 = time::count_cycles(&melody[0]);

  let mut highpass_contour: SampleBuffer = vec![MFf; n_samples];
  let mut lowpass_contour: SampleBuffer = Vec::with_capacity(n_samples);

  // The default position of the lowpass filter.
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

pub fn renderable<'render>(cps: f32, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  let mullet = match arf.energy {
    Energy::Low => 2f32.powi(12i32),
    Energy::Medium => 2f32.powi(10i32),
    Energy::High => 2f32.powi(8i32),
  };
  let len_cycles: f32 = time::count_cycles(&melody[0]);
  let soids = druidic_soids::overs_sawtooth(mullet);

  let bp: Bp2 = get_bp(cps, melody, arf, len_cycles);

  let mut knob_mods: KnobMods2 = KnobMods2::unit();
  knob_mods.0.push(amp_microtransient(arf.visibility, arf.energy, arf.presence));
  knob_mods.2.push(pmod_chorus(arf.visibility, arf.energy, arf.presence));
  let n_samples = (SRf * len_cycles / 2f32) as usize;

  let dynamics = dynamics::gen_organic_amplitude(10, n_samples, arf.visibility);

  let expr = (dynamics, vec![1f32], vec![0f32]);

  let delays_note = vec![delay::passthrough];
  let delays_room = vec![];
  let reverbs_note: Vec<ReverbParams> = vec![];
  let reverbs_room: Vec<ReverbParams> = vec![];

  let stem: Stem2 = (
    melody,
    soids,
    expr,
    bp,
    knob_mods,
    delays_note,
    delays_room,
    reverbs_note,
    reverbs_room,
  );

  Renderable2::Instance(stem)
}
