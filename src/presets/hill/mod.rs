use super::*;
pub mod perc;
pub mod chords;
pub mod bass;
pub mod kick;
pub mod hats;
pub mod lead;


pub fn role_presets<'render>() -> RolePreset<'render> {
    RolePreset::new(
        "hill",
        bass::renderable,
        chords::renderable,
        lead::renderable,
        kick::renderable,
        perc::renderable,
        hats::renderable,
    )
}