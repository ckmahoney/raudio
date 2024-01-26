use hound;
use std::env;
use std::fs::File;
use std::io::{Write, BufReader};

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

fn convolve(signal: &[f32], impulse: &[f32]) -> Vec<f32> {
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
