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
}

pub mod pmod {
  use super::*;

  pub fn reece(soids: &Soids, n: usize) -> Soids {
    if n == 0 {
      panic!("Must provide a nonzero value for n")
    }

    let mut new_soids = (vec![], vec![], vec![]);
    const v: f32 = pi_2 / 2f32;
    let dv: f32 = v as f32 / n as f32;

    // create two wet copies of each soid and reduce amplitude of each by half
    soids.0.iter().enumerate().for_each(|(i, amp)| {
      let gain = 0.5 * soids.0[i] / n as f32;
      let mul = soids.1[i];
      let offset = soids.2[i];

      // add one element past and future to pi

      for i in 0..n {
        new_soids.0.push(gain / ((i + 1) as f32));
        new_soids.1.push(mul);
        new_soids.2.push(offset + i as f32 * dv);

        new_soids.0.push(gain / ((i + 1) as f32));
        new_soids.1.push(mul);
        new_soids.2.push(offset - i as f32 * dv);
      }
    });
    new_soids
  }
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
