use crate::analysis::delay::{DelayParams, passthrough};
use crate::reverb::convolution::ReverbParams;
use crate::types::timbre::{Enclosure, SpaceEffects, Positioning, AmpContour, Distance, Echo};
use rand::{self, Rng, rngs::ThreadRng};

/// Given a client request for positioning and artifacting a melody,
/// produce application parameters to create the effect.
/// 
/// enclosure contributes to reverb and delay time
/// distance contributes to gain and reverb
/// echoes contributes to delay
/// complexity contributes to reverb, reverb as saturation, and delay times 
fn positioning(cps:f32, enclosure:&Enclosure, Positioning {complexity, distance, echo}:&Positioning) -> SpaceEffects  {
    let delays:Vec<DelayParams>=vec![];
    let reverbs:Vec<ReverbParams>=vec![];
    let mut gain = 1f32;
    let mut rng = rand::thread_rng();


    match distance {
        Distance::Far => 1f32,
        Distance::Near => 3f32,
        Distance::Adjacent => 2f32
    };

    match enclosure {
        Enclosure::Spring => 1,
        Enclosure::Room => 2,
        Enclosure::Hall => 3,
        Enclosure::Vast => 4,
    };

    match echo {
        None => vec![passthrough],
        Some(Echo::Slapback) => vec![
            gen_slapback(&mut rng, *complexity)
        ],
        Some(Echo::Trailing) => vec![
            gen_trailing(cps, &mut rng, *complexity)
        ]
    };

    SpaceEffects {
        delays,
        reverbs,
        gain: 0f32
    }
}


/// short delay with loud echo
/// works best with percussive or plucky sounds
fn gen_slapback(rng:&mut ThreadRng, complexity:f32) -> DelayParams {
    let n_echoes = if complexity < 0.5f32 { 1 } else { 2 };
    let len_seconds:f32 = rng.gen::<f32>().powi(2i32)/2f32;
    let gain:f32 = 0.5f32 + rng.gen::<f32>()/3f32;
    DelayParams { mix: 0.5f32, len_seconds, n_echoes, gain }
}

/// longer delay with fading echoes
fn gen_trailing(cps:f32, rng:&mut ThreadRng, complexity:f32) -> DelayParams {
    let n_echoes = if complexity < 0.33f32 { 
            rng.gen_range(3..5) 
        } else if complexity < 0.66 { 
            rng.gen_range(4..7)
        } else {
            rng.gen_range(5..9)
    };
    let rate = 1f32 / 2f32 * rng.gen_range(1..5) as f32;
    let len_seconds:f32 = rate / cps;
    let gain:f32 = 0.66 * rng.gen::<f32>();
    DelayParams { mix: 0.5f32, len_seconds, n_echoes, gain }
}