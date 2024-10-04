use super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};
use rand::{thread_rng, Rng};

static one:f32 = 1f32;

pub mod ratio {
    use super::*;

    /// Add a copy of soids where all mul are multiplied by 'k'
    /// gain affects the amplitude of added copies. original sinus are not affected.
    pub fn constant(soids:&Soids, k:f32, gain: f32) -> Soids {
        let mut ret = soids.clone();
        let gain:f32 = gain * 0.5f32; // this adds one mul per mul, so halve the volume of both.
        soids.0.iter().enumerate().for_each(|(i,m)|{
            // ret.0[i] *= gain;

            ret.0.push(gain);
            ret.1.push(m*k);
            ret.2.push(0f32);
        });
        ret
    }
}

pub mod detune {
    use super::*;

    /// Detunes the multipliers with n additional voices using frequency distortion
    /// Produces a neuro reece effect
    pub fn reece(soids:&Soids, n:usize, depth: f32) -> Soids {
        let mut ret:Soids = soids.clone();
        let mut rng =  thread_rng();
        let max_distance = 8f32;
        let applied_distance = one + (depth * (max_distance - one));
        
        let gain = one / (soids.0.len() * n) as f32;
        (0..n).into_iter().for_each(|i| {
            soids.1.iter().for_each(|m| {
                // put even indicies above the multiplier,
                // odd go below. 
                let mul = if i % 2 == 0 {
                    m * 2f32.powf(rng.gen::<f32>() * applied_distance/48f32)
                } else {
                    m * 2f32.powf(rng.gen::<f32>() * -applied_distance/48f32)
                };
                ret.0.push(gain);
                ret.1.push(mul);
                ret.2.push(0f32);
            })
        });
        ret
    }
}

pub mod noise {
    use crate::druid::noise::NoiseColor;
    use super::*;

    pub fn reso() -> Soids {
        let mut rng = thread_rng();
        let focal:f32 = 7f32 + 5f32 * rng.gen::<f32>();
        let mut soids:Soids = (vec![1f32], vec![focal], vec![0f32]);

        concat(&vec![
            soids.clone(),
            ratio::constant(&soids.clone(), 2f32, 0.33f32),
            ratio::constant(&soids.clone(), 0.66f32, 0.11f32),
            ratio::constant(&soids.clone(), 0.2f32, 0.1f32),
            ratio::constant(&soids.clone(), 3f32, 0.001f32),
        ])
    }

    pub fn rank(register:usize, color:NoiseColor, gain:f32) -> Soids {
        let n = 13 * (register+1);
        let mut rng = thread_rng();
        let mut soids:Soids = (vec![], vec![], vec![]);
        for u in 0..n {
            let m = rng.gen::<f32>();
            let r = register as f32 +m;
            let a = gain * match color {
                NoiseColor::Pink => (r/MAX_REGISTER as f32).sqrt(),
                NoiseColor::Violet => (r/MAX_REGISTER as f32).powi(2i32),
                _ => rng.gen::<f32>(),
            };
            let a:f32 = 1f32;
            let o = pi2 * rng.gen::<f32>();
            soids.0.push(a);
            soids.1.push(2f32.powf(r));
            soids.2.push(o);
        };
        soids
    }
}

pub mod amod {
    use super::*;

        pub fn reece(soids:&Soids, n:usize) -> Soids {
            
        if n == 0 {
            panic!("Must provide a nonzero value for n")
        }
        let mut new_soids = soids.clone();
        const v:usize = 16;
        
        soids.0.iter().enumerate().for_each(|(i, amp)| {
            let mul = soids.1[i];
            let offset = soids.2[i];

            // add one element above and one element below ai / 48

            for i in 0..n {
                new_soids.0.push(0.5f32 * one / ((i+1) as f32).powi(2i32));
                let modulated_over = 2f32.powf(i as f32/v as f32);
                new_soids.1.push(mul * modulated_over);
                new_soids.2.push(offset);
            }
            for i in 0..n {
                new_soids.0.push(one / ((i+1) as f32).sqrt());
                new_soids.0.push(one);
                let modulated_under = one - (2f32.powf(i as f32/v as f32) - one);
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
    pub fn triangle(soids:&Soids, n:usize) -> Soids {
        let ref_freq = 2f32.powf(NFf.log2() - (n as f32).log2());
        let samples = soids::overs_triangle(ref_freq);
        let mut ret:Soids = soids.clone();

        soids.0.iter().enumerate().for_each(|(i, amp)| {
            let mul = soids.1[i];
            let offset = soids.2[i];
            let rescaled_muls:Vec<f32> = samples.1.clone().iter().map(|m| mul * m ).collect();

            ret.0.extend(samples.0.clone());
            ret.1.extend(rescaled_muls);
            ret.2.extend(samples.2.clone());
        });
        ret
    }
    /// applies a short form square wave to each member of the soids
    /// n parameter describes how many multipliers to add of the modulation series
    pub fn square(soids:&Soids, n:usize) -> Soids {
        let ref_freq = 2f32.powf(NFf.log2() - (n as f32).log2());
        let samples = soids::overs_square(ref_freq);
        let mut ret:Soids = soids.clone();
  
        soids.0.iter().enumerate().for_each(|(i, amp)| {
            let mul = soids.1[i];
            let offset = soids.2[i];
            let rescaled_muls:Vec<f32> = samples.1.clone().iter().map(|m| mul * m ).collect();

            ret.0.extend(samples.0.clone());
            ret.1.extend(rescaled_muls);
            ret.2.extend(samples.2.clone());
        });
        ret
    }

    /// applies a short form sawtooth wave to each member of the soids
    /// n parameter describes how many multipliers to add of the modulation series
    pub fn sawtooth(soids:&Soids, n:usize) -> Soids {
        let ref_freq = 2f32.powf(NFf.log2() - (n as f32).log2());
        let samples = soids::overs_sawtooth(ref_freq);
        let mut ret:Soids = soids.clone();

        soids.0.iter().enumerate().for_each(|(i, amp)| {
            let mul = soids.1[i];
            let offset = soids.2[i];
            let rescaled_muls:Vec<f32> = samples.1.clone().iter().map(|m| mul * m ).collect();

            ret.0.extend(samples.0.clone());
            ret.1.extend(rescaled_muls);
            ret.2.extend(samples.2.clone());
        });
        ret
    }
}

pub mod pmod {
    use super::*;

    pub fn reece(soids:&Soids, n:usize) -> Soids {
            
        if n == 0 {
            panic!("Must provide a nonzero value for n")
        }

        let mut new_soids = soids.clone();
        const v:f32 = pi;
        let dv:f32 = v as f32 / n as f32;
        
        soids.0.iter().enumerate().for_each(|(i, amp)| {
            let mul = soids.1[i];
            let offset = soids.2[i];

            // add one element past and future to pi

            for i in 0..n {
                new_soids.0.push(0.5f32 * one / ((i+1) as f32));
                new_soids.1.push(mul);
                new_soids.2.push(offset + i as f32 * dv);

                new_soids.0.push(0.5 * one / ((i+1) as f32));
                new_soids.1.push(mul);
                new_soids.2.push(offset - i as f32 * dv);
            }

        });
        new_soids
    }
}


pub type SoidMod = (fn (&Soids, n:usize) -> Soids, f32);

/// Given a seed soids and a collection of (soid_fx, gain), 
/// Applies the soid fx to the collection at adjusted volume
/// and return the new (modulated) soids
pub fn map(soids:&Soids, n:usize, fx:Vec<SoidMod>) -> Soids {
    let mut additions:Vec<Soids> = vec![];
    let mut ret = soids.clone();
    for (f, weight) in fx { 
        let mut ss:Soids = f(soids, n);
        ss.0.iter_mut().for_each(|mut amp| *amp *= weight);
        additions.push(ss)
    }
    // move all of the neew additions into a copy of the original
    for (mut amps, mut muls, mut offsets) in additions {
        ret.0.extend(amps);
        ret.1.extend(muls);
        ret.2.extend(offsets);
    };
    ret
}

/// Given a collection of sinusoidal args, merge them all into a single Soids tuple.
pub fn concat(soids:&Vec<Soids>) -> Soids {
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