static neg: f32 = -1f32;
static one: f32 = 1f32;
static two: f32 = 2f32;
static half: f32 = 0.5f32;

pub type OldRangerDeprecated = fn(usize, f32, f32) -> f32;
pub type Mixer = (f32, OldRangerDeprecated);
pub type Weight = f32;
pub type WOldRangerDeprecateds = Vec<(Weight, OldRangerDeprecated)>;

/// Collection of optional xformers for amplitude, frequency, and phase.
pub type Modders = [Option<WOldRangerDeprecateds>; 3];

pub static example_options: [OldRangerDeprecated; 3] = [a, b, c];
/// Desmos is a handly tool for previewing contours! This sketch shows a, b, and c
/// as of May 23 2024
/// https://www.desmos.com/calculator/ar9rw3klcs

/// Transformer based on logistic function for output in range [0, 1]
/// Only one conform method is allowed. It should maintain the contour of the input.
fn conform(y: f32) -> f32 {
  // mutation looks good in desmos; can remove this subtraction for a more pure conformation
  let z = y - 0.5;

  let denom: f32 = one + (3f32 * (1.5f32 - z)).exp();
  one / denom
}

/// Given a point (k, x, d) and group of weighted OldRangerDeprecateds,
/// Apply the weighted sum of all OldRangerDeprecateds at (k,x,d)
pub fn mix(k: usize, x: f32, d: f32, mixers: &WOldRangerDeprecateds) -> f32 {
  let weight = mixers.iter().fold(0f32, |acc, wr| acc + wr.0);
  if weight > 1f32 {
    panic!(
      "Cannot mix OldRangerDeprecateds whose total weight is more than 1. Got {}",
      weight
    )
  };

  mixers.iter().fold(0f32, |y, (w, OldRangerDeprecated)| {
    y + (w * OldRangerDeprecated(k, x, d))
  })
}

/// Model based on (1/x)
/// Horizontal: left
/// Vertical: bottom
pub fn a(k: usize, x: f32, d: f32) -> f32 {
  if x == 0f32 {
    return 1f32;
  }

  let y = one / (k as f32 * x * x.sqrt());
  conform(y)
}

/// Model based on (1/x^2)
/// Horizontal: left
/// Vertical: bottom
pub fn b(k: usize, x: f32, d: f32) -> f32 {
  if x == 0f32 {
    return 1f32;
  }

  let y = 0.1f32 * (k as f32).sqrt() / (x * x);
  conform(y)
}

/// Model inspired by the logistic function
/// Horizontal: left
/// Vertical: bottom
pub fn c(k: usize, x: f32, d: f32) -> f32 {
  let p = -0.75f32 * (one + x * (half * k as f32).log10());
  let y = (two / (one - p.exp())) - one;
  conform(y)
}

#[cfg(test)]
mod test {
  use super::*;

  const MONICS: [usize; 59] = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31,
    32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59,
  ];
  const DOMAIN: [f32; 48000] = {
    let mut array = [0.0; 48000];
    let mut i = 0;
    while i < 48000 {
      array[i] = i as f32 / 48000.0;
      i += 1;
    }
    array
  };

  const min: f32 = 0f32;
  const max: f32 = 1f32;
  const d: f32 = 1f32;

  #[test]
  fn test_valid_range() {
    for (i, OldRangerDeprecated) in (&example_options).iter().enumerate() {
      for k in MONICS {
        let mut has_value = false;
        let mut not_one = false;
        for x in DOMAIN {
          let y = OldRangerDeprecated(k, x, d);
          if y > 0f32 && !has_value {
            has_value = true
          };
          if y < 1f32 && !not_one {
            not_one = true
          };
          assert!(
            y >= min,
            "OldRangerDeprecated {} must not produce values below {}",
            i,
            min
          );
          assert!(
            y <= max,
            "OldRangerDeprecated {} must not produce values above {}",
            i,
            max
          );
        }
        assert!(
          has_value,
          "OldRangerDeprecated {} must not be 0 valued over its domain",
          i
        );
        assert!(
          not_one,
          "OldRangerDeprecated {} must not be 1 valued over its domain",
          i
        );
      }
    }
  }

  #[test]
  fn test_mix() {
    let mixers: Vec<Mixer> = (&example_options)
      .iter()
      .map(|OldRangerDeprecated| (1f32 / example_options.len() as f32, *OldRangerDeprecated))
      .collect();
    for k in MONICS {
      let mut has_value = false;
      let mut not_one = false;
      for x in DOMAIN {
        let y = mix(k, x, d, &mixers);
        if y > 0f32 && !has_value {
          has_value = true
        };
        if y < 1f32 && !not_one {
          not_one = true
        };
        assert!(
          y >= min,
          "Mixing OldRangerDeprecateds must not produce values below {}",
          min
        );
        assert!(
          y <= max,
          "Mixing OldRangerDeprecateds must not produce values above {}",
          max
        );
      }
      assert!(
        has_value,
        "Mixing OldRangerDeprecateds must not be 0 valued over its domain"
      );
      assert!(
        not_one,
        "Mixing OldRangerDeprecateds must not be 1 valued over its domain"
      );
    }
  }
}
