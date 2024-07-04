//! # Additive Synthesis Engine
//! 
//! This module provides an implementation of an additive synthesis engine
//! with support for modulation and automation of parameters such as amplitude,
//! frequency, and phase over time. The `ninja` function is the main entry point
//! for generating a synthesized audio buffer.

use crate::druid::applied_modulation::{Dressing, ModulationEffect, Modifiers, ModifiersHolder};
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

type Thresh = (f32, f32);

pub struct Ctx<'render> {
    freq: f32,
    span: &'render Span,
    thresh: &'render Thresh,
}

pub struct FeelingHolder {
    bp: Bp,
    expr: Expr,
    dressing: Dressing,
    modifiers: ModifiersHolder
}

pub struct Feeling<'render> {
    bp: &'render Bp,
    expr: &'render Expr,
    dressing: &'render Dressing,
    modifiers: Modifiers<'render>
}

/// Additive synthesis with dynamic modulators
pub fn nin<'render>(
    Ctx { freq, span, thresh: &(gate_thresh, clip_thresh) }: &'render Ctx,
    FeelingHolder { bp, expr, dressing, modifiers }: FeelingHolder
) -> Vec<f32> {
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
                        let amp = amplifier * am * filter(p, frequency, &bp);
                        let p0 = frequency * pi2 * t;
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
                        let p0 = frequency * pi2 * t;
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
    use crate::analysis::xform_freq;
    use crate::druid::applied_modulation::{AmplitudeModParams, PhaseModParams, FrequencyModParams, ModulationEffect};
    use crate::druid::melodic;
    use crate::files;
    use crate::render;
    use crate::render::engrave;
    use super::*;
    use crate::music::lib::x_files;

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


    #[test]
    fn test_ninja_vibrato() {
        let test_name = "sawtooth-freq-vibrato";

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
        let gtr_arg = PhaseModParams { rate: 4f32, depth:1f32,  offset: 0.0};
        let effect = ModulationEffect::Warp(gtr_arg);
        let modifiers:Modifiers = (
            &vec![], 
            &vec![effect], 
            &vec![], 
            &vec![]
        );
        let signal = ninja(&span, thresh, freq, &expr, &bp, &dressing, modifiers);
        write_test_asset(&signal, &test_name)
    }

    fn feeling_lead(freq:f32, amp:f32) -> FeelingHolder {
        let gtr_arg1 = PhaseModParams { rate: 4f32, depth:2f32,  offset: 0.0};
        let effect1 = ModulationEffect::Vibrato(gtr_arg1);
        let modifiers1:ModifiersHolder = (
            vec![], 
            vec![], 
            vec![effect1], 
            vec![]
        );

        let (amps1, muls1, offs1) = melodic::square(freq);
        FeelingHolder {
            expr: (vec![amp],vec![1f32],vec![0f32]),
            bp: (vec![MFf],vec![NFf]),
            dressing: Dressing::new(amps1, muls1, offs1),
            modifiers: modifiers1
        }
    }

    fn feeling_chords(freq:f32, amp:f32) -> FeelingHolder {
        let gtr_arg2 = AmplitudeModParams { freq: 3f32, depth:1f32,  offset: 0.0};
        let effect2 = ModulationEffect::Tremelo(gtr_arg2);
        let modifiers2:ModifiersHolder = (
            vec![effect2], 
            vec![], 
            vec![], 
            vec![]
        );

        let (amps2, muls2, offs2) = melodic::triangle(freq);

        FeelingHolder {
            expr: (vec![amp],vec![1f32],vec![0f32]),
            bp: (vec![MFf],vec![NFf]),
            dressing: Dressing::new(amps2, muls2, offs2),
            modifiers: modifiers2
        }
    }

    #[test]
    fn test_ninja_xfiles_lead() {
        let melody = x_files::lead_melody();
        let line= &melody[0];
        let test_name = "x_files_lead";

        let mut signal:Vec<f32> = Vec::new();
        for syn_midi in line {
            let freq = x_files::root * xform_freq::midi_to_freq(syn_midi.1);

            //copypastas
            let span:Span = (x_files::cps, syn_midi.0);
            let thresh: (f32, f32) = (0f32, 1f32);
            let expr = (vec![xform_freq::velocity_to_amplitude(syn_midi.2)],vec![1f32],vec![0f32]);
            let bp = (vec![MFf],vec![NFf]);
            let dressing = Dressing::new(
                melodic::amps_sawtooth(freq), 
                melodic::muls_sawtooth(freq), 
                melodic::phases_sawtooth(freq)
            );
            let gtr_arg = PhaseModParams { rate: 4f32, depth:0.24f32,  offset: 0.0};
            let effect = ModulationEffect::Vibrato(gtr_arg);
            let modifiers:Modifiers = (
                &vec![], 
                &vec![], 
                &vec![effect], 
                &vec![]
            );
            let mut samples = ninja(&span, thresh, freq, &expr, &bp, &dressing, modifiers);
            signal.append(&mut samples)
        }
        
        write_test_asset(&signal, &test_name)
    }

    #[test]
    fn test_ninja_xfiles_piano() {
        let melody = x_files::piano_melody();
        let line= &melody[0];
        let test_name = "x_files_piano";

        let mut signal:Vec<f32> = Vec::new();
        for syn_midi in line {
            let freq = x_files::root * xform_freq::midi_to_freq(syn_midi.1);

            //copypastas
            let span:Span = (x_files::cps, syn_midi.0);
            let thresh: (f32, f32) = (0f32, 1f32);
            let expr = (vec![xform_freq::velocity_to_amplitude(syn_midi.2)],vec![1f32],vec![0f32]);
            let bp = (vec![MFf],vec![NFf]);
            let dressing = Dressing::new(
                melodic::amps_sawtooth(freq), 
                melodic::muls_sawtooth(freq), 
                melodic::phases_sawtooth(freq)
            );
            let gtr_arg = PhaseModParams { rate: 4f32, depth:1f32,  offset: 0.0};
            let effect = ModulationEffect::Warp(gtr_arg);
            let modifiers:Modifiers = (
                &vec![], 
                &vec![effect], 
                &vec![], 
                &vec![]
            );
            let mut samples = ninja(&span, thresh, freq, &expr, &bp, &dressing, modifiers);
            signal.append(&mut samples)
        }

        write_test_asset(&signal, &test_name)
    }


    #[test]
    fn test_ninja_xfiles_render() {
        let melody1 = x_files::lead_melody();
        let melody2 = x_files::piano_melody();
        let line1 = &melody1[0];
        let line2 = &melody2[0];
        let test_name = "x_files_render";

        let rs:Vec<(Vec<Vec<(f32, i32, i8)>>, fn (f32,f32) -> FeelingHolder)> = vec![
            (melody1, feeling_lead),
            (melody2, feeling_chords),
        ];

        let mut channels:Vec<SampleBuffer> = Vec::new();
        let common_thresh:Thresh = (0f32, 1f32);

        for (midi_melody, synth_gen) in rs {
            if midi_melody.len() == 1 {
                let mut channel_signal:SampleBuffer = Vec::new();
                let line = &midi_melody[0];
                for syn_midi in line {
                    let freq = x_files::root * xform_freq::midi_to_freq(syn_midi.1);
                    let ctx:Ctx = Ctx {
                        span: &(x_files::cps, syn_midi.0),
                        freq,
                        thresh: &common_thresh
                    };
                    let feeling = synth_gen(freq, xform_freq::velocity_to_amplitude(syn_midi.2));
                    channel_signal.append(&mut nin(&ctx, feeling));
                }
                
                channels.push(channel_signal)
            } else {
                panic!("Need to implement polyphonic melody")
            }
            
        }

        match render::pad_and_mix_buffers(channels) {
            Ok(signal) => {
                write_test_asset(&signal, &test_name)
            },
            Err(msg) => {
                panic!("Failed to mix and render audio: {}", msg)
            }
        }
    }
}