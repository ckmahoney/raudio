use hound::Sample;
use rustfft::num_traits::sign;
use crate::analysis;

pub type AmpModulation = Vec<f32>; // must be in range of [0, 1]

pub mod contour;
pub mod lifespan;

/// activation function for bandpass filter. True indicates frequency is OK; false says to filter it out.
pub fn bandpass_filter(filter:&BandpassFilter, phr:&Phrasing, freq:f32, i:usize, n:usize) -> bool {
    let min_frequency = filter.2.0;
    let max_frequency = filter.2.1;
    match filter.0 {
        FilterMode::Linear => {
            match filter.1 {
                FilterPoint::Constant => {
                    return freq > min_frequency && freq < max_frequency;
                },
                FilterPoint::Mid => {
                    true
                },
                FilterPoint::Tail => {
                    true
                }
            }
        },
        FilterMode::Logarithmic => {
            panic!("No implementation for a logarithmic mixer yet")
        }
    }
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