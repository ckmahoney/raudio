pub mod blend; 
pub mod engrave;
pub mod ifft;
pub mod ninja;
pub mod realize; 

use crate::synth::{SR, SRf, MFf, MF, NFf, NF, pi2, pi, SampleBuffer};
use crate::analysis::{delay, xform_freq};
use crate::druid::{Elementor, Element, ApplyAt, melody_frexer, inflect};
use crate::druid::applied_modulation::{self, update_mods};
use crate::monic_theory::tone_to_freq;
use crate::phrasing::lifespan::{self};
use crate::phrasing::contour::{Expr, Position, sample, apply_contour};
use crate::reverb::convolution;
use crate::time;
use crate::types::timbre::{AmpContour, Arf, AmpLifespan};
use crate::types::synthesis::{GlideLen, Modifiers, ModifiersHolder, Note, Range, Bp, Clippers, Dressing, Dressor};
use crate::types::render::{Melody,Span, Stem, Feel};
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

/// Render an audio sample for an applied polyphonic or monophonic melody.
fn channel(cps:f32, root:f32, (melody, dressor, feel, mods, delays):&Stem) -> SampleBuffer {
    let line_buffs:Vec<SampleBuffer> = melody.iter().map(|line| {
        let mut channel_samples:Vec<SampleBuffer> = Vec::new();

        let len_cycles:f32 = time::count_cycles(line);
        let append_delay = time::samples_of_dur(cps, longest_delay_length(&delays));
        let signal_len = time::samples_of_cycles(cps, len_cycles) + append_delay;
        let durs:Vec<f32> = line.iter().map(|(d,_,_)| time::duration_to_cycles(*d)).collect();
        line.iter().enumerate().for_each(|(i, (duration, tone, amp))| {
            let freq = root * tone_to_freq(tone);
            let moment = summit(cps, root, freq, durs[i], &dressor(freq), &feel, &delays);
            channel_samples.push(moment);
        });

        let mixed = overlapping(signal_len, cps, durs, &channel_samples);
        trim_zeros(mixed)
    }).collect();

    match pad_and_mix_buffers(line_buffs) {
        Ok(sig) => sig,
        Err(msg) => panic!("Failed to mix and render channel: {}", msg)
    }
}

/// Convolution and delay effects may produce a long tail of empty signal.
/// Remove it.
pub fn trim_zeros(signal:SampleBuffer) -> SampleBuffer {
    let last_sound = find_last_audible_index(&signal, 0.001);
    match last_sound {
        None => signal, 
        Some(ind) => (&signal[0..ind]).to_vec()
    }
}
fn find_last_audible_index(vec: &Vec<f32>, thresh:f32) -> Option<usize> {
    for (i, &value) in vec.iter().enumerate().rev() {
        if value.abs() > thresh{
            return Some(i);
        }
    }
    None
}


/// Given a list of signals that may overlap with one another (e.g. long delay or release times)
/// Create a sample representing their ordered mixing.
pub fn overlapping(len:usize, cps:f32, durs:Vec<f32>, samples:&Vec<SampleBuffer>) -> SampleBuffer {
    let mut signal:SampleBuffer = vec![0f32; len];
    durs.iter().enumerate().fold(0, |pos, (i, dur)| { 
        for (j,s) in samples[i].iter().enumerate() {
            signal[pos + j] += s
        }
        pos + time::samples_of_dur(cps, *dur)
    });
    signal
}

/// Render a signal from contextual and decorative paramters. 
/// Returns a SampleBuffer representing the moment produced from this request.
pub fn summit<'render>(
    cps:f32, 
    root: f32, 
    fundamental:f32,
    n_cycles:f32,
    dressing:&Dressing,
    Feel { bp, expr, modifiers, clippers }: &'render Feel,
    delays: &Vec::<delay::DelayParams>
) -> Vec<f32> {
    let append_delay = time::samples_of_dur(cps, longest_delay_length(delays));
    let n_samples = crate::time::samples_of_cycles(cps, n_cycles) + append_delay;
    let mut sig = vec![0f32; n_samples];
    let (gate_thresh, clip_thresh) = clippers;

    let (modAmp, modFreq, modPhase, modTime) = modifiers;
    for delay_params in delays {
        for j in 0..n_samples {
            for replica_n in 0..(delay_params.n_echoes.max(1)) {
                let gain =  delay::gain(j, replica_n, delay_params);
                if gain < *gate_thresh {
                    continue;
                }

                // do not advace p with the delay; it should use the "p" of its source just delayed in time
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

                if (am*gain) < *gate_thresh {
                    continue;
                }

                for (i, &m) in multipliers.iter().enumerate() {
                    let a0 = amplifiers[i];
                    if a0 > *gate_thresh {
                        let amplifier = modAmp.iter().fold(a0, |acc, ma| ma.apply(t, acc)); 
                        if (amplifier*am*gain) < *gate_thresh {
                            continue
                        }
                        let k = i + 1;
                        let f0:f32 = m * fm * fundamental;
                        let frequency = modFreq.iter().fold(f0, |acc, mf| mf.apply(t, acc)); 
                        let amp = gain * amplifier * am * filter(p, frequency, &bp);
                        let p0 = frequency * pi2 * t;
                        let phase = modPhase.iter().fold(p0, |acc, mp| mp.apply(t, acc)); 
                        
                        v += amp * phase.sin();
                    }
                };

                // apply gating and clipping to the summed sample
                if v.abs() > *clip_thresh {
                    sig[j] += v.signum() * (*clip_thresh);
                } else if v.abs() >= *gate_thresh {
                    sig[j] += v;
                } else {
                    // don't mix this [too quiet] sample!
                }
            }
        }
    }
    sig
}

/// Given a list of stems and how to represent them in space, 
/// Generate the signals and apply reverberation. Return the new signal.
fn combine(cps:f32, root:f32, stems:&Vec<Stem>, reverbs:&Vec<convolution::ReverbParams>) -> SampleBuffer {
    let mut channels:Vec<SampleBuffer> = stems.iter().map(|stem| channel(cps, root, &stem)).collect();
    match pad_and_mix_buffers(channels) {
        Ok(signal) => {
            if reverbs.len() == 0 {
                signal
            } else {
                reverbs.iter().fold(signal, |sig, params| {
                    trim_zeros(convolution::of(&sig, &params))
                })
            }
        },
        Err(msg) => {
            panic!("Failed to mix and render audio: {}", msg)
        }
    }
}


#[cfg(test)]
mod test {
    use convolution::ReverbParams;

    use super::*;
    use crate::{files, phrasing};
    use crate::music::lib::{happy_birthday, x_files};
    use crate::druid::melodic;
    static TEST_DIR:&str = "dev-audio/render";

    fn write_test_asset(signal:&SampleBuffer, test_name:&str) {
        files::with_dir(TEST_DIR);
        let filename = format!("{}/{}.wav", TEST_DIR, test_name);
        engrave::samples(SR, &signal, &filename);
    } 

    fn modifiers_lead() -> ModifiersHolder {
        let mut rng = rand::thread_rng();
        let vibrato = applied_modulation::gen_vibrato(2f32, 13f32, 0f32, 1f32, 0f32, 0f32, &mut rng);
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
        let tremelo = applied_modulation::gen_tremelo(2f32, 13f32, 0f32, 1f32, 0f32, 0f32, &mut rng);
        (
            // vec![tremelo], 
            vec![], 
            vec![], 
            vec![], 
            vec![]
        )
    }

    fn feeling_lead() -> Feel {
        Feel {
            expr: (lifespan::mod_lifespan(SR, 1f32, &AmpLifespan::Snap, 1, 1f32),vec![1f32],vec![0f32]),
            bp: (vec![MFf],vec![NFf]),
            modifiers: modifiers_lead(),
            clippers: (0f32, 1f32)
        }
    }

    fn feeling_chords() -> Feel {
        Feel {
            expr: (vec![1f32],vec![1f32],vec![0f32]),
            bp: (vec![MFf],vec![NFf]),
            modifiers: modifiers_chords(), 
            clippers: (0f32, 1f32)
        }
    }

    #[test]
    fn test_combine() {
        let mods_chords:ModifiersHolder = modifiers_chords();
        let mods_lead:ModifiersHolder = modifiers_lead();

        let delays_lead = vec![
            delay::DelayParams { mix: 0.25f32, gain: 0.66, len_seconds: 0.33f32, n_echoes: 5}
        ];
        let delays_chords = vec![
            delay::DelayParams { mix: 0f32, gain: 0f32, len_seconds: 0.15f32, n_echoes: 5}
        ];

        let stems:Vec<Stem> = vec![
            (happy_birthday::lead_melody(), melodic::dress_square as fn(f32) -> Dressing, feeling_lead(), mods_lead, &delays_lead),
            (happy_birthday::lead_melody(), melodic::dress_sawtooth as fn(f32) -> Dressing, feeling_chords(), mods_chords, &delays_chords)
        ];

        let reverbs:Vec<convolution::ReverbParams> = vec![
            ReverbParams { mix: 0.005, amp: 0.2, dur: 3f32, rate: 0.1 }
        ];

        let result = combine(happy_birthday::cps, happy_birthday::root, &stems, &reverbs);
        write_test_asset(&result, "combine");
        println!("Completed test render")
    }
    use crate::types::timbre::{ SpaceEffects, Positioning, Distance, Echo, Enclosure};
    use rand::seq::SliceRandom;
    use rand::thread_rng;

    fn gen_enclosure() -> Enclosure {
        let mut rng = thread_rng();
        *[Enclosure::Spring, Enclosure::Room, Enclosure::Hall, Enclosure::Vast].choose(&mut rng).unwrap()
    }

    fn gen_positioning() -> Positioning {
        let mut rng = thread_rng();

        Positioning {
            distance: *[Distance::Far, Distance::Near,Distance::Adjacent].choose(&mut rng).unwrap(),
            // echo: *[Some(Echo::Slapback), Some(Echo::Trailing), None].choose(&mut rng).unwrap(),
            echo: *[Some(Echo::Trailing)].choose(&mut rng).unwrap(),
            complexity: if rng.gen::<f32>() < 0.25 { 0f32 } else { rng.gen() } 
        }
    }


    use crate::inp::arg_xform;

    #[test]
    fn test_space_effects() {
        for i in 0..3 {
            let mods_chords:ModifiersHolder = modifiers_chords();
            let mods_lead:ModifiersHolder = modifiers_lead();

            let enclosure = gen_enclosure();
            let se_lead:SpaceEffects = arg_xform::positioning(happy_birthday::cps, &enclosure, &gen_positioning());
            let se_chords:SpaceEffects = arg_xform::positioning(happy_birthday::cps, &enclosure, &gen_positioning());
            let stems:Vec<Stem> = vec![
                (happy_birthday::lead_melody(), melodic::dress_square as fn(f32) -> Dressing, feeling_lead(), mods_lead, &se_lead.delays),
                // (happy_birthday::lead_melody(), melodic::dress_sawtooth as fn(f32) -> Dressing, feeling_chords(), mods_chords, &se_chords.delays)
            ];

            // let result = combine(happy_birthday::cps, happy_birthday::root, &stems, &reverbs);
            let result = combine(happy_birthday::cps, happy_birthday::root, &stems, &se_lead.reverbs);
            write_test_asset(&result, &format!("combine_with_space_{}", i));
            println!("Completed test render")
        }
    }
}