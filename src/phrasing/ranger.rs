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
/// `mul` The current multiplier wrt the fundamental  
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
/// `mul` The current multiplier wrt the fundamental  
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
/// `mul` The current multiplier wrt the fundamental  
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
/// `mul` The current multiplier wrt the fundamental  
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
/// `mul` The current multiplier wrt the fundamental  
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



/// A continuous frequency mod that adds detuning to the harmonic.
/// 
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier wrt the fundamental  
/// `n_cycles` Total duration of this event in cycles  
/// `pos_cycles` The current position in the event (in cycles)  
/// 
/// ## Knob Params
/// `a`: The depth of detune to apply. 0 is none, 1 is maximum amount. 
/// `b`: The intensity of detune to apply. 0 is more transparent, 1 is very visible.  
/// `c`: unused. 
/// 
/// ## Observations
/// 
/// ## Desmos 
/// 
/// ## Returns
pub fn pmod_chorus(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let t:f32 = pos_cycles/n_cycles;
    -f32::sin(t * knob.a * pi2 * mul)*pi2.powf(knob.b)
}



/// A oneshot amplitude modulation adding a "huh" like breathing in for a word.
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier wrt the fundamental  
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

/// A oneshot amplitdue modulation for rapid decay.
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier wrt the fundamental  
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
/// `mul` The current multiplier wrt the fundamental  
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
/// `mul` The current multiplier wrt the fundamental  
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
/// `mul` The current multiplier wrt the fundamental  
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
    (0.5f32 + (y - (y + 0.5).floor())).powi(2i32)
}

/// A continuous amplitdue modulation for smooth sine envelopes.
///
/// ## Arguments
/// `cps` Instantaneous playback rate as cycles per second  
/// `fund` The reference fundamental frequency  
/// `mul` The current multiplier wrt the fundamental  
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
/// https://www.desmos.com/calculator/ag0vtteimu
/// 
/// ## Returns
pub fn amod_oscillation_sine(knob: &Knob, cps: f32, fund: f32, mul: f32, n_cycles: f32, pos_cycles: f32) -> f32 {
    let t:f32 = pos_cycles/n_cycles;
    let osc_rate:f32 = 2f32.powf(-4f32 * knob.a);
    let osc_mod_mul:f32 = 2f32.powf(knob.a * 4f32);
    1f32 - (pi * t * osc_mod_mul).sin().abs().powi(4i32)
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
