use hound::{self, WavReader, WavSpec, WavWriter};
use rubato::{FftFixedInOut, Resampler};
use std::collections::HashMap;
use std::sync::Arc;

pub enum AudioFormat {
  Mono(Vec<f32>),                 // Single-channel audio
  Stereo(Vec<f32>, Vec<f32>),     // Separate left and right channels
  Interleaved(Vec<f32>),          // Interleaved stereo samples
}

/// Reads metadata from a WAV file.
///
/// # Parameters
/// - `path`: Path to the WAV file.
///
/// # Returns
/// A `HashMap<String, String>` containing key-value pairs of metadata.
///
/// # Errors
/// Returns an error if the file cannot be read or does not contain metadata.
pub fn read_metadata(path: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
  let reader = WavReader::open(path)?;
  let spec = reader.spec();

  let mut metadata = HashMap::new();
  metadata.insert("Channels".to_string(), spec.channels.to_string());
  metadata.insert("Sample Rate".to_string(), spec.sample_rate.to_string());
  metadata.insert("Bits Per Sample".to_string(), spec.bits_per_sample.to_string());
  metadata.insert("Sample Format".to_string(), format!("{:?}", spec.sample_format));

  // Add more metadata if available in the WAV header.
  Ok(metadata)
}
/// Writes metadata to a WAV file.
///
/// # Parameters
/// - `path`: Path to the WAV file where metadata will be written.
/// - `metadata`: A `HashMap<String, String>` containing key-value pairs of metadata.
///
/// # Returns
/// A result indicating success or failure.
///
/// # Errors
/// Returns an error if the file cannot be opened or metadata cannot be written.
pub fn write_metadata(path: &str, metadata: &HashMap<String, String>) -> Result<(), Box<dyn std::error::Error>> {
  // Read the existing WAV file to copy its audio data.
  let mut reader = WavReader::open(path)?;
  let spec = reader.spec();
  let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();

  // Create a new WAV file to write metadata and audio data.
  let new_path = format!("{}_with_metadata.wav", path);
  let mut writer = WavWriter::create(&new_path, spec)?;

  // Write audio data.
  for sample in samples {
    writer.write_sample(sample)?;
  }

  // Note: The Hound crate does not natively support writing custom metadata (e.g., INFO chunks).
  // You may need a different library or custom logic for full metadata support.
  // For now, we only demonstrate preserving basic WAV properties.

  writer.finalize()?;
  Ok(())
}

/// Extracts metadata from a WAV file.
pub fn get_audio_metadata(path: &str) -> Result<hound::WavSpec, Box<dyn std::error::Error>> {
  let reader = hound::WavReader::open(path)?;
  Ok(reader.spec())
}

/// Reads an audio file and returns the audio buffer and sample rate.
///
/// # Parameters
/// - `path`: Path to the WAV file.
///
/// # Returns
/// A tuple containing:
/// - `Vec<Vec<f32>>`: A vector of per-channel samples, where each channel is a vector of normalized samples.
/// - `u32`: Sample rate of the WAV file.
///
/// # Errors
/// Returns an error if the file cannot be opened or has an unsupported bit depth or format.
pub fn read_audio_file(path: &str) -> Result<(Vec<Vec<f32>>, u32), Box<dyn std::error::Error>> {
  let mut reader = hound::WavReader::open(path)?;
  let spec = reader.spec();

  let num_channels = spec.channels as usize;
  let mut channel_samples: Vec<Vec<f32>> = vec![vec![]; num_channels];

  match spec.bits_per_sample {
    8 => {
      for (i, sample) in reader.samples::<i8>().enumerate() {
        let sample = (sample.unwrap() as f32 - 128.0) / 128.0; // Normalize [-1, 1]
        channel_samples[i % num_channels].push(sample);
      }
    }
    16 => {
      for (i, sample) in reader.samples::<i16>().enumerate() {
        let sample = sample.unwrap() as f32 / i16::MAX as f32;
        channel_samples[i % num_channels].push(sample);
      }
    }
    24 => {
      let raw_samples = read_24bit_samples(reader.into_inner())?;
      for (i, sample) in raw_samples.iter().enumerate() {
        channel_samples[i % num_channels].push(*sample);
      }
    }
    32 => {
      for (i, sample) in reader.samples::<i32>().enumerate() {
        let sample = sample.unwrap() as f32 / i32::MAX as f32;
        channel_samples[i % num_channels].push(sample);
      }
    }
    _ => return Err(format!("Unsupported bit depth: {}", spec.bits_per_sample).into()),
  };

  let channel_samples_override: Vec<Vec<f32>> =
    channel_samples.iter().map(|ys| (*ys).iter().map(|x| *x * 0.33f32).collect()).collect();

  Ok((channel_samples_override, spec.sample_rate))
}
/// Reads 24-bit samples from a WAV file.
fn read_24bit_samples<R: std::io::Read + std::io::Seek>(
  mut raw_reader: R,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
  let mut buffer = vec![];
  raw_reader.read_to_end(&mut buffer)?;

  const MAX_24BIT: f32 = (1 << 23) as f32; // 2^(24-1), maximum positive value for 24-bit signed integer
  let samples: Vec<f32> = buffer
    .chunks_exact(3)
    .map(|chunk| {
      let value = ((chunk[2] as i32) << 16) | ((chunk[1] as i32) << 8) | (chunk[0] as i32);
      let value = if value & 0x800000 != 0 {
        value | !0xFFFFFF // Sign-extend if negative
      } else {
        value
      };
      value as f32 / MAX_24BIT // Normalize 24-bit to [-1, 1]
    })
    .collect();

  Ok(samples)
}

/// Resource allowance profiles for audio processing.
pub enum ResourceAllowance {
  Low,    // Low-resource environment (e.g., edge devices, low-powered VMs)
  Medium, // Balanced environment (e.g., general-purpose servers)
  High,   // High-resource environment (e.g., dedicated server, powerful workstation)
}

impl ResourceAllowance {
  /// Get FFT resampling parameters (chunk size and overlap) based on the resource allowance.
  fn fft_params(&self) -> (usize, usize) {
    match self {
      ResourceAllowance::Low => (64, 1), // Minimal latency, suitable for low-resource systems
      ResourceAllowance::Medium => (128, 2), // Balanced trade-off
      ResourceAllowance::High => (512, 4), // High-quality processing
    }
  }

  /// Default resource allowance (High).
  fn default() -> Self {
    ResourceAllowance::High
  }
}

/// Resamples audio from one sample rate to another.
///
/// # Parameters
/// - `samples`: Input audio samples (non-interleaved).
/// - `from_rate`: Original sample rate.
/// - `to_rate`: Target sample rate.
/// - `resource_allowance`: Optional resource profile for FFT settings.
/// - `channels`: Number of audio channels.
///
/// # Returns
/// A vector of resampled channels as `Vec<Vec<f32>>`.
///
/// # Errors
/// Returns an error if resampling fails.

pub fn resample_audio(
  samples: &[f32], from_rate: u32, to_rate: u32, resource_allowance: Option<ResourceAllowance>, channels: usize,
) -> Vec<Vec<f32>> {
  let resource_profile = resource_allowance.unwrap_or_else(ResourceAllowance::default);
  let (chunk_size, overlap_chunks) = resource_profile.fft_params();

  let mut resampler = FftFixedInOut::<f32>::new(from_rate as usize, to_rate as usize, chunk_size, overlap_chunks)
    .expect("Failed to create resampler");

  // Process each channel separately
  let input: Vec<Arc<[f32]>> =
    samples.chunks_exact(channels).map(|chunk| Arc::from(chunk.to_vec().into_boxed_slice())).collect();
  let output = resampler.process(&input, None).expect("Resampling failed");

  output.into_iter().map(|chunk| chunk.to_vec()).collect()
}

/// Sequences and combines multiple audio buffers into a single buffer.
///
/// # Parameters
/// - `buffers`: A vector of vectors, where each inner vector represents an audio buffer.
///
/// # Returns
/// A single flattened vector of audio samples.
#[inline]
pub fn sequence_samples(buffers: Vec<Vec<f32>>) -> Vec<f32> {
  buffers.into_iter().flatten().collect()
}

/// Writes a multi-channel audio buffer to a WAV file.
///
/// # Parameters
/// - `path`: Path to the output WAV file.
/// - `buffers`: A slice of vectors representing audio samples for each channel.
/// - `sample_rate`: Sample rate to be written into the WAV file.
///
/// # Returns
/// A result indicating success or failure.
///
/// # Errors
/// Returns an error if the file cannot be created or samples cannot be written.
pub fn write_audio_file(path: &str, buffers: &[Vec<f32>], sample_rate: u32) -> Result<(), Box<dyn std::error::Error>> {
  let channels = buffers.len() as u16;
  let spec = hound::WavSpec {
    channels,
    sample_rate,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int,
  };

  let mut writer = hound::WavWriter::create(path, spec)?;
  let num_samples = buffers[0].len();
  for i in 0..num_samples {
    for buffer in buffers {
      let sample = buffer[i].clamp(-1.0, 1.0) * i16::MAX as f32;
      writer.write_sample(sample as i16)?;
    }
  }
  Ok(())
}
