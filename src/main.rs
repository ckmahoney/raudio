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
mod synth_config;
mod demo;
mod files;
mod druid;
mod midi;
mod music;
mod phrasing;
mod preset;
mod presets;
mod render;
mod sequence;
mod synth;
mod time;
mod types;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!(r#"Usage: raudio "/abs/to/playbook.json" "/abs/to/audio.wav""#);
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


// fn contrib_to_osc(contrib:&Contrib) -> timbre::BaseOsc {
//     use timbre::BaseOsc::*;
//     let mut rng = thread_rng(); 
//     let opts:Vec<timbre::BaseOsc> = match contrib.role {
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

// /// Given a part to render as Contrib,
// /// create a spectral domain bandpass filter
// /// error: should also factor the root of composition as well
// fn contrib_to_bandpass(contrib:&Contrib) -> BandpassFilter {
//     static min_supported_frequency:f32 = 22.0;
//     static max_supported_frequency:f32 = 23998.0;

//     let min_max = match contrib.visibility {
//         Visibility::Hidden => {
//             // strict bandfiltering 
//             (2f32.powi(contrib.register as i32), 2f32.powi((contrib.register + 2u32) as i32))
//         },
//         Visibility::Background => {
//             // loose bandfiltering 
//             (2f32.powi(contrib.register as i32 - 1i32), 2f32.powi((contrib.register + 3u32) as i32))
//         },
//         Visibility::Foreground => {
//             // some bandfiltering 
//             (2f32.powi(contrib.register as i32 - 1i32), 2f32.powi((contrib.register + 4u32) as i32))
//         },
//         Visibility::Visible => {
//             // functional bandfiltering 
//             (min_supported_frequency, max_supported_frequency)
//         }
//     };
//     (FilterMode::Linear, FilterPoint::Constant, min_max)
// }


