use hound;
use crate::render;


/// Resamples a given signal to a specified length using linear interpolation.
///
/// # Arguments
/// * `signal` - A slice of the input signal to be resampled.
/// * `target_length` - The desired length of the output signal.
///
/// # Returns
/// A `Vec<f32>` containing the resampled signal.
pub fn resample(signal: &[f32], target_length: usize) -> Vec<f32> {
    if target_length == 0 || signal.is_empty() {
        return Vec::new();
    }
    let mut resampled = Vec::with_capacity(target_length);

    let scale = (signal.len() - 1) as f32 / (target_length - 1) as f32;
    for i in 0..target_length {
        let float_idx = i as f32 * scale;
        let idx = float_idx as usize;
        let frac = float_idx - idx as f32;

        // Linear interpolation
        let next_idx = (idx + 1).min(signal.len() - 1);
        let interpolated_value = (1.0 - frac) * signal[idx] + frac * signal[next_idx];
        resampled.push(interpolated_value);
    }
    resampled
}

pub fn tidy(signal: &mut[f32], length:usize) {
    let mut resampled = resample(&signal, length);
    render::normalize(&mut resampled);
}


/// Performs full convolution on a signal and an impulse response.
/// 
/// This method includes all possible overlaps between the signal and the impulse response.
/// The length of the output is `signal.len() + impulse.len() - 1`.
/// 
/// Benefits: It provides a complete representation of the convolution, including edge effects.
/// Costs: Results in the longest output, which might include significant zero-padding effects at the edges.
/// Lossiness: Non-lossy, as it includes all interactions.
/// 
/// # Arguments
/// * `signal` - A slice of the input signal.
/// * `impulse` - A slice of the impulse response.
/// 
/// # Returns
/// A `Vec<f32>` containing the full convolution result.
pub fn full(signal: &[f32], impulse: &[f32]) -> Vec<f32> {
    let mut convolved = vec![0.0_f32; signal.len() + impulse.len() - 1];
    for (i, &s) in signal.iter().enumerate() {
        for (j, &imp) in impulse.iter().enumerate() {
            convolved[i + j] += s * imp;
        }
    }
    convolved
}

/// Performs full convolution on a signal and an impulse response and then resamples
/// the result back to the original signal length.
///
/// # Arguments
/// * `signal` - A slice of the input signal.
/// * `impulse` - A slice of the impulse response.
///
/// # Returns
/// A `Vec<f32>` containing the convolved and resampled signal.
pub fn full_resample(signal: &[f32], impulse: &[f32]) -> Vec<f32> {
    let convolved = full(signal, impulse);
    resample(&convolved, signal.len())
}


/// Performs same convolution on a signal and an impulse response.
/// 
/// This method pads the input signal to ensure the output has the same length as the input signal.
/// 
/// Benefits: Output length matches the input signal length, useful for consistent dimensionality.
/// Costs: Padding can introduce artifacts at the edges.
/// Lossiness: Potentially lossy at the edges due to padding.
/// 
/// # Arguments
/// * `signal` - A slice of the input signal.
/// * `impulse` - A slice of the impulse response.
/// 
/// # Returns
/// A `Vec<f32>` containing the same convolution result.
pub fn same(signal: &[f32], impulse: &[f32]) -> Vec<f32> {
    let pad = impulse.len() / 2;
    let mut padded_signal = vec![0.0_f32; signal.len() + 2 * pad];
    padded_signal[pad..pad + signal.len()].copy_from_slice(signal);

    let mut convolved = vec![0.0_f32; signal.len()];
    for (i, &s) in padded_signal.iter().enumerate().take(signal.len() + pad) {
        for (j, &imp) in impulse.iter().enumerate() {
            if i + j < convolved.len() {
                convolved[i + j] += s * imp;
            }
        }
    }
    convolved
}

/// Performs valid convolution on a signal and an impulse response.
/// 
/// This method only includes the parts of the signal that fully overlap with the impulse response,
/// resulting in a shorter output.
/// 
/// Benefits: No edge effects due to zero-padding.
/// Costs: Results in a shorter output, potentially missing some interactions.
/// Lossiness: Lossy, as it excludes partial overlaps.
/// 
/// # Arguments
/// * `signal` - A slice of the input signal.
/// * `impulse` - A slice of the impulse response.
/// 
/// # Returns
/// A `Vec<f32>` containing the valid convolution result.
pub fn valid(signal: &[f32], impulse: &[f32]) -> Vec<f32> {
    if impulse.len() > signal.len() {
        return vec![];
    }

    let mut convolved = vec![0.0_f32; signal.len() - impulse.len() + 1];
    for (i, window) in signal.windows(impulse.len()).enumerate() {
        convolved[i] = window.iter().zip(impulse).map(|(&s, &imp)| s * imp).sum();
    }
    convolved
}


mod audio_utils {
    use hound;
    use itertools::Either;
    use itertools::Itertools;
    use std::io::{Error, ErrorKind};

    pub enum AudioData {
        Mono(Vec<f32>),
        Stereo(Vec<f32>, Vec<f32>),
    }

    pub fn read_wav(file_path: &str, max_length: usize) -> Result<AudioData, Error> {
        let mut reader = hound::WavReader::open(file_path).map_err(|e| Error::new(ErrorKind::Other, e))?;
        let spec = reader.spec();
    
        match spec.channels {
            1 => {
                let samples = reader.samples::<i32>().filter_map(Result::ok).map(|y| y as f32 / 2.0f32.powi(23)).take(max_length);
                let mono: Vec<f32> = samples.collect();
                Ok(AudioData::Mono(mono))
            }
            2 => {
                let mut left = Vec::with_capacity(max_length);
                let mut right = Vec::with_capacity(max_length);
                reader.samples::<i32>().filter_map(Result::ok).enumerate().for_each(|(i, sample)| {
                    let sample_f32 = sample as f32 / 2.0f32.powi(23);
                    if i % 2 == 0 {
                        if left.len() < max_length { left.push(sample_f32); }
                    } else {
                        if right.len() < max_length { right.push(sample_f32); }
                    }
                });
                Ok(AudioData::Stereo(left, right))
            }
            _ => Err(Error::new(ErrorKind::InvalidData, "Unsupported number of channels")),
        }
    }

    pub fn mix_down_to_mono(audio_data: AudioData) -> AudioData {
        match audio_data {
            AudioData::Mono(_) => audio_data,
            AudioData::Stereo(left, right) => {
                let mono = left.into_iter().zip(right.into_iter()).map(|(l, r)| (l + r) / 2.0).collect();
                AudioData::Mono(mono)
            },
        }
    }

}

pub fn convolve(signal: &[f32], impulse: &[f32]) -> Vec<f32> {
    let mut convolved = vec![0.0_f32; signal.len() + impulse.len() - 1];
    let mut min: f32 = 0.0;
    let mut max: f32 = 0.0;
    for (i, &s) in signal.iter().enumerate() {
        for (j, &imp) in impulse.iter().enumerate() {
            convolved[i + j] += s * imp;
            min = min.min(convolved[i+j]);
            max = max.max(convolved[i+j]);
        }
    }
    let amplitude_range = max - min;

    convolved.iter_mut().for_each(|sample| {
        *sample = (*sample - min) / amplitude_range * 2.0 - 1.0;
    });
    convolved
}



fn write_wav(file_path: &str, samples: &[i32], spec: hound::WavSpec) {
    let mut writer = hound::WavWriter::create(file_path, spec).expect("Failed to create WAV file");
    for &sample in samples {
        writer.write_sample(sample).expect("Failed to write sample");
    }
    writer.finalize().expect("Failed to finalize WAV file");
}

fn render(input_file: &str, output_file: &str, samples: &[f32]) {
    let reader = hound::WavReader::open(input_file).expect("Failed to open input file");
    let spec = reader.spec();

    let mono_spec = hound::WavSpec {
        channels: 1, // Mono
        sample_rate: spec.sample_rate,
        bits_per_sample: 32, // For f32 samples
        sample_format: hound::SampleFormat::Float, // Set to Float for f32
    };

    let mut writer = hound::WavWriter::create(output_file, mono_spec).expect("Failed to create output file");
    for &sample in samples {
        writer.write_sample(sample).expect("Failed to write sample");
    }
    writer.finalize().expect("Failed to finalize output file");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convolution() {
        let file_a = "/home/naltroc/clones/raudio-synth/dev-audio/guitar.wav";
        let file_b = "/home/naltroc/clones/raudio-synth/dev-audio/kick.wav";
        let output_file = "/home/naltroc/clones/raudio-synth/dev-audio/convolution-guitar-kick-100000.wav";

        let signal_a = audio_utils::read_wav(file_a, 100000).expect("Failed to read file a");
        let signal_b = audio_utils::read_wav(file_b, 100000).expect("Failed to read file b");

        let mixed_signal_a = audio_utils::mix_down_to_mono(signal_a);
        let mixed_signal_b = audio_utils::mix_down_to_mono(signal_b);

        if let audio_utils::AudioData::Mono(data1) = mixed_signal_a {
            let preview = &data1[..std::cmp::min(10, data1.len())]; // Safely take up to 10 elements
            if let audio_utils::AudioData::Mono(data2) = mixed_signal_b {
                let convolved_signal = convolve(&data1, &data2);

                render(&file_a, &output_file, &convolved_signal);
            } else {
                panic!("Expected mono signal for signal b");
            }
        } else {
            panic!("Expected mono signal for signal a");
        }
    }

    
}
