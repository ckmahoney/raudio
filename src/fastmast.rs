use crate::analysis::sampler::{read_audio_file, write_audio_file, AudioFormat};
use crate::analysis::groovability::groove_optimizer;
use crate::render;
use rubato::{SincFixedIn, SincInterpolationParameters, Resampler, SincInterpolationType, WindowFunction};
use crate::synth::SR;

/// Load and resample audio if necessary
fn load_and_resample_audio(input_path: &str, target_sample_rate: u32) -> (Vec<Vec<f32>>, u32) {
    let (audio, input_sample_rate) = read_audio_file(input_path)
        .unwrap_or_else(|err| panic!("Failed to read input file '{}': {}", input_path, err));

    // Debug: Write raw input for verification
    render::engrave::write_audio(input_sample_rate as usize, audio.clone(), "rawback.wav");

    if input_sample_rate != target_sample_rate {
        let resample_ratio = target_sample_rate as f64 / input_sample_rate as f64;
        let sinc_params = SincInterpolationParameters {
            sinc_len: 128,
            f_cutoff: 0.95,
            interpolation: SincInterpolationType::Cubic,
            oversampling_factor: 32,
            window: WindowFunction::BlackmanHarris2,
        };

        let mut resampler = SincFixedIn::<f32>::new(
            resample_ratio,
            10.0,
            sinc_params,
            1024, // Chunk size
            audio.len(), // Number of channels
        )
        .unwrap();

        let input_slices: Vec<&[f32]> = audio.iter().map(|ch| ch.as_slice()).collect();
        let mut resampled_audio: Vec<Vec<f32>> = vec![Vec::new(); audio.len()];

        let mut remaining_samples = input_slices[0].len();
        let mut offset = 0;

        while remaining_samples > 0 {
            let chunk_size = remaining_samples.min(1024); // Use at most 1024 samples per chunk
            let chunk: Vec<Vec<f32>> = input_slices
                .iter()
                .map(|&ch| {
                    let mut padded_chunk = vec![0.0; 1024];
                    padded_chunk[..chunk_size].copy_from_slice(&ch[offset..offset + chunk_size]);
                    padded_chunk
                })
                .collect();

            let chunk_slices: Vec<&[f32]> = chunk.iter().map(|v| v.as_slice()).collect();

            let processed_chunk = resampler.process(&chunk_slices, None).unwrap();

            for (channel_idx, channel_data) in processed_chunk.iter().enumerate() {
                resampled_audio[channel_idx].extend(channel_data);
            }

            remaining_samples -= chunk_size;
            offset += chunk_size;
        }

        // Debug: Write resampled output for verification
        render::engrave::write_audio(target_sample_rate as usize, resampled_audio.clone(), "resampled.wav");
        (resampled_audio, target_sample_rate)
    } else {
        println!("No resampling needed; input sample rate matches target sample rate.");
        (audio, input_sample_rate)
    }
}

fn process_audio(audio: Vec<Vec<f32>>, num_channels: usize) -> Vec<Vec<f32>> {
    let input_signal = if num_channels == 1 {
        AudioFormat::Mono(audio[0].clone())
    } else if num_channels == 2 {
        AudioFormat::Stereo(audio[0].clone(), audio[1].clone())
    } else {
        panic!("Unsupported number of channels: {}", num_channels);
    };

    let processed = groove_optimizer(
        input_signal,
        0.8,  // Transient emphasis
        0.6,  // Dynamic range control
        0.5,  // Rhythmic threshold
        50.0, // Time constant (ms)
    );

    match processed {
        AudioFormat::Mono(samples) => vec![samples],
        AudioFormat::Stereo(left, right) => vec![left, right],
        AudioFormat::Interleaved(_) => panic!("Unexpected interleaved format after processing"),
    }
}



/// Write processed audio to a file
fn write_audio(output_path: &str, audio: &[Vec<f32>], sample_rate: u32) {
    let max_samples = audio.iter().map(|ch| ch.len()).max().unwrap();
    let padded_audio: Vec<Vec<f32>> = audio
        .iter()
        .map(|ch| {
            let mut ch_padded = ch.clone();
            ch_padded.resize(max_samples, 0.0); // Pad with silence
            ch_padded
        })
        .collect();

    write_audio_file(output_path, &padded_audio, sample_rate);
}

/// Apply danceability optimization
pub fn apply_danceability(input_path: &str, output_path: &str) {
    use crate::synth::SR;

    // Step 1: Load and resample audio
    let (audio, target_sample_rate) = load_and_resample_audio(input_path, SR as u32);
    let num_channels = audio.len();

    // Step 2: Process audio
    let processed_audio = process_audio(audio, num_channels);

    // Step 3: Write audio
    write_audio(output_path, &processed_audio, target_sample_rate);
}



/// Test function for apply_danceability
#[test]
fn test_apply_danceability_function() {
    let input_path = "/home/naltroc/Music/Music/Wallpaper/RoomFeel/Polunkite.wav";
    let output_path = "test-output-danceability2.wav";

    println!("Testing apply_danceability from '{}' to '{}'", input_path, output_path);

    // Call the apply_danceability function
    apply_danceability(input_path, output_path);

    // Verify output
    use std::path::Path;
    assert!(
        Path::new(output_path).exists(),
        "Output file '{}' was not created.",
        output_path
    );

    // Validate the output
    let (output_audio, output_sample_rate) = read_audio_file(output_path)
        .unwrap_or_else(|err| panic!("Failed to read output file '{}': {}", output_path, err));
    assert_eq!(output_sample_rate, crate::synth::SR as u32, "Sample rate mismatch.");
    assert!(!output_audio.is_empty(), "Output audio is empty.");
    assert_eq!(output_audio.len(), 2, "Expected 2 channels in output audio.");

    println!("apply_danceability test passed, output written to '{}'", output_path);
}
