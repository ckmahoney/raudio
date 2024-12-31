use super::*;
pub mod bass;
pub mod chords;
pub mod lead;
pub use crate::presets::bright::hats;
pub use crate::presets::bright::kick;
pub use crate::presets::bright::perc;

pub fn map_role_preset<'render>() -> RolePreset<'render> {
  RolePreset {
    label: "Fum",
    kick: crate::presets::bright::kick::renderable,
    perc: crate::presets::bright::perc::renderable,
    hats: crate::presets::bright::hats::renderable,
    chords: chords::renderable,
    lead: lead::renderable,
    bass: bass::renderable,
  }
}

/// Reduces the intensity of modulation for higher frequencies
/// Designed for qualatative use, to "thin out" the instrument
/// as its melody rises.
pub fn attenuate_mod_index_by_freq(freq: f32) -> f32 {
  2f32.powf(1f32 - freq.log2())
}

/// Returns a value in (0, sqrt(2)) representing how much to increase mod index based on velocity
/// Scaled such that velocity ~70% is neutral (mod_scale_index=0.989)
/// 10% results in mod_index cancellation   return = mod_scale_index = 0
/// 10% results in mod_index attenuation    return = mod_scale_index = 0.141
/// 30% results in mod_index attenuation    return = mod_scale_index = 0.424
/// 50% results in mod_index attenuation    return = mod_scale_index = 0.707
/// 70% results in mod_index identity       return = mod_scale_index = 0.989
/// 90% results in mod_index expansion      return = mod_scale_index = 1.272
/// 100% results in mod_index expansion     return = mod_scale_index = 1.414
pub fn attenuate_mod_index_by_vel(velocity: f32) -> f32 {
  let x = velocity.clamp(0f32, 1f32).powi(2i32);
  (x * 2f32).powf(0.5f32)
}

#[test]
fn test_attenuate_mod_index_by_vel() {
  for v in 0..=10 {
    let velocity = v as f32 / 10f32;
    let result = attenuate_mod_index_by_vel(velocity);
    println!("Got x {} result {}", velocity, result);
    assert!(
      result >= 0f32 && result <= 1.42f32,
      "Must not exceed the range of (0, 1.42) but actually got {}",
      result
    );
  }
}

pub fn mod_index_by_note(note: &Note) -> f32 {
  attenuate_mod_index_by_vel(note.2) * attenuate_mod_index_by_freq(note_to_freq(note))
}

pub fn mod_index_by_arf(arf: &Arf) -> f32 {
  let mut rng = thread_rng();
  match arf.energy {
    Energy::Low => in_range(&mut rng, 0.3f32, 0.5f32),
    Energy::Medium => in_range(&mut rng, 0.5f32, 0.75f32),
    Energy::High => in_range(&mut rng, 0.9f32, 1f32),
  };
  0.3f32
}

pub fn mod_index_by_moment(note: &Note, arf: &Arf) -> f32 {
  mod_index_by_arf(arf) * mod_index_by_note(note)
}

/// Represents the fading effect of gain attenuation at deeply nested modulators
//// where k represents the depth of the modulator
pub fn cascaded_gain(gain: f32, k: i32) -> f32 {
  if gain == 1f32 {
    return 1f32;
  };
  if gain < 1f32 {
    gain.powf((1i32 + 2i32 * k) as f32)
  } else {
    gain.powf(1f32 / (1i32 + 2i32 * k) as f32)
  }
}

#[test]
fn test_mod_index_by_moment() {
  let arf = Arf {
    mode: Mode::Melodic,
    role: Role::Chords,
    register: 10,
    visibility: Visibility::Foreground,
    energy: Energy::High,
    presence: Presence::Legato,
  };
  let amp = 1f32;

  // create a melody of same note ascending by octave
  let line: Vec<Note> = (4..15).map(|register| ((1, 1), (register as i8, (1, 0, 3)), amp)).collect();

  let mut prev = 1000f32;
  for note in line {
    let result = mod_index_by_moment(&note, &arf);
    assert!(prev > result, "Must have mod_index that diminishes as frequency increases. But here the last value was {} and we just got {} for register {}", prev, result, note.1.0);
    prev = result;
  }
}
