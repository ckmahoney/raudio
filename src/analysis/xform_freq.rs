use crate::synth::SampleBuffer;
use crate::types::render::{Melody, Midi, ScoreEntry};
use crate::types::synthesis::{Bandpass, Duration, Freq, Monae, Mote, Note, Tone};

use crate::monic_theory::tone_to_freq;
use crate::time;

pub fn midi_to_freq(midi_note: i32) -> f32 {
  const A4: f32 = 440.0;
  const A4_MIDI: i32 = 69;
  const SEMITONES_IN_OCTAVE: f32 = 12.0;

  A4 * (2.0_f32).powf((midi_note - A4_MIDI) as f32 / SEMITONES_IN_OCTAVE)
}

pub fn velocity_to_amplitudedb(velocity: i8) -> f32 {
  const MIN_VELOCITY: i8 = 0;
  const MAX_VELOCITY: i8 = 127;
  const MIN_DB: f32 = -60.0;
  const MAX_DB: f32 = 0.0;

  let velocity = velocity.clamp(MIN_VELOCITY, MAX_VELOCITY);
  let db = MIN_DB + (velocity as f32 / MAX_VELOCITY as f32) * (MAX_DB - MIN_DB);
  10.0_f32.powf(db / 20.0)
}
pub fn velocity_to_amplitude(velocity: i8) -> f32 {
  velocity as f32 / 127 as f32
}

// fn midi_to_mote(cps:f32, (duration, note, amplitude):&Midi) -> Mote {
//     let frequency = midi::note_to_frequency(*note as f32);
//     let amp = midi::map_amplitude(*amplitude as f32);
//     let dur = duration / cps;

//     (dur, frequency, amp)
// }

fn note_to_mote(cps: f32, (ratio, tone, ampl): &Note) -> Mote {
  (time::dur(cps, ratio), tone_to_freq(tone), *ampl)
}

// fn fill_zeros(cps:f32, n_cycles:f32) -> SampleBuffer {
//     let n_samples = (time::samples_per_cycle(cps) as f32 * n_cycles) as usize;
//     vec![0f32; n_samples]
// }

// /// Given a list of score part, create a list of motes.
// pub fn midi_entry_to_motes(cps:f32, entry:ScoreEntry<Midi>) -> Melody<Mote> {
//     let midi_mels = entry.1;
//     midi_mels.into_iter().map(|midi_mel|
//         midi_mel.into_iter().map(|mid| midi_to_mote(cps, &mid)).collect()
//     ).collect()
// }

// /// Given a list of score part, create a list of motes.
// pub fn note_entry_to_motes(cps:f32, entry:ScoreEntry<Note>) -> Melody<Mote> {
//     let midi_mels = entry.1;
//     midi_mels.into_iter().map(|midi_mel|
//         // midi_mel.into_iter().map(|note| note_to_mote(cps, &note)).collect()
//         midi_mel.into_iter().map(|note| note_to_mote(cps, &note)).collect()
//     ).collect()
// }

// pub fn process_midi_parts(parts: Vec::<ScoreEntry<Midi>>, cps: f32) -> Vec<Melody<Mote>> {
//     parts.into_iter().map(|entry|
//         midi_entry_to_motes(cps, entry)
//     ).collect()
// }

// pub fn process_note_parts(parts: Vec::<ScoreEntry<Note>>, cps: f32) -> Vec<Melody<Mote>> {
//     parts.into_iter().map(|entry|
//         note_entry_to_motes(cps, entry)
//     ).collect()
// }

// /// Generate a list of Note for testing (misnomer of tone, it's older code)
// pub fn test_tone(d:i32, register:i8, n:usize) -> Vec<Note> {
//     let monic:i8 = 1;
//     let rotation:i8 =0;
//     let dur:Duration = (d, 1i32);

//     let qs:Vec<i8> = vec![0];
//     let mut mel:Vec<Note> = Vec::with_capacity(n);
//     let q = 0;
//     let monic = 1;
//     let monae:Monae = (rotation,q, monic);
//     let tone:Tone = (register, monae);
//     for i in 0..n {
//         mel.push((dur, tone, 1f32));
//     }
//     mel
// }
