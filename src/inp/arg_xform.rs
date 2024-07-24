use crate::analysis::delay::DelayParams;
use crate::reverb::convolution::ReverbParams;
use crate::types::timbre::{SpaceEffects, Positioning, AmpContour, Distance, Echo};



/// Given a client request for positioning and artifacting a melody,
/// produce application parameters to create the effect.
fn design(Positioning {complexity, contour, distance, echo}:&Positioning) -> SpaceEffects  {
    let delays:Vec<DelayParams>=vec![];
    let reverbs:Vec<ReverbParams>=vec![];
    SpaceEffects {
        delays,
        reverbs,
        gain: 0f32
    }
}