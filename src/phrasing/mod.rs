use crate::analysis;
use hound::Sample;
use rustfft::num_traits::sign;

pub type AmpModulation = Vec<f32>; // must be in range of [0, 1]
use crate::synthesis::FilterPoint;
use crate::timbre::{AmpContour, BandpassFilter, FilterMode, Phrasing};

pub mod contour;
pub mod dynamics;
pub mod lifespan;
pub mod micro;
pub mod older_ranger;
pub mod ranger;

use crate::synth::{pi2, SR};
