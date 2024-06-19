use hound::Sample;
use rustfft::num_traits::sign;
use crate::analysis;

pub type AmpModulation = Vec<f32>; // must be in range of [0, 1]
use crate::timbre::{Phrasing, FilterMode, BandpassFilter, AmpContour};
use crate::synthesis::FilterPoint;

pub mod contour;
pub mod lifespan;
pub mod micro;
pub mod ranger;

use crate::synth::{pi2, SR};
use once_cell::sync::Lazy;
