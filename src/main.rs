#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(non_snake_case)]
#![allow(unused_must_use)]
#![allow(non_upper_case_globals)]

use std::env;
use std::process;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::types::synthesis;
use crate::types::synthesis::*;
use crate::types::timbre;
use crate::types::timbre::*;
use crate::types::render::*;

mod arg_parse;
mod analysis;
pub use analysis::monic_theory;
mod demo;
mod files;
mod druid;
mod music;
mod phrasing;
mod presets;
mod render;
mod synth;
pub use analysis::time;
mod types;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!(r#"Usage: raudio "/abspath/in/to/playbook.json" "/abspath/out/to/audio.wav""#);
        process::exit(1);
    }

    let file_path = &args[1];
    let out_path = &args[2];
    // render_playbook(out_path, file_path);
}

// fn render_playbook(out_path: &str, playbook_path: &str) {
//     use std::path::Path;
//     use std::fs;

//     match arg_parse::load_score_from_file(&playbook_path) {
//         Ok(score) => {
//             match render_score(out_path, score) {
//                 Ok(_) => {
//                     println!("{}", out_path)
//                 },
//                 Err(msg) => {
//                     eprint!("Problem while writing {}", msg)
//                 }
//             }
//         },
//         Err(msg) => {
//             panic!("Failed to open score: {}", msg)
//         }
//     }
// }


// fn arf_to_osc(arf:&Arf) -> timbre::BaseOsc {
//     use timbre::BaseOsc::*;
//     let mut rng = thread_rng(); 
//     let opts:Vec<timbre::BaseOsc> = match arf.role {
//         Role::Kick => {
//             vec![Bell]
//         },
//         Role::Perc => {
//             vec![Noise]
//         },
//         Role::Hats => {
//             vec![Bell, Noise]
//         },
//         Role::Bass => {
//             vec![Sawtooth, Square, Sine]
//         },
//         Role::Chords => {
//             vec![Poly, Square, Sine]
//         },
//         Role::Lead => {
//             vec![Triangle, Square, Sine]
//         },
//     };
//     opts.choose(&mut rng).unwrap().clone()
// }

// /// Given a part to render as Arf,
// /// create a spectral domain bandpass filter
// /// error: should also factor the root of composition as well
// fn arf_to_bandpass(arf:&Arf) -> BandpassFilter {
//     static min_supported_frequency:f32 = 22.0;
//     static max_supported_frequency:f32 = 23998.0;

//     let min_max = match arf.visibility {
//         Visibility::Hidden => {
//             // strict bandfiltering 
//             (2f32.powi(arf.register as i32), 2f32.powi((arf.register + 2i8) as i32))
//         },
//         Visibility::Background => {
//             // loose bandfiltering 
//             (2f32.powi(arf.register as i32 - 1i32), 2f32.powi((arf.register + 3i8) as i32))
//         },
//         Visibility::Foreground => {
//             // some bandfiltering 
//             (2f32.powi(arf.register as i32 - 1i32), 2f32.powi((arf.register + 4i8) as i32))
//         },
//         Visibility::Visible => {
//             // functional bandfiltering 
//             (min_supported_frequency, max_supported_frequency)
//         }
//     };
//     (FilterMode::Linear, FilterPoint::Constant, min_max)
// }



// pub fn render_score(filename:&str, score:Score) -> Result<(), core::fmt::Error> {
//     files::with_dir(filename);
//     let lens:Vec::<f32> = (&score.parts).iter()
//     .map(|(_, melody)| 
//         melody[0].iter()
//         .fold(0f32, |acc, note| acc + time::dur(score.conf.cps, &note.0)) 
//     ).collect();

//     let mut phr = Phrasing { 
//         cps: score.conf.cps,
//         form: Timeframe {
//             cycles: lens[0],
//             p: 0f32,
//             instance: 0
//         },
//         arc: Timeframe {
//             cycles: lens[0],
//             p: 0f32,
//             instance: 0
//         },
//         line: Timeframe {
//             cycles: lens[0],
//             p: 0f32,
//             instance: 0
//         },
//         note: Timeframe {
//             cycles: -1.0,
//             p: 0f32,
//             instance: 0
//         }
//     };
// use crate::types::timbre::Role;
// use presets::{kick, snare, hats};
// let mut pre_mix_buffs:Vec<synth::SampleBuffer> = Vec::new();
//     for (arf, melody) in &score.parts {
//         let preset = match arf.role {
//             Role::Kick => kick::synth(),
//             Role::Perc => snare::synth(),
//             Role::Hats => hats::synth(),
//             _ => panic!("Need to finish this match handler"),
//         };
//         let osc:BaseOsc = contrib_to_osc(&contrib);
//         let sound = Sound {
//             bandpass: contrib_to_bandpass(&contrib),
//             energy: contrib.energy,
//             presence : contrib.presence,
//             pan: 0f32,
//         };
//         //@art-choice Create modulators in a diferent way
//         let mbs:preset::SomeModulators = preset::SomeModulators {
//             amp: Some(match contrib.ampex.0 {
//                 AmpLifespan::Drone => {
//                     preset::drone::amod
//                 },
//                 AmpLifespan::Pluck => {
//                     preset::pluck::amod_tanh
//                 },
//                 _ => {
//                     preset::none::amod
//                 }   
//             }),
//             freq: None,
//             phase: None,
//         };
//         for line in melody {
//             let mut line_buff:synth::SampleBuffer = engrave::color_line(score.conf.cps, &line, &osc, &sound, &mut phr, &mbs)
//                 .into_iter()
//                 .flatten()
//                 .collect();
//             pre_mix_buffs.push(line_buff)
//         }
//     }

//     match render::pad_and_mix_buffers(pre_mix_buffs) {
//         Ok(signal) => {
//             render::samples_f32(44100, &signal, &filename);
//             Ok(())
//         },
//         Err(msg) => {
//             panic!("Failed to mix and render audio: {}", msg)
//         }
//     }
// }
