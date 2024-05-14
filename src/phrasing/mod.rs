use hound::Sample;
use rustfft::num_traits::sign;
use crate::analysis;

pub type AmpModulation = Vec<f32>; // must be in range of [0, 1]
use crate::timbre::{Phrasing, FilterMode, BandpassFilter, AmpContour};
use crate::synthesis::FilterPoint;

pub mod contour;
pub mod lifespan;

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
/// @art-curr And a dynamic highpass filter updated by FilterMode
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
    let y = min_frequency + 2f32.powf(p * df.log2());
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
