mod common;

use raudio_synth::gen::WaveformGenerator;

#[test]
fn test_waveform_generator() {
    let config = common::test_config();
    let sample_rate = config.sample_rate as f64;
    let frequency = 400;
    let duration_in_seconds = 2;
    let num_samples = (sample_rate * duration_in_seconds as f64) as usize;

    // Generate waveforms
    let mut sine_gen = raudio_synth::gen::sine_wave_generator(&config, frequency as f32);
    let mut square_gen = raudio_synth::gen::square_wave_generator(&config, frequency as f32);
    let mut sawtooth_gen = raudio_synth::gen::sawtooth_wave_generator(&config, frequency as f32);
    let mut triangle_gen = raudio_synth::gen::triangle_wave_generator(&config, frequency as f32);

    // Write each waveform to a WAV file
    write_waveform_to_wav(&mut sine_gen, num_samples, &common::test_audio_name(&config, "optimized_sine"));
    write_waveform_to_wav(&mut square_gen, num_samples, &common::test_audio_name(&config, "optimized_square"));
    write_waveform_to_wav(&mut sawtooth_gen, num_samples, &common::test_audio_name(&config, "optimized_sawtooth"));
    write_waveform_to_wav(&mut triangle_gen, num_samples, &common::test_audio_name(&config, "optimized_triangle"));
}


fn write_waveform_to_wav(generator: &mut WaveformGenerator, num_samples: usize, file_name: &str) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(file_name, spec).unwrap();

    for _ in 0..num_samples {
        let sample = generator.next_sample();
        writer.write_sample(sample as f32).unwrap();
    }
    writer.finalize().unwrap();
    println!("Completed writing waveform to {}", file_name);
}