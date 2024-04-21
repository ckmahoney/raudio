use crate::types::synthesis::Ratio;
use crate::synth::SR;

/// Given dynamic playback rate and constant sample rate, 
/// determines the number of samples required to recreate
/// one second of audio signal.
pub fn samples_per_cycle(cps:f32) -> usize {
    (SR as f32 / cps) as usize
}

pub fn cycles_from_n(cps:f32, n:usize) -> f32 {
    let one = samples_per_cycle(cps) as f32;
    n as f32/one
}

pub fn samples_of_duration(cps:f32, d:&Ratio) -> usize {
    ((SR as f32 / cps) * dur(cps, &d)) as usize
}

pub fn samples_of_dur(cps:f32, dur:f32) -> usize {
    ((SR as f32 / cps) * dur) as usize
}

pub fn samples_from_dur(cps:f32, dur:f32) -> usize {
    ((SR as f32 / cps) * dur) as usize
}

pub fn samples_of_cycles(cps:f32, k:f32) -> usize {
    (samples_per_cycle(cps) as f32 * k) as usize
}

pub fn dur(cps: f32, ratio:&Ratio) -> f32 {
    (ratio.0 as f32 / ratio.1 as f32)/cps
}

pub fn duration_to_cycles((numerator, denominator):Ratio) -> f32 {
    numerator as f32/denominator as f32
}