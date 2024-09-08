use super::*;
use crate::types::synthesis::{ModifiersHolder,Soids};

const one:f32=1f32;
/// Generates sinusoids representing a stack of octaves (even harmonics) from the fundamental.
pub fn octave(freq:f32) -> Soids {
    let n = (NFf / freq / 2f32) as usize;
    let muls:Vec<f32> = (1..n).step_by(2).map(|ku| ku as f32).collect();
    let amps:Vec<f32> = (0..muls.len()).into_iter().enumerate().map(|(i,ku)| one/(i as f32 + one)).collect();
    let offs:Vec<f32> = (0..muls.len()).into_iter().map(|ku| 0f32).collect();
    (muls, amps, offs)
}