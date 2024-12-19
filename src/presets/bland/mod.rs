use std::i32::MAX;

use super::*;
pub mod bass;
pub mod chords;
pub mod hats;
pub mod kick;
pub mod lead;
pub mod perc;


pub struct BlandCon<'render> {
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
    Energy::Low => 0, Energy::Medium => -1, Energy::High => -2
  };
  2f32.powi(height.clamp(7, MAX_REGISTER - 1))
}


impl<'render> Conventions<'render> for BlandCon<'render> {
  fn get_bp(cps: f32, mel: &'render Melody<Note>, arf: &Arf) -> Bp2 {
    bp2_unit()
  }
}

pub fn map_role_preset<'render>() -> RolePreset<'render> {
  RolePreset {
    label: "bland",
    kick: kick::renderable,
    perc: perc::renderable,
    hats: hats::renderable,
    chords: chords::renderable,
    lead: lead::renderable,
    bass: bass::renderable,
  }
}
