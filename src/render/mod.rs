pub mod blend; 
pub mod engrave;
pub mod ifft;
pub mod ninja;
pub mod realize; 

use crate::synth::{SR, SRf, MFf, MF, NFf, NF, pi2, pi, SampleBuffer};
use crate::analysis::delay;
use crate::reverb::convolution;
use crate::types::render::{Span, Stem, FeelingHolder};
use self::realize::normalize_channels;
use crate::druid::applied_modulation::update_mods;
use crate::time;
use crate::types::render::{Melody};
use crate::druid::{Elementor, Element, ApplyAt, melody_frexer, inflect};
use crate::phrasing::lifespan;
use crate::phrasing::contour::{Expr, Position, sample, apply_contour};
use crate::types::timbre::{AmpContour,Arf};
use crate::types::synthesis::{GlideLen, Note, Range, Bp, Clippers};
use rand;
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;


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

pub fn rescale(samples: &[f32], original_freq: f32, target_freq: f32) -> Vec<f32> {
    let ratio = original_freq / target_freq;
    let new_length = (samples.len() as f32 * ratio) as usize;
    let mut resampled = Vec::with_capacity(new_length);

    for i in 0..new_length {
        let float_idx = i as f32 / ratio;
        let idx = float_idx as usize;
        let next_idx = if idx + 1 < samples.len() { idx + 1 } else { idx };
        
        // Linear interpolation
        let sample = if idx != next_idx {
            let fraction = float_idx.fract();
            samples[idx] * (1.0 - fraction) + samples[next_idx] * fraction
        } else {
            samples[idx]
        };

        resampled.push(sample);
    }

    resampled
}

pub fn normalize(buffer: &mut Vec<f32>) {
    if buffer.is_empty() {
        return;
    }

    let max_amplitude = buffer.iter().map(|&sample| sample.abs()).fold(0.0, f32::max);

    if max_amplitude != 0.0 {
        buffer.iter_mut().for_each(|sample| *sample /= max_amplitude);
    }
}

pub fn norm_scale(buffer: &mut Vec<f32>, scale:f32) {
    if buffer.is_empty() {
        return;
    }

    let max_amplitude = buffer.iter().map(|&sample| sample.abs()).fold(0.0, f32::max);

    if max_amplitude != 0.0 {
        buffer.iter_mut().for_each(|sample| *sample /= scale * max_amplitude);
    }
}


pub fn amp_scale(buffer:&mut Vec<f32>, amp: f32) {
    buffer.iter_mut().for_each(|sample| *sample *= amp)
}

pub fn mix_and_normalize_buffers(buffers: Vec<Vec<f32>>) -> Result<Vec<f32>, &'static str> {
    realize::mix_buffers(&mut buffers.clone())
}

pub fn pad_and_mix_buffers(buffers: Vec<Vec<f32>>) -> Result<Vec<f32>, &'static str> {
    if buffers.is_empty() {
        return Ok(Vec::new());
    }

    let max_buffer_length = buffers.iter().map(|b| b.len()).max().unwrap_or(0);
    let padded_buffers = buffers.into_iter()
    .map(|buffer| {
        let mut padded = buffer;
        padded.resize(max_buffer_length, 0.0);
        padded
    })
    .collect();

    mix_and_normalize_buffers(padded_buffers)
}


pub fn arf(cps:f32, contour:&AmpContour, melody:&Melody<Note>, synth:&Elementor, arf:Arf) -> SampleBuffer {
    let melody_frexd = melody_frexer(&melody, GlideLen::None, GlideLen::None);
    let mut channels:Vec<SampleBuffer> = Vec::with_capacity(melody.len());
    let mut seed:ThreadRng = thread_rng();
    
    // @art-choice: apply a visible dynamic amp mix for the synth as a whole
    let mod_amp_synth:f32 = 0.5f32 + 0.5 * seed.gen::<f32>();

    for (index, line_frexd) in melody_frexd.iter().enumerate() {
        let mut line_buff:SampleBuffer = Vec::new();
        let line = &melody[index];
        // @art-choice: apply a background dynamic amp for the melodies within as well
        let mod_amp_melody:f32 = 0.8f32 + 0.2 * seed.gen::<f32>();

        for (jindex, frex) in line_frexd.iter().enumerate() {
            let mod_amp_dither:f32 = 0.99 + 0.01 * seed.gen::<f32>();

            let dur = time::duration_to_cycles(line[jindex].0);
            let amp = line[jindex].2;
            let at = ApplyAt { frex: *frex, span: (cps, dur) };
            let weight_modulation:f32 = mod_amp_dither * mod_amp_melody * mod_amp_synth;
            let applied:Elementor = synth.iter().map(|(w, r)| (weight_modulation* *w * amp, *r)).collect();
            line_buff.append(&mut inflect(
                &frex, 
                &at, 
                &applied, 
                &arf.visibility,
                &arf.energy,
                &arf.presence
            ));
        }
        channels.push(line_buff)
    }

    match realize::mix_buffers(&mut channels) {
        Ok(mut mixdown) => {
            let cont = lifespan::mod_lifespan(
                mixdown.len()/2, 
                1f32, 
                &lifespan::select_lifespan(&contour), 
                1usize, 
                1f32
            );
            apply_contour(&mut mixdown, &cont);
            mixdown
        },
        Err(msg) => panic!("Error while preparing mixdown: {}", msg)
    }

}


fn longest_delay_length(ds:&Vec<delay::DelayParams>) -> f32 {
    ds.iter().fold(0f32, |max, params| (params.len_seconds * params.n_echoes as f32).max(max))
}
 
use crate::analysis::xform_freq;


pub struct Ctx<'render> {
    freq: f32,
    span: &'render Span,
    thresh: &'render Clippers,
}
fn channel(cps:f32, root:f32, stem:&Stem) -> SampleBuffer {
    // let line_buffs:Vec<SampleBuffer> = midi_melody.iter().map(|line| {
    //     let mut channel_samples:Vec<SampleBuffer> = Vec::new();

    //     let len_cycles:f32 = line.iter().map(|(d,_,_)| d).sum();
    //     let append_delay = time::samples_of_dur(cps, longest_delay_length(&ds));
    //     let signal_len = crate::time::samples_of_cycles(cps, len_cycles) + append_delay;

    //     for syn_midi in line {
    //         let freq = root * xform_freq::midi_to_freq(syn_midi.1);
    //         let ctx:Ctx = Ctx {
    //             span: &(cps, syn_midi.0),
    //             freq,
    //             thresh: &common_thresh
    //         };
            
    //         channel_samples.push(ninj(&ctx, &feeling, &ds, &reverbs));
    //     }

    //     let durs:Vec<f32> = line.iter().map(|(d,_,_)| *d).collect();
    //     stitch(signal_len, cps, durs, &mut channel_samples)
    // }).collect();
    vec![0f32]

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

fn channels(cps:f32, root:f32, stems:&Vec<Stem>, reverbs:Vec<&convolution::ReverbParams>) -> SampleBuffer {
    let mut rng = rand::thread_rng();
    
    let reverbs:Vec<&convolution::ReverbParams> = vec![
        &convolution::ReverbParams { mix: 1f32, amp: 0.5f32, dur: 0.01f32, rate: 0.8f32},
        &convolution::ReverbParams { mix: 0.3f32, amp: 0.5f32, dur: 4f32, rate: 0.1f32},
    ];

    let mut channels:Vec<SampleBuffer> = Vec::new();
    // for (index, (midi_melody, synth_gen)) in stems.iter().enumerate() {
    //     match pad_and_mix_buffers(line_buffs) {
    //         Ok(channel_signal) => {
    //             channels.push(channel_signal)
    //         },
    //         Err(msg) => {
    //             panic!("Failed to mix and render audio: {}", msg)
    //         }
    //     }
    // }

    // match pad_and_mix_buffers(channels) {
    //     Ok(signal) => signal,
    //     Err(msg) => {
    //         panic!("Failed to mix and render audio: {}", msg)
    //     }
    // }

    vec![0f32]
}


/// Additive synthesis with dynamic modulators supporting inline delay
pub fn summit<'render>(
    Ctx { freq, span, thresh: &(gate_thresh, clip_thresh) }: &'render Ctx,
    FeelingHolder { bp, expr, dressing, modifiers, clippers }: &'render FeelingHolder,
    delays: &Vec::<delay::DelayParams>,
    reverbs: &Vec::<&convolution::ReverbParams>,
) -> Vec<f32> {
    let append_delay = time::samples_of_dur(span.0, longest_delay_length(delays));
    let n_samples = crate::time::samples_of_cycles(span.0, span.1) + append_delay;
    let mut sig = vec![0f32; n_samples];

    let (modAmp, modFreq, modPhase, modTime) = modifiers;

    for delay_params in delays {
        let samples_per_echo: usize = time::samples_from_dur(1f32, delay_params.len_seconds);
        for j in 0..n_samples {
            for replica_n in 0..(delay_params.n_echoes.max(1)) {
                let gain = if replica_n == 0 {
                    1f32 - delay_params.mix
                } else {
                    delay::gain(j, replica_n, delay_params)
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
    if reverbs.len() == 0 {
        return sig
    }

    let wets:Vec<SampleBuffer> = reverbs.iter().map(|params| {
        convolution::of(&sig, &params)
    }).collect();
    match pad_and_mix_buffers(wets) {
        Ok(signal) => signal,
        Err(msg) => panic!("Uncaught error while mixing in reverbs: {}",msg)
    }
}















fn fake_gen<'render>() -> Vec<Stem<'render>> {
    use crate::music::lib::x_files;
    use crate::druid::applied_modulation::{Dressing, ModulationEffect, Modifiers, ModifiersHolder, gen_vibrato, gen_tremelo};

    fn modifiers_lead() -> ModifiersHolder {
        let mut rng = rand::thread_rng();
        let vibrato = gen_vibrato(2f32, 13f32, 0f32, 1f32, 0f32, 0f32, &mut rng);
        (
            vec![], 
            vec![], 
            vec![vibrato], 
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
        use crate::druid::melodic;
        let (amps1, muls1, offs1) = melodic::square(freq);
        FeelingHolder {
            expr: (vec![amp, amp/4f32, amp/10f32, amp/10f32,  0f32],vec![1f32],vec![0f32]),
            // bp: (vec![MFf, MFf*2f32, MFf*4f32],vec![NFf, NFf/2f32, NFf/4f32]),
            bp: (vec![1000f32, 700f32, 200f32],vec![5000f32]),
            dressing: Dressing::new(amps1, muls1, offs1),
            modifiers:modifiers_lead(), 
            clippers: (0f32, 1f32)
        }
    }

    fn feeling_chords(freq:f32, amp:f32) -> FeelingHolder {
        use crate::druid::melodic;
        let (amps2, muls2, offs2) = melodic::triangle(freq);

        FeelingHolder {
            expr: (vec![amp],vec![1f32],vec![0f32]),
            bp: (vec![MFf],vec![NFf]),
            dressing: Dressing::new(amps2, muls2, offs2),
            modifiers:modifiers_chords(), 
            clippers: (0f32, 1f32)
        }
    }
    
    let melody1 = x_files::lead_melody();
    let melody2 = x_files::piano_melody();
    let stems:Vec<(Vec<Vec<(f32, i32, i8)>>, fn (f32,f32) -> FeelingHolder)> = vec![
        (melody1, feeling_lead),
        (melody2, feeling_chords),
    ];
    let ds:Vec<delay::DelayParams> = vec![
        delay::DelayParams { mix: 0.5f32, gain: 0.3f32, len_seconds: 0.66f32, n_echoes: 3},
        delay::DelayParams { mix: 0.5f32, gain: 0.5f32, len_seconds: 1.5f32, n_echoes: 2}
    ];
    let ms:Vec<fn () -> ModifiersHolder> = vec![
        modifiers_lead,
        modifiers_chords,
    ];

    let initial_mods:Vec<ModifiersHolder> = ms.iter().map(|mod_gen| mod_gen()).collect();
    vec![]
}
