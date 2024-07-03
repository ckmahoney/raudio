//! # Additive Synthesis Engine
//! 
//! This module provides an implementation of an additive synthesis engine
//! with support for modulation and automation of parameters such as amplitude,
//! frequency, and phase over time. The `ninja` function is the main entry point
//! for generating a synthesized audio buffer.

use crate::druid::applied_modulation::{Dressing, ModulationEffect, Modifiers};
use crate::druid::compute::{ModulationMode};
use crate::synth::{SR, SRf, MFf, MF, NFf, NF, pi2, pi, SampleBuffer};
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

/// Synthesis with dynamic modulators
pub fn ninja<'render>(
    span: &Span,
    thresh: (f32, f32),
    freq: f32,
    expr: &Expr,
    bp: &Bp,
    dressing: &Dressing,
    modifiers: Modifiers<'render>,
) -> SampleBuffer {
    let n_samples = crate::time::samples_of_cycles(span.0, span.1);
    let mut sig = vec![0f32; n_samples];

    let (modAmp, modFreq, modPhase, modTime) = modifiers;

    for j in 0..n_samples {
        let p: f32 = j as f32 / n_samples as f32;
        let t0:f32 = j as f32 / SRf;
        let t: f32 = modTime.iter().fold(t0, |acc, mt| mt.apply(t0, acc)); 
        let mut v: f32 = 0f32;

        // sample the amp, freq, and phase offset envelopes
        let mut am = sample(&expr.0, p);
        let mut fm = sample(&expr.1, p);
        let mut pm = sample(&expr.2, p);

        let multipliers = &dressing.multipliers;
        let amplifiers = &dressing.amplitudes;
        let phases = &dressing.offsets;
        let (gate_thresh, clip_thresh) = thresh;

        // preliminary filter. Saves a lot of compute!
        if am > gate_thresh {
            for (i, &m) in multipliers.iter().enumerate() {
                let a0 = amplifiers[i];
                if a0 > 0f32 {
                    let amplifier = modAmp.iter().fold(a0, |acc, ma| ma.apply(t, acc)); 
                    if amplifier != 0f32 {
                        let k = i + 1;
                        let f0:f32 = m * fm * freq;
                        let frequency = modFreq.iter().fold(f0, |acc, mf| mf.apply(t, acc)); 
                        let amp = amplifier * am * filter(p, frequency, bp);
                        // Note that phase gets the unmodulated frequency
                        let p0 = f0 * pi2 * t;
                        let phase = modPhase.iter().fold(p0, |acc, mp| mp.apply(t, acc)); 
                        v += amp * phase.sin();
                    }
                }
            }

            // apply gating and clipping to the summed sample
            if v.abs() > clip_thresh {
                sig[j] += v.signum() * clip_thresh;
            }
            if v.abs() >= gate_thresh {
                sig[j] += v;
            }
        }
    }

    sig
}

#[cfg(test)]
mod test {
    use crate::druid::applied_modulation::{AmplitudeModParams,ModulationEffect};
    use crate::druid::melodic;
    use crate::files;
    use crate::render::engrave;
    use super::*;

    static TEST_DIR:&str = "dev-audio/ninja";

    fn write_test_asset(signal:&SampleBuffer, test_name:&str) {
        files::with_dir(TEST_DIR);
        let filename = format!("{}/{}.wav", TEST_DIR, test_name);
        engrave::samples(SR, &signal, &filename);
    } 


    #[test]
    fn test_ninja() {
        let test_name = "sawtooth-plain";

        let span:Span = (1.2f32, 4f32);
        let thresh: (f32, f32) = (0f32, 1f32);
        let freq = 222f32;
        let expr = (vec![1f32],vec![1f32],vec![0f32]);
        let bp = (vec![MFf],vec![NFf]);
        let dressing = Dressing::new(
            melodic::amps_sawtooth(freq), 
            melodic::muls_sawtooth(freq), 
            melodic::phases_sawtooth(freq)
        );
        let modifiers:Modifiers = (&vec![], &vec![], &vec![], &vec![]);
        let signal = ninja(&span, thresh, freq, &expr, &bp, &dressing, modifiers);

        write_test_asset(&signal, &test_name)
    }
    
    #[test]
    fn test_ninja_tremelo() {
        let test_name = "sawtooth-amp-tremelo";

        let span:Span = (1.2f32, 4f32);
        let thresh: (f32, f32) = (0f32, 1f32);
        let freq = 222f32;
        let expr = (vec![1f32],vec![1f32],vec![0f32]);
        let bp = (vec![MFf],vec![NFf]);
        let dressing = Dressing::new(
            melodic::amps_sawtooth(freq), 
            melodic::muls_sawtooth(freq), 
            melodic::phases_sawtooth(freq)
        );
        let gtr_arg = AmplitudeModParams { freq: 18.25, depth: 1.0, offset: 0.0};
        let effect = ModulationEffect::Tremelo(gtr_arg);
        let modifiers:Modifiers = (
            &vec![effect], 
            &vec![], 
            &vec![], 
            &vec![]
        );
        let signal = ninja(&span, thresh, freq, &expr, &bp, &dressing, modifiers);
        write_test_asset(&signal, &test_name)
    }
}