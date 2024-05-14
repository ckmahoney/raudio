use std::collections::HashMap;
use crate::types::synthesis::Ratio;
use crate::synth::SR;

impl Duration {
    fn new(whole_seconds: i32, divisor: i32) -> Self {
        if divisor == 0 {
            panic!("divisor cannot be zero");
        }
        Duration { whole_seconds, divisor }
    }

    fn value(&self) -> f64 {
        self.whole_seconds as f64 / self.divisor as f64
    }

    fn duration_to_size_map() -> HashMap<Self, usize> {
        let mut map = HashMap::new();
        map.insert(Self::new(15, 1), 3);
        map.insert(Self::new(30, 1), 4);
        map.insert(Self::new(60, 1), 5);
        map.insert(Self::new(90, 1), 6);
        map.insert(Self::new(120, 1), 5);
        map.insert(Self::new(240, 1), 4);
        map
    }
}


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
    samples_from_dur(cps, dur)
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

use std::time::{Instant};

/// Measures the execution time of a function.
///
/// # Arguments
///
/// * `f` - A closure to execute for which the execution time is measured.
///
/// # Returns
///
/// A tuple containing the result of the function and the duration it took to execute.
pub fn measure<T, F: FnOnce() -> T>(f: F) -> (T, std::time::Duration) {
    let start = Instant::now(); // Start timing before the function is called.
    let result = f(); // Call the function and store the result.
    let duration = start.elapsed(); // Calculate how long it took to call the function.
    (result, duration) // Return the result and the duration.
}

/// Given a duration in seconds and select constraints,
/// Return the cps and size required to produce a track of length n_seconds
type Timing = (f64, usize);
fn get_timing(n_seconds:f64, min_cps:f64, base:f64, cpc:f64, min_size:usize) -> Timing {
    let n_cycles = base.powi(min_size as i32) * cpc;
    let cps = n_cycles / n_seconds;
    if cps < min_cps {
        get_timing(n_seconds, min_cps, base, cpc, min_size + 1)
    } else {
        (cps, min_size)
    }

}
#[derive(Hash, Eq, PartialEq)]
struct Duration {
    whole_seconds: i32,
    divisor: i32,
}


/// Given a length in seconds, get an approximate size (assuming base 2, cpc 4) for that duration. 
/// Supports up to 5 minutes with the internal default map.
pub fn get_approx_size(seconds: f64) -> usize {
    let durations = [
        (15.0, 3),
        (30.0, 4),
        (60.0, 5),
        (90.0, 5),
        (120.0, 6),
        (200.0, 6),
        (300.0, 7),
    ];

    durations
        .iter()
        .filter(|(duration, _)| *duration >= seconds)
        .min_by(|(d1, _), (d2, _)| d1.partial_cmp(d2).unwrap())
        .map_or(0, |(_, size)| *size) // Return 0 if no match is found
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_timing() {
        let n_seconds:f64 = 15.0;
        let min_cps:f64 = 0.5;
        let min_size:usize = 3;
        let base:f64 = 2.0;
        let cpc:f64 = 4.0;


        let expected:Timing = (0f64, 0usize);
        let actual:Timing = get_timing(n_seconds, min_cps, base, cpc, min_size);
        println!("got timing {:?}", actual)
    }
    #[test]
    fn test_with_size_map() {
        let test_cases = vec![
            (7.0, 0.5, 3, 2.0, 4.0),   // These tuples are (seconds, min_cps, min_size, base, cpc)
            (12.0, 0.5, 3, 2.0, 4.0),
            (17.0, 0.5, 4, 2.0, 4.0),
            (60.0, 0.5, 5, 2.0, 4.0),
            (61.0, 0.5, 6, 2.0, 4.0),
            (89.9, 0.5, 6, 2.0, 4.0),
            (94.0, 0.5, 5, 2.0, 4.0),
            (123.0, 0.5, 5, 2.0, 4.0),
            (245.0, 0.5, 6, 2.0, 4.0),
            (299.0, 0.5, 5, 2.0, 4.0),
        ];
    
        for (seconds, min_cps, min_size, base, cpc) in test_cases {
            let size = get_approx_size(seconds);
            let actual: Timing = get_timing(seconds, min_cps, base, cpc, size);
            println!("Test for {} seconds -> Expected size: {}, Got a timing: {:?}", seconds, size, actual);
        }
    }
}

