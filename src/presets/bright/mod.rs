use std::i32::MAX;

use super::*;
pub mod bass;
pub mod chords;
pub mod hats;
pub mod kick;
pub mod lead;
pub mod perc;

pub struct BrightCon<'render> {
  _marker: PhantomData<&'render ()>,
}

// some old notes
// 8 is the optimal value for high energy because using 7 often has the same appearance but costs 2x more
// 10 is clearly different than 8
// 12 is clearly different than 10
// also noting that 8 and 9 not so different, 10 and 11 somewhat different
// edit nov 13, just used 9 instead of 8 because adding soid_fx doubled the number of soids.

/// Given an arf,
/// Determine how tall its synth should be by setting a reference fundamental here.
pub fn get_mullet(register: i8, energy: Energy) -> f32 {
  let height = register as i32
    + match energy {
      Energy::Low => 1,
      Energy::Medium => 0,
      Energy::High => -1,
    };
  2f32.powi(height.clamp(7, MAX_REGISTER - 1))
}

/// Given a line,
/// define a lowpass contour by interpolating from the provided onset/decay/release breakpoints
/// for the provided stable (begin & end), peak, and sustain levels.
/// Guaranteed that a complete ODR will always fit in each noteevent's duration.
pub fn pointwise_lowpass(cps: f32, line: &Vec<Note>, level_macro: &LevelMacro, odr_macro: &ODRMacro) -> SampleBuffer {
  let n_samples = time::samples_of_line(cps, line);
  let mut contour: SampleBuffer = Vec::with_capacity(n_samples);

  for (i, note) in (*line).iter().enumerate() {
    let Levels { peak, sustain, stable } = level_macro.gen();
    let applied_peak = peak.clamp(1f32, MAX_REGISTER as f32 - MIN_REGISTER as f32);

    let n_samples_note: usize = time::samples_of_note(cps, note);

    let dur_seconds = time::step_to_seconds(cps, &(*note).0);
    let odr: ODR = (odr_macro.gen()).fit_in_samples(cps, dur_seconds);

    let (n_samples_ramp, n_samples_fall, n_samples_hold, n_samples_kill) = eval_odr(cps, n_samples_note, &odr);

    let curr_freq: f32 = note_to_freq(note);

    let stable_freq_base = stable * curr_freq.log2();
    for j in 0..n_samples_note {
      let cutoff_freq: f32 = if j < n_samples_ramp {
        // onset
        let p = j as f32 / n_samples_ramp as f32;
        2f32.powf(applied_peak * p + stable_freq_base)
      } else if j < n_samples_ramp + n_samples_fall {
        // decay
        let p = (j - n_samples_ramp) as f32 / n_samples_fall as f32;
        let d_sustain = p * (1f32 - sustain);
        2f32.powf((applied_peak - applied_peak * d_sustain) + stable_freq_base)
      } else if j < n_samples_ramp + n_samples_fall + n_samples_hold {
        let p = (j - n_samples_ramp - n_samples_fall) as f32 / n_samples_hold as f32;
        // sustain
        2f32.powf(applied_peak * sustain + stable_freq_base)
      } else {
        // release
        let p = (j - n_samples_ramp - n_samples_fall - n_samples_hold) as f32 / n_samples_kill as f32;
        let d_sustain = (1f32 - p) * (applied_peak * sustain);
        2f32.powf(d_sustain + stable_freq_base)
      };

      contour.push(cutoff_freq);
    }
  }
  contour
}

impl<'render> Conventions<'render> for BrightCon<'render> {
  fn get_bp(cps: f32, mel: &'render Melody<Note>, arf: &Arf) -> Bp2 {
    match arf.role {
      Role::Bass => {
        let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);

        let level_macro: LevelMacro = LevelMacro {
          peak: [2.5f32, 2.9f32],
          sustain: [0.25f32, 0.3f32],
          stable: [1f32, 1.2f32],
        };

        let odr_macro = ODRMacro {
          onset: [30.0, 60f32],
          decay: [1130.0, 1400f32],
          release: [310.0, 600f32],

          mo: vec![MacroMotion::Constant],
          md: vec![MacroMotion::Constant],
          mr: vec![MacroMotion::Constant],
        };

        (
          vec![MFf],
          pointwise_lowpass(cps, &mel[high_index], &level_macro, &odr_macro),
          vec![],
        )
      }
      Role::Chords => {
        let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);

        let level_macro: LevelMacro = LevelMacro {
          peak: [1.5f32, 2.2f32],
          sustain: [0.25f32, 0.3f32],
          stable: [1f32, 1.2f32],
        };

        let odr_macro = ODRMacro {
          onset: [30.0, 60f32],
          decay: [1130.0, 2400f32],
          release: [310.0, 400f32],

          mo: vec![MacroMotion::Constant],
          md: vec![MacroMotion::Constant],
          mr: vec![MacroMotion::Constant],
        };

        (
          vec![MFf],
          pointwise_lowpass(cps, &mel[high_index], &level_macro, &odr_macro),
          vec![],
        )
      }
      Role::Lead => {
        let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);

        let level_macro: LevelMacro = LevelMacro {
          peak: [2f32, 3.2f32],
          sustain: [0.3f32, 0.5f32],
          stable: [1f32, 1.2f32],
        };

        let odr_macro = ODRMacro {
          onset: [20.0, 260f32],
          decay: [430.0, 2400f32],
          release: [510.0, 1800f32],

          mo: vec![MacroMotion::Constant],
          md: vec![MacroMotion::Constant],
          mr: vec![MacroMotion::Constant],
        };

        (
          vec![MFf],
          pointwise_lowpass(cps, &mel[high_index], &level_macro, &odr_macro),
          vec![],
        )
      }
      _ => (vec![MFf], vec![NFf], vec![]),
    }
  }
}

pub fn map_role_preset<'render>() -> RolePreset<'render> {
  RolePreset {
    label: "bright",
    kick: kick::renderable,
    perc: perc::renderable,
    hats: hats::renderable,
    chords: chords::renderable,
    lead: lead::renderable,
    bass: bass::renderable,
  }
}
