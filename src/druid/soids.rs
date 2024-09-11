use super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};

const zero:f32=0f32;
const one:f32=1f32;

/// Generates sinusoids representing a stack of octaves (even harmonics) from the fundamental.
/// Intended to be tall but focused on the low end.
pub fn octave(freq:f32) -> Soids {
    let n = (NFf / freq / 2f32) as usize;
    let mut muls:Vec<f32> = vec![1f32];
    muls.append(&mut (2..n).step_by(2).map(|ku| ku as f32).collect::<Vec<f32>>());
    let l =muls.len();
    let amps:Vec<f32> = (0..muls.len()).into_iter().enumerate().map(|(i,ku)| 
        one/((i as f32 + one).powi(3i32))
    ).collect();
    let offs:Vec<f32> = (0..l).into_iter().map(|ku| 0f32).collect();
    (amps, muls, offs) 
}


/// Generates sinusoids representing undertone placement using a "square" pattern.
/// Intended to be tall but focused on the low end.
pub fn under_square(freq:f32) -> Soids {
    let n = (freq / MFf / 2f32).floor() as usize;

    let mut muls:Vec<f32> = vec![];
    let mut amps:Vec<f32> = vec![];
    let mut offs:Vec<f32> = vec![];

    for i in (1..n).step_by(2) { 
        muls.push(one/(i as f32));
        amps.push(one/(i as f32).powi(3i32));
        if (i + 1)/2 % 2 == 0 {
            offs.push(zero)
        } else {
            offs.push(pi)
        }
    }

    (amps, muls, offs) 
}

use rand::Rng;
pub enum NoiseType {
    Violet,
    Equal,
    Pink
}

pub fn noise(freq:f32, noise_type:NoiseType) -> Soids {
    let max_mul: f32 = NFf / freq; 
    let mut rng = rand::thread_rng();

    let n_stages = max_mul.log2().floor() as usize;
    let mut muls = vec![];
    let mut amps = vec![];
    let mut offs = vec![];

    let min_entries_dec:i32 = if n_stages > 8 {
        2i32
    } else if n_stages > 6 {
        3i32
    } else if n_stages > 3 {
        4i32
    } else { 5i32 };
    let color = match noise_type {
        NoiseType::Violet => 2i32,
        NoiseType::Equal => 0i32,
        NoiseType::Pink => -2i32
    };
    for i in 0..n_stages {
        let n_entries = 2i32.pow((min_entries_dec + i as i32).max(0) as u32);
        let amp = 1f32/(2f32.powi(i as i32)).powi(color);
        for j in 0..n_entries {
            let r1:f32 = rng.gen::<f32>();
            let r2:f32 = rng.gen::<f32>();
            muls.push(2f32.powf(i as f32 + r1 - 1f32));
            amps.push(amp);
            offs.push(pi2 * r2)
        }
    };
    (amps, muls, offs)
}