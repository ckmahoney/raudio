pub fn samples(pathname: &str) -> Vec<f32> {
    let mut reader = hound::WavReader::open(pathname).unwrap();
    let spec = reader.spec();
    if spec.sample_format != hound::SampleFormat::Float {
        panic!("Unsupported sample format. Only 32-bit float WAV files are supported.");
    }
    if spec.bits_per_sample != 32 {
        panic!("Unsupported bit depth. Only 32-bit float WAV files are supported.");
    }

    reader.samples::<f32>().map(|s| s.unwrap()).collect()
}