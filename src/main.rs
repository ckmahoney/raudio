pub mod freq_forms;
pub mod time_forms;
pub mod synth_config;
pub mod convolve;
pub mod gen;
pub mod sequence;
pub mod envelope;
pub mod mix;
pub mod modulate;
pub mod render;
pub mod phrase;
pub mod canvas;

use wendy;
use wendy::synth::{RenderConfig, HarmonicSelector};
use clap::{App, Arg};
use std::fs;
use std::path::Path;

const DEST: &str = "audio-out";

fn ensure_directory_exists(dir: &str) {
    let path = Path::new(dir);
    if !path.exists() {
        fs::create_dir_all(path).expect("Failed to create directory");
    }
}

fn main() {
    ensure_directory_exists(DEST);
    let matches = App::new("Audio Generator")
        .version("1.0")
        .author("Cortland Mahoney")
        .about("Generates audio files based on harmonics")
        .arg(Arg::with_name("max_harmonic")
             .required(true)
             .index(1)
             .help("The maximum harmonic to generate"))
        .arg(Arg::with_name("selector")
             .long("selector")
             .short('s')
             .takes_value(true)
             .possible_values(&["all", "odd", "even", "geometric", "constant"])
             .help("Sets the harmonic selector"))
        .arg(Arg::with_name("geometric_ratio")
             .long("geometric-ratio")
             .takes_value(true)
             .help("Sets the ratio for geometric harmonic selector"))
        .arg(Arg::with_name("constant_value")
             .long("constant-value")
             .takes_value(true)
             .help("Sets the value for constant harmonic selector"))
        .arg(Arg::with_name("sample_rate")
             .long("sample-rate")
             .short('u')
             .takes_value(true)
             .help("Sets the sample rate"))
        // ... additional arguments for amplitude scaling, cps, harmonic selector, offset ...
        .get_matches();

    let max_harmonic = matches.value_of("max_harmonic").unwrap().parse::<usize>().unwrap();
    let sample_rate = matches.value_of("sample_rate").unwrap_or("44100").parse::<usize>().unwrap();
    let selector = match matches.value_of("selector") {
        Some("odd") => HarmonicSelector::Odd,
        Some("even") => HarmonicSelector::Even,
        Some("geometric") => {
            let ratio = matches.value_of("geometric_ratio").expect("Geometric ratio is required for geometric selector").parse::<f32>().unwrap();
            HarmonicSelector::Geometric(ratio)
        }
        Some("constant") => {
            let value = matches.value_of("constant_value").expect("Constant value is required for constant selector").parse::<f32>().unwrap();
            HarmonicSelector::Constant(value)
        }
        _ => HarmonicSelector::All,
    };
    let config = RenderConfig {
        sample_rate,
        amplitude_scaling: 1.0f32,
        cps: 1.0f32,              
    };
    let start:usize = 1;
    let offset:f32= 0.0;
    let selector_str = match selector {
        HarmonicSelector::All => "all",
        HarmonicSelector::Odd => "odd",
        HarmonicSelector::Even => "even",
        HarmonicSelector::Geometric(_) => "geometric",
        HarmonicSelector::Constant(_) => "constant",
    };
    // println!("Running with args {},{},{},{}", selector_str, start, max_harmonic, sample_rate);
    

    let sum_filename = format!("audio-out/sum_{}_{}_{}_{}.wav", selector_str, start, max_harmonic, sample_rate);
    let mut path = Path::new(&sum_filename);
    if !path.exists() {
        println!("Computing sum");
        let sum_result = wendy::sum_periods(&config, &selector, start, max_harmonic, offset); 
        println!("Writing result...");
        render::samples_f32(config.sample_rate, &sum_result, &sum_filename);
        std::mem::drop(sum_result);
        println!("Wrote sum audio to {}", sum_filename);
    } else {
        println!("Asset exists: {}", sum_filename);
    };

    
    let convolve_filename = format!("audio-out/convolve_{}_{}_{}_{}.wav", selector_str, start, max_harmonic, sample_rate);
    path = Path::new(&convolve_filename);
    if !path.exists() {
        println!("Computing convolution");
        let convolve_result = wendy::convolve_periods(&config, &selector, start, max_harmonic, offset);
        println!("Writing result...");
        render::samples_f32(config.sample_rate, &convolve_result, &convolve_filename);
        println!("Wrote convolution audio to {}", convolve_filename);
    } else {
        println!("Asset exists: {}", convolve_filename);
    }
    return;
}