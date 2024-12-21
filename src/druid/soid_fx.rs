use super::*;
use crate::types::synthesis::{ModifiersHolder, Soids};
use rand::{thread_rng, Rng};

static one: f32 = 1f32;

fn mul_metrics(soids:&Soids) -> (f32, f32) {
  let mut min_mul = 100f32;
  let mut max_mul = 0f32;
  for mul in soids.1.iter() {
    min_mul = min_mul.min(*mul);
    max_mul = max_mul.max(*mul);
  }
  (min_mul, max_mul)
}

// fn degrade(soids:&Soids, q:f32) -> Soids {
//   if soids.1.len() < 8 {
//     return soids.clone()
//   }
//   // the min/max percentages of elements removed. 
//   // goes on the idea that you need 7 soids to create a distinct mask, 
//   // so a 1/8 chance means 8 or more soids may start to sound degraded on each call.
//   let (minp, maxp) = (1f32/8f32, 1f32/2f32);
//   let (min_mul, max_mul) = mul_metrics(soids);

//   let cap_octaves = 8f32;
//   let n_octaves = (max_mul/min_mul).log2().min(cap_octaves);
//   let n_percent_removals = minp + (maxp - minp) * n_octaves / cap_octaves;
//   let n_removals = n_percent_removals * soids.len() as usize;

// }

/// Methods for operating in terms of rational expressions (over multiplier) on a collection of soids
pub mod ratio {
  use super::*;

  /// Add a copy of soids where all mul are multiplied by 'k'
  /// gain affects the amplitude of added copies. original sinus are not affected.
  pub fn constant(soids: &Soids, k: f32, gain: f32) -> Soids {
    let mut ret = soids.clone();
    let gain: f32 = gain * 0.5f32; // this adds one mul per mul, so halve the volume of both.
    soids.0.iter().enumerate().for_each(|(i, m)| {
      // ret.0[i] *= gain;

      ret.0.push(gain);
      ret.1.push(m * k);
      ret.2.push(0f32);
    });
    ret
  }

  /// Create a copy of soids where all mul are multiplied by a constant factor
  /// to produce a boost at the "perfect fifth"
  /// range adds octaves up (0 is 1 octave)
  /// gain affects the amplitude of added copies. original sinus are not affected.
  pub fn fifth_up(soids: &Soids, range: usize, gain: f32) -> Soids {
    let a = soids.0.len() * range + 1;
    let mut ret: Soids = (Vec::with_capacity(a), Vec::with_capacity(a), Vec::with_capacity(a));

    for b in 0..(range + 1) {
      let k: f32 = 2f32.powi(b as i32) * 1.5f32;
      let amp = gain / 2f32.powi(b as i32);
      soids.0.iter().enumerate().for_each(|(i, m)| {
        ret.0.push(amp);
        ret.1.push(m * k);
        ret.2.push(0f32);
      });
    }

    ret
  }

  /// Create a copy of soids where all mul are multiplied by a constant factor
  /// to produce a boost at the "perfect fifth"
  /// range adds octaves up (0 is 1 octave)
  /// gain affects the amplitude of added copies. original sinus are not affected.
  pub fn quince_up(soids: &Soids, range: usize, gain: f32) -> Soids {
    let a = soids.0.len() * range + 1;
    let mut ret: Soids = (Vec::with_capacity(a), Vec::with_capacity(a), Vec::with_capacity(a));

    for b in 0..(range + 1) {
      let k: f32 = 2f32.powi(b as i32) * 1.5f32.powi(2i32);
      let amp = gain / 2f32.powi(b as i32);
      soids.0.iter().enumerate().for_each(|(i, m)| {
        ret.0.push(amp);
        ret.1.push(m * k);
        ret.2.push(0f32);
      });
    }

    ret
  }

  /// Create a copy of soids where all mul are multiplied by a constant factor
  /// to produce a boost at the "perfect fifth"
  /// range adds octaves up (0 is 1 octave)
  /// gain affects the amplitude of added copies. original sinus are not affected.
  pub fn dquince_up(soids: &Soids, range: usize, gain: f32) -> Soids {
    let a = soids.0.len() * range + 1;
    let mut ret: Soids = (Vec::with_capacity(a), Vec::with_capacity(a), Vec::with_capacity(a));

    for b in 0..(range + 1) {
      let k: f32 = 2f32.powi(b as i32) * 1.5f32.powi(3i32);
      let amp = gain / 2f32.powi(b as i32);
      soids.0.iter().enumerate().for_each(|(i, m)| {
        ret.0.push(amp);
        ret.1.push(m * k);
        ret.2.push(0f32);
      });
    }

    ret
  }
}

pub mod noise {
  use super::*;
  use crate::druid::noise::NoiseColor;

  pub fn reso() -> Soids {
    let mut rng = thread_rng();
    let focal: f32 = 7f32 + 5f32 * rng.gen::<f32>();
    let mut soids: Soids = (vec![1f32], vec![focal], vec![0f32]);

    concat(&vec![
      soids.clone(),
      ratio::constant(&soids.clone(), 2f32, 0.33f32),
      ratio::constant(&soids.clone(), 0.66f32, 0.11f32),
      ratio::constant(&soids.clone(), 0.2f32, 0.1f32),
      ratio::constant(&soids.clone(), 3f32, 0.001f32),
    ])
  }

  pub fn resof(focal: f32) -> Soids {
    let mut rng = thread_rng();
    let mut soids: Soids = (vec![0.5f32], vec![focal], vec![0f32]);

    concat(&vec![
      soids.clone(),
      ratio::constant(&soids.clone(), 2f32, 0.133f32),
      ratio::constant(&soids.clone(), 0.66f32, 0.011f32),
      ratio::constant(&soids.clone(), 0.2f32, 0.0132),
      ratio::constant(&soids.clone(), 3f32, 0.001f32),
    ])
  }

  pub fn rank(register: usize, color: NoiseColor, gain: f32) -> Soids {
    let n = 13 * 2i32.pow(register as u32) as usize;
    let mut rng = thread_rng();
    let mut soids: Soids = (vec![], vec![], vec![]);
    for u in 0..n {
      let r = rng.gen_range(1..n) as f32;
      let a = gain
        * match color {
          NoiseColor::Pink => (r / MAX_REGISTER as f32).sqrt(),
          NoiseColor::Violet => (r / MAX_REGISTER as f32).powi(2i32),
          _ => rng.gen::<f32>(),
        };
      let a: f32 = 1f32;
      let o = pi2 * rng.gen::<f32>();
      soids.0.push(a);
      soids.1.push(2f32.powf(r));
      soids.2.push(o);
    }
    soids
  }
}

pub mod amod {
  use super::*;

  /// Uses amp and fmod to create a reece effect
  pub fn reece(soids: &Soids, n: usize) -> Soids {
    if n == 0 {
      panic!("Must provide a nonzero value for n")
    }
    let mut new_soids = soids.clone();
    const v: usize = 16;

    soids.0.iter().enumerate().for_each(|(i, amp)| {
      let mul = soids.1[i];
      let offset = soids.2[i];

      // add one element above and one element below i / 48

      for i in 0..n {
        // over
        new_soids.0.push(0.5f32 * one / ((i + 1) as f32).powi(2i32));
        let modulated_over = 2f32.powf(i as f32 / v as f32);
        new_soids.1.push(mul * modulated_over);
        new_soids.2.push(offset);

        // under

        new_soids.0.push(one / ((i + 1) as f32).sqrt());
        new_soids.0.push(one);
        let modulated_under = one - (2f32.powf(i as f32 / v as f32) - one);
        new_soids.1.push(mul * modulated_under);
        new_soids.2.push(offset);
      }
    });
    new_soids
  }

  // log2 based amplitude attenuation, scaled by k 
  pub fn attenuate_bin_k(soids:&Soids, k:f32) -> Soids {
    if k == 0f32 {
      panic!("Must provide a nonzero value for k")
    }
    let mut new_soids = (vec![], vec![], vec![]);

    soids.0.iter().enumerate().for_each(|(i, amp)| {
      let mul = soids.1[i];
      let offset = soids.2[i];

      let modulated_amp = amp / (k * mul.log2());
      new_soids.0.push(modulated_amp);
      new_soids.1.push(mul);
      new_soids.2.push(offset);
    });
    new_soids
  }

  pub fn gain(soids:&Soids, gain:f32) -> Soids {
    let mut ret = soids.clone();
    ret.0.iter_mut().for_each(|amp| *amp *=  gain);
    ret
  }
}

pub mod fmod {
  use super::*;

  /// applies a short form triangle wave to each member of the soids
  /// n parameter describes how many multipliers to add of the modulation series
  pub fn triangle(soids: &Soids, n: usize) -> Soids {
    let ref_freq = 2f32.powf(NFf.log2() - (n as f32).log2());
    let samples = soids::overs_triangle(ref_freq);
    let mut ret: Soids = soids.clone();

    soids.0.iter().enumerate().for_each(|(i, amp)| {
      let mul = soids.1[i];
      let offset = soids.2[i];
      let rescaled_muls: Vec<f32> = samples.1.clone().iter().map(|m| mul * m).collect();

      ret.0.extend(samples.0.clone());
      ret.1.extend(rescaled_muls);
      ret.2.extend(samples.2.clone());
    });
    ret
  }
  /// applies a short form square wave to each member of the soids
  /// n parameter describes how many multipliers to add of the modulation series
  pub fn square(soids: &Soids, n: usize) -> Soids {
    let ref_freq = 2f32.powf(NFf.log2() - (n as f32).log2());
    let samples = soids::overs_square(ref_freq);
    let mut ret: Soids = soids.clone();

    soids.0.iter().enumerate().for_each(|(i, amp)| {
      let mul = soids.1[i];
      let offset = soids.2[i];
      let rescaled_muls: Vec<f32> = samples.1.clone().iter().map(|m| mul * m).collect();

      ret.0.extend(samples.0.clone());
      ret.1.extend(rescaled_muls);
      ret.2.extend(samples.2.clone());
    });
    ret
  }

  /// applies a short form sawtooth wave to each member of the soids
  /// n parameter describes how many multipliers to add of the modulation series
  pub fn sawtooth(soids: &Soids, n: usize) -> Soids {
    let ref_freq = 2f32.powf(NFf.log2() - (n as f32).log2());
    let samples = soids::overs_sawtooth(ref_freq);
    let mut ret: Soids = soids.clone();

    soids.0.iter().enumerate().for_each(|(i, amp)| {
      let mul = soids.1[i];
      let offset = soids.2[i];
      let rescaled_muls: Vec<f32> = samples.1.clone().iter().map(|m| mul * m).collect();

      ret.0.extend(samples.0.clone());
      ret.1.extend(rescaled_muls);
      ret.2.extend(samples.2.clone());
    });
    ret
  }

  /// For every soid, replaces with two soids detuned by intensity k of reference interval b
  /// Using frequency difference scaling like (f-kb)/f
  pub fn reece(soids: &Soids, b: f32, k: usize) -> Soids {
    let mut new_soids = (vec![], vec![], vec![]);
    let ref_b: f32 = crate::analysis::fit(2f32.powi(-((k + 1) as i32)), b);

    // create two wet copies of each soid and reduce amplitude of each by half
    soids.0.iter().enumerate().for_each(|(i, amp)| {
      let gain = 0.5 * soids.0[i];
      let mul = soids.1[i];
      let octave = mul.log2().floor();
      let offset = soids.2[i];

      // scale the frequency modulation with the applied octave
      // for even xformation
      let mf = if octave == 0f32 { ref_b } else { ref_b / octave };

      // add one element above and below f
      new_soids.0.push(gain);
      new_soids.1.push(mul * (1f32 - mf));
      new_soids.2.push(offset);

      new_soids.0.push(gain);
      new_soids.1.push(mul * (1f32 + mf));
      new_soids.2.push(offset);
    });
    new_soids
  }

  pub fn reece2(soids: &Soids, b: f32) -> Soids {
    let mut new_soids = (vec![], vec![], vec![]);

    // create two wet copies of each soid and reduce amplitude of each by half
    soids.0.iter().enumerate().for_each(|(i, amp)| {
      let gain = 0.5 * soids.0[i];
      let mul = soids.1[i];
      let octave = mul.log2().floor();
      let offset = soids.2[i];

      // scale the frequency modulation with the applied octave
      // for even xformation
      let mf = b / mul.log2();

      // add one element above and below f
      new_soids.0.push(gain);
      new_soids.1.push(mul * (1f32 - mf));
      new_soids.2.push(offset);

      new_soids.0.push(gain);
      new_soids.1.push(mul * (1f32 + mf));
      new_soids.2.push(offset);
    });
    new_soids
  }
}

pub mod ffilter {
  use super::*;

  /// Finds all indices in the input where the predicate returns `true`.
  ///
  /// # Example
  /// ```
  /// let numbers = vec![1, 2, 3, 2, 4];
  /// let indices = find_all_indices(numbers, |&x| x == 2);
  /// assert_eq!(indices, vec![1, 3]);
  /// ```
  fn find_all_indices<T>(iter: impl IntoIterator<Item = T>, predicate: impl Fn(&T) -> bool) -> Vec<usize> {
    iter.into_iter()
        .enumerate()
        .filter_map(|(i, item)| if predicate(&item) { Some(i) } else { None })
        .collect()
  }


  /// Removes elements from the vector at the specified sorted indices.
  ///
  /// # Requirements
  /// - `indices` must be sorted in ascending order.
  /// - If `indices` contains out-of-bounds values, they will be ignored.
  ///
  /// # Example
  /// ```
  /// let mut numbers = vec![10, 20, 30, 40, 50];
  /// let indices_to_remove = vec![1, 3];
  /// remove_all_at_sorted_indices(&mut numbers, &indices_to_remove);
  /// assert_eq!(numbers, vec![10, 30, 50]);
  /// ```
  #[inline]
  fn remove_all_at_sorted_indices<T>(vec: &mut Vec<T>, indices: &[usize]) {
      for &index in indices.iter().rev() { // Process in reverse order
          if index < vec.len() {
              vec.remove(index);
          }
      }
  }

  /// Returns all soids with a multiplier larger than x
  pub fn greater_than(soids: &Soids, x:f32) -> Soids {
    let mut ret = soids.clone();
    let indices_to_remove = find_all_indices(soids.1.iter(), |&m| *m > x);
    remove_all_at_sorted_indices(&mut ret.0, &indices_to_remove);
    remove_all_at_sorted_indices(&mut ret.1, &indices_to_remove);
    remove_all_at_sorted_indices(&mut ret.2, &indices_to_remove);
    ret
  }

  /// Returns all soids with a multiplier larger than x
  pub fn less_than(soids: &Soids, x:f32) -> Soids {
    let mut ret = soids.clone();
    let indices_to_remove = find_all_indices(soids.1.iter(), |&m| *m < x);
    remove_all_at_sorted_indices(&mut ret.0, &indices_to_remove);
    remove_all_at_sorted_indices(&mut ret.1, &indices_to_remove);
    remove_all_at_sorted_indices(&mut ret.2, &indices_to_remove);
    ret
  }
}

pub mod pmod {
  use super::*;

  /// Given the farthest offset in terms of π, create `n` equally distributed phase offsets
  /// with linearly falling amplitude for each SOID. Uses exponential distribution inline.
  pub fn reece_chorus(soids: &Soids, n: usize, max_offset: f32) -> Soids {
      if n == 0 {
          panic!("Must provide a nonzero value for n");
      }

      let mut new_soids = (vec![], vec![], vec![]);

      let distribute_exponential = |i: usize| -> f32 {
          let normalized = i as f32 / n as f32;
          normalized.powf(0.5f32) * max_offset
      };

      soids.0.iter().enumerate().for_each(|(i, amp)| {
          let base_amp = amp / n as f32;
          let base_mul = soids.1[i];
          let base_offset = soids.2[i];

          for j in 0..n {
              let offset = distribute_exponential(j);
              new_soids.0.push(base_amp);
              new_soids.1.push(base_mul);
              new_soids.2.push(base_offset + base_mul.log2() * offset); // Apply exponential offset
          }
      });

      new_soids
  }
}

pub fn filter_do<F, M>(
  soids: &Soids,
  modulation: M,
  predicate: F,
) -> Soids
where
  F: Fn(&(f32, f32, f32)) -> bool,
  M: Fn(&Soids) -> Soids,
{
  let mut passes = (vec![], vec![], vec![]);
  let mut fails = (vec![], vec![], vec![]);

  for i in 0..soids.0.len() {
      let amp = soids.0[i];
      let mul = soids.1[i];
      let offset = soids.2[i];
      let soid = (amp, mul, offset);

      if predicate(&soid) {
          passes.0.push(amp);
          passes.1.push(mul);
          passes.2.push(offset);
      } else {
          fails.0.push(amp);
          fails.1.push(mul);
          fails.2.push(offset);
      }
  }

  let modulated_passes = modulation(&passes);

  // Combine the modulated and unmodulated parts
  let merged = (
      [modulated_passes.0, fails.0].concat(),
      [modulated_passes.1, fails.1].concat(),
      [modulated_passes.2, fails.2].concat(),
  );

  merged
}


pub fn filter_or<F, M, N>(
  soids: &Soids,
  modulation1: M,
  modulation2: N,
  predicate: F,
) -> Soids
where
  F: Fn(&(f32, f32, f32)) -> bool,
  M: Fn(&Soids) -> Soids,
  N: Fn(&Soids) -> Soids,
{
  let mut passes = (vec![], vec![], vec![]);
  let mut fails = (vec![], vec![], vec![]);

  for i in 0..soids.0.len() {
      let amp = soids.0[i];
      let mul = soids.1[i];
      let offset = soids.2[i];
      let soid = (amp, mul, offset);

      if predicate(&soid) {
          passes.0.push(amp);
          passes.1.push(mul);
          passes.2.push(offset);
      } else {
          fails.0.push(amp);
          fails.1.push(mul);
          fails.2.push(offset);
      }
  }

  let modulated_passes = modulation1(&passes);
  let modulated_fails = modulation2(&fails);

  // Combine the modulated and unmodulated parts
  let merged = (
      [modulated_passes.0, modulated_fails.0].concat(),
      [modulated_passes.1, modulated_fails.1].concat(),
      [modulated_passes.2, modulated_fails.2].concat(),
  );

  merged
}



pub type SoidMod = (fn(&Soids, n: usize) -> Soids, f32);

/// Given a seed soids and a collection of (soid_fx, gain),
/// Applies the soid fx to the collection at adjusted volume
/// and return the new (modulated) soids
pub fn map(soids: &Soids, n: usize, fx: Vec<SoidMod>) -> Soids {
  let mut additions: Vec<Soids> = vec![];
  let mut ret = soids.clone();
  for (f, weight) in fx {
    let mut ss: Soids = f(soids, n);
    ss.0.iter_mut().for_each(|mut amp| *amp *= weight);
    additions.push(ss)
  }
  // move all of the neew additions into a copy of the original
  for (mut amps, mut muls, mut offsets) in additions {
    ret.0.extend(amps);
    ret.1.extend(muls);
    ret.2.extend(offsets);
  }
  ret
}

pub fn empty_soids() -> Soids {
  (vec![], vec![], vec![])
}

/// Given a collection of sinusoidal args, merge them all into a single Soids tuple.
pub fn concat(soids: &Vec<Soids>) -> Soids {
  let mut amps = Vec::new();
  let mut muls = Vec::new();
  let mut offsets = Vec::new();

  for (s_amps, s_muls, s_offsets) in soids {
    amps.extend(s_amps);
    muls.extend(s_muls);
    offsets.extend(s_offsets);
  }

  (amps, muls, offsets)
}

pub mod chordlike {
  use super::*;

  /// A collection of monics that produce a major chord.
  /// `freq` The reference frequency (fudamental)
  /// `range` The number of harmonic octaves to include. 0 is one octave, 1 is two octaves, etc.
  pub fn major(freq: f32, range: usize) -> Soids {
    let mut max_mul: f32 = NFf / freq;

    if max_mul < 1f32 {
      return empty_soids();
    }

    if max_mul % 2f32 == 0f32 {
      max_mul -= 1f32
    };

    let monics: Vec<f32> = vec![1f32, 3f32, 5f32];
    let mut amps: Vec<f32> = vec![];
    let mut muls: Vec<f32> = vec![];
    let mut offsets: Vec<f32> = vec![];
    for i in 0..(range + 1) {
      for m in &monics {
        let mul = m * 2f32.powi(i as i32);
        if mul <= max_mul {
          let amp = one / (mul * 2f32.powi(i as i32));
          amps.push(amp);
          muls.push(mul);
          offsets.push(0f32);
        }
      }
    }

    (amps, muls, offsets)
  }

  /// A collection of monics that produce a major seventh chord.
  /// `freq` The reference frequency (fudamental)
  /// `range` The number of harmonic octaves to include. 0 is one octave, 1 is two octaves, etc.
  pub fn major_seven(freq: f32, range: usize) -> Soids {
    let mut max_mul: f32 = NFf / freq;

    if max_mul < 1f32 {
      return empty_soids();
    }

    if max_mul % 2f32 == 0f32 {
      max_mul -= 1f32
    };

    let monics: Vec<f32> = vec![1f32, 3f32, 5f32, 1.5f32 * 5f32];
    let mut amps: Vec<f32> = vec![];
    let mut muls: Vec<f32> = vec![];
    let mut offsets: Vec<f32> = vec![];
    for i in 0..(range + 1) {
      for m in &monics {
        let mul = m * 2f32.powi(i as i32);
        if mul <= max_mul {
          let amp = one / (mul * 2f32.powi(i as i32));
          amps.push(amp);
          muls.push(mul);
          offsets.push(0f32);
        }
      }
    }

    (amps, muls, offsets)
  }

  /// A collection of monics that produce a minor seventh chord.
  /// `freq` The reference frequency (fudamental)
  /// `range` The number of harmonic octaves to include. 0 is one octave, 1 is two octaves, etc.
  pub fn minor_seven(freq: f32, range: usize) -> Soids {
    let mut max_mul: f32 = NFf / freq;

    if max_mul < 1f32 {
      return empty_soids();
    }

    if max_mul % 2f32 == 0f32 {
      max_mul -= 1f32
    };

    let monics: Vec<f32> = vec![1f32, 3f32, 5f32, 1.5f32 * 5f32];
    let mut amps: Vec<f32> = vec![];
    let mut muls: Vec<f32> = vec![];
    let mut offsets: Vec<f32> = vec![];
    for i in 0..(range + 1) {
      for m in &monics {
        let k: f32 = 2f32.powi(1i32 + i as i32);
        let mul = crate::analysis::fit(k, one / m);
        if mul <= max_mul {
          let amp = one / (mul * k);
          amps.push(amp);
          muls.push(mul);
          offsets.push(0f32);
        }
      }
    }

    (amps, muls, offsets)
  }

  /// A collection of monics that produce a minor chord.  
  /// This uses undertone minor: where the fundamental (reference) functions as the perfect fifth of the minor chord.
  /// `freq` The reference frequency (fudamental).  
  /// `range` The number of harmonic octaves to include. 0 is one octave, 1 is two octaves, etc.  
  pub fn minor(freq: f32, range: usize) -> Soids {
    let mut max_mul: f32 = NFf / freq;

    if max_mul < 1f32 {
      return empty_soids();
    }

    if max_mul % 2f32 == 0f32 {
      max_mul -= 1f32
    };

    let monics: Vec<f32> = vec![1f32, 3f32, 5f32];
    let mut amps: Vec<f32> = vec![];
    let mut muls: Vec<f32> = vec![];
    let mut offsets: Vec<f32> = vec![];
    for i in 0..(range + 1) {
      for m in &monics {
        let k: f32 = 2f32.powi(1i32 + i as i32);
        let mul = crate::analysis::fit(k, one / m);
        if mul <= max_mul {
          let amp = one / (mul * k);
          amps.push(amp);
          muls.push(mul);
          offsets.push(0f32);
        }
      }
    }

    (amps, muls, offsets)
  }

  /// Convenient preset to make the "relative minor" chord.
  /// This uses undertone minor: where the fundamental (reference) functions as the perfect fifth of the minor chord.
  /// `freq` The reference frequency (fudamental).  
  /// `range` The number of harmonic octaves to include. 0 is one octave, 1 is two octaves, etc.  
  pub fn minor_offset(freq: f32, range: usize) -> Soids {
    let mut max_mul: f32 = NFf / freq;

    if max_mul < 1f32 {
      return empty_soids();
    }

    if max_mul % 2f32 == 0f32 {
      max_mul -= 1f32
    };

    let monics: Vec<f32> = vec![1f32, 3f32, 5f32];
    let mut amps: Vec<f32> = vec![];
    let mut muls: Vec<f32> = vec![];
    let mut offsets: Vec<f32> = vec![];

    let freq_offset: f32 = 1.5f32.powi(-4i32);
    for i in 0..(range + 1) {
      for m in &monics {
        let k: f32 = 2f32.powi(1i32 + i as i32);
        let mul = crate::analysis::fit(k, freq_offset / m);
        if mul <= max_mul {
          let amp = one / (mul * k);
          amps.push(amp);
          muls.push(mul);
          offsets.push(0f32);
        }
      }
    }

    (amps, muls, offsets)
  }

  pub fn dimdom(freq: f32, range: usize) -> Soids {
    let mut max_mul: f32 = NFf / freq;

    if max_mul < 1f32 {
      return empty_soids();
    }

    if max_mul % 2f32 == 0f32 {
      max_mul -= 1f32
    };

    let monics: Vec<f32> = vec![1f32, 3f32, 5f32];
    let mut amps: Vec<f32> = vec![];
    let mut muls: Vec<f32> = vec![];
    let mut offsets: Vec<f32> = vec![];

    let freq_offset_overs: f32 = 1.5f32.powi(2i32);
    let freq_offset_unders: f32 = 1.5f32.powi(-2i32);
    for i in 0..(range + 1) {
      // overtones
      for m in &monics {
        let k: f32 = 2f32.powi(i as i32);
        let mul = crate::analysis::fit(k, freq_offset_overs * m);
        if mul <= max_mul {
          let amp = one / (mul * k).powi(1i32);
          amps.push(amp);
          muls.push(mul);
          offsets.push(0f32);
        }
      }

      // undertones
      for m in &monics {
        let k: f32 = 2f32.powi(1i32 + i as i32);
        let mul = crate::analysis::fit(k, freq_offset_unders / m);
        if mul <= max_mul {
          let amp = one / (mul * k).powi(1i32);
          amps.push(amp);
          muls.push(mul);
          offsets.push(0f32);
        }
      }
    }

    (amps, muls, offsets)
  }
}

use std::collections::{BTreeMap,HashMap};



/// Merges redundant entries in a `Soids` tuple by combining amplitudes and removing duplicates.
///
/// This function reduces redundancy in the input `Soids` by grouping elements with the same
/// multiplier (`muls`) and phase offset (`phase`). Amplitudes (`amps`) are averaged for entries
/// in the same group. The merging process uses quantized keys to ensure precise grouping
/// while avoiding floating-point comparison issues.
///
/// # Arguments
/// - `soids`: A reference to the `Soids` tuple, containing:
///   - `soids.0`: Vector of amplitudes (`amps`)
///   - `soids.1`: Vector of multipliers (`muls`)
///   - `soids.2`: Vector of phase offsets (`phase`)
///
/// # Returns
/// A new `Soids` tuple with merged entries:
/// - `merged_amps`: Averaged amplitudes for each unique `(mul, phase)` pair
/// - `merged_muls`: Unique multipliers (`muls`)
/// - `merged_phases`: Unique phase offsets
///
/// # Example
/// ```
/// let soids = (
///     vec![0.25, 0.25, 0.5, 1.0],
///     vec![440.0, 440.0, 880.0, 440.0],
///     vec![0.0, 0.0, 0.5, 0.0],
/// );
///
/// let merged = merge_soids(&soids);
/// assert_eq!(
///     merged,
///     (
///         vec![0.5, 0.5, 1.0], // Averaged amplitudes
///         vec![440.0, 880.0, 440.0], // Unique multipliers
///         vec![0.0, 0.5, 0.0] // Unique phase offsets
///     )
/// );
/// ```
///
/// # Notes
/// - Phase offsets and multipliers are quantized using a scaling factor (`scale`) to avoid
///   floating-point precision issues when grouping entries.
/// - Use this function to optimize the representation of a `Soids` for synthesis or processing.
///
/// # Panics
/// This function does not panic, as it safely handles empty or invalid inputs.
pub fn merge_soids(soids: &Soids) -> Soids {
    let mut grouped: BTreeMap<(i32, i32), (f32, usize)> = BTreeMap::new();
    let scale = 1_000_000; // Scale factor for quantization

    for i in 0..soids.0.len() {
        let amp = soids.0[i];
        let mul_key = (soids.1[i] * scale as f32).round() as i32;
        let phase_key = (soids.2[i] * scale as f32).round() as i32;

        let entry = grouped.entry((mul_key, phase_key)).or_insert((0.0, 0));
        entry.0 += amp; // Accumulate amplitude
        entry.1 += 1;   // Increment count
    }

    let mut merged_amps = Vec::new();
    let mut merged_muls = Vec::new();
    let mut merged_phases = Vec::new();

    for ((mul_key, phase_key), (total_amp, count)) in grouped {
        merged_amps.push(total_amp / count as f32); // Average amplitude
        merged_muls.push(mul_key as f32 / scale as f32);
        merged_phases.push(phase_key as f32 / scale as f32);
    }

    (merged_amps, merged_muls, merged_phases)
}
