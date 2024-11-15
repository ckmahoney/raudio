use std::os::unix::thread;

use super::*;
use super::*;
use crate::druid::{self, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::types::synthesis::{ModifiersHolder, Soids};

fn expr() -> Expr {
  let ampenv = amp_expr(4f32);
  (ampenv, vec![1f32], vec![0f32])
}

/// create a harmonic pallette texture like a house stab
fn generate_rich_texture(arf: &Arf) -> Soids {
  let soids = druidic_soids::id();
  match arf.energy {
    Energy::Low => soids,
    Energy::Medium => soid_fx::fmod::triangle(&soids, 4),
    Energy::High => soid_fx::fmod::sawtooth(&soids, 8),
  }
}

fn amp_knob_staccatto(
  visibility: Visibility, energy: Energy, presence: Presence,
) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  let mut rng = thread_rng();
  let sustain = match presence {
    Presence::Staccatto => 0f32,
    Presence::Legato => 0.66f32,
    Presence::Tenuto => 1f32,
  };

  let decay_rate = match energy {
    Energy::Low => rng.gen::<f32>() / 5f32,
    Energy::Medium => 0.25 + rng.gen::<f32>() / 2f32,
    Energy::High => 0.66f32 + 0.33 * rng.gen::<f32>(),
  };

  (
    Knob {
      a: sustain,
      b: decay_rate,
      c: 0.0,
    },
    ranger::amod_pluck,
  )
}

fn amp_knob_legato(
  visibility: Visibility, energy: Energy, presence: Presence,
) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  let mut rng = thread_rng();
  let contour = match visibility {
    Visibility::Foreground => 0.1 * rng.gen::<f32>(),
    Visibility::Visible => 0.1 * rng.gen::<f32>(),
    Visibility::Background => 0.3 + 0.2 * rng.gen::<f32>(),
    Visibility::Hidden => 0.45 + 0.45 * rng.gen::<f32>(),
  };

  let osc_rate = match energy {
    Energy::Low => rng.gen::<f32>() / 8f32,
    Energy::Medium => 0.2 + rng.gen::<f32>() / 8f32,
    Energy::High => 0.33f32 + 0.33 * rng.gen::<f32>(),
  };

  return (
    Knob {
      a: contour,
      b: 1f32,
      c: osc_rate,
    },
    ranger::amod_wavelet_morphing,
  );
}

fn amp_knob_tenuto(
  visibility: Visibility, energy: Energy, presence: Presence,
) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  let mut rng = thread_rng();
  let osc_rate = match energy {
    Energy::Low => 0.1 * rng.gen::<f32>(),
    Energy::Medium => 0.3 + 0.2 * rng.gen::<f32>(),
    Energy::High => 0.45 + 0.45 * rng.gen::<f32>(),
  };

  let time_scale = match energy {
    Energy::Low => rng.gen::<f32>() / 5f32,
    Energy::Medium => 0.2 + rng.gen::<f32>() / 4f32,
    Energy::High => 0.33f32 + 0.66 * rng.gen::<f32>(),
  };

  return (
    Knob {
      a: osc_rate,
      b: time_scale,
      c: 0f32,
    },
    ranger::amod_oscillation_sin_mul,
  );
}

fn amp_knob(
  visibility: Visibility, energy: Energy, presence: Presence,
) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  match presence {
    Presence::Staccatto => amp_knob_staccatto(visibility, energy, presence),
    Presence::Legato => amp_knob_legato(visibility, energy, presence),
    Presence::Tenuto => amp_knob_tenuto(visibility, energy, presence),
  }
}

pub fn renderable<'render>(melody: &'render Melody<Note>, arf: &Arf) -> Renderable<'render> {
  let soids: Soids = generate_rich_texture(arf);
  println!("Got soids {:?}", soids);
  let mut knob_mods: KnobMods = KnobMods::unit();
  knob_mods.0.push(amp_knob(arf.visibility, arf.energy, arf.presence));

  let feel: Feel = Feel {
    bp: (vec![MFf], vec![NFf]),
    modifiers: (vec![], vec![], vec![], vec![]),
    clippers: (0f32, 1f32),
  };

  let stem = (melody, soids, expr(), feel, knob_mods, vec![delay::passthrough]);
  Renderable::Instance(stem)
}
