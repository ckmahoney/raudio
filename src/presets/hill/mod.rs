use super::*;
pub mod bass;
pub mod chords;
pub mod hats;
pub mod kick;
pub mod lead;
pub mod perc;

pub fn get_bp<'render>(cps: f32, mel: &'render Melody<Note>, arf: &Arf, len_cycles: f32) -> Bp2 {
  match arf.presence {
    Presence::Staccatto => bp_wah(cps, mel, arf, len_cycles),
    Presence::Legato => bp_sighpad(cps, mel, arf, len_cycles),
    Presence::Tenuto => bp_cresc(cps, mel, arf, len_cycles),
  }
}

pub fn map_role_preset<'render>() -> RolePreset<'render> {
  RolePreset {
    label: "hill",
    kick: kick::renderable,
    perc: perc::renderable,
    hats: hats::renderable,
    chords: chords::renderable,
    lead: lead::renderable,
    bass: bass::renderable,
  }
}
