//! # Additive Synthesis Engine
//! 
//! This module provides an implementation of an additive synthesis engine
//! with support for modulation and automation of parameters such as amplitude,
//! frequency, and phase over time. The `ninja` function is the main entry point
//! for generating a synthesized audio buffer.

use crate::analysis::delay;
use crate::analysis::xform_freq;
use crate::druid::applied_modulation::{Dressing, ModulationEffect, Modifiers, ModifiersHolder};
use crate::druid::compute::{ModulationMode};
use crate::synth::{SR, SRf, MFf, MF, NFf, NF, pi2, pi, SampleBuffer};
use crate::types::synthesis::{Frex, GlideLen, Bp,Range, Direction, Duration, FilterPoint, Radian, Freq, Monae, Mote, Note, Tone};
use crate::types::timbre::{BandpassFilter, Energy, Presence, BaseOsc, Sound, FilterMode, Timeframe, Phrasing};
use crate::types::render::{self, Span};
use crate::time;
use crate::phrasing::contour::{Expr, Position, sample};
use crate::phrasing::ranger::{Ranger, Modders, Mixer, WRangers, mix, example_options};
use rand::Rng;
use rand::rngs::ThreadRng;
use serde::de;

/// Returns an amplitude identity or cancellation value
/// for the given frequency and bandpass settings
/// 
/// idea: enable attenuation by providing conventional Q settings wrt equalization/filtering.
/// That is, Ratio Q for how wide the attenuation reaches and Mod Q for how much to attenuate.
fn filter(progress:f32, freq:f32, bandpass:&Bp) -> Range {
    let p = progress.max(0f32).min(1f32);
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
    let n_samples = time::samples_of_cycles(span.0, span.1);
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

// /// high feedback gain per echo
// fn hfb_decay_delay(j:usize, delay_params: &delay::DelayParams, render_at_offset: RenderFn) -> f32 {
//     let samples_per_echo: usize = time::samples_from_dur(1f32, delay_params.len_seconds);
//     let min_distance =  samples_per_echo;
//     if j < min_distance {
//         return (1f32 - delay_params.mix) * render_at_offset(0)
//     }

//     let max_distance = delay_params.n_echoes * samples_per_echo;

//     let sample_points:Vec<usize> = (1..(delay_params.n_echoes-1)).map(|x| x * samples_per_echo).collect();
//     let dry = render_at_offset(0);
//     let wet = sample_points.iter().fold(0f32, |v, t| {
//         let y = if j > *t {
//             render_at_offset(*t)
//         } else {
//             0f32
//         };
//         let n: f32 = *t as f32 / samples_per_echo as f32;
//         let gain: f32 = 0.99f32.powf(n);
//         v + gain * y
//     });
//     (1f32 - delay_params.mix) * dry + delay_params.mix * wet
// }

fn longest_delay_length(ds:&Vec<delay::DelayParams>) -> f32 {
    ds.iter().fold(0f32, |max, params| (params.len_seconds * params.n_echoes as f32).max(max))
}


/// Additive synthesis with dynamic modulators supporting inline delay
pub fn ninj<'render>(
    Ctx { freq, span, thresh: &(gate_thresh, clip_thresh) }: &'render Ctx,
    FeelingHolder { bp, expr, dressing, modifiers }: &'render FeelingHolder,
    delays: &Vec::<delay::DelayParams>,
) -> Vec<f32> {
    let append_delay = time::samples_of_dur(span.0, longest_delay_length(delays));
    let n_samples = crate::time::samples_of_cycles(span.0, span.1) + append_delay;
    let mut sig = vec![0f32; n_samples];

    let (modAmp, modFreq, modPhase, modTime) = modifiers;

    for delay_params in delays {
        let samples_per_echo: usize = time::samples_from_dur(1f32, delay_params.len_seconds);
        for j in 0..n_samples {
            for replica_n in 0..(delay_params.n_echoes) {
                let gain = if replica_n == 0 {
                    1f32 - delay_params.mix
                } else {
                    delay_params.mix * delay::gain(j, replica_n, delay_params)
                };
                if gain < gate_thresh {
                    continue;
                }

                let offset_j = replica_n * samples_per_echo;

                let p: f32 = (offset_j + j) as f32 / n_samples as f32;
                let t0:f32 = (offset_j + j) as f32 / SRf;
                let t: f32 = modTime.iter().fold(t0, |acc, mt| mt.apply(t0, acc)); 
                let mut v: f32 = 0f32;

                // sample the amp, freq, and phase offset envelopes
                let mut am = sample(&expr.0, p);
                let mut fm = sample(&expr.1, p);
                let mut pm = sample(&expr.2, p);

                let multipliers = &dressing.multipliers;
                let amplifiers = &dressing.amplitudes;
                let phases = &dressing.offsets;

                if (am*gain) < gate_thresh {
                    continue;
                }

                for (i, &m) in multipliers.iter().enumerate() {
                    let a0 = amplifiers[i];
                    if a0 > 0f32 {
                        let amplifier = modAmp.iter().fold(a0, |acc, ma| ma.apply(t, acc)); 
                        if (amplifier*am*gain) < gate_thresh {
                            continue
                        }
                        let k = i + 1;
                        let f0:f32 = m * fm * freq;
                        let frequency = modFreq.iter().fold(f0, |acc, mf| mf.apply(t, acc)); 
                        let amp = gain * amplifier * am * filter(p, frequency, &bp);
                        let p0 = frequency * pi2 * t;
                        let phase = modPhase.iter().fold(p0, |acc, mp| mp.apply(t, acc)); 
                        v += amp * phase.sin();
                    }
                };

                // apply gating and clipping to the summed sample
                if v.abs() > clip_thresh {
                    sig[j] += v.signum() * clip_thresh;
                } else if v.abs() >= gate_thresh {
                    sig[j] += v;
                }
            }
        }
    }

    sig
}

/// Additive synthesis with dynamic modulators supporting inline delay
pub fn nun<'render>(
    Ctx { freq, span, thresh: &(gate_thresh, clip_thresh) }: &'render Ctx,
    FeelingHolder { bp, expr, dressing, modifiers }: &'render FeelingHolder,
    delays: &Vec::<delay::DelayParams>,
) -> Vec<f32> {
    let n_samples = crate::time::samples_of_cycles(span.0, span.1);
    let mut sig = vec![0f32; n_samples];

    let (modAmp, modFreq, modPhase, modTime) = modifiers;

    for delay_params in delays {
        let samples_per_echo: usize = time::samples_from_dur(1f32, delay_params.len_seconds);
        for j in 0..n_samples {
            for replica_n in 0..(delay_params.n_echoes) {
                let gain = if replica_n == 0 {
                    1f32 - delay_params.mix
                } else {
                    delay_params.mix * delay::gain(j, replica_n, delay_params)
                };
                if gain < gate_thresh {
                    continue;
                }

                let offset_j = replica_n * samples_per_echo;

                let p: f32 = (offset_j + j) as f32 / n_samples as f32;
                let t0:f32 = (offset_j + j) as f32 / SRf;
                let t: f32 = modTime.iter().fold(t0, |acc, mt| mt.apply(t0, acc)); 
                let mut v: f32 = 0f32;

                // sample the amp, freq, and phase offset envelopes
                let mut am = sample(&expr.0, p);
                let mut fm = sample(&expr.1, p);
                let mut pm = sample(&expr.2, p);

                let multipliers = &dressing.multipliers;
                let amplifiers = &dressing.amplitudes;
                let phases = &dressing.offsets;

                if (am*gain) < gate_thresh {
                    continue;
                }

                for (i, &m) in multipliers.iter().enumerate() {
                    let a0 = amplifiers[i];
                    if a0 > 0f32 {
                        let amplifier = modAmp.iter().fold(a0, |acc, ma| ma.apply(t, acc)); 
                        if (amplifier*am*gain) < gate_thresh {
                            continue
                        }
                        let k = i + 1;
                        let f0:f32 = m * fm * freq;
                        let frequency = modFreq.iter().fold(f0, |acc, mf| mf.apply(t, acc)); 
                        let amp = gain * amplifier * am * filter(p, frequency, &bp);
                        let p0 = frequency * pi2 * t;
                        let phase = modPhase.iter().fold(p0, |acc, mp| mp.apply(t, acc)); 
                        v += amp * phase.sin();
                    }
                };

                // apply gating and clipping to the summed sample
                if v.abs() > clip_thresh {
                    sig[j] += v.signum() * clip_thresh;
                } else if v.abs() >= gate_thresh {
                    sig[j] += v;
                }
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

/// Given a list of durs representing the basis note durations and a playback rate,
/// Return a list of indicies for a SampleBuffer indicating each note's "start" position.
/// Useful when mixing in delayed copies of a signal.
pub fn insertion_marks(cps:f32, durs:&Vec<f32>) -> Vec<usize> {
    let mut marks = Vec::with_capacity(durs.len());
    let (_, marks) = durs.iter()
        .fold((0, marks), |(acc, mut ms), dur| {
                ms.push(acc);
                (acc + time::samples_from_dur(cps, *dur), ms)
            }
        );
    marks
}

pub fn ssstitch((root, cps):(f32, f32), thresh:&Thresh, line:&Vec<(f32, i32, i8)>, feeling:&FeelingHolder, delays: &Vec<delay::DelayParams>) -> SampleBuffer {
    let mut samples:Vec<SampleBuffer> = Vec::with_capacity(line.len());
    let insert_points:Vec<usize> = insertion_marks(cps, &line.iter().map(|(d,_,_)| *d).collect());
    let mut wet_buf:SampleBuffer = Vec::new();
    
    for (i, syn_midi) in line.iter().enumerate() {
                    
        let freq = root * xform_freq::midi_to_freq(syn_midi.1);
        let ctx:Ctx = Ctx {
            span: &(cps, syn_midi.0),
            freq,
            thresh
        };
        let start_point = insert_points[i];
        let sample = nun(&ctx, &feeling, &delays);
        for (j, y) in sample.iter().enumerate() {
            wet_buf[i+j] += y
        }

    }

    wet_buf
}

pub fn stitch(len:usize, cps:f32, durs:Vec<f32>, samples:&mut Vec<SampleBuffer>) -> SampleBuffer {
    let mut signal:SampleBuffer = vec![0f32; len];
    durs.iter().enumerate().fold(0, |pos, (i, dur)| { 
        for (j,s) in samples[i].iter().enumerate() {
            signal[pos + j] += s
        }
        pos + time::samples_of_dur(cps, *dur)
    });
    signal
}

#[cfg(test)]
mod test {
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

    fn modifiers_lead() -> ModifiersHolder {
        let mut rng = rand::thread_rng();
        let vibrato = gen_vibrato(2f32, 13f32, 0f32, 1f32, 0f32, 0f32, &mut rng);
        (
            vec![], 
            vec![], 
            // vec![vibrato], 
            vec![], 
            vec![]
        )
    }

    fn modifiers_chords() -> ModifiersHolder {
        let mut rng = rand::thread_rng();
        let tremelo = gen_tremelo(2f32, 13f32, 0f32, 1f32, 0f32, 0f32, &mut rng);
        (
            vec![tremelo], 
            vec![], 
            vec![], 
            vec![]
        )
    }

    fn feeling_lead(freq:f32, amp:f32) -> FeelingHolder {
        let (amps1, muls1, offs1) = melodic::square(freq);
        FeelingHolder {
            expr: (vec![amp, amp/4f32, amp/10f32, amp/10f32,  0f32],vec![1f32],vec![0f32]),
            // bp: (vec![MFf, MFf*2f32, MFf*4f32],vec![NFf, NFf/2f32, NFf/4f32]),
            bp: (vec![1000f32, 700f32, 200f32],vec![5000f32]),
            dressing: Dressing::new(amps1, muls1, offs1),
            modifiers:modifiers_lead()
        }
    }

    fn feeling_chords(freq:f32, amp:f32) -> FeelingHolder {
        let (amps2, muls2, offs2) = melodic::triangle(freq);

        FeelingHolder {
            expr: (vec![amp],vec![1f32],vec![0f32]),
            bp: (vec![MFf],vec![NFf]),
            dressing: Dressing::new(amps2, muls2, offs2),
            modifiers:modifiers_chords()
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


    fn gen_tremelo(min_f:f32, max_f:f32, min_d:f32, max_d:f32, min_o:f32, max_o:f32, rng:&mut ThreadRng) -> ModulationEffect {
        let gtr_arg = AmplitudeModParams { 
            freq: min_f + rng.gen::<f32>() * (max_f * min_f), 
            depth: min_d + rng.gen::<f32>() * (max_d - min_d),  
            offset: min_o + rng.gen::<f32>() * (max_o - min_o),  
        };
        ModulationEffect::Tremelo(gtr_arg)
    }

    fn gen_vibrato(min_f:f32, max_f:f32, min_d:f32, max_d:f32, min_o:f32, max_o:f32, rng:&mut ThreadRng) -> ModulationEffect {
        let gtr_arg = PhaseModParams { 
            rate: min_f + rng.gen::<f32>() * (max_f * min_f), 
            depth: min_d + rng.gen::<f32>() * (max_d - min_d),  
            offset: min_o + rng.gen::<f32>() * (max_o - min_o),  
        };
        ModulationEffect::Vibrato(gtr_arg)
    }

    /// Reroll some params for the mods using the test/example updater defined here
    fn update_mods(holder:&ModifiersHolder, rng:&mut ThreadRng) -> ModifiersHolder {
        (
            holder.0.iter().map(|m| update_modifier(m, rng)).collect(),
            holder.1.iter().map(|m| update_modifier(m, rng)).collect(),
            holder.2.iter().map(|m| update_modifier(m, rng)).collect(),
            holder.3.iter().map(|m| update_modifier(m, rng)).collect(),
        )
    }

    fn update_modifier(modifier:&ModulationEffect, rng:&mut ThreadRng) -> ModulationEffect {
        match *modifier {
            ModulationEffect::Tremelo(ref tremelo) => {
                let gtr_arg:AmplitudeModParams = AmplitudeModParams {
                    freq: tremelo.freq,
                    depth: rng.gen(), 
                    offset: tremelo.offset
                };
                ModulationEffect::Tremelo(gtr_arg)
            },
            ModulationEffect::Vibrato(ref vibrato) => {
                let gtr_arg:PhaseModParams = PhaseModParams {
                    rate: vibrato.rate,
                    depth: rng.gen(), 
                    offset: vibrato.offset
                };
                ModulationEffect::Vibrato(gtr_arg)
            },
            ModulationEffect::Noise(ref noise) => {
                let gtr_arg:PhaseModParams = PhaseModParams {
                    rate: noise.rate,
                    depth: rng.gen(), 
                    offset: noise.offset
                };
                ModulationEffect::Vibrato(gtr_arg)
            },
            ModulationEffect::Chorus(ref chorus) => {
                let gtr_arg:PhaseModParams = PhaseModParams {
                    rate: chorus.rate,
                    depth: rng.gen(), 
                    offset: chorus.offset
                };
                ModulationEffect::Vibrato(gtr_arg)
            },
            _ => *modifier
        }
    }

    #[test]
    fn test_ninja_xfiles_render() {
        let melody1 = x_files::lead_melody();
        let melody2 = x_files::piano_melody();
        let test_name = "x_files_render";
        let mut rng = rand::thread_rng();

        let rs:Vec<(Vec<Vec<(f32, i32, i8)>>, fn (f32,f32) -> FeelingHolder)> = vec![
            (melody1, feeling_lead),
            (melody2, feeling_chords),
        ];

        let ms:Vec<fn () -> ModifiersHolder> = vec![
            modifiers_lead,
            modifiers_chords,
        ];

        let initial_mods:Vec<ModifiersHolder> = ms.iter().map(|mod_gen| mod_gen()).collect();

        let mut channels:Vec<SampleBuffer> = Vec::new();
        let common_thresh:Thresh = (0f32, 1f32);

        for (index, (midi_melody, synth_gen)) in rs.iter().enumerate() {
            if midi_melody.len() == 1 {
                let stem_name = format!("{}-stem-{}", test_name, index);
                let mut channel_signal:SampleBuffer = Vec::new();
                let line = &midi_melody[0];
                for syn_midi in line {
                    
                    let freq = x_files::root * xform_freq::midi_to_freq(syn_midi.1);
                    let ctx:Ctx = Ctx {
                        span: &(x_files::cps, syn_midi.0),
                        freq,
                        thresh: &common_thresh
                    };
                    let f:FeelingHolder = synth_gen(freq, xform_freq::velocity_to_amplitude(syn_midi.2));
                    let feeling:FeelingHolder = FeelingHolder {
                        bp: f.bp,
                        expr: f.expr,
                        dressing: f.dressing,
                        // modifiers: update_mods(&initial_mods[index], &mut rng)
                        modifiers: initial_mods[index].clone()
                    };
                    channel_signal.append(&mut nin(&ctx, feeling));
                }

                write_test_asset(&channel_signal, &stem_name);
                
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

    #[test]
    fn test_ninja_xfiles_with_delay() {
        let melody1 = x_files::lead_melody();
        let melody2 = x_files::piano_melody();
        let test_name = "x_files_delay";
        let mut rng = rand::thread_rng();

        let rs:Vec<(Vec<Vec<(f32, i32, i8)>>, fn (f32,f32) -> FeelingHolder)> = vec![
            (melody1, feeling_lead),
            (melody2, feeling_chords),
        ];

        let ds:Vec<delay::DelayParams> = vec![
            delay::DelayParams { mix: 0.5f32, gain: 0.99, len_seconds: 0.15f32, n_echoes: 5}
        ];

        let ms:Vec<fn () -> ModifiersHolder> = vec![
            modifiers_lead,
            modifiers_chords,
        ];

        let initial_mods:Vec<ModifiersHolder> = ms.iter().map(|mod_gen| mod_gen()).collect();

        let mut channels:Vec<SampleBuffer> = Vec::new();
        let common_thresh:Thresh = (0f32, 1f32);

        for (index, (midi_melody, synth_gen)) in rs.iter().enumerate() {
            if midi_melody.len() == 1 {
                let stem_name = format!("{}-stem-{}", test_name, index);
                let mut channel_signal:SampleBuffer = Vec::new();
                let line = &midi_melody[0];

                for syn_midi in line {
                    
                    let freq = x_files::root * xform_freq::midi_to_freq(syn_midi.1);
                    let ctx:Ctx = Ctx {
                        span: &(x_files::cps, syn_midi.0),
                        freq,
                        thresh: &common_thresh
                    };
                    let f:FeelingHolder = synth_gen(freq, xform_freq::velocity_to_amplitude(syn_midi.2));
                    let feeling:FeelingHolder = FeelingHolder {
                        bp: f.bp,
                        expr: f.expr,
                        dressing: f.dressing,
                        modifiers: update_mods(&initial_mods[index], &mut rng)
                    };
                    channel_signal.append(&mut ninj(&ctx, &feeling, &ds));
                }

                write_test_asset(&channel_signal, &stem_name);
                
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



    #[test]
    fn test_ninja_xfiles_stitched_delay() {
        let melody1 = x_files::lead_melody();
        let melody2 = x_files::piano_melody();
        let test_name = "x_files_stitched_delay";
        let mut rng = rand::thread_rng();

        let rs:Vec<(Vec<Vec<(f32, i32, i8)>>, fn (f32,f32) -> FeelingHolder)> = vec![
            (melody1, feeling_lead),
            (melody2, feeling_chords),
        ];

        let ds:Vec<delay::DelayParams> = vec![
            delay::DelayParams { mix: 0.5f32, gain: 0.99, len_seconds: 0.15f32, n_echoes: 5}
        ];

        let ms:Vec<fn () -> ModifiersHolder> = vec![
            modifiers_lead,
            modifiers_chords,
        ];

        let initial_mods:Vec<ModifiersHolder> = ms.iter().map(|mod_gen| mod_gen()).collect();

        let mut channels:Vec<SampleBuffer> = Vec::new();
        let common_thresh:Thresh = (0f32, 1f32);

        for (index, (midi_melody, synth_gen)) in rs.iter().enumerate() {
            if midi_melody.len() == 1 {
                let stem_name = format!("{}-stem-{}", test_name, index);
                let mut channel_samples:Vec<SampleBuffer> = Vec::new();
                let line = &midi_melody[0];

                let len_cycles:f32 = line.iter().map(|(d,_,_)| d).sum();

                let append_delay = time::samples_of_dur(x_files::cps, longest_delay_length(&ds));
                let signal_len = crate::time::samples_of_cycles(x_files::cps, len_cycles) + append_delay;

                for syn_midi in line {
                    
                    let freq = x_files::root * xform_freq::midi_to_freq(syn_midi.1);
                    let ctx:Ctx = Ctx {
                        span: &(x_files::cps, syn_midi.0),
                        freq,
                        thresh: &common_thresh
                    };
                    let f:FeelingHolder = synth_gen(freq, xform_freq::velocity_to_amplitude(syn_midi.2));
                    let feeling:FeelingHolder = FeelingHolder {
                        bp: f.bp,
                        expr: f.expr,
                        dressing: f.dressing,
                        modifiers: update_mods(&initial_mods[index], &mut rng)
                    };
                    channel_samples.push(ninj(&ctx, &feeling, &ds));
                }

                let durs:Vec<f32> = line.iter().map(|(d,_,_)| *d).collect();
                let channel_signal = stitch(signal_len, x_files::cps, durs, &mut channel_samples);
                
                write_test_asset(&channel_signal, &stem_name);
                
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