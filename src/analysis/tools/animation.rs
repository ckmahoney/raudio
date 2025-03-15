/// This module provides methods for performing changes in dynamics. 
/// The Animator structs specify the allowed parameters that can be animated
/// and their types of motion.


use super::*;

pub enum DynamicMotion {
    Expander(TRAnimator),
    Compressor(TRAnimator),
}

pub enum RangeDistribution {
    Equal,  // any value in range of left/right
    Left,  // more likely the first value 
    Right, // more likely the second value
    Standard // bell curve
}


pub struct AVal {
    left: f32, 
    right: f32, 
    dist: RangeDistribution
}


/// keys are the property to animate
/// values are a tuple of min/max values and spread type (equal, left, right, standard)
pub struct TRAnimator {
    threshold: AVal,
    ratio: AVal,
    mthreshold: MacroMotion,
    mratio: MacroMotion
}

