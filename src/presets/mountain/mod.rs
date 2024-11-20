use super::*;
pub mod bass;
pub mod chords;
pub mod hats;
pub mod kick;
pub mod lead;
pub mod perc;



/// Generates a macro for ambient music, designed for long, evolving fades suitable for lead and pad synths.
/// Adjusts cycle length, fade shape, and dynamic range based on visibility, energy, and presence.
pub fn amp_onset(visibility: Visibility, energy: Energy, presence: Presence) -> KnobPair {
  let visibility_gain = match visibility {
      Visibility::Hidden => [0.0, 0.2],
      Visibility::Background => [0.2, 0.5],
      Visibility::Foreground => [0.5, 0.8],
      Visibility::Visible => [0.8, 1.0],
  };

  let spectral_mod = match energy {
      Energy::Low => [0.2, 0.5],
      Energy::Medium => [0.4, 0.8],
      Energy::High => [0.7, 1.0],
  };

  let duration_mod = match presence {
      Presence::Staccatto => [0.1, 0.2],
      Presence::Tenuto => [0.3, 0.6],
      Presence::Legato => [0.7, 1.0],
  };

  (
      KnobMacro {
          a: visibility_gain,
          b: spectral_mod,
          c: duration_mod,
          ma: grab_variant(vec![MacroMotion::Forward, MacroMotion::Constant]),
          mb: grab_variant(vec![MacroMotion::Mean, MacroMotion::Reverse]),
          mc: grab_variant(vec![MacroMotion::Forward, MacroMotion::Constant]),
      },
      ranger::amod_cycle_fadein_4_16,
  )
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

/// Create bandpass automations with respect to Arf and Melody
fn bp_cresc<'render>(cps: f32, mel: &'render Melody<Note>, arf: &Arf) -> Bp2 {
  let len_cycles = time::count_cycles(&mel[0]);
  let size = (len_cycles.log2() - 1f32).max(1f32); // offset 1 to account for lack of CPC. -1 assumes CPC=2
  let rate_per_size = match arf.energy {
    Energy::Low => 0.5f32,
    Energy::Medium => 1f32,
    Energy::High => 2f32,
  };
  let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);
  let n_samples: usize = ((len_cycles / 2f32) as usize).max(1) * SR;

  let (highpass, lowpass): (Vec<f32>, Vec<f32>) = if let Visibility::Visible = arf.visibility {
    match arf.energy {
      Energy::Low => (
        filter_contour_triangle_shape_highpass(lowest_register, highest_register, n_samples, size * rate_per_size),
        vec![NFf],
      ),
      _ => (
        vec![MFf],
        filter_contour_triangle_shape_lowpass(lowest_register, n_samples, size * rate_per_size),
      ),
    }
  } else {
    (vec![MFf], vec![NFf])
  };

  let levels = Levels::new(0.7f32, 4f32, 0.5f32);
  let odr = ODR {
    onset: 60.0,
    decay: 1330.0,
    release: 110.0,
  };

  (highpass, lowpass, vec![])
}

/// Create bandpass automations with respect to Arf and Melody
fn bp_wah<'render>(cps: f32, mel: &'render Melody<Note>, arf: &Arf) -> Bp2 {
  let len_cycles = time::count_cycles(&mel[0]);
  let size = (len_cycles.log2() - 1f32).max(1f32); // offset 1 to account for lack of CPC. -1 assumes CPC=2
  let rate_per_size = match arf.energy {
    Energy::Low => 0.5f32,
    Energy::Medium => 1f32,
    Energy::High => 2f32,
  };
  let ((lowest_register, low_index), (highest_register, high_index)) = find_reach(mel);
  let n_samples: usize = ((len_cycles / 2f32) as usize).max(1) * SR;

  let levels = Levels::new(0.7f32, 4f32, 0.5f32);

  let level_macro: LevelMacro = LevelMacro {
    stable: match arf.energy {
      Energy::Low => [1f32, 3f32],
      Energy::Medium => [2f32, 3f32],
      Energy::High => [2f32, 3f32],
    },
    peak: match arf.energy {
      Energy::Low => [2.0f32, 3.0f32],
      Energy::Medium => [3.75f32, 5f32],
      Energy::High => [4f32, 8f32],
    },
    sustain: match arf.visibility {
      Visibility::Visible => [0.85f32, 1f32],
      Visibility::Foreground => [0.5f32, 1.0f32],
      Visibility::Background => [0.2f32, 0.5f32],
      Visibility::Hidden => [0.0132, 0.1f32],
    },
  };

  // Increased ODR values for slower, more gradual changes suitable for ambient music
  let odr_macro = ODRMacro {
    onset: [180.0, 360f32],   // Previously 60.0, 120f32
    decay: [460.0, 600f32],   // Previously 230.0, 300f32
    release: [330.0, 400f32], // Previously 110.0, 200f32
    mo: vec![MacroMotion::Constant],
    md: vec![MacroMotion::Constant],
    mr: vec![MacroMotion::Constant],
  };
  let highpass = if let Energy::Low = arf.energy {
    filter_contour_triangle_shape_highpass(lowest_register, highest_register, n_samples, size * rate_per_size)
  } else {
    vec![MFf]
  };
  (
    highpass,
    mask_wah(cps, &mel[low_index], &level_macro, &odr_macro),
    vec![],
  )
}

pub fn get_bp<'render>(cps: f32, mel: &'render Melody<Note>, arf: &Arf) -> Bp2 {
  match arf.presence {
    Presence::Staccatto => bp_wah(cps, mel, arf),
    Presence::Legato => bp_sighpad(cps, mel, arf),
    Presence::Tenuto => bp_cresc(cps, mel, arf),
  }
}


pub struct MountainCon<'render> {
  _marker: PhantomData<&'render ()>,
}

impl<'render> Conventions<'render> for MountainCon<'render> {
  fn get_bp(cps: f32, mel: &'render Melody<Note>, arf: &Arf) -> Bp2 {
    match arf.presence {
      Presence::Staccatto => bp_wah(cps, mel, arf),
      Presence::Legato => bp_sighpad(cps, mel, arf),
      Presence::Tenuto => bp_cresc(cps, mel, arf),
    }
  }
}

pub fn map_role_preset<'render>() -> RolePreset<'render> {
  RolePreset {
    label: "mountain",
    kick: kick::renderable,
    perc: perc::renderable,
    hats: hats::renderable,
    chords: chords::renderable,
    lead: lead::renderable,
    bass: bass::renderable,
  }
}
