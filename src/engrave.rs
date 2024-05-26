use crate::synth::{MF, NF,SampleBuffer};
use crate::types::synthesis::{Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::types::render::*;
use crate::types::timbre::{BandpassFilter, Energy, Presence, BaseOsc, Sound, FilterMode, Timeframe, Phrasing, Ampex};

use crate::preset::{Modulators, Ctx, Coords};
use crate::{decor, phrasing, AmpLifespan};
use crate::preset;
use crate::envelope;
use crate::song;
use crate::midi;
use crate::midi::Midi;
use crate::monic_theory::tone_to_freq;
use crate::synth;
use crate::time;
pub use crate::analysis::fit;

use std::f32::consts::PI;
use crate::synth::SR;
pub static pi2:f32 = PI*2.;
pub static pi:f32 = PI;

use crate::render::realize;
use crate::render::normalize;


/// activation function for bandpass filter. True indicates frequency is OK; false says to filter it out.
fn bandpass_filter(filter:&BandpassFilter, phr:&Phrasing, freq:f32, i:usize, n:usize) -> bool {
    phrasing::bandpass_filter(filter, freq, i as f32 / n as f32)
}

#[inline]
/// additive synthesizer taking monic modulators in the shape of a "rhodes sine"
fn mgen_sine(cps:f32, note:&Note, ext:usize, sound:&Sound, dir:Direction, phr:&mut Phrasing, mbs: &preset::SomeModulators) -> synth::SampleBuffer {
    let frequency = tone_to_freq(&note.1);
    let ampl = &note.2;
    let ks = ((SR as f32 / frequency) as usize).max(1);
    // let n_samples = (time::samples_per_cycle(cps) as f32 * time::dur(cps, &note.0)) as usize;
    let n_samples = time::samples_per_cycle(cps) as usize;
    
    let mut sig:Vec<f32> = vec![0.0; n_samples];

    let dir:Direction = Direction::Constant;

    
    let m8s:preset::Modulators = decor::gen_from(cps, &note, mbs);

    phr.note.cycles = note.0.1  as f32 / note.0.0 as f32;        
    for k in (1..=ks).filter(|x| *x == 1usize ||  x % 2 == 0) {
        for j in 0..n_samples {
            phr.note.p = j as f32 / n_samples as f32;
            let coords = Coords { cps, k, i: j};
            let ctx = Ctx { 
                root:frequency, 
                dur_seconds: time::dur(coords.cps, &note.0), 
                extension: ext 
            };
            let f = frequency * k as f32 * (m8s.freq)(&coords, &ctx, &sound, &dir, &phr);
            let p_extra = phr.note.p * phr.note.cycles / phr.line.cycles;
            if phrasing::bandpass_filter(&sound.bandpass,  f, phr.line.p + p_extra) {
                let amp = ampl * (m8s.amp)(&coords, &ctx, &sound, &dir, &phr);
                let phase = f * 2.0 * PI * (j as f32 / SR as f32) + (m8s.phase)(&coords, &ctx, &sound, &dir, &phr);
                sig[j] += amp * phase.sin() / (k * k) as f32;
            } else {
                continue
            }
        }
    }
    normalize(&mut sig);
    sig
}

#[inline]
fn mgen_square(cps:f32, note:&Note, ext:usize, sound:&Sound, dir:Direction, phr:&mut Phrasing, mbs: &preset::SomeModulators) -> synth::SampleBuffer {
    let frequency = tone_to_freq(&note.1);
    let ampl = &note.2;
    let ks = ((SR as f32 / frequency) as usize).max(1);
    let n_samples = time::samples_per_cycle(cps) as usize;
    
    let mut sig:Vec<f32> = vec![0.0; n_samples];

    let dir:Direction = Direction::Constant;

    let m8s:preset::Modulators = decor::gen_from(cps, &note, mbs);

    let c = 4f32/pi;

    phr.note.cycles = note.0.1  as f32 / note.0.0 as f32;        
    for k in (1..=ks).filter(|x| x % 2 == 1) {
        for j in 0..n_samples {
            phr.note.p = j as f32 / n_samples as f32;
            let coords = Coords { cps, k, i: j};
            let ctx = Ctx { 
                root:frequency, 
                dur_seconds: time::dur(coords.cps, &note.0), 
                extension: ext 
            };
            let f = frequency * k as f32 * (m8s.freq)(&coords, &ctx, &sound, &dir, &phr);
            if bandpass_filter(&sound.bandpass, phr, f, j, n_samples) {
                let amp = ampl * (m8s.amp)(&coords, &ctx, &sound, &dir, &phr);
                let phase = f * 2.0 * PI * (j as f32 / SR as f32) + (m8s.phase)(&coords, &ctx, &sound, &dir, &phr);
                sig[j] += c * amp * phase.sin() / k as f32;
            } else {
                continue
            }
        }
    }
    normalize(&mut sig);
    sig
}

#[inline]
fn mgen_triangle(cps:f32, note:&Note, ext:usize, sound:&Sound, dir:Direction, phr:&mut Phrasing, mbs: &preset::SomeModulators) -> synth::SampleBuffer {
    let frequency = tone_to_freq(&note.1);
    let ampl = &note.2;
    let ks = ((SR as f32 / frequency) as usize).max(1);
    let n_samples = time::samples_per_cycle(cps) as usize;
    
    let mut sig:Vec<f32> = vec![0.0; n_samples];

    let dir:Direction = Direction::Constant;

    let m8s:preset::Modulators = decor::gen_from(cps, &note, mbs);

    let c = 8f32/(pi *pi);
    
    phr.note.cycles = note.0.1  as f32 / note.0.0 as f32;        
    for k in (1..=ks).filter(|x| x % 2 == 1) {
        let sign = (-1f32).powi(1i32 + k as i32);

        for j in 0..n_samples {
            phr.note.p = j as f32 / n_samples as f32;
            let coords = Coords { cps, k, i: j};
            let ctx = Ctx { 
                root:frequency, 
                dur_seconds: time::dur(coords.cps, &note.0), 
                extension: ext 
            };
            let f = frequency * k as f32 * (m8s.freq)(&coords, &ctx, &sound, &dir, &phr);
            if !bandpass_filter(&sound.bandpass, phr, f, j, n_samples) {
                continue
            } else {
                let amp = ampl * (m8s.amp)(&coords, &ctx, &sound, &dir, &phr);
                let phase = f * 2.0 * PI * (j as f32 / SR as f32) + (m8s.phase)(&coords, &ctx, &sound, &dir, &phr);
                sig[j] += c * amp * phase.cos() / (k * k) as f32;
            }
        }
    }
    normalize(&mut sig);
    sig
}

#[inline]
fn mgen_sawtooth(cps:f32, note:&Note, ext:usize, sound:&Sound, dir:Direction, phr:&mut Phrasing, mbs: &preset::SomeModulators) -> synth::SampleBuffer {
    let frequency = tone_to_freq(&note.1);
    let ampl = &note.2;
    let ks = ((SR as f32 / frequency) as usize).max(1);
    let n_samples = time::samples_per_cycle(cps) as usize;
    
    let mut sig:Vec<f32> = vec![0.0; n_samples];

    let dir:Direction = Direction::Constant;

    let m8s:preset::Modulators = decor::gen_from(cps, &note, mbs);


    let c = 2f32/pi;
    for k in 1..=ks {
        let sign = (-1f32).powi(1i32 + k as i32);

        for j in 0..n_samples {
            phr.note.p = j as f32 / n_samples as f32;
            let coords = Coords { cps, k, i: j};
            let ctx = Ctx { 
                root:frequency, 
                dur_seconds: time::dur(coords.cps, &note.0), 
                extension: ext 
            };
            let f = frequency * k as f32 * (m8s.freq)(&coords, &ctx, &sound, &dir, &phr);
            if !bandpass_filter(&sound.bandpass, phr, f, j, n_samples) {
                continue
            } else {
                let amp = ampl * (m8s.amp)(&coords, &ctx, &sound, &dir, &phr) / k as f32;
                let phase = f * sign * 2.0 * PI * (j as f32 / SR as f32) + (m8s.phase)(&coords, &ctx, &sound, &dir, &phr);
                sig[j] += c * amp * phase.sin();
            }
        }
    }
    normalize(&mut sig);
    sig
}

/**
 * todo
 * 
 * mgen_all is the only integer based mgen I should be using.
 * The "square", "triangle", "sawtooth", and "sine" harmonic selection can all be replaced by a respective amod.
 * 
 * Since A = infinite product (amods)
 * 
 * we can easily choose an amplitude modulation to enable or disable the monic.
 * In fact, since these are constant values, 
 * we could even pre-compute the weights at a reference frequency of 1Hertz as our basis for each. 
 */

#[inline]
fn mgen_all(cps:f32, note:&Note, ext:usize, sound:&Sound, dir:Direction, phr:&mut Phrasing, mbs: &preset::SomeModulators) -> synth::SampleBuffer {
    let frequency = tone_to_freq(&note.1);
    let ampl = &note.2;
    let ks = ((SR as f32 / frequency) as usize).max(1);
    let n_samples = time::samples_per_cycle(cps) as usize;
    
    let mut sig:Vec<f32> = vec![0.0; n_samples];

    let m8s:preset::Modulators = decor::gen_from(cps, &note, mbs);


    for k in 1..=ks {
        for j in 0..n_samples {
            phr.note.p = j as f32 / n_samples as f32;
            let coords = Coords { cps, k, i: j};
            let ctx = Ctx { 
                root:frequency, 
                dur_seconds: time::dur(coords.cps, &note.0), 
                extension: ext 
            };
            let f = frequency * k as f32 * (m8s.freq)(&coords, &ctx, &sound, &dir, &phr);
            if !bandpass_filter(&sound.bandpass, phr, f, j, n_samples) {
                continue
            } else {
                let amp = ampl * (m8s.amp)(&coords, &ctx, &sound, &dir, &phr);
                let phase = f * 2.0 * PI * (j as f32 / SR as f32) + (m8s.phase)(&coords, &ctx, &sound, &dir, &phr);
                sig[j] += amp * phase.sin();
            }
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
    (time::dur(cps,ratio), tone_to_freq(tone), *ampl)
}

fn fill_zeros(cps:f32, n_cycles:f32) -> SampleBuffer {
    let n_samples = (time::samples_per_cycle(cps) as f32 * n_cycles) as usize;
    vec![0f32; n_samples]
}

/// Deprecated method (was the MVP) for generating expressive signals
#[inline]
fn color_mod_note(cps:f32, note:&Note, osc:&BaseOsc, sound:&Sound, dir:Direction, phr:&mut Phrasing, mbs: &preset::SomeModulators) -> SampleBuffer {
    let (duration, (_, (_,_, monic)), amp) = note;
    let d = time::dur(cps, duration);
    let adur:f32 = 0.002;
    let ext:usize = 1;

    if *amp == 0f32 {
        return fill_zeros(cps, d)
    }

    let mut buf = match osc {
        BaseOsc::Sine => {
            mgen_sine(cps, note, ext, sound, dir, phr, mbs)
        },
        BaseOsc::Triangle => {
            mgen_triangle(cps, note, ext, sound, dir, phr, mbs)
        },
        BaseOsc::Square => {
            mgen_square(cps, note, ext, sound, dir, phr, mbs)
        },
        BaseOsc::Sawtooth => {
            mgen_sawtooth(cps, note, ext, sound, dir, phr, mbs)
        },
        // @art-choice Use a different waveform based on monic
        // @art-curr Select richist waveforms for lowest monics and purest for highest
        BaseOsc::Poly => {
            let (duration, (_, (_,_, monic)), amp) = note;
            let d = time::dur(cps, duration);
            let adur:f32 = 2f32/1000f32;

            match monic {
                1 => {
                    mgen_sawtooth(cps, note, ext, sound, dir, phr, mbs)
                },
                3 => {
                    mgen_square(cps, note, ext, sound, dir, phr, mbs)
                },
                5 => {
                    mgen_triangle(cps, note, ext, sound, dir, phr, mbs)
                },
                _ => {
                    mgen_sine(cps, note, ext, sound, dir, phr, mbs)
                }
            }
        },
        BaseOsc::All => {
            mgen_all(cps, note, ext, sound, dir, phr, mbs)
        },
        _ => {
            panic!("Need to implement the matcher for osc type {:?}", osc)
        }
    };
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

pub fn render_line(cps:f32, line: &Vec<Note>, osc:&BaseOsc, sound:&Sound, phr:&mut Phrasing, preset: &preset::SomeModulators) -> Vec<synth::SampleBuffer> {
    let dir = Direction::Constant;
    let len = line.len() as f32;
    let n_cycles = line.iter().fold(0f32, |acc, note| acc + time::duration_to_cycles(note.0));

    phr.line.cycles = n_cycles;
    
    line.iter().enumerate().map(|(index, &note)| {
        phr.line.p = realize::dur_to(&line, index) / n_cycles;
        color_mod_note(cps, &note, &osc, &sound, dir, phr, preset)
    }).collect()
}

/// Deprecated method (was the MVP) for rendering expressive lines
pub fn color_line(cps:f32, notes: &Vec<Note>, osc:&BaseOsc, sound:&Sound, phr:&mut Phrasing, mbs: &preset::SomeModulators) -> Vec<synth::SampleBuffer> {
    let dir = Direction::Constant;
    phr.line.cycles = notes.iter().fold(0f32, |acc, &note| acc + note.0.1 as f32 / note.0.0 as f32 );

    notes.iter().map(|&note| {
        color_mod_note(cps, &note, &osc, &sound, dir, phr, mbs)
    }).collect()
}

/// Generate a list of Note for testing (misnomer of tone, it's older code)
pub fn test_tone(d:i32, register:i8, n:usize) -> Vec<Note> {
    let monic:i8 = 1;
    let rotation:i8 =0;
    let dur:Duration = (d, 1i32);
    
    let qs:Vec<i8> = vec![0];
    let mut mel:Vec<Note> = Vec::with_capacity(n);
    let q = 0;
    let monic = 1;
    let monae:Monae = (rotation,q, monic);
    let tone:Tone = (register, monae);
    for i in 0..n {
        mel.push((dur, tone, 1f32));
    }
    mel
}

#[cfg(test)]
mod test {
    use rustfft::Length;

    use super::*;
    use crate::song::x_files;
    use crate::song::happy_birthday;

    use crate::render; 
    use crate::files;

    static test_dir:&str = "dev-audio/engrave";

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
        let dir = Direction::Constant;

        // this test has one arc containing one line
        // so use the same duration for each of form/arc/line
        let mut phr = Phrasing {
            cps, 
            form: Timeframe {
                cycles: track.duration,
                p: 0f32,
                instance: 0
            },
            arc: Timeframe {
                cycles: track.duration,
                p: 0f32,
                instance: 0
            },
            line: Timeframe {
                cycles: track.duration,
                p: 0f32,
                instance: 0
            },
            note: Timeframe {
                cycles: 0f32,
                p: 0f32,
                instance: 0
            }
        };
        let mbs = preset::SomeModulators {
            amp: None,
            freq: None,
            phase: None,
        };
        
        for (contrib, mels_notes) in track.parts {
            // iterate over the stack of lines
            for mel_notes in mels_notes {
                let sound = Sound {
                    bandpass: (FilterMode::Logarithmic, FilterPoint::Tail, (1f32, 24000f32)),
                    energy: Energy::Medium,
                    presence : Presence::Legato,
                    pan: 0f32,
                };
                buffs.push(render_line(cps, &mel_notes, &BaseOsc::Sine, &sound, &mut phr, &mbs));
            }
        }

        let mixers:Vec<synth::SampleBuffer> = buffs.into_iter().map(|buff|
            buff.into_iter().flatten().collect()
        ).collect();

        files::with_dir(test_dir);
        match render::pad_and_mix_buffers(mixers) {
            Ok(signal) => {
                render::samples_f32(44100, &signal, &format!("{}/happy_birthday.wav", test_dir));
            },
            Err(err) => {
                println!("Problem while mixing buffers. Message: {}", err)
            }
        }
    }

    #[test]
    fn test_song_happy_birthday_percs() {
        use crate::presets;

        let mbs:preset::SomeModulators = preset::SomeModulators {
            amp: Some(presets::kick::amod),
            freq: Some(presets::kick::fmod),
            phase: Some(presets::kick::pmod),
        };
        let track = happy_birthday::get_track();
        let cps = track.conf.cps;
        let mut buffs:Vec<Vec<synth::SampleBuffer>> = Vec::new();
        let dir = Direction::Constant;
        let osc = &BaseOsc::All;

        // this test has one arc containing one line
        // so use the same duration for each of form/arc/line
        let mut phr = Phrasing {
            cps, 
            form: Timeframe {
                cycles: track.duration,
                p: 0f32,
                instance: 0
            },
            arc: Timeframe {
                cycles: track.duration,
                p: 0f32,
                instance: 0
            },
            line: Timeframe {
                cycles: track.duration,
                p: 0f32,
                instance: 0
            },
            note: Timeframe {
                cycles: 0f32,
                p: 0f32,
                instance: 0
            }
        };
        
        for (contrib, mels_notes) in track.parts {
            // iterate over the stack of lines
            for mel_notes in mels_notes {
                let sound = Sound {
                    bandpass: (FilterMode::Linear, FilterPoint::Tail, (1f32, 24000f32)),
                    energy: Energy::Low,
                    presence : Presence::Legato,
                    pan: 0f32,
                };
                buffs.push(render_line(cps, &mel_notes, &osc, &sound, &mut phr, &mbs));
            }
        }

        let mixers:Vec<synth::SampleBuffer> = buffs.into_iter().map(|buff|
            buff.into_iter().flatten().collect()
        ).collect();

        files::with_dir(test_dir);
        match render::pad_and_mix_buffers(mixers) {
            Ok(signal) => {
                render::samples_f32(44100, &signal, &format!("{}/happy_birthday-kick.wav", test_dir));
            },
            Err(err) => {
                println!("Problem while mixing buffers. Message: {}", err)
            }
        }
    }


    #[test]
    fn test_tone_kick() {
        use crate::presets;
        let n_cycles = 16f32;

        let mbs:preset::SomeModulators = preset::SomeModulators {
            amp: Some(presets::kick::amod),
            freq: Some(presets::kick::fmod),
            phase: Some(presets::kick::pmod),
        };

        let cps = 2.1f32;
        let mut buffs:Vec<Vec<synth::SampleBuffer>> = Vec::new();
        let dir = Direction::Constant;
        let osc = &BaseOsc::Sine;

        // this test has one arc containing one line
        // so use the same duration for each of form/arc/line
        let mut phr = Phrasing {
            cps, 
            form: Timeframe {
                cycles: n_cycles,
                p: 0f32,
                instance: 0
            },
            arc: Timeframe {
                cycles: n_cycles,
                p: 0f32,
                instance: 0
            },
            line: Timeframe {
                cycles: n_cycles,
                p: 0f32,
                instance: 0
            },
            note: Timeframe {
                cycles: 0f32,
                p: 0f32,
                instance: 0
            }
        };
        
    
        let sound = Sound {
            bandpass: (FilterMode::Logarithmic, FilterPoint::Constant, (MF as f32, NF as f32)),
            energy: Energy::Medium,
            presence : Presence::Staccatto,
            pan: 0f32,
        };

        let register= 6i8;
        let melody = test_tone(1i32, register, n_cycles as usize);

        let notebufs = render_line(cps, &melody, &osc, &sound, &mut phr, &mbs);
        buffs.push(notebufs);

        let mixers:Vec<synth::SampleBuffer> = buffs.into_iter().map(|buff|
            buff.into_iter().flatten().collect()
        ).collect();

        files::with_dir(test_dir);
        match render::pad_and_mix_buffers(mixers) {
            Ok(signal) => {
                render::samples_f32(44100, &signal, &format!("{}/test-tone-freq-{}-n-{}.wav", test_dir, register, n_cycles));
            },
            Err(err) => {
                println!("Problem while mixing buffers. Message: {}", err)
            }
        }
    }


    // #[test]
    // fn test_test_tone() {
    //     let n_cycles = 4;
    //     let cps = 1.8f32;
    //     let mut buffs:Vec<Vec<synth::SampleBuffer>> = Vec::new();
    //     let register = 6;
    //     let melody:Vec<Note> = test_tone(1i32, register, n_cycles);

    //     let dir = Direction::Constant;

    //     let length = melody.iter().fold(0f32, |acc, &note| acc + time::duration_to_cycles(note.0));

    //     let mbs = preset::SomeModulators {
    //         amp: None,
    //         freq: None,
    //         phase: None,
    //     };

    //     // this test has one noteevent
    //     // so use the same duration for all members
    //     let mut phr = Phrasing { 
    //         cps, 
    //         form: Timeframe {
    //             cycles: length,
    //             p: 0f32,
    //             instance: 0
    //         },
    //         arc: Timeframe {
    //             cycles: length,
    //             p: 0f32,
    //             instance: 0
    //         },
    //         line: Timeframe {
    //             cycles: length,
    //             p: 0f32,
    //             instance: 0
    //         },
    //         note: Timeframe {
    //             cycles: length,
    //             p: 0f32,
    //             instance: 0
    //         }
    //     };

    //     let sound = Sound {
    //         bandpass: (FilterMode::Logarithmic, FilterPoint::Tail, (1000 as f32, NF as f32)),
    //         energy: Energy::Medium,
    //         presence : Presence::Staccatto,
    //         pan: 0f32,
    //     };

    //     let notebufs = render_line(cps, &melody,&BaseOsc::Sine, &sound, &mut phr, &mbs);
    //     buffs.push(notebufs);

    //     let mixers:Vec<synth::SampleBuffer> = buffs.into_iter().map(|buff|
    //         buff.into_iter().flatten().collect()
    //     ).collect();

    //     files::with_dir(test_dir);
    //     match render::pad_and_mix_buffers(mixers) {
    //         Ok(signal) => {
    //             render::samples_f32(44100, &signal, &format!("{}/test-tone-freq-{}-n-{}.wav", test_dir, register, n_cycles));
    //         },
    //         Err(err) => {
    //             println!("Problem while mixing buffers. Message: {}", err)
    //         }
    //     }
    // }
}