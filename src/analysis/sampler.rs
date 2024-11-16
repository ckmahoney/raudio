use hound;
use rubato::{FftFixedInOut, Resampler};
use std::sync::Arc;

/// Reads an audio file and returns the audio buffer and sample rate.
pub fn read_audio_file(path: &str) -> Result<(Vec<f32>, u32), Box<dyn std::error::Error>> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();

    let samples: Vec<f32> = match spec.bits_per_sample {
        16 => reader
            .samples::<i16>()
            .map(|s| s.unwrap() as f32 / i16::MAX as f32)
            .collect(),
        24 => read_24bit_samples(reader.into_inner())?,
        32 => reader
            .samples::<i32>()
            .map(|s| s.unwrap() as f32 / i32::MAX as f32)
            .collect(),
        _ => return Err(format!("Unsupported bit depth: {}", spec.bits_per_sample).into()),
    };

    Ok((samples, spec.sample_rate))
}

/// Reads 24-bit samples from a WAV file.
fn read_24bit_samples<R: std::io::Read + std::io::Seek>(
    mut raw_reader: R,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let mut buffer = vec![];
    raw_reader.read_to_end(&mut buffer)?;

    let samples: Vec<f32> = buffer
        .chunks_exact(3)
        .map(|chunk| {
            let value = ((chunk[2] as i32) << 16) | ((chunk[1] as i32) << 8) | (chunk[0] as i32);
            let value = if value & 0x800000 != 0 {
                value | !0xFFFFFF // Sign-extend if negative
            } else {
                value
            };
            value as f32 / 8388607.0 // Normalize 24-bit to [-1, 1]
        })
        .collect();

    Ok(samples)
}

/// Resamples audio from one sample rate to another.
pub fn resample_audio(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    let mut resampler = FftFixedInOut::<f32>::new(
        from_rate as usize,
        to_rate as usize,
        128, // Chunk size
        2,   // Overlap-add buffer chunks
    )
    .expect("Failed to create resampler");

    let input: Vec<Arc<[f32]>> = vec![Arc::from(samples.to_vec().into_boxed_slice())];
    let output = resampler.process(&input, None).expect("Resampling failed");

    output.into_iter().flat_map(|chunk| chunk.to_vec()).collect()
}

/// Sequences and combines multiple audio buffers.
pub fn sequence_samples(buffers: Vec<Vec<f32>>) -> Vec<f32> {
    buffers.into_iter().flatten().collect()
}

/// Writes an audio buffer to a WAV file.
pub fn write_audio_file(path: &str, buffer: &[f32], sample_rate: u32) -> Result<(), Box<dyn std::error::Error>> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)?;
    for sample in buffer {
        let pcm_sample = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        writer.write_sample(pcm_sample)?;
    }
    Ok(())
}

/// Parses a pattern string (e.g., "kick1 kick2 snare1") into a sequence of (category, index) pairs.
pub fn parse_pattern(pattern: &str) -> Vec<(String, usize)> {
    pattern
        .split_whitespace()
        .map(|s| {
            let (cat, num) = s.split_at(s.len() - 1);
            (cat.to_string(), num.parse().unwrap())
        })
        .collect()
}
