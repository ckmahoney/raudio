use crate::convolve;
use crate::render;
use crate::synth;
use hound;
use std::result;
use std::{fs, io::BufReader};

pub fn read_samples(file_name:&str) -> Result<synth::SampleBuffer, String> {
    if !fs::metadata(file_name).is_ok() {
        return Err(format!("File not found: {}", file_name));
    }
    let reader = BufReader::new(fs::File::open(file_name).map_err(|e| e.to_string())?);
    let mut wav_reader = hound::WavReader::new(reader)
        .map_err(|e| format!("Error reading WAV file '{}': {}", file_name, e))?;

    let res = wav_reader.samples::<f32>()
        .map(|s| s.unwrap() as f32 )
        .collect();

    Ok(res)
}

pub fn carry(config:&synth::RenderConfig, carrier:&synth::SampleBuffer, colors:Vec<&synth::SampleBuffer>) {
    let mut result = colors.iter().fold(carrier.to_owned(), |carr, impulse| {
        convolve::full(&carr, &impulse)
    });
    render::normalize(&mut result)
}

// Given a carrier and its frequency (required for wavelet scaling)
// apply the provided files as a coloration layer to the carrier
// return the new buffer or error on file handling
pub fn with_samples(freq:f32, carrier:&synth::SampleBuffer, file_names: &Vec<&str>) -> Result<synth::SampleBuffer, String> {
    for &file_name in file_names {
        if !fs::metadata(file_name).is_ok() {
            return Err(format!("File not found: {}", file_name));
        }
    }

    let mut result = carrier.to_owned();

    for &file_name in file_names {
        match read_samples(file_name) {
            Ok(primitive_samples) => {
                let samples = render::rescale(&primitive_samples, 1.0f32, freq);
                result = convolve::full_resample(&result, &samples);
            },
            Err(msg) => {
                println!("Caught error while reading file {}. Message: {}", file_name, msg)
            }
        }
    }
    Ok(convolve::tidy(&mut result, carrier.len()))
}

pub fn with_samples_echo(freq:f32, carrier:&synth::SampleBuffer, file_names: &Vec<&str>, echos: usize) -> Result<Vec<f32>, String> {
    for &file_name in file_names {
        if !fs::metadata(file_name).is_ok() {
            return Err(format!("File not found: {}", file_name));
        }
    }

    let mut result = carrier.to_owned();
    
    for &file_name in file_names {
        match read_samples(file_name) {
            Ok(primitive_samples) => {
                let echoed:Vec<f32> = (0..echos)
                .flat_map(|_| primitive_samples.iter())
                .cloned()
                .collect();
    
                let samples = render::rescale(&echoed, 1.0f32, freq);
                result = convolve::full_resample(&result, &samples);
            },
            Err(msg) => {
                println!("Caught error while reading file {} for echoes. Message: {}", file_name, msg)
            }
        }
    }
    Ok(convolve::tidy(&mut result, carrier.len()))
}

#[test]
fn test_color_samples_two_known_sums() {
    let config = synth::RenderConfig {
        sample_rate: 44100,
        amplitude_scaling: 1.0,
        cps: 1.0,
    };

    // !! Remember that the max allowed harmonic for a 3000 hertz sine is 14.
    let file_names:Vec<&str> = vec!["audio-out/sum_odd_1_20_44100.wav", "audio-out/sum_all_1_30_44100.wav"];
    let frequencies:Vec<f32> = vec![100.0, 600.0, 3000.0];
    let duration = 3.0f32;
    

    for &freq in &frequencies {
        println!("Testing frequency {}", freq);
        let samples = synth::sample_ugen(&config, synth::silly_sine, duration, freq);
        let before_name = format!("dev-audio/color_car_{}_hz.wav", freq);
        render::samples_f32(config.sample_rate, &samples, &before_name);

        println!("Coloring frequency {}", freq);
        match with_samples(freq, &samples, &file_names) {
            Ok(result) => {
                let filename = format!("dev-audio/color_res_sum_{}_hz.wav", freq);
                render::samples_f32(config.sample_rate, &result, &filename);
            },
            Err(msg) => {
                println!("Error while running test: {}", msg)
            }
        }
    }
}
#[test]
fn test_color_samples_two_known_convolutions() {
    let config = synth::RenderConfig {
        sample_rate: 44100,
        amplitude_scaling: 1.0,
        cps: 1.0,
    };

    // !! Remember that the max allowed harmonic for a 3000 hertz sine is 14.
    let file_names:Vec<&str> = vec!["audio-out/convolve_odd_1_5_44100.wav", "audio-out/convolve_all_1_5_44100.wav"];
    let frequencies:Vec<f32> = vec![100.0, 600.0, 3000.0];
    let duration = 3.0f32;
    
    for &freq in &frequencies {
        println!("Testing frequency {}", freq);
        let samples = synth::sample_ugen(&config, synth::silly_sine, duration, freq);
        let before_name = format!("dev-audio/color_car_{}_hz.wav", freq);
        render::samples_f32(config.sample_rate, &samples, &before_name);

        println!("Coloring frequency {}", freq);
        match with_samples(freq, &samples, &file_names) {
            Ok(result) => {
                let filename = format!("dev-audio/color_res_convolve_{}_hz.wav", freq);
                render::samples_f32(config.sample_rate, &result, &filename);
            },
            Err(msg) => {
                println!("Error while running test: {}", msg)
            }
        }
    }
}


#[test]
fn test_color_samples_two_known_convolutions_meow() {
    let config = synth::RenderConfig {
        sample_rate: 44100,
        amplitude_scaling: 1.0,
        cps: 1.0,
    };

    // !! Remember that the max allowed harmonic for a 3000 hertz sine is 14.
    let file_names:Vec<&str> = vec!["dev-audio/cat-meow-6226.mp3"];
    let frequencies:Vec<f32> = vec![100.0, 600.0, 3000.0];
    let duration = 3.0f32;
    
    for &freq in &frequencies {
        println!("Testing frequency {}", freq);
        let samples = synth::sample_ugen(&config, synth::silly_sine, duration, freq);
        let before_name = format!("dev-audio/color_car_{}_hz.wav", freq);
        render::samples_f32(config.sample_rate, &samples, &before_name);

        println!("Coloring frequency {}", freq);
        match with_samples(freq, &samples, &file_names) {
            Ok(result) => {
                let filename = format!("dev-audio/color_res_convolve_{}_hz_meow.wav", freq);
                render::samples_f32(config.sample_rate, &result, &filename);
            },
            Err(msg) => {
                println!("Error while running test: {}", msg)
            }
        }
    }
}


#[test]
fn test_color_samples_convolution_reverb() {
    let config = synth::RenderConfig {
        sample_rate: 44100,
        amplitude_scaling: 1.0,
        cps: 1.0,
    };

    // !! Remember that the max allowed harmonic for a 3000 hertz sine is 14.
    let file_names:Vec<&str> = vec!["dev-audio/reverb-sample.wav"];
    let frequencies:Vec<f32> = vec![100.0, 600.0, 3000.0];
    let duration = 3.0f32;
    
    for &freq in &frequencies {
        println!("Testing frequency {}", freq);
        let samples = synth::sample_ugen(&config, synth::silly_sine, duration, freq);
        let before_name = format!("dev-audio/color_car_{}_hz.wav", freq);
        render::samples_f32(config.sample_rate, &samples, &before_name);

        println!("Coloring frequency {}", freq);
        match with_samples(freq, &samples, &file_names) {
            Ok(result) => {
                let filename = format!("dev-audio/color_res_convolve_{}_hz_reverb.wav", freq);
                render::samples_f32(config.sample_rate, &result, &filename);
            },
            Err(msg) => {
                println!("Error while running test: {}", msg)
            }
        }
    }
}