use crate::analysis::volume::db_to_amp;
use crate::analysis::delay::{DelayParams, passthrough};
use crate::reverb::convolution::ReverbParams;
use crate::types::render::{DruidicScoreEntry};
use crate::types::timbre::{Enclosure, SpaceEffects, Positioning, AmpContour, Distance, Echo};
use rand::{self, Rng, rngs::ThreadRng};

pub fn entry<C>((_, arf, _):&DruidicScoreEntry<C>) {
    
    
}

/// Given a client request for positioning and artifacting a melody,
/// produce application parameters to create the effect.
/// 
/// `enclosure` contributes to reverb and delay time
/// `distance` contributes to gain and reverb
/// `echoes` contributes to delay
/// `complexity` contributes to reverb, reverb as saturation, and delay times 
pub fn positioning(cps:f32, enclosure:&Enclosure, Positioning {complexity, distance, echo}:&Positioning) -> SpaceEffects  {
    let mut rng = rand::thread_rng();
    let gain:f32 = match distance {
        Distance::Adjacent => 1f32,
        Distance::Near => db_to_amp(-6f32),
        Distance::Far => db_to_amp(-12f32),
    };

    SpaceEffects {
        delays: gen_delays(&mut rng, cps, echo, *complexity),
        reverbs: gen_reverbs(&mut rng, cps, distance, enclosure, *complexity),
        gain
    }
}

fn gen_delays(rng:&mut ThreadRng, cps:f32, echo:&Option<Echo>, complexity:f32) -> Vec<DelayParams> {
    match *echo {
        None => vec![passthrough],
        Some(Echo::Slapback) => vec![
            gen_slapback(rng, complexity)
        ],
        Some(Echo::Trailing) => vec![
            gen_trailing(cps, rng, complexity)
        ]
    }
}

/// Create a saturation layer and room layer
fn gen_reverbs(rng:&mut ThreadRng, cps:f32, distance:&Distance, enclosure:&Enclosure, complexity:f32) -> Vec<ReverbParams> {
    let gain = match distance {
        Distance::Far => rng.gen::<f32>().powf(0.25f32),
        Distance::Adjacent => rng.gen::<f32>(),
        Distance::Near => rng.gen::<f32>().powi(4i32),
    };
    let gain:f32=0.1f32;

    let rate:f32 = match enclosure {
        Enclosure::Spring => rng.gen::<f32>().powi(8i32).min(0.05),
        Enclosure::Room => rng.gen::<f32>().powi(2i32).min(0.25).max(0.75),
        Enclosure::Hall => rng.gen::<f32>(),
        Enclosure::Vast => rng.gen::<f32>().powf(0.5f32).max(0.5),
    };

    let mul_seconds:f32 = match distance {
        Distance::Far => 1.5f32,
        Distance::Adjacent => 1f32,
        Distance::Near => 0.5f32,
    }; 
    let mul_seconds:f32=1f32;

    let enclosure = Enclosure::Vast;
    let dur:f32 = 32f32 * mul_seconds * match enclosure {
        Enclosure::Spring => rng.gen::<f32>().powi(8i32).min(0.05),
        Enclosure::Room => rng.gen::<f32>().powi(2i32).min(0.5).max(0.1),
        Enclosure::Hall => rng.gen::<f32>(),
        Enclosure::Vast => rng.gen::<f32>().powf(0.25f32).max(0.5),
    } / cps;

    let enclosure:Enclosure = Enclosure::Vast;
    let amp:f32 = gain * match enclosure {
        Enclosure::Spring => 2f32.powi(-8i32),
        Enclosure::Room => 2f32.powi(-9i32),
        Enclosure::Hall => 2f32.powi(-10i32),
        Enclosure::Vast => 2f32.powi(-8i32),
    } * complexity.powf(0.25f32);

    let mix:f32 = match distance {
        Distance::Far => 0.1f32,
        Distance::Adjacent => 0.1f32,
        Distance::Near => 0.1f32,
    };
    let mix:f32 = 1f32;
    vec![
        gen_saturation(cps, complexity),
        ReverbParams { mix, amp, dur, rate }
    ]
}

fn gen_saturation(cps:f32, complexity:f32) -> ReverbParams {
    ReverbParams { mix: 0.5f32, rate: 0.005 * complexity/cps, dur: complexity/cps, amp: complexity.powi(3) }
}

/// Very transparent and live sounding room effect
fn gen_spring(distance:&Distance) -> ReverbParams {
    ReverbParams { mix: 0.05, amp: 1f32, dur: 1f32, rate: 1f32 }
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