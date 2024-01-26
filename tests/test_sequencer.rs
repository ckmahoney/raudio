mod common;

use rand::seq::SliceRandom;
use rand::thread_rng;
use raudio_synth::synth_config::SynthConfig;

#[test]
fn test_write_sequenced_melody() {
    let config = &common::test_config();
    let melody = [400.0, 600.0, 500.0, 700.0, 800.0, 600.0, 500.0, 400.0];
    let waveform_functions = vec![
        raudio_synth::freq_forms::sawtooth,
        raudio_synth::freq_forms::triangle,
        raudio_synth::freq_forms::sine,
    ];

    let mut rng = thread_rng();
    let mut complete_sequence: Vec<f32> = Vec::new();

    for (index, &frequency) in melody.iter().enumerate() {
        let ugen = waveform_functions.choose(&mut rng).unwrap();
        let note_duration = index + 1;
        let mut sequence = Vec::with_capacity(note_duration as usize);
        let num_samples = (config.sample_rate as f32 * note_duration as f32 / config.cps).floor() as i32;
        for i in 0..num_samples {
            let sample = ugen(config, i as u32, frequency, None);
            sequence.push(sample);
        }

        complete_sequence.extend(sequence);
    }

    let label = "melody-test";
    write_sequence_to_file(&config, &complete_sequence, label);
}


fn write_sequence_to_file(config: &SynthConfig, sequence: &[f32], label: &str) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: config.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Int,
    };
    let filename = common::test_audio_name(&config, label);
    let mut writer = hound::WavWriter::create(filename.clone(), spec).unwrap();

    for &sample in sequence {
        writer.write_sample(sample).unwrap(); 
    }
    writer.finalize().unwrap();
    println!("Completed writing test waveform {}", filename);
}