use hound::{WavSpec, WavWriter, SampleFormat};
use rand::random;
use std::f32::consts::PI;
use std::collections::HashMap;


mod test {
    use super::*;
    #[test]
    fn testwhite_noise() {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create("white_noise.wav", spec).unwrap();
        let duration = 10; // Duration of noise in seconds
        let sample_count = spec.sample_rate * duration;

        for _ in 0..sample_count {
            writer.write_sample(generate_white_noise()).unwrap();
        }
        writer.finalize().unwrap();
    }

    #[test]
    fn test_perceptual_noise() {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 44100,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };

        let mut writer = WavWriter::create("perceptual_noise.wav", spec).unwrap();
        let duration = 10; // Duration of noise in seconds
        let sample_count = spec.sample_rate * duration;

        for _ in 0..sample_count {
            let sample = generate_white_noise();
            let filtered_sample = apply_fletcher_munson_filter(sample);
            writer.write_sample(filtered_sample).unwrap();
        }
        writer.finalize().unwrap();
    }
}

/// Generates a simple white noise sample
fn generate_white_noise() -> f32 {
    random::<f32>() * 2.0 - 1.0f32
}

/// Applies a simple Fletcher-Munson curve filter.
/// This is a placeholder; you would need to implement the actual curve.
fn apply_fletcher_munson_filter(sample: f32) -> i16 {
    let adjusted_sample = sample * 0.5; // Reduce amplitude by half

    (adjusted_sample * i16::MAX as f32) as i16
}


fn main() {
    let spec = WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create("fletcher_munsen_noise.wav", spec).unwrap();
    let duration = 10; // Duration of noise in seconds
    let sample_count = spec.sample_rate * duration;

    // Fletcher-Munson EQ settings
    let eq = fletcher_munsen_eq();

    for i in 0..sample_count {
        let sample = generate_white_noise();
        let filtered_sample = apply_fletcher_munsen_filter(sample, &eq, i, spec.sample_rate);
        writer.write_sample(filtered_sample).unwrap();
    }
    writer.finalize().unwrap();
}


/// Applies the Fletcher-Munson curve EQ based on frequency register
fn apply_fletcher_munsen_filter(sample: f32, eq: &HashMap<u32, f32>, i: u32, sample_rate: u32) -> i16 {
    let frequency_bin = (i % sample_rate) / 1000; // Simplified frequency bin calculation
    let attenuation = *eq.get(&frequency_bin).unwrap_or(&1.0); // Default to no attenuation if not specified

    // Apply attenuation and convert to 16-bit PCM
    (sample * attenuation * i16::MAX as f32) as i16
}

/// Defines the Fletcher-Munson EQ settings
fn fletcher_munsen_eq() -> HashMap<u32, f32> {
    let mut eq = HashMap::new();
    eq.insert(10, db_to_amp(-10.0));
    eq.insert(11, db_to_amp(-5.0));
    eq.insert(12, db_to_amp(-5.0));
    eq.insert(13, db_to_amp(-5.0));
    eq.insert(14, db_to_amp(-5.0));
    eq
}

/// Converts decibels to amplitude
fn db_to_amp(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}
