use hound::Sample;

use crate::synth::{SR, MF, NF,SampleBuffer, pi, pi2};
use crate::types::timbre::{AmpContour,AmpLifespan,BandpassFilter, Energy, Presence, BaseOsc, Sound2, FilterMode, Timeframe, Phrasing, Ampex};
use crate::types::synthesis::{Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::monic_theory::tone_to_freq;
use crate::presets::{Modulators, Ctx, Coords, DModulators};
use crate::preset;
use crate::synth;
use crate::phrasing::bandpass_filter;

use crate::time;

/// Given an index in the line, return the cumulative cycles passed to reach that point in time.
fn dur_to(line:&Vec<Note>, pos:usize) -> f32 {
    if pos == 0 { 
        0f32
    } else {
        let notes = &line[0..pos];
        notes.iter().fold(0f32, |acc, &note| acc + time::duration_to_cycles(note.0))
    }
}

/// Render a time varying line of notes.
pub fn render_line(line: &Vec<Note>, sound:&Sound2, phr:&mut Phrasing, preset: &DModulators) -> SampleBuffer {
    let dir = Direction::Constant;
    let len = line.len() as f32;
    let n_cycles = line.iter().fold(0f32, |acc, note| acc + time::duration_to_cycles(note.0));
    let ext = 1;
    phr.line.cycles = n_cycles;
    
    let mut buff:SampleBuffer = Vec::new();

    for (index, &note) in line.iter().enumerate() {
        //@bug not correct implementation of p. needs to be decided by accumulative position not index
        phr.line.p = dur_to(&line, index) / n_cycles;
        buff.append(&mut mgen_overs(&note, &sound, phr, preset))
    }
    buff
}


#[inline]
/// Iterates over all available overtones for the given note to produce an audio sample.
fn mgen_overs(note:&Note, sound:&Sound2, phr:&mut Phrasing, m8s: &DModulators) -> synth::SampleBuffer {
    let dur = &note.0;
    let fund = tone_to_freq(&note.1);
    let ampl = &note.2;
    let ks = ((SR as f32 / fund) as usize).max(1);
    let n_samples = time::samples_per_cycle(phr.cps) as usize;
    let mut sig:Vec<f32> = vec![0.0; n_samples];

    let fmod = m8s.freq.as_ref().unwrap();

    for j in 0..n_samples {
        phr.note.p = j as f32 / n_samples as f32;
        for k in 1..=ks {
            let coords = Coords { cps:phr.cps, k, i: j};
            let ctx = Ctx { 
                root: fund, 
                dur_seconds: time::dur(coords.cps, dur), 
                extension: sound.extension 
            };
            let freq = if m8s.freq.is_some() {
                fund * k as f32 * (m8s.freq.as_ref().unwrap())(&coords, &ctx, &sound, &phr)
            } else {
                fund * k as f32
            };
            if !bandpass_filter(&sound.bandpass, freq, j as f32 / n_samples as f32) {
                continue
            } else {
                let amp = if m8s.amp.is_some() {
                    ampl * (m8s.amp.as_ref().unwrap())(&coords, &ctx, &sound, &phr)
                } else {
                    *ampl
                };

                let phase = if m8s.phase.is_some() {
                    freq * pi2 * (j as f32 / SR as f32) + (m8s.phase.as_ref().unwrap())(&coords, &ctx, &sound,  &phr)
                } else {
                    freq * pi2 * (j as f32 / SR as f32)
                };

                sig[j] += amp * phase.sin();
            }
        }
    }
    normalize(&mut sig);
    sig
}

/// Given a group of same-length SampleBuffers,
/// Sum their signals and apply normalization if necessary.
pub fn mix_buffers(buffs: &mut Vec<Vec<f32>>) -> Result<Vec<f32>, &'static str> {
    if buffs.is_empty() {
        return Ok(Vec::new());
    }

    let buffer_length = buffs.first().unwrap().len();
    if buffs.iter().any(|b| b.len() != buffer_length) {
        return Err("Buffers do not have the same length");
    }
    normalize_channels(buffs);
    let mut mixed_buffer:Vec<f32> = vec![0.0; buffer_length];
    for buffer in buffs {
        for (i, sample) in buffer.into_iter().enumerate() {
            mixed_buffer[i] += *sample;
        }
    }

    let max_amplitude = mixed_buffer.iter().map(|&sample| sample.abs()).fold(0.0, f32::max);
    if max_amplitude != 0.0 && max_amplitude > 1.0 {
        mixed_buffer.iter_mut().for_each(|sample| *sample /= max_amplitude);
    }

    Ok(mixed_buffer)
}


fn normalize(signal: &mut Vec<f32>) {
    let max_amplitude = signal.iter().map(|&sample| sample.abs()).fold(0.0, f32::max);
    if max_amplitude != 0.0 && max_amplitude > 1.0 {
        signal.iter_mut().for_each(|sample| *sample /= max_amplitude);
    }
}
fn find_global_max_amplitude(channels: &[Vec<f32>]) -> f32 {
    channels
        .iter()
        .flat_map(|channel| channel.iter())
        .map(|&sample| sample.abs())
        .fold(0.0, f32::max)
}

fn scale_by(channel: &mut [f32], global_max: f32) {
    if global_max > 1.0 {
        channel.iter_mut().for_each(|sample| *sample /= global_max);
    }
}

fn calculate_normalization_factor(channels: &[Vec<f32>]) -> f32 {
    if channels.is_empty() {
        return 1.0;
    }

    let length = channels[0].len();
    let mut summed_signal = vec![0.0; length];

    for channel in channels {
        for (i, &sample) in channel.iter().enumerate() {
            summed_signal[i] += sample;
        }
    }

    let max_amplitude = summed_signal.iter().map(|&sample| sample.abs()).fold(0.0, f32::max);
    if max_amplitude > 1.0 {
        return max_amplitude;
    }

    1.0 // No normalization needed
}

pub fn normalize_channels(channels: &mut Vec<Vec<f32>>) {
    if channels.is_empty() || channels.iter().any(|channel| channel.is_empty()) {
        return;
    }

    let global_max = find_global_max_amplitude(channels);
    if global_max == 0.0 {
        return;
    }

    if global_max > 1.0 {
        for channel in channels.iter_mut() {
            scale_by(channel, global_max);
        }
    }

    let normalization_factor = calculate_normalization_factor(channels);

    if normalization_factor > 1.0 {
        for channel in channels.iter_mut() {
            channel.iter_mut().for_each(|sample| *sample /= normalization_factor);
        }
    }
}


#[test]
fn main() {
    let mut kick = vec![-2.8, 2.8, 1.5, -1.5];  // Example data
    let mut perc = vec![-0.5, 0.5, -0.3, 0.3];
    let mut hats = vec![0.1, -0.1, 0.05, -0.05];

    let mut channels = vec![kick, perc, hats];

    normalize_channels(&mut channels);

    for (i, channel) in channels.iter().enumerate() {
        println!("Channel {}: {:?}", i + 1, channel);
    }
}
