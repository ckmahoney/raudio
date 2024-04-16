use crate::synth::SampleBuffer;
use crate::types::synthesis::*;
use crate::types::render::*;

use crate::song;
use crate::midi;
use crate::midi::Midi;
use crate::monic_theory::tone_to_freq;
use crate::synth;

use std::f32::consts::PI;
use crate::synth::SR;
pub static pi2:f32 = PI*2.;
pub static pi:f32 = PI;

pub enum Ugen {
    Sine,
    Square
}

fn normalize(signal: &mut Vec<f32>) {
    let max_amplitude = signal.iter().map(|&sample| sample.abs()).fold(0.0, f32::max);
    if max_amplitude != 0.0 && max_amplitude > 1.0 {
        signal.iter_mut().for_each(|sample| *sample /= max_amplitude);
    }
}

/// Given dynamic playback rate and constant sample rate, 
/// determines the number of samples required to recreate
/// one second of audio signal.
fn samples_per_cycle(cps:f32) -> usize {
    (SR as f32 / cps) as usize
}

fn fit(a:f32, b:f32) -> f32 {
    if b >= a && b < (a*2.) {
        return b
    } else if b < a {
        return fit(a, b*2.0)
    } else {
        return fit (a, b/2.0)
    }
}

fn dur (cps: f32, ratio:&Ratio) -> f32 {
    (ratio.0 as f32 / ratio.1 as f32)/cps
}

/// Better for linear modulation of amplitude
fn db_to_amp(db:f32) -> f32 {
    10f32.powf(db/20f32)
}

/// Given the current tempo and the number of cycles to span,
/// Create a -60dB to 0dB amplitude curve lasting k cycles.
fn exp_env_k_cycle_db_60_0(cps:f32, k:f32) -> Vec<f32> {
    let n_samples = (samples_per_cycle(cps) as f32 * k) as usize;
    let minDb = -60f32;
    let maxDb = 0f32;
    let dDb = (maxDb - minDb)/n_samples as f32;
    (0..=n_samples).map(|i|
        minDb + i as f32 * dDb
    ).collect()
}


/// 4/pi * sin(kx)/k for odd k > 0 
fn ugen_square(cps:f32, amod:f32, note:&Note) -> synth::SampleBuffer {
    let freq = tone_to_freq(&note.1);
    let k = ((SR as f32 / freq) as usize).max(1).min(50);
    let n_samples = (samples_per_cycle(cps) as f32 * dur(cps, &note.0)) as usize;

    let phase = 0f32;
    let c = 4f32/pi;

    let mut sig:Vec<f32> = vec![0.0; n_samples];
    println!("square k {} for note {:?}", k , note);

    for i in (1..=k).filter(|x| x % 2 == 1) {
        let f = freq * i as f32;
        for j in 0..n_samples {
            let phase = 2.0 * PI * f * (j as f32 / SR as f32);
            sig[j] += amod * c * (phase.sin() / k as f32);
        }
    }
    normalize(&mut sig);
    sig
} 


/// sin(kx)/k for even k > 0 
fn ugen_sine(cps:f32, amod:f32, note:&Note) -> synth::SampleBuffer {
    let freq = tone_to_freq(&note.1);
    let k = ((SR as f32 / freq) as usize).max(1).min(50);
    let n_samples = (samples_per_cycle(cps) as f32 * dur(cps, &note.0)) as usize;

    let phase = 0f32;
    let c = 4f32/pi;

    println!("sine k {} for note {:?}", k , note);
    let mut sig:Vec<f32> = vec![0.0; n_samples];
    for i in (1..=k).filter(|x| *x == 1usize ||  x % 2 == 0) {
        let f = freq * i as f32;
        for j in 0..n_samples {
            let phase = 2.0 * PI * f * (j as f32 / SR as f32);
            sig[j] += amod * phase.sin() / k as f32;
        }
    }
    normalize(&mut sig);
    sig
} 


fn midi_to_mote(cps:f32, (duration, note, amplitude):&Midi) -> Mote {
    let frequency = midi::note_to_frequency(*note as f32);
    let amp = midi::map_amplitude(*amplitude as f32);
    let dur = duration / cps;

    (dur, frequency, amp)
}

fn note_to_mote(cps:f32, (ratio, tone, ampl):&Note) -> Mote {
    (dur(cps,ratio), tone_to_freq(tone), *ampl)
}

fn mix_envelope(env:&SampleBuffer, buf:&mut SampleBuffer) {
    if env.len() > buf.len() {
        panic!("Unable to mix envelopes with greater length than the target signal")
    }

    for i in 0..env.len() {
        buf[i] *= env[i]
    }
}

fn render_note(cps:f32, note:&Note) -> SampleBuffer {
    let (duration, (_, (_,_, monic)), amp) = note;
    let d = dur(cps, duration);
    let adur:f32 = if d > 1.1f32 { 1f32 } else if d > 0.25 { 0.125 } else  { 1f32/64f32 } ;
    let aenv = exp_env_k_cycle_db_60_0(cps, adur);

    let mut buf = match monic {
        1 => {
            ugen_square(cps, 1f32, note)
        },
        3 => {
            ugen_square(cps, 0.5f32, note)
        },
        5 => {
            ugen_sine(cps, 1f32, note)
        },
        _ => {
            ugen_sine(cps, 0.5f32, note)
        }
    };
    mix_envelope(&aenv, &mut buf);
    buf
}

/// Given a list of score part, create a list of motes. 
pub fn midi_entry_to_motes(cps:f32, entry:ScoreEntry<Midi>) -> Melody<Mote> {
    let midi_mels = entry.1;
    midi_mels.into_iter().map(|midi_mel| 
        midi_mel.into_iter().map(|mid| midi_to_mote(cps, &mid)).collect()
    ).collect()
}

/// Given a list of score part, create a list of motes. 
pub fn note_entry_to_motes(cps:f32, entry:ScoreEntry<Note>) -> Melody<Mote> {
    let midi_mels = entry.1;
    midi_mels.into_iter().map(|midi_mel| 
        // midi_mel.into_iter().map(|note| note_to_mote(cps, &note)).collect()
        midi_mel.into_iter().map(|note| note_to_mote(cps, &note)).collect()
    ).collect()
}

pub fn process_midi_parts(parts: Vec::<ScoreEntry<Midi>>, cps: f32) -> Vec<Melody<Mote>> {
    parts.into_iter().map(|entry|
        midi_entry_to_motes(cps, entry)
    ).collect()
}

pub fn process_note_parts(parts: Vec::<ScoreEntry<Note>>, cps: f32) -> Vec<Melody<Mote>> {
    parts.into_iter().map(|entry|
        note_entry_to_motes(cps, entry)
    ).collect()
}
pub fn transform_to_monic_buffers(cps:f32, notes: &Vec<Note>) -> Vec<synth::SampleBuffer> {
    notes.iter().map(|&note| {
        render_note(cps, &note)
    }).collect()
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::song::x_files;
    use crate::song::happy_birthday;

    use crate::render; 
    use crate::files;
    #[test]
    // fn test_song_x_files() {
    //     let track = x_files::get_track();
    //     let cps = track.conf.cps;
    //     let processed_parts = process_midi_parts(track.parts, cps);
    //     let mut buffs:Vec<Vec<synth::SampleBuffer>> = Vec::new();

    //     for mote_mels in processed_parts {
    //         for mel_mote in mote_mels {
    //             buffs.push(transform_to_sample_buffers(cps, &mel_mote))
    //         }
    //     }

    //     let mixers:Vec<synth::SampleBuffer> = buffs.into_iter().map(|buff|
    //         buff.into_iter().flatten().collect()
    //     ).collect();

    //     files::with_dir("dev-audio");
    //     match render::pad_and_mix_buffers(mixers) {
    //         Ok(signal) => {
    //             render::samples_f32(44100, &signal, "dev-audio/x_files.wav");
    //         },
    //         Err(err) => {
    //             println!("Problem while mixing buffers. Message: {}", err)
    //         }
    //     }
    // }


    #[test]
    fn test_song_happy_birthday() {
        let track = happy_birthday::get_track();
        let cps = track.conf.cps;
        let mut buffs:Vec<Vec<synth::SampleBuffer>> = Vec::new();

        for (contrib, mels_notes) in track.parts {
            for mel_notes in mels_notes {
                buffs.push(transform_to_monic_buffers(cps, &mel_notes));
            }
        }

        let mixers:Vec<synth::SampleBuffer> = buffs.into_iter().map(|buff|
            buff.into_iter().flatten().collect()
        ).collect();

        files::with_dir("dev-audio");
        match render::pad_and_mix_buffers(mixers) {
            Ok(signal) => {
                render::samples_f32(44100, &signal, "dev-audio/happy_birthday.wav");
            },
            Err(err) => {
                println!("Problem while mixing buffers. Message: {}", err)
            }
        }
    }
}