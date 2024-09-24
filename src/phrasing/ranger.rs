use std::os::unix::thread;

use crate::analysis::volume::db_to_amp;
/// # Rangers
/// 
/// These methods offer per-multipler modulation at gentime on a per-sample basis.
/// They all accept some common parameters which have identical meaning across all Ranger functions.
/// They each also have a required "knob" macro which offers three additional range values, to be defined per-method.
/// Generally, knob values of 0 indicate a passthrough effect while knob values of 1 indicate maximal affected modulation.

use crate::types::synthesis::Range;
use crate::synth::{MFf, NFf, SR, SRf, pi, pi2,pi_2,pi_4};
static one:f32 = 1f32;
static half:f32 = 0.5f32;

const MIN_DB:f32=-30f32;
const MAX_DB:f32=0f32;

#[derive(Copy,Clone,Debug)]
/// A set of three dials for managing the parameters of these predefined methods.
/// All values of a, b, or c are standard range [0, 1]
/// The consuming method must define how each of a/b/c are used, if at all.
pub struct Knob {
    pub a: Range,
    pub b: Range,
    pub c: Range,
}

/// A dynamically dispatched callback for modulating amplitude OR frequency OR phase OR time.  
/// Signature:  
/// `knob, cps, fundamental, multiplier, n_cycles, pos_cycles` -> `modulation value`
pub type Ranger  = fn(&Knob, f32, f32, f32, f32, f32) -> f32;

pub type KnobbedRanger = (Knob, Ranger);

#[derive(Clone,Debug)]
pub struct KnobMods (pub Vec<KnobbedRanger>, pub Vec<KnobbedRanger>, pub Vec<KnobbedRanger>);
impl KnobMods {
    pub fn unit() -> Self {
        KnobMods (vec![], vec![], vec![])
    }
}

/// Example noop modulation function applied to the amplitude modulation context. 
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Returns
/// A value for multiplying the gentime amplitude coefficient.
pub fn amod_noop(knob:&Knob, cps:f32, fund:f32, mul:f32, n_cycles:f32, pos_cycles:f32) -> f32 {
    one
}

/// Example noop modulation function applied to the frequency modulation context. 
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Returns
/// A value for multiplying the gentime frequency.
pub fn fmod_noop(knob:&Knob, cps:f32, fund:f32, mul:f32, n_cycles:f32, pos_cycles:f32) -> f32 {
    one
}

/// Example noop modulation function applied to the phase offset context. 
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Returns
/// A value for adding to the gentime phase offset.
pub fn pmod_noop(knob:&Knob, cps:f32, fund:f32, mul:f32, n_cycles:f32, pos_cycles:f32) -> f32 {
    0f32
}

/// A oneshot frequency mod that starts the note octaves higher from its fundamental, logarithmically sweeping down to the k frequency.
/// This modulator has an in-place (1/4) coefficient on the Nyquist Frequency, to prevent aliasing issues that are unexpectedly cropping up. 
/// 
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// `a`: The mix of sweep to apply. 0 indicates no pitch modulation; 1 indicates the maximum possible pitch modulation.  
/// `b`: The decay rate of sweep. 0 indicates the fastest sweep, while 1 indicates a sweep lasting the duration of the note event.  
/// `c`: unused. 
/// 
/// ## Observations
/// The smallest observable effect happens when a=epsilon and b=0.
/// This creats a one octave frequency sweep at most.
/// when b is >= 0.95, the reference frequency is never reached. 
/// 
/// ## Desmos 
/// https://www.desmos.com/calculator/r7fqush5st  
/// 
/// ## Returns
pub fn fmod_sweepdown(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let a = if knob.a == 1f32 {
        // it is bugging out when a=1. this is fine
        0.995 
    } else {
        knob.a
    };
    if a == 0f32{
        return one
    }

    let max_mul: f32 = (NFf/4f32) / (mul * fund); 
    if max_mul < 1f32 || mul.log2() > 5f32 {
        return 1f32
    }
    let t:f32 = pos_cycles/n_cycles;
    let b_coef:f32 = -(100f32 - 95f32*knob.b);
    let decay_mul:f32 = (b_coef * t).exp();
    let scaled_mul = 2f32.powf(a * max_mul.log2());
    one + decay_mul * scaled_mul
}

/// A continuous frequency mod that adds detuning to the harmonic.
/// 
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// `a`: The amount of detune to apply. 0 is none, 1 is maximum amount. 
/// `b`: Decay rate.  0 for faster decay, 1 for slower decay.
/// `c`: unused. 
/// 
/// ## Observations
/// 
/// ## Desmos 
/// for decay rate: https://www.desmos.com/calculator/ze5vckie3q
/// 
/// ## Returns
pub fn fmod_vibrato(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let lowest = (4f32/9f32)*2f32;
    let highest = (9f32/4f32)/2f32;
    let applied_lower_bound = (1f32-lowest) * knob.a;
    let applied_upper_bound = (highest-1f32) * knob.a;

    let t:f32 = pos_cycles/n_cycles;
    let decay = (1f32-t).powf(f32::tan(pi_2*(1f32 - 0.995f32 * knob.b)));
    let y = f32::sin(t * fund.log2());
    
    if y == 0f32 {
        1f32
    } else if y > 0f32 {
        1f32 + decay * y * applied_upper_bound
    } else {
        1f32 - decay * y * applied_lower_bound
    }
}


/// A continuous frequency mod that adds a vintage tape pitch bend effect.
/// 
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// `a`: The amount of detune to apply. 0 is none, 1 is maximum amount. 
/// `b`: unused.  
/// `c`: unused.  
/// 
/// ## Observations
/// 
/// ## Desmos 
/// for decay rate: https://www.desmos.com/calculator/ze5vckie3q
/// 
/// ## Returns
pub fn fmod_warble(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let max_distance_lowest = 1f32 - (31f32/32f32);
    let max_distance_highest = (34f32/32f32) - 1f32;
    let applied_distance_lowest = 1f32 - (knob.a * max_distance_lowest);
    let applied_distance_highest = 1f32 + (knob.a * max_distance_highest);
    let range = applied_distance_highest - applied_distance_lowest;

    let t:f32 = cps * pos_cycles/n_cycles;
    let y = (1f32 + f32::sin(pi2 * t)) / 2f32; // value in [0, 1]
    let val = applied_distance_lowest + y * range;
    val
}



/// A continuous frequency mod that adds detuning to the harmonic.
/// 
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// `a`: The depth of detune to apply. 0 is none, 1 is maximum amount.  
/// `b`: The intensity of detune to apply. 0 is more transparent, 1 is very visible.   
/// `c`: Modulation factor.
///
/// 
/// ## Observations
/// This is a subtle and highly effective modulation. So transparent while very colorful.
/// 
/// ## Desmos 
/// 
/// ## Returns
pub fn pmod_chorus(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let t:f32 = pos_cycles/n_cycles;
    (f32::sin(t * knob.a * pi2 * mul)*pi.powf(knob.b)).powf(1f32 + 0f32 * knob.c * 2f32)
}

pub fn pmod_weird(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let t:f32 = pos_cycles/n_cycles;
    knob.a * t.powf(8f32 - 7f32 * knob.b) * pi * (t * mul.sqrt()).sin() * fund * mul
}

/// A constant peak at some point up to 4 octaves above the fundamental.
/// 
/// Works by damping all other frequencies.
/// Offers a damping intensity scalor.
/// 
/// ## Knob params 
/// `a`: frequency selector   
/// `b`: resonant factor  
/// `c`: damping intensity  
pub fn amod_peak(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let max_mul: f32 = NFf / (mul * fund); 
    let max_freq:f32 = max_mul.max(4f32);
    let target_freq:f32 = fund * 2f32.powf(knob.a * max_freq);
    let df = (target_freq - fund*mul).abs();
    if df < 10f32 {
        1f32
    } else { 
        db_to_amp(knob.b * -60f32)
    }
}


/// A oneshot amplitude modulation adding a "huh" like breathing in for a word.
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// 
/// `a`: The decay rate of the amplitude contour. 0 indicates the biggest breath decay, and 1 offers the least noticable.  
/// `b`: unused.   
/// `c`: unused.  
/// 
/// ## Observations
/// ## Desmos 
/// 
/// ## Returns
pub fn amod_breath(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let t:f32 = pos_cycles/n_cycles;
    let decay = (t).powf(f32::tan(pi_4*(1f32 - (0.8f32 + 0.2 * knob.a))));

    decay
}



/// A oneshot amplitude contouring for microtransients. 
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// 
/// `a`: The total length of the amplitude contour. 0 is the shortest microtransient (12ms) while 1 is the longest (60ms). 
/// `b`: The decay mode of the amplitude contour. 0 is rapid decay, 0.5 is linear, 1 is slow decay.    
/// `c`: Tempo response. 0 for cps invariant durations. 1 for durations that scale with time. Rounds down.   
pub fn amod_microtransient(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    const min_ms:f32 = 12f32;
    const max_ms:f32 = 60f32;  // Adjusted to 60ms as per your earlier comment

    let playback_scaling:f32 = if knob.c.floor() == 0.0 { 1.0 } else { 1.0 / cps };
    let length_ms = (min_ms + (max_ms - min_ms) * knob.a) * playback_scaling;
    let length_seconds = length_ms / 1000f32;
    let seconds_per_sample = 1.0f32 / SRf;

    let n_samples = (length_seconds * SRf).floor() as usize;
    let t:f32 = pos_cycles / n_cycles;
    let curr_sample = (t * n_cycles * SRf / cps).floor() as usize;
    let curr_sample = curr_sample.min(n_samples);

    if curr_sample >= n_samples {
        return 0.0;
    }

    let p:f32 = curr_sample as f32 / n_samples as f32;
    let y = one - (curr_sample as f32 /n_samples as f32);
    db_to_amp(-120f32 + 120f32*y)
}


/// A oneshot amplitude contouring adding a linear rise in amplitude (decibel scaled)
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// 
/// `a`: The total length of the amplitude contour. 0 is off (passthrough) and 1 is half the duration of the noteevent. 
/// `b`: The dynamic range of the contour. 0 is no change (passthrough), 1 is full decibel scale (-90 to 0).    
/// `c`: Multipler delay. 0 is no delay, 0.5 scales time by pow2, 1 scales time by pow4  
pub fn amod_fadein(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    // Prevent division by zero if pos_cycles is 0
    if pos_cycles == 0.0 {
        return 0.0;
    }

    let env_length = n_cycles * knob.a;
    let t = pos_cycles / env_length;

    // If the position exceeds the envelope length, return full amplitude
    if t >= 1.0 {
        return 1.0;
    }

    // Use multiplier to scale the time response
    let s_t = t * mul.powf(1f32/3f32);

    // Dynamic range in decibels
    let dynamic_range_db = -60.0 * knob.b; // Full fade-in is -60dB to 0dB

    // Delay scale based on time and knob.c
    let delay_scale = s_t.powf(1.0 + 3.0 * knob.c); 

    // Calculate amplitude in decibels and convert to linear
    let amplitude_db = dynamic_range_db * delay_scale;
    1f32 - db_to_amp(amplitude_db)
}

#[test]
fn test_amod_fadein_monotonic_increasing() {
    const EPSILON: f32 = 1e-6;
    let knob = Knob { a: 1.0, b: 0.5, c: 0.5 };
    let cps: f32 = 1.3;  
    let fund: f32 = 440.0; 
    let mul: f32 = 1.0;

    let n_cycles: f32 = 1.0;
    let mut last_value = 0.0; // Start with minimum amplitude

    for sample in 0..SR {
        let pos_cycles = sample as f32 / SRf;
        let result = amod_fadein(&knob, cps, fund, mul, n_cycles, pos_cycles);
        assert!(result >= last_value, "Value at sample {} was not monotonically increasing. Prev sample {} Curr sample {} ", sample, last_value, result);
        last_value = result;
    }

    assert!((1.0 - last_value).abs() <= EPSILON, "Final value was not close to 1: {}", last_value);
}



#[test]
fn test_amod_microtransient_monotonic_decreasing() {
    const EPSILON: f32 = 1e-6;
    let knob = Knob { a: 0.0, b: 0.5, c: 0.0 };
    let cps: f32 = 1.3;  
    let fund: f32 = 440.0; 
    let mul: f32 = 1.0;

    let n_cycles: f32 = 1.0;
    let mut last_value = 1.0; 

    for sample in 0..SR {
        let pos_cycles = sample as f32 / SRf;
        let result = amod_microtransient(&knob, cps, fund, mul, n_cycles, pos_cycles);
        assert!(result <= last_value, "Value at sample {} was not monotonically decreasing. Prev sample {} Curr sample {} ", sample, last_value, result);
        last_value = result;
    }

    assert!(last_value <= EPSILON, "Final value was not close to zero: {}", last_value);
}


    // use a negative parabola whose peak is at (0.5, 1)
    // so it is linear at the center of [0, 1]
    // and gradually falls to either side with contour
    // f(x) = 1-1*2*(x-0.5)(x-0.5) 
    //   0.25 ^ 2 = 0.0625
    //   0.25 ^ 0.5 = 0.5

/// A oneshot amplitdue modulation for rapid decay.
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// 
/// `a`: The decay rate of the amplitude contour. 0 indicates the fastest decay, and 1 offers the longest decay.  
/// `b`: unused.   
/// `c`: unused.  
/// 
/// ## Observations
/// 
/// Values of a below 0.5 are good for very rapid decays. Below 0.3 is microtransient territory.
/// Values below 0.95 produce a 0 value before the end of t. 
/// Values above 0.95 will never reach 0 for t.
/// 
/// ## Desmos 
/// https://www.desmos.com/calculator/luahaj4rwa  
/// 
/// ## Returns
pub fn amod_impulse(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let t:f32 = pos_cycles/n_cycles;
    let decay_rate:f32 = 105f32*(one-knob.a);
    let contrib1 = ((half/(one-(-t).exp()))-half).tanh();
    let contrib2 = one - (decay_rate*t-pi).tanh();
    half * contrib1 * contrib2
}

/// A oneshot amplitdue modulation for rapid decay.
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// 
/// `a`: Base decay rate. Value of 0 is the shortest possible base decay, and 1 is the longest.  
/// `b`: Upper harmonic decay sensitivity. Value of 0 means all harmonics have same decay. Value of 1 means later harmonics decay much more rapidly than early harmonics.  
/// `c`: unused.  
/// 
/// ## Desmos 
/// https://www.desmos.com/calculator/jw8vzd2ie1
/// 
/// ## Returns
pub fn amod_pluck(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let max_mul: f32 = NFf / (mul * fund); 
    let t:f32 = pos_cycles/n_cycles;
    let base_decay_rate:f32 = 5f32 + 20f32 * (1f32 - knob.a);
    let decay_mod_add:f32 = 120f32 * (knob.b).powi(2i32) * (mul/max_mul);
    let decay_rate:f32 = base_decay_rate + decay_mod_add;
    1f32/(decay_rate*t).exp()
}


/// A oneshot amplitdue modulation for slow decay.
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// 
/// `a`: Base decay rate. Value of 0 has the least overall volume. Value of 1 has the most overall volume.   
/// `b`: Upper harmonic stretch. Value of 0 means all multipliers have the same decay rate. Value of 1 adds tiny volume as mul increases. Has more prominent effect for low a values. For a=1, is nearly invisible.
/// `c`: unused.  
/// 
/// ## Desmos 
/// https://www.desmos.com/calculator/4gc7quzfvw
/// 
/// ## Returns
pub fn amod_burp(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let max_mul: f32 = NFf / (mul * fund);
    let t:f32 = pos_cycles/n_cycles;
    let base_decay_rate:f32 = 2f32 + (8f32 * knob.a);
    let decay_mod_add:f32 = 2f32 * knob.b * mul/max_mul;
    (1f32 - t.powf(base_decay_rate + decay_mod_add)).powi(5i32)
}

/// A continuous amplitdue modulation for periodic falling linear contour.
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// 
/// `a`: Tremelo period. Value of 0 is 1 hits per note, and value of 1 is 2^4 hits per note.  
/// `b`: unused.   
/// `c`: unused.  
/// 
/// ## Desmos 
/// https://www.desmos.com/calculator/2bf9xonkci
/// 
/// ## Returns
pub fn amod_oscillation_tri(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let t:f32 = pos_cycles/n_cycles;
    let osc_rate:f32 = 2f32.powf(-4f32 * knob.a);
    let offset_b:f32 = osc_rate / 2f32;
    let y:f32 = (osc_rate - t + offset_b)/osc_rate;
    one - knob.b * (0.5f32 + (y - (y + 0.5).floor())).powi(2i32)
}

/// A continuous amplitdue modulation for smooth sine envelopes.
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// 
/// `a`: Tremelo period. Value of 0 is 1 hits per note, and value of 1 is 2^4 hits per note.  
/// `b`: Tremelo intensity. Value of 0.01 is barely perceptable, least dynamic range. Value of 1 is maximum dynamic range.   
/// `c`: unused.  
/// 
/// ## Desmos 
/// https://www.desmos.com/calculator/ag0vtteimu
/// 
/// ## Returns
pub fn amod_oscillation_sine(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let t:f32 = pos_cycles/n_cycles;
    let osc_mod_mul:f32 = 2f32.powf(knob.a * 4f32);
    1f32 - knob.b * (pi2 * pos_cycles * osc_mod_mul).sin().abs().powi(4i32)
}

/// A continuous amplitdue taking as long as possible to reach the peak.
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// 
/// `a`:   
/// `b`:   
/// `c`: unused.  
/// 
/// 
/// ## Returns
pub fn amod_slowest(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {

    let t:f32 = if knob.a == 0f32 {
        pos_cycles/n_cycles
    } else {
        (n_cycles - pos_cycles)/n_cycles
    };
    let cycle:f32 = (knob.a * 2f32) - 1f32;
    let curr_db:f32 = -(MAX_DB-MIN_DB) * t;
    db_to_amp(curr_db)
}

/// ## Returns
pub fn amod_lfo_sine(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {

    let t:f32 = pos_cycles/n_cycles;
    let m = (fund*mul).log2();

    let s = 16f32;
    let y = (cps*t*pi2*m*m/(s*3f32)).cos().powi(2i32) * (cps*t*pi2*m.powf(0.5f32)/s).sin().powi(2i32);
    let v = y * 0.5 + 0.5;
    y / (m)
}


/// A continuous amplitude contour that rises and falls.
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// 
/// `a`: Intensity of contour. 0 is the smoothest/most natural contour. 1 has extreme amplitude peaking and steeper gradients.  
/// `b`: Direction of modulation.   
///   - `0` for a plucky/finite effect: starts at full amplitude and falls to 0.   
///   - `1` for a pad/infinite effect: starts at 0 (silence) and rises to max amplitude. Rounds to ceil/floor.  
/// `c`: Oscillation rate. 0 is slowest contour, 1 is up to twice the oscillation rate.  
/// 
/// ## Desmos 
/// https://www.desmos.com/calculator/fev4uqu12x
/// 
/// ## Returns
pub fn amod_wavelet_morphing(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let c:f32 = (one+mul).log2();
    let t:f32 = c*pos_cycles/n_cycles;
    let tx:f32 = c*t;
    // need to slow down time 

    let mod_c = (knob.c * 2f32).max(0.005f32);
    let sig = (pi2*tx*mod_c).cos().powi(2i32);
    let y = (-5f32*tx).exp() * sig;
    let y = (-1f32*tx).exp() * sig;

    let intensity = 12f32.powf(knob.a);
    let v = y.powf(one/(c*intensity))/(fund*mul).log2().powi(2i32);

    if knob.b.round() == 1f32 { 
        let d_amp = MIN_DB * (one-v);
        db_to_amp(d_amp);
        one-v
    } else {
        let d_amp = MIN_DB * v;
        db_to_amp(d_amp);
        v
    }
}

/// a static amod that animates by linear fall 
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier with respect to the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// 
/// 
pub fn amod_stick(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let t:f32 = pos_cycles/n_cycles;
    let osc_rate = mul.log2()/4f32;
    let osc_rate = if mul > 1f32 { one / osc_rate } else { osc_rate };
    let y = osc_rate * (1f32-t) % 1f32;
    y
}

pub fn amod_glideupdown(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let t:f32 = (12f32 * pos_cycles/n_cycles) % 1f32;
    let max_mul: f32 = (NFf/4f32) / fund; 
    let q_factor:f32 = 0.1f32;
    
    let lower_t = t - q_factor;
    let upper_t = t + q_factor; 
    
    let lower_bound_mul = if (t - lower_t) < 0.5f32 {
        2f32 * lower_t * max_mul 
    }  else {
        2f32 * (1f32-lower_t) * max_mul
    };

    let upper_bound_mul = if upper_t < 0.5f32 {
        2f32 * upper_t * max_mul
    }  else {
        2f32 * (1f32-upper_t) * max_mul
    };
    
    // println!("t {} mul {} lower_t {} upper_t {} lower_bound_mul {} upper_bound_mul {} ", t, mul, lower_t, upper_t, lower_bound_mul,upper_bound_mul);
    // if t > 0.008f32 {
    //     panic!("Kill it ")
    // }

    if lower_bound_mul < mul && mul < upper_bound_mul {
        1f32
    } else {
        0f32
    }
}
 
pub fn amod_cutupdown(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let t:f32 = (3f32 * pos_cycles/n_cycles) % 1f32;
    let max_mul: f32 = (NFf * 0.75) / fund; 
    let q_factor:f32 = 1f32;

    let cur_mul:f32 = if t < 0.5f32 {
        2f32 * t * max_mul 
    }  else {
        2f32 * (1f32-t) * max_mul
    };


    if cur_mul == mul {
        one
    } else if (cur_mul - q_factor) < mul {
        let y = (mul - q_factor) / (cur_mul - q_factor);
        y.powi(2i32)
    } else if (cur_mul + q_factor) > mul {
        let y = (mul - q_factor) / (cur_mul - q_factor);
        y.powi(2i32)
    } else {
        0f32
    }
}



/// alternates between even and odd harmonioc
pub fn amod_seesaw(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let rate:f32 = 1f32;
    let t:f32 = (rate * pos_cycles/n_cycles) % 1f32;
    let mod_rate:f32 = (4f32*pi2*t).cos().powi(2i32) * 32f32;

    if mul.floor() % 2f32 == 0f32 {
        let amp_evens = (mod_rate * pi * t * half).cos();
        amp_evens.powi(2i32)
    } else {
        let amp_odds = (mod_rate * pi * t * half).sin();
        amp_odds.powi(2i32)
    }
}




///
/// `a`: Detune amount. 0 is none; 0.5 is just noticable/vintage; 1 is noticable.
/// `b`: Detune mix. 0 is none; 1 is completely mixed.
pub fn amod_detune(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let t:f32 = pos_cycles/n_cycles;
    let osc_mod_mul:f32 = 2f32.powf(knob.a * 4f32);
    one - knob.b * (t * mul.powf(2f32 * knob.a) *  pi).sin().abs()
}

pub fn fmod_geo(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let d = if mul < 1f32 {
        (1f32/mul).log2()
    } else {
        mul.log2()
    };
    let t:f32 = pos_cycles/n_cycles;

    (1.5f32).powf(t*d);
    d
}

pub fn amod_collage(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let d = if mul < 1f32 {
        (1f32/mul).log2()
    } else {
        mul.log2()
    };
    let t:f32 = pos_cycles/n_cycles;
    let b = d.floor();
    let mod_rate:f32 = if b % 5f32 == 0f32 {
        one/3f32
    } else if b % 4f32 == 0f32 {
        one / 2.5f32
    } else if b % 3f32 == 0f32 {
        one / 2f32 
    } else if b % 2f32 == 0f32 {
        one / 1.5f32
    } else {
        1.5f32
    };


    (pi2 * t * mod_rate).cos().powi(2i32)
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::analysis;

    static n_samples:usize = 1000;
    static cps:f32 = 1.4;
    static fund:f32 = 1.9;
    static muls:[f32;4] = [1f32, 10f32, 20f32, 200f32];
    static n_cycles:f32= 1.5f32;

    #[test]
    fn test_fmod_sweepdown_a1_b0() {  
        for &mul in &muls {
            let mut signal:Vec<f32> = Vec::with_capacity(n_samples);
            let knob:Knob = Knob { a:1f32, b:0f32, c:0f32};
            // Generate a buffer of modulation envelope data 
            for i in 0..n_samples {
                let pos_cycles = n_cycles * (i as f32/ n_samples as f32);
                signal.push(fmod_sweepdown(&knob, cps, fund, mul, n_cycles, pos_cycles))
            }
            let f:f32 = mul * fund;
            let mut bads:Vec<(usize,f32)> = Vec::new();

            assert!(signal.iter().all(|&v| analysis::is_fmod_range(f, v)), "fmod_sweepdown must only produce valid frequency modulation results");
            assert!(analysis::is_monotonically_decreasing(&signal), "fmod_sweepdown must produce only values that are constantly decreasing.");
        }
    }
}
