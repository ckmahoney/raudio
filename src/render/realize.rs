use hound::Sample;

use crate::synth::{SR, MF, NF,SampleBuffer, pi, pi2};
use crate::types::timbre::{AmpContour,AmpLifespan,BandpassFilter, Energy, Presence, BaseOsc, Sound2, FilterMode, Timeframe, Phrasing};
use crate::types::synthesis::{Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::monic_theory::tone_to_freq;
use crate::synth;

use crate::time;

/// Given an index in the line, return the cumulative cycles passed to reach that point in time.
pub fn dur_to(line:&Vec<Note>, pos:usize) -> f32 {
    if pos == 0 { 
        0f32
    } else {
        let notes = &line[0..pos];
        notes.iter().fold(0f32, |acc, &note| acc + time::duration_to_cycles(note.0))
    }
}


/// Allow zero padding buffers to some degree, 
/// Or error if over the given threshold. 
pub fn massage_buffers_or_die(buffs:&mut Vec<Vec<f32>>, err_thresh:usize) -> Result<(), &'static str>  {
    let lens = (&buffs).iter().map(|b| b.len());
    let max = lens.clone().fold(0, usize::max);
    if max - lens.fold(1000, usize::min) > err_thresh {
        return Err("Buffers do not have the same length and exceed allowed length difference");
    };
    for buff in buffs.iter_mut() {
        if buff.len() != max {
            buff.append(&mut vec![0f32; max - buff.len()])
        }
    };
    Ok(())
}

/// Given a group of same-length SampleBuffers,
/// Sum their signals and apply normalization if necessary.
pub fn mix_buffers(buffs: &mut Vec<Vec<f32>>) -> Result<Vec<f32>, &'static str> {
    if buffs.is_empty() {
        return Ok(Vec::new());
    }

    let max_length = (&buffs).iter().map(|channel| channel.len()).max().unwrap_or(0);
    normalize_channels(buffs);
    let mut mixed_buffer:Vec<f32> = vec![0.0; max_length];
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

    let max_length = channels.iter().map(|channel| channel.len()).max().unwrap_or(0);
    let mut summed_signal = vec![0.0; max_length];

    for channel in channels {
        for (i, &sample) in channel.iter().enumerate() {
            if i < summed_signal.len() {
                summed_signal[i] += sample;
            }
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
