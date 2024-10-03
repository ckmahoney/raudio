use super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};

static one:f32 = 1f32;


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
        let samples = soids::overs_sawtooth(2f32.powi(9i32));
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