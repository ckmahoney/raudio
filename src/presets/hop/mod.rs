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

  let flex_mode = match visibility {
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

  (
    KnobMacro {
      a: onset_duration,
      b: flex_mode,
      c: dynamic_range,
      ma: grab_variant(vec![MacroMotion::Reverse, MacroMotion::Constant]),
      mb: grab_variant(vec![MacroMotion::Reverse, MacroMotion::Constant]),
      mc: grab_variant(vec![MacroMotion::Reverse, MacroMotion::Constant]),
    },
    if let Presence::Staccatto = presence {
      ranger::amod_cycle_fadein_0p031_0p125
    } else {
      ranger::amod_cycle_fadein_0p125_1
    },
  )
}

pub struct HopCon<'render> {
  _marker: PhantomData<&'render ()>,
}

impl<'render> Conventions<'render> for HopCon<'render> {
  fn get_bp(cps: f32, mel: &'render Melody<Note>, arf: &Arf) -> Bp2 {
    match arf.role {
      Role::Bass => bp_bark(cps, mel, arf, 1f32),
      Role::Lead => bp_bark(cps, mel, arf, 2f32),
      _ => bp2_unit(),
    }
  }
}

pub fn map_role_preset<'render>() -> RolePreset<'render> {
  RolePreset {
    label: "hop",
    kick: kick::renderable,
    perc: perc::renderable,
    hats: hats::renderable,
    chords: chords::renderable,
    lead: lead::renderable,
    bass: bass::renderable,
  }
}
