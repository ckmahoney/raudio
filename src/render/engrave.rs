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
