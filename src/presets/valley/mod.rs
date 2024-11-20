use std::i32::MAX;

use super::*;
pub mod bass;
pub mod chords;
pub mod hats;
pub mod kick;
pub mod lead;
pub mod perc;



pub fn amp_onset(visibility: Visibility, energy: Energy, presence: Presence) -> KnobPair {
  let onset_duration = match presence {
      Presence::Staccatto => [0.1, 0.3], 
      Presence::Tenuto => [0.4, 0.6],    
      Presence::Legato => [0.7, 1.0],   
  };


let flex_mode =  match visibility {
  Visibility::Hidden => [0.8, 1f32],
  Visibility::Background => [0.3, 0.8],
  Visibility::Foreground => [0.2, 0.4],
  Visibility::Visible => [0.5, 0.5],
};


let dynamic_range = match energy {
        Energy::Low => [0.3, 0.5],
        Energy::Medium => [0.6, 0.8],
        Energy::High => [0.7, 1.0],
    };

    (KnobMacro {
        a: onset_duration,
        b: flex_mode,
        c: dynamic_range,
        ma: grab_variant(vec![MacroMotion::Forward, MacroMotion::Random, MacroMotion::Reverse, MacroMotion::Constant]),
        mb: grab_variant(vec![MacroMotion::Forward, MacroMotion::Random, MacroMotion::Reverse, MacroMotion::Constant]),
        mc: grab_variant(vec![MacroMotion::Forward, MacroMotion::Random, MacroMotion::Reverse, MacroMotion::Constant]),
    },
    match presence {
      Presence::Staccatto => ranger::amod_cycle_fadein_0p031_0p125,
      Presence::Legato => ranger::amod_cycle_fadein_0p125_1,
      Presence::Tenuto => ranger::amod_cycle_fadein_1_4,
    }
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


pub fn get_bp<'render>(cps: f32, mel: &'render Melody<Note>, arf: &Arf) -> Bp2 {
  match arf.presence {
    Presence::Staccatto => bp_wah(cps, mel, arf),
    Presence::Legato => bp_sighpad(cps, mel, arf),
    Presence::Tenuto => bp_cresc(cps, mel, arf),
  }
}

trait Conventions<'render> {
  fn get_bp(cps: f32, mel: &'render Melody<Note>, arf: &Arf) -> Bp2;
}

pub struct ValleyCon<'render> {
  _marker: PhantomData<&'render ()>,
}

// some old notes
// 8 is the optimal value for high energy because using 7 often has the same appearance but costs 2x more
  // 10 is clearly different than 8
  // 12 is clearly different than 10
  // also noting that 8 and 9 not so different, 10 and 11 somewhat different
  // edit nov 13, just used 9 instead of 8 because adding soid_fx doubled the number of soids.

/// Given an arf, 
/// Determine how tall its synth should be by setting its fundamental here.
pub fn get_mullet(arf:&Arf) -> f32 {
  let height = arf.register as i32 + match arf.energy {
    Energy::Low => 2, Energy::Medium => 0, Energy::High => -2
  };
  2f32.powi(height.clamp(7, MAX_REGISTER - 1))
}


impl<'render> Conventions<'render> for ValleyCon<'render> {
  // fn get_bp(&self, cps: f32, mel: &'render Melody<Note>, arf: &Arf, len_cycles: f32) -> Bp2 {
  //   match arf.presence {
  //     Presence::Staccatto => bp_wah(cps, mel, arf),
  //     Presence::Legato => bp_sighpad(cps, mel, arf),
  //     Presence::Tenuto => bp_cresc(cps, mel, arf),
  //   }
  // }


  fn get_bp(cps: f32, mel: &'render Melody<Note>, arf: &Arf) -> Bp2 {
    match arf.role {
      Role::Bass => match arf.visibility {
        Visibility::Foreground => bp_bark(cps, mel, arf, 1f32),
        Visibility::Visible => bp_bark(cps, mel, arf, 0.75f32),
        Visibility::Hidden => bp_bark(cps, mel, arf, 0.5f32),
        _ => bp_sighpad(cps, mel, arf),
      },
      Role::Chords => match arf.visibility {
        Visibility::Foreground =>bp_bark(cps, mel, arf, 0.5f32),
        Visibility::Hidden => bp_cresc(cps, mel, arf),
        _ => bp_sighpad(cps, mel, arf),
      },
      Role::Lead => match arf.presence {
        Presence::Legato => bp_sighpad(cps, mel, arf),
        Presence::Staccatto => bp_bark(cps, mel, arf, 1.5f32),
        Presence::Tenuto => bp2_unit()
      },
      _ => bp2_unit()
  }
}
}

pub fn map_role_preset<'render>() -> RolePreset<'render> {
  RolePreset {
    label: "valley",
    kick: kick::renderable,
    perc: perc::renderable,
    hats: hats::renderable,
    chords: chords::renderable,
    lead: lead::renderable,
    bass: bass::renderable,
  }
}
