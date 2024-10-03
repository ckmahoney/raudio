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
        one/((i as f32 + one).powi(1i32))
    ).collect();
    let offs:Vec<f32> = (0..l).into_iter().map(|ku| 0f32).collect();
    (amps, muls, offs) 
}

// all integer components with inverse cubic amp decay
pub fn integer_overs(freq:f32) -> Soids {
    let n = (NFf / freq) as usize;
    let mut muls:Vec<f32> = vec![1f32];
    muls.append(&mut (1..n).map(|ku| ku as f32).collect::<Vec<f32>>());
    let l =muls.len();
    let amps:Vec<f32> = (0..muls.len()).into_iter().enumerate().map(|(i,ku)| 
        one/((i as f32 + one).powi(3i32))
    ).collect();
    let offs:Vec<f32> = vec![0f32; l];
    (amps, muls, offs) 
}

// all integer components with inverse cubic amp decay
pub fn integer_unders(freq:f32) -> Soids {
    let n = (freq / MFf / 2f32) as usize;
    let mut muls:Vec<f32> = vec![1f32];
    muls.append(&mut (1..n).map(|ku| 1f32/ku as f32).collect::<Vec<f32>>());
    let l =muls.len();
    let amps:Vec<f32> = (0..muls.len()).into_iter().enumerate().map(|(i,ku)| 
        one/((i as f32 + one).powi(3i32))
    ).collect();
    let offs:Vec<f32> = vec![0f32; l];
    (amps, muls, offs) 
}

/// The most basic set of soids: a plain sinusoid
pub fn id() -> Soids {
    return (vec![1f32], vec![1f32], vec![0f32])
}

// all integer components with constant amp
pub fn unit(freq:f32) -> Soids {
    let n = (NFf / freq) as usize;
    let mut muls:Vec<f32> = vec![1f32];
    muls.append(&mut (1..n).map(|ku| ku as f32).collect::<Vec<f32>>());
    let l =muls.len();
    let amps:Vec<f32> = vec![1f32; l];
    let offs:Vec<f32> = vec![0f32; l];
    (amps, muls, offs) 
}

/// Generates sinusoids representing undertone placement using an "octave" pattern.
pub fn under_octave(freq: f32) -> Soids {
    let n = (freq / MFf / 2f32) as usize;
    
    // Start with the first undertone (1) and then add reciprocals of even harmonics
    let mut muls: Vec<f32> = vec![1.0];
    muls.append(&mut (2..n).step_by(2).map(|ku| 1.0 / (ku as f32)).collect::<Vec<f32>>());
    
    let l = muls.len();
    
    // Generate amplitudes with cubic decay similar to the octave function
    let amps: Vec<f32> = (0..l).map(|i| 1.0 / ((i as f32 + 1.0).powi(3))).collect();
    
    // No phase offset
    let offs: Vec<f32> = vec![0.0; l];
    
    (amps, muls, offs)
}

/// Generates sinusoids representing undertone placement using a "square" pattern.
/// Intended to be tall but focused on the low end.
pub fn under_experiement(freq:f32) -> Soids {
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

pub fn under_square(freq: f32) -> Soids {
    let n = (freq / MFf / 2f32).floor() as usize;

    let mut muls: Vec<f32> = vec![];
    let mut amps: Vec<f32> = vec![];
    let mut offs: Vec<f32> = vec![];

    for i in (1..=n).step_by(2) { 
        muls.push(1.0 / (i as f32));
        amps.push(4.0 / (pi * i as f32));  
        if (i + 1) / 2 % 2 == 0 {
            offs.push(0.0);  
        } else {
            offs.push(pi);  
        }
    }

    (amps, muls, offs)
}

pub fn under_sawtooth(freq: f32) -> Soids {
    let n = (freq / MFf / 2f32).floor() as usize;

    let mut muls: Vec<f32> = vec![];
    let mut amps: Vec<f32> = vec![];
    let mut offs: Vec<f32> = vec![];

    for i in 1..=n {
        muls.push(1.0 / (i as f32));  // Undertone frequency placement
        amps.push(2.0 / (std::f32::consts::PI * i as f32));  // Sawtooth amplitude scaling
        offs.push(0.0);  // No phase shift for sawtooth wave
    }

    (amps, muls, offs)
}

/// Generates sinusoids representing undertone placement using a "triangle" pattern.
pub fn under_triangle(freq: f32) -> Soids {
    let n = (freq / MFf / 2f32).floor() as usize;

    let mut muls: Vec<f32> = vec![];
    let mut amps: Vec<f32> = vec![];
    let mut offs: Vec<f32> = vec![];

    for i in (1..=n).step_by(2) {
        muls.push(1.0 / (i as f32));  // Undertone frequency placement (odd harmonics)
        let sign = if (i - 1) / 2 % 2 == 0 { 1.0 } else { -1.0 };  // Alternating sign
        amps.push(sign * 8.0 / (std::f32::consts::PI * std::f32::consts::PI * (i as f32).powi(2)));  // Triangle amplitude scaling
        offs.push(0.0);  // No phase shift for triangle wave
    }

    (amps, muls, offs)
}


pub fn overs_square(freq: f32) -> Soids {
    let n = (NFf / freq) as usize;

    let harmonics: Vec<usize> = (1..=n).filter(|&i| i % 2 != 0).collect();
    let multipliers: Vec<f32> = harmonics.iter().map(|&i| i as f32).collect();
    let amplitudes: Vec<f32> = harmonics.iter().map(|&i| 4f32 / (pi * i as f32)).collect();
    let offsets: Vec<f32> = vec![0f32; multipliers.len()];

    (amplitudes, multipliers, offsets)
}

pub fn overs_triangle(freq:f32) -> Soids {
    let n = (NFf / freq) as usize;
    let multipliers:Vec<f32> =  (1..=n).filter(|i| i % 2 != 0).map(|i| i as f32).collect();
    
    let amplitudes:Vec<f32> = (1..=n).filter(|i| i % 2 != 0).map(|i| {
        let sign = if (i - 1) / 2 % 2 == 0 { 1f32 } else { -1f32 };
        sign * 8f32 / (pi * pi * (i as f32).powi(2))
    }).collect();

    let offsets:Vec<f32> = vec![0f32; multipliers.len()];
    (amplitudes, multipliers, offsets)
}

pub fn overs_sawtooth(freq:f32) -> Soids {
    let n = (NFf / freq) as usize;

    let multipliers:Vec<f32> = (1..=n).map(|x| x as f32).collect();
    let amplitudes:Vec<f32> = (1..=n).map(|i| 2f32 / (pi * i as f32)).collect();
    let offsets:Vec<f32> = vec![0f32; multipliers.len()];

    (amplitudes, multipliers, offsets)

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