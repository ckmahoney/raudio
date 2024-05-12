use hound::Sample;
use rustfft::num_traits::sign;

pub type AmpModulation = Vec<f32>; // must be in range of [0, 1]

pub mod contour;
pub mod lifespan;