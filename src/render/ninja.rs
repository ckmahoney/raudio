//! # Additive Synthesis Engine
//! 
//! This module provides an implementation of an additive synthesis engine
//! with support for modulation and automation of parameters such as amplitude,
//! frequency, and phase over time. The `ninja` function is the main entry point
//! for generating a synthesized audio buffer.

use crate::druid::applied_modulation::{Dressing, ModulationEffect};
use crate::druid::compute::{ModulationMode};
use crate::synth::{SR, MFf, MF, NFf, NF, pi2, pi, SampleBuffer};
use crate::types::synthesis::{Frex, GlideLen, Bp,Range, Direction, Duration, FilterPoint, Radian, Freq, Monae, Mote, Note, Tone};
use crate::types::timbre::{BandpassFilter, Energy, Presence, BaseOsc, Sound, FilterMode, Timeframe, Phrasing};
use crate::types::render::{Span};
use crate::phrasing::contour::{Expr, Position, sample};
use crate::phrasing::ranger::{Ranger, Modders, Mixer, WRangers, mix, example_options};


/// Returns an amplitude identity or cancellation value
/// for the given frequency and bandpass settings
/// 
/// idea: enable attenuation by providing conventional Q settings wrt equalization/filtering.
/// That is, Ratio Q for how wide the attenuation reaches and Mod Q for how much to attenuate.
fn filter(p:f32, freq:f32, bandpass:&Bp) -> Range {
    let min_f = sample(&bandpass.0, p).max(MF as f32);
    let max_f = sample(&bandpass.1, p).min(NF as f32);
    if freq < min_f || freq > max_f {
        return 0f32
    } else {
      return 1f32  
    }
}

/// Main synthesis function for generating the audio buffer.
/// 
/// # Parameters
/// 
/// - `frex`: Frequency context.
/// - `expr`: Expression contours and automation envelopes.
/// - `span`: Cycles and cycles per second.
/// - `bp`: Bandpass filter settings.
/// - `values`: Multipliers, amplifiers, and phases.
/// - `modders`: Modulation functions.
/// - `params`: Modulation parameters.
/// - `thresh`: Tuple of gate and clip thresholds.
/// 
/// # Returns
/// 
/// - `SampleBuffer`: The synthesized audio buffer.
pub fn ninja(
    frex: &Frex,
    expr: &Expr,
    span: &Span,
    bp: &Bp,
    dressing: &Dressing,
    modulations: &Vec<ModulationEffect>,
    thresh: (f32, f32)
) -> SampleBuffer {
    let (glide_from, maybe_prev, freq, maybe_next, glide_to) = frex;
    let n_samples = crate::time::samples_of_cycles(span.0, span.1);
    let mut sig = vec![0f32; n_samples];

    for j in 0..n_samples {
        let p: f32 = j as f32 / n_samples as f32;
        let t: f32 = j as f32 / 44100.0; // Assuming sample rate of 44100 Hz

        let mut am = sample(&expr.0, p);
        let mut fm = sample(&expr.1, p);
        let mut pm = sample(&expr.2, p);

        let mut v: f32 = 0f32;

        let multipliers = &dressing.multipliers;
        let amplifiers = &dressing.amplitudes;
        let phases = &dressing.offsets;

        for (i, &m) in multipliers.iter().enumerate() {
            let k = i + 1;
            let frequency = m * fm * freq;
            let amplifier = amplifiers.get(i).cloned().unwrap_or(0.0);

            if amplifier > 0f32 {
                let amp = amplifier * am * filter(p, frequency, bp) ;
                if amp != 0f32 {
                    let phase = (frequency * std::f32::consts::PI * 2.0 * t) + pm;
                    v += amp * phase.sin();
                }
            }
        }

        let (gate_thresh, clip_thresh) = thresh;
        if v.abs() > clip_thresh {
            let sign: f32 = if v > 0f32 { 1f32 } else { -1f32 };
            sig[j] += sign * clip_thresh;
        }
        if v.abs() >= gate_thresh {
            sig[j] += v;
        }
    }

    sig
}