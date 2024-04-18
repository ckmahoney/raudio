use crate::synth::SampleBuffer;
use crate::types::synthesis::*;
use crate::types::render::*;
use crate::types::timbre::BaseOsc;

use crate::envelope;
use crate::song;
use crate::midi;
use crate::midi::Midi;
use crate::monic_theory::tone_to_freq;
use crate::synth;
use crate::time;

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

fn fit(a:f32, b:f32) -> f32 {
    if b >= a && b < (a*2.) {
        return b
    } else if b < a {
        return fit(a, b*2.0)
    } else {
        return fit (a, b/2.0)
    }
}


/// 4/pi * sum(sin(kx)/k for odd k > 0)
fn ugen_square(cps:f32, amod:f32, note:&Note) -> synth::SampleBuffer {
    let freq = tone_to_freq(&note.1);
    let k = ((SR as f32 / freq) as usize).max(1).min(101);
    let n_samples = (time::samples_per_cycle(cps) as f32 * time::dur(cps, &note.0)) as usize;

    let phase = 0f32;

    let mut sig:Vec<f32> = vec![0.0; n_samples];

    let c = 4f32/pi;
    for i in (1..=k).filter(|x| x % 2 == 1) {
        let f = cps * freq * i as f32;
        for j in 0..n_samples {
            let phase = 2.0 * PI * f * (j as f32 / SR as f32);
            sig[j] += amod * c * (phase.sin() / i as f32);
        }
    }
    normalize(&mut sig);
    sig
} 

/// 8 / pi^2 * sum(cos ks / (k*k)) for odd k > 0
fn ugen_triangle(cps:f32, amod:f32, note:&Note) -> synth::SampleBuffer {
    let freq = tone_to_freq(&note.1);
    let k = ((SR as f32 / freq) as usize).max(1).min(101);
    let n_samples = (time::samples_per_cycle(cps) as f32 * time::dur(cps, &note.0)) as usize;

    let phase = 0f32;

    let mut sig:Vec<f32> = vec![0.0; n_samples];

    let c = 8f32/(pi *pi);
    for i in (1..=k).filter(|x| x % 2 == 1) {
        let f = cps * freq * i as f32;
        for j in 0..n_samples {
            let phase = 2.0 * PI * f * (j as f32 / SR as f32);
            sig[j] += amod * c * (phase.cos() / (i*i) as f32);
        }
    }
    normalize(&mut sig);
    sig
}

/// 2/pi * sum((-1f32).powi(k) * sin(kf) / k) for k > 0
fn ugen_sawtooth(cps:f32, amod:f32, note:&Note) -> synth::SampleBuffer {
    let freq = tone_to_freq(&note.1);
    let k = ((SR as f32 / freq) as usize).max(1).min(51);
    let n_samples = (time::samples_per_cycle(cps) as f32 * time::dur(cps, &note.0)) as usize;

    let phase = 0f32;

    let mut sig:Vec<f32> = vec![0.0; n_samples];

    let c = 2f32/pi;
    for i in 1..=k {
        let f = cps * freq * i as f32;
        for j in 0..n_samples {
            let sign = (-1f32).powi(i as i32);
            let phase = sign * 2.0 * PI * f * (j as f32 / SR as f32);
            sig[j] += amod * c * (phase.sin() / i as f32);
        }
    }
    normalize(&mut sig);
    sig
}

/// sin(kx)/k for even k > 0 
fn ugen_sine(cps:f32, amod:f32, note:&Note) -> synth::SampleBuffer {
    let freq = tone_to_freq(&note.1);
    let k = ((SR as f32 / freq) as usize).max(1).min(51);
    let n_samples = (time::samples_per_cycle(cps) as f32 * time::dur(cps, &note.0)) as usize;

    let phase = 0f32;
    let c = 4f32/pi;

    let mut sig:Vec<f32> = vec![0.0; n_samples];
    for i in (1..=k).filter(|x| *x == 1usize ||  x % 2 == 0) {
        let f = cps * freq * 1.0001f32.powi(i as i32) * i as f32;
        for j in 0..n_samples {
            let phase = 2.0 * PI * f * (j as f32 / SR as f32);
            sig[j] += amod * phase.sin() / (i) as f32;
        }
    }
    normalize(&mut sig);
    sig
} 

// static msg:&str = r#"
// Consider including a mgen collection of functions
// which use the harmonic and phase structure of base
// waveforms like sine, square, sawtooth, triangle and 
// accepts the following args:

// mod_phase: fn (k, i) -> Radian
// mod_amp: fn (k, i) -> Range
// mod_freq: fn (k, i) -> f32
// "#;



fn midi_to_mote(cps:f32, (duration, note, amplitude):&Midi) -> Mote {
    let frequency = midi::note_to_frequency(*note as f32);
    let amp = midi::map_amplitude(*amplitude as f32);
    let dur = duration / cps;

    (dur, frequency, amp)
}

fn note_to_mote(cps:f32, (ratio, tone, ampl):&Note) -> Mote {
    (time::dur(cps,ratio), tone_to_freq(tone), *ampl)
}

fn color_note(cps:f32, note:&Note, osc:&BaseOsc) -> SampleBuffer {
    let (duration, (_, (_,_, monic)), amp) = note;
    let d = time::dur(cps, duration);
    let adur:f32 = 0.002;
    let breath = envelope::db_env_n(time::samples_of_cycles(cps, adur), -60f32, 0f32);
    let envelope = envelope::gen_env(cps, note, breath.len());

    let mut buf = match osc {
        BaseOsc::Sine => {
            ugen_sine(cps, 1f32, note)
        },
        BaseOsc::Triangle => {
            ugen_triangle(cps, 0.5f32, note)
        },
        BaseOsc::Square => {
            ugen_square(cps, 1f32, note)
        },
        BaseOsc::Sawtooth => {
            ugen_sawtooth(cps, 1f32, note)
        }
    };
    envelope::mix_envelope(&breath, &mut buf, 0);
    envelope::mix_envelope(&envelope, &mut buf, breath.len());
    buf
}

fn render_note(cps:f32, note:&Note) -> SampleBuffer {
    let (duration, (_, (_,_, monic)), amp) = note;
    let d = time::dur(cps, duration);
    let adur:f32 = 0.002;
    let breath = envelope::db_env_n(time::samples_of_cycles(cps, adur), -60f32, 0f32);
    let envelope = envelope::gen_env(cps, note, breath.len());

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
    envelope::mix_envelope(&breath, &mut buf, 0);
    envelope::mix_envelope(&envelope, &mut buf, breath.len());
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

pub fn color_line(cps:f32, notes: &Vec<Note>, osc:&BaseOsc) -> Vec<synth::SampleBuffer> {
    notes.iter().map(|&note| {
        color_note(cps, &note, &osc)
    }).collect()
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::song::x_files;
    use crate::song::happy_birthday;

    use crate::render; 
    use crate::files;
    // #[test]
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
                buffs.push(color_line(cps, &mel_notes, &BaseOsc::Sine));
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