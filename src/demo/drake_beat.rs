use super::out_dir;
use std::iter::FromIterator;

use crate::files;
use crate::synth::{MF, NF, SR, SampleBuffer, pi, pi2};
use crate::types::synthesis::{Ampl, Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::types::render::{Melody};
use crate::types::timbre::{FilterMode, Sound, Sound2, Energy, Presence, Timeframe, Phrasing};
use crate::{presets, render};
use crate::time;
use presets::{PartialModulators, DModulators, kick, snare};

static demo_name:&str = "drakebeat";

fn make_line(durations:Vec<Duration>, registers:Vec<i8>, amps:Vec<Ampl>, overs:bool) -> Vec<Note> {
    let len = durations.len();
    if len != registers.len() || len != amps.len() {
        panic!("Must provide the same number of components per contributor. Got actual lengths for duration {} register {} amp {}", len, registers.len(), amps.len());
    }

    let mut line:Vec<Note> = Vec::with_capacity(len);
    for (i, duration) in durations.iter().enumerate() {
        let register = registers[i];
        let amp = amps[i];
        line.push(test_note(*duration, register, amp, overs))
    }
    line
}

fn make_melody(durations:Vec<Duration>, registers:Vec<i8>, amps:Vec<Ampl>, overs:bool) -> Melody<Note> {
    vec![
        make_line(durations, registers, amps, overs)
    ]
}

/// given a length, duration, ampltidue, and space selection, 
/// create a note in the register.
fn test_note(duration:Duration, register:i8, amp:f32, overs:bool) -> Note {
    let monic:i8 = 1;
    let rotation:i8 = 0;
    
    let q:i8 = if overs { 0 } else { 1 };
    let monic = 1;
    let monae:Monae = (rotation,q, monic);
    (duration, (register, monae), amp)
}

/// Produces the (kick, perc, hats) tuple 
fn make_melodies() -> [Melody<Note>; 3] {
    let tala_hats:Vec<Duration> = vec![(1i32, 4i32); 16];
    let tala_perc:Vec<Duration> = vec![
        (1i32,1i32), // rest
        (3i32,4i32),
        (5i32,4i32),
        (1i32,1i32)
    ];
    let tala_kick:Vec<Duration> = vec![
        (1i32, 2i32),
        (1i32, 2i32),
        (1i32, 1i32), // rest
        (2i32, 1i32)
    ];

    let amp_hats:Vec<Ampl> = vec![
        0.5f32, 0.5f32, 1f32, 0.5f32,
        0.5f32, 0.5f32, 0.66f32, 0.5f32,
        0.5f32, 0.5f32, 0.75f32, 0.5f32,
        0.5f32, 0.5f32, 0.66f32, 0.5f32,
    ];

    let amp_perc:Vec<Ampl> = vec![
        0f32, 1f32, 0.66f32, 0.75f32
    ];

    let amp_kick:Vec<Ampl> = vec![
        1f32, 0.66f32, 0f32, 1f32
    ];

    let register_hats:Vec<i8> = vec![10i8; amp_hats.len()];
    let register_perc:Vec<i8> = vec![8i8; amp_perc.len()];
    let register_kick:Vec<i8> = vec![5i8; amp_kick.len()];

    [
        make_melody(tala_kick, register_kick, amp_kick, true),
        make_melody(tala_perc, register_perc, amp_perc, true),
        make_melody(tala_hats, register_hats, amp_hats, true),
    ]
}

fn make_sounds() -> [Sound2; 3] {
    let allpass = (FilterMode::Linear, FilterPoint::Constant, (MF as f32, NF as f32));
    let sound_hats:Sound2= Sound2{
        bandpass: allpass,
        extension: 1usize
    };
    let sound_perc:Sound2= Sound2{
        bandpass: allpass,
        extension: 1usize
    };
    let sound_kick:Sound2= Sound2{
        bandpass: allpass,
        extension: 1usize
    };
    [   
        sound_kick,
        sound_perc, 
        sound_hats
    ]
}

fn make_synths() -> [DModulators; 3] {
    let synth_kick:DModulators = PartialModulators {
        amp: Some(kick::amod2),
        freq: Some(kick::fmod2),
        phase: Some(kick::pmod2),
    }.dynamize();

    let synth_perc:DModulators = PartialModulators {
        amp: Some(kick::amod2),
        freq: Some(kick::fmod2),
        phase: Some(kick::pmod2),
    }.dynamize();

    let synth_hats:DModulators = PartialModulators {
        amp: Some(kick::amod2),
        freq: Some(kick::fmod2),
        phase: Some(kick::pmod2),
    }.dynamize();

    [
        synth_kick,
        synth_perc,
        synth_hats
    ]
}
fn gen_signal_composite(cps:f32, sound:&Sound2, melody:&Melody<Note>, m8s:&DModulators) -> SampleBuffer {
    let n_cycles:f32 = melody[0].iter().fold(0f32, |acc, note| acc + time::duration_to_cycles(note.0));

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

    let mut voices:Vec<SampleBuffer> = Vec::new();
    let snare_energy = Energy::Medium;
    
    for line in melody.iter() {
        let mut channels = vec![
            // crate::render::realize::render_line(&line, &sound, &mut phr, &m8s),
            snare::render_line(&line, &snare_energy, &sound, &mut phr)
        ];
        let buff:SampleBuffer = match render::realize::mix_buffers(&mut channels) {
            Ok(signal) => signal,
            Err(msg) => panic!("Error while mixing signal components {}", msg)
        };
        voices.push(buff)
    }

    render::realize::normalize_channels(&mut voices);
    match render::realize::mix_buffers(&mut voices) {
        Ok(signal) => signal,
        Err(msg) => panic!("Error while rendering signal {}", msg)
    }
}

fn gen_signal_m8s(cps:f32, sound:&Sound2, melody:&Melody<Note>, m8s:&DModulators) -> SampleBuffer {
    let n_cycles:f32 = melody[0].iter().fold(0f32, |acc, note| acc + time::duration_to_cycles(note.0));
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

    let mut voices:Vec<SampleBuffer> = melody.iter().map(|line|
        crate::render::realize::render_line(&line, &sound, &mut phr, &m8s)
    ).collect();

    render::realize::normalize_channels(&mut voices);
    match render::realize::mix_buffers(&mut voices) {
        Ok(signal) => signal,
        Err(msg) => panic!("Error while rendering signal {}", msg)
    }
}

fn demonstrate() {
    let cps:f32 = 1.15;
    let labels:Vec<&str> = vec!["kick", "perc"];
    let labels:Vec<&str> = vec!["kick", "perc", "hat"];
    let melodies = make_melodies();
    let sounds = make_sounds();
    let synths = make_synths();

    let mut buffs:Vec<SampleBuffer> = Vec::with_capacity(melodies.len());

    files::with_dir(out_dir);
    for (i, label) in labels.iter().enumerate() {
        let filename = format!("{}/{}-{}.wav", out_dir, demo_name, label);
        let signal:SampleBuffer = if i == 0 {
            gen_signal_m8s(cps, &sounds[i], &melodies[i], &synths[i])
        } else {
            println!("Rendering gen_signal_composite {}", label);
            gen_signal_composite(cps, &sounds[i], &melodies[i], &synths[i])
        };
        render::engrave::samples(SR, &signal, &filename);
        buffs.push(signal);
        println!("Rendered stem {}", filename);
    }

    match render::realize::mix_buffers(&mut buffs) {
        Ok(mixdown) => {
            let filename = format!("{}/{}.wav", out_dir, demo_name);
            render::engrave::samples(SR, &mixdown, &filename);
            println!("Rendered mixdown {}", filename);
        },
        Err(msg) => panic!("Error while preparing mixdown: {}", msg)
    }
}

#[test]
fn test() {
    demonstrate()
}

// struct Contribution {
//     range: (f32, f32),
//     activation_fn: Box<dyn Fn(f32) -> f32>,
//     gen_fn: Box<dyn Fn(f32) -> f32>,
// }

// fn interpolate_functions(contributions: &[Contribution], t: f32, x: f32) -> f32 {
//     let mut total_weight = 0.0;
//     let mut result = 0.0;

//     for contribution in contributions {
//         let (start_x, end_x) = contribution.range;
//         if x >= start_x && x <= end_x {
//             let weight = (contribution.activation_fn)(x);
//             result += weight * (contribution.gen_fn)(t);
//             total_weight += weight;
//         }
//     }

//     if total_weight > 0.0 {
//         result / total_weight
//     } else {
//         0.0  // Default value if no contributions are valid
//     }
// }

// fn find_bounds(contributions: &[Contribution], t: f32) -> (f32, f32) {
//     let mut min_value = f32::INFINITY;
//     let mut max_value = f32::NEG_INFINITY;

//     let step = 0.001; // Step size for iterating over the input domain
//     for x in (0..=1000).map(|i| i as f32 * step / 1000.0) {
//         let value = interpolate_functions(contributions, t, x);
//         if value < min_value {
//             min_value = value;
//         }
//         if value > max_value {
//             max_value = value;
//         }
//     }

//     (min_value, max_value)
// }

// #[test]
// fn tmain() {
//     let a = Contribution {
//         range: (0.0, 0.5),
//         activation_fn: Box::new(|x: f32| (pi * x + pi / 2.0).sin()),
//         gen_fn: Box::new(|_: f32| 0.0), // Placeholder generator function
//     };

//     let b = Contribution {
//         range: (0.25, 0.75),
//         activation_fn: Box::new(|x: f32| (pi * x).sin()),
//         gen_fn: Box::new(|_: f32| 0.0), // Placeholder generator function
//     };

//     let c = Contribution {
//         range: (0.5, 1.0),
//         activation_fn: Box::new(|x: f32| (pi * x - pi / 2.0).sin()),
//         gen_fn: Box::new(|_: f32| 0.0), // Placeholder generator function
//     };

//     let contributions:Vec<Contribution> = vec![a, b, c];

//     let t = 1.0;
//     let (min_value, max_value) = find_bounds(&contributions, t);

//     let scale_factor = if max_value != min_value {
//         1.0 / (max_value - min_value)
//     } else {
//         1.0
//     };

//     let x_values = vec![0.0, 0.1, 0.25, 0.3, 0.5, 0.6, 0.75, 0.9, 1.0];
//     for x in x_values {
//         let result = interpolate_functions(&contributions, t, x);
//         let normalized_result = (result - min_value) * scale_factor;
//         let clamped_result = normalized_result.max(0.0).min(1.0); // Ensure the result is clamped to [0, 1]
//         println!("Interpolated and normalized value at t = {}, x = {}: {}", t, x, clamped_result);
//     }
// }