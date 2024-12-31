pub fn samples(sample_rate: usize, samples: &Vec<f32>, filename: &str) {
  use std::path::Path;
  let p: &Path = Path::new(filename);
  let spec = hound::WavSpec {
    channels: 1,
    sample_rate: sample_rate as u32,
    bits_per_sample: 32,
    sample_format: hound::SampleFormat::Float,
  };
  let mut writer = hound::WavWriter::create(p, spec).unwrap();
  for &sample in samples {
    writer.write_sample(sample).unwrap();
  }
  writer.finalize().unwrap();
}

pub fn samples2(sample_rate: usize, samples: &Vec<f32>, filename: &str) {
  use std::path::Path;
  let p: &Path = Path::new(filename);
  let spec = hound::WavSpec {
    channels: 2, // Stereo
    sample_rate: sample_rate as u32,
    bits_per_sample: 32,
    sample_format: hound::SampleFormat::Float,
  };
  let mut writer = hound::WavWriter::create(p, spec).unwrap();
  for chunk in samples.chunks(2) {
    if chunk.len() == 2 {
      // Write left and right channel samples
      writer.write_sample(chunk[0]).unwrap(); // Left
      writer.write_sample(chunk[1]).unwrap(); // Right
    } else {
      // Handle the case where the input has an odd number of samples
      writer.write_sample(chunk[0]).unwrap(); // Write the remaining single sample as mono
      writer.write_sample(0.0).unwrap(); // Pad with silence for the other channel
    }
  }
  writer.finalize().unwrap();
}

pub fn write_audio(sample_rate: usize, signals: Vec<Vec<f32>>, filename: &str) {
  if signals.len() == 1 {
    // Mono signal, use samples
    samples(sample_rate, &signals[0], filename);
  } else if signals.len() == 2 {
    // Stereo signal, interleave and use samples2
    let left = &signals[0];
    let right = &signals[1];

    // Ensure both channels have the same length
    let min_len = left.len().min(right.len());
    let interleaved: Vec<f32> =
      left[..min_len].iter().zip(right[..min_len].iter()).flat_map(|(&l, &r)| vec![l, r]).collect();

    samples2(sample_rate, &interleaved, filename);
  } else {
    panic!("Only mono and stereo signals are supported!");
  }
}
