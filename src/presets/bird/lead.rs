use std::os::unix::thread;

use super::*;
use super::*;
use crate::druid::{self, soids as druidic_soids};
use crate::phrasing::{contour::Expr, ranger::KnobMods};
use crate::types::synthesis::{ModifiersHolder, Soids};

fn amp_knob_principal(rng: &mut ThreadRng, arf: &Arf) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  return (
    Knob {
      a: match arf.presence {
        Presence::Staccatto => in_range(rng, 0f32, 0.5f32),
        Presence::Legato => in_range(rng, 0.33f32, 0.88f32),
        Presence::Tenuto => in_range(rng, 0.88f32, 1f32),
      },
      b: match arf.energy {
        Energy::High => in_range(rng, 0f32, 0.2f32),
        Energy::Medium => in_range(rng, 0.2f32, 0.3f32),
        Energy::Low => in_range(rng, 0.3f32, 0.5f32),
      },
      c: 0.0,
    },
    ranger::amod_burp,
  );
}

fn amp_knob_attenuation(rng: &mut ThreadRng, arf: &Arf) -> (Knob, fn(&Knob, f32, f32, f32, f32, f32) -> f32) {
  return (
    Knob {
      a: match arf.energy {
        Energy::High => in_range(rng, 0.5f32, 0.8f32),
        Energy::Medium => in_range(rng, 0.3f32, 0.4f32),
        Energy::Low => in_range(rng, 0.0f32, 0.31f32),
      },
      b: match arf.visibility {
        Visibility::Visible => in_range(rng, 0.8f32, 1f32),
        Visibility::Foreground => in_range(rng, 0.5f32, 0.8f32),
        _ => in_range(rng, 0f32, 0.3f32),
      },
      c: 0.0,
    },
    ranger::amod_detune,
  );
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

pub fn expr(arf: &Arf) -> Expr {
  (vec![visibility_gain(arf.visibility)], vec![1f32], vec![0f32])
}

pub fn renderable<'render>(melody: &'render Melody<Note>, arf: &Arf) -> Renderable<'render> {
  let mut rng = thread_rng();
  //# id component
  let soids = druidic_soids::id();

  let feel: Feel = Feel {
    bp: (vec![MFf], vec![NFf]),
    modifiers: (vec![], vec![], vec![], vec![]),
    clippers: (0f32, 1f32),
  };

  let mut rng = thread_rng();

  let mut knob_mods: KnobMods = KnobMods::unit();
  knob_mods.0.push(amp_knob(arf.visibility, arf.energy, arf.presence));
  knob_mods.0.push(amp_knob_principal(&mut rng, &arf));
  knob_mods.0.push(amp_knob_attenuation(&mut rng, &arf));

  let height = match arf.visibility {
    Visibility::Hidden => 1,
    Visibility::Background => 2,
    Visibility::Foreground => 4,
    Visibility::Visible => 8,
  };
  let soids = if rng.gen::<f32>() < 0.5f32 {
    soid_fx::chordlike::major(32f32, height)
  } else {
    soid_fx::chordlike::minor(32f32, height)
  };

  let maybe_adds: Vec<Soids> = vec![
    soid_fx::ratio::dquince_up(&soids, 3, 0.001f32),
    soid_fx::ratio::quince_up(&soids, 2, 0.01f32),
    soid_fx::ratio::fifth_up(&soids, 1, 0.11f32),
  ];

  let n = match arf.energy {
    Energy::Low => 0,
    Energy::Medium => 1,
    Energy::High => 2,
  };

  let mut selected_mods = maybe_adds.into_iter().take(n).collect::<Vec<_>>();
  selected_mods.push(soids);
  let soids = soid_fx::concat(&selected_mods);
  let stem = (melody, soids, expr(arf), feel, knob_mods, vec![delay::passthrough]);

  Renderable::Group(vec![stem])
}
