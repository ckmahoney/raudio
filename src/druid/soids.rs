use super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};

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