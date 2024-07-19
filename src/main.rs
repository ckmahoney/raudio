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
mod reverb;
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
    render_playbook(out_path, file_path);
}

fn render_playbook(out_path: &str, playbook_path: &str) {
    use std::path::Path;
    use std::fs;

    match arg_parse::load_score_from_file(&playbook_path) {
        Ok(score) => {
            match render_score(out_path, score) {
                Ok(_) => {
                    println!("{}", out_path)
                },
                Err(msg) => {
                    eprint!("Problem while writing {}", msg)
                }
            }
        },
        Err(msg) => {
            panic!("Failed to open score: {}", msg)
        }
    }
}


pub fn render_score(filename:&str, score:DruidicScore) -> Result<(), core::fmt::Error> {
    files::with_dir(filename);
    let lens:Vec::<f32> = (&score.parts)
        .iter()
        .map(|(_, _, melody)| 
            melody[0].iter()
            .fold(0f32, |acc, note| acc + time::dur(score.conf.cps, &note.0)) 
        )
    .collect();

    let mut pre_mix_buffs:Vec<synth::SampleBuffer> = Vec::new();
    for (contour, arf, melody) in &score.parts {
        let preset = presets::select(&arf);
        let synth = preset(&arf);
        let mut signal = render::arf(score.conf.cps, contour, &melody, &synth, *arf);
        pre_mix_buffs.push(signal)
    }

    match render::pad_and_mix_buffers(pre_mix_buffs) {
        Ok(signal) => {
            render::engrave::samples(44100, &signal, &filename);
            Ok(())
        },
        Err(msg) => {
            panic!("Failed to mix and render audio: {}", msg)
        }
    }
}


#[test]
fn test_render_playbook() {
    render_playbook("src/demo/test-druidic-render.wav", "src/demo/test-druidic-score.json")
}