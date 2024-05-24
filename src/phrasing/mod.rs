use hound::Sample;
use rustfft::num_traits::sign;
use crate::analysis;

pub type AmpModulation = Vec<f32>; // must be in range of [0, 1]
use crate::timbre::{Phrasing, FilterMode, BandpassFilter, AmpContour};
use crate::synthesis::FilterPoint;

pub mod contour;
pub mod lifespan;
pub mod ranger;

use crate::synth::{pi2, SR};
use once_cell::sync::Lazy;

pub static filter_points:[FilterPoint; 3] = [
    FilterPoint::Constant,
    FilterPoint::Head,
    FilterPoint::Tail,
];

pub static filter_modes:[FilterMode; 2] = [
    FilterMode::Linear,
    FilterMode::Logarithmic,
];

pub static UNIT_FADE_FORWARD: Lazy<AmpModulation> = Lazy::new(|| {
    contour::gen_contour(SR, 1f32, &AmpContour::Fade, false)
});

pub static UNIT_FADE_REVERSE: Lazy<AmpModulation> = Lazy::new(|| {
    contour::gen_contour(SR, 1f32, &AmpContour::Fade, true)
});

pub static UNIT_SURGE_FORWARD: Lazy<AmpModulation> = Lazy::new(|| {
    contour::gen_contour(SR, 1f32, &AmpContour::Surge, false)
});

pub static UNIT_SURGE_REVERSE: Lazy<AmpModulation> = Lazy::new(|| {
    contour::gen_contour(SR, 1f32, &AmpContour::Surge, true)
});

/// activation function for bandpass filter as a function of absolute frequency and progress.
/// 
/// Configurable with timbre::BandpassFilter.
/// 
/// Returns boolan, true says frequency is OK and false says to filter it out.
/// static @art-curr Use a static lowpass filter from the given max_freq
/// @art-curr A dynamic highpass filter configured by FilterMode
pub fn bandpass_filter(filter:&BandpassFilter, freq:f32, p:f32) -> bool {
    let (mode, direction, (min_f, max_f)) = filter;
    let min_frequency = *min_f;
    let max_frequency = *max_f;

    match direction {
        FilterPoint::Constant => {
            return freq > min_frequency && freq < max_frequency;
        },
        _ => {
            if freq > max_frequency {
                return false
            }
        }
    };

    let reverse = match direction {
        FilterPoint::Head => {
            true
        },
        FilterPoint::Tail => {
            false
        },
        _ => {
            panic!("Impossible path")
        }
    };

    // use a contour to set the current filter bounds.
    // When input freq is within the bounds, allow passage.
    // Else filter it out
    let df = max_frequency - min_frequency;
    let ref_mod = match mode {
        FilterMode::Linear => {
            if reverse {
                &UNIT_FADE_REVERSE
            } else {
                &UNIT_FADE_FORWARD
            }
        },
        FilterMode::Logarithmic => {
            if reverse {
                &UNIT_SURGE_REVERSE
            } else {
                &UNIT_SURGE_FORWARD
            }
        }
    };

    let mut ind = (p * ref_mod.len() as f32) as usize;
    if ind == ref_mod.len() {
        ind = ind - 1;
    }
    let yi = ref_mod[ind];
    let r = df.log2();
    let d_cap = freq - 2f32.powf(r.floor());

    // increase the two components:
    // primary power which increases the min frequency logarithmically 
    // additional contribution incrementing min frequency linearly (to reduce loss)
    // when p == 0 the min allowed value is min_frequency
    // when p == 1 the min allowed value is min_frequency + frequency
    
    let y = min_frequency + 2f32.powf(yi * r) + (yi * d_cap);
    let ok = freq >= min_frequency + y;
    return ok
}

/// Given a vector that might be an amplitude modulator,
/// verify it meets the requirements to modulate a signal's amplitude:
/// Must have all elements be in [0, 1]
/// and must not be ID or 0 matrix 
pub fn verify_amp_mod(vec:Vec<f32>) -> bool {
    if vec.is_empty() {
        return false 
    }

    let min = vec.iter().fold(1f32, |acc, y| if *y < acc { *y } else { acc } );
    let max = vec.iter().fold(0f32, |acc, y| if *y > acc { *y } else { acc } );
    if min < 0f32 || max > 1f32 {
        return false 
    }

    let rms = analysis::volume::rms(&vec);
    if rms == 0f32 || rms == 1f32 {
        return false
    }

    return true
}


pub fn gen_cocktail(n:usize)-> Vec<ranger::Mixer> {
    use rand;
    use rand::Rng;
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();

    if n > ranger::options.len() {
        panic!("Requested more rangers than are available. Repeating the same ranger is the same as boosting its weight.")
    }

    let weights:Vec<f32> = if n == 1usize {
        vec![1f32]
    } else {
        let init = rng.gen();
        let mut ws = vec![init];
        for i in 0..(n-1) {
            let rem = 1f32 - ws.iter().sum::<f32>();
            let next = if i == (n-2) { rem } else {
                rng.gen::<f32>() * rem
            };
            ws.push(next) 
        }

        ws
    };

    let mut opts = ranger::options.to_vec();
    opts.shuffle(&mut rng);
    let rangers:Vec<ranger::Ranger> = opts.to_vec().iter().cloned().take(n).collect();   
    weights.into_iter().zip(rangers.into_iter()).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    const MONICS: [usize; 59] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59
    ];

    const DOMAIN: [f32; 48000] = {
        let mut array = [0.0; 48000];
        let mut i = 0;
        while i < 48000 {
            array[i] = i as f32 / 48000.0;
            i += 1;
        }
        array
    };


    #[test]
    fn test_gen_mixer() {
        let n:usize = 3;
        let d = 1f32;
        let min = 0f32;
        let max = 1f32;

        let mixers = gen_cocktail(n);
        for k in MONICS {
            let kf = k as f32;
            let mut has_value = false;
            let mut not_one = false;
            for x in DOMAIN {
                let y = ranger::mix(kf, x, d, &mixers);
                if y > 0f32 && !has_value {
                    has_value = true
                };
                if y < 1f32 && !not_one {
                    not_one = true
                };
                assert!(y >= min, "Mixing rangers must not produce values below {}", min);
                assert!(y <= max, "Mixing rangers must not produce values above {}", max);
            }
            assert!(has_value, "Mixing rangers must not be 0 valued over its domain");
            assert!(not_one, "Mixing rangers must not be 1 valued over its domain");
        }
    }

}