extern crate hound;
extern crate rubberband;

use rubberband::{Rubberband, Settings};
use std::f64::consts::PI;

fn sine_wave(frequency: f64, duration_secs: u32, sample_rate: u32) -> Vec<f32> {
    (0..(sample_rate * duration_secs))
        .map(|x| ((2.0 * PI * frequency * x as f64 / sample_rate as f64).sin() * 0.5) as f32)
        .collect()
}

fn pitch_shift(input: &[f32], sample_rate: u32, shift_ratio: f64) -> Vec<f32> {
    let mut rubberband = Rubberband::new(sample_rate as u32, 1, Settings::default());

    rubberband.set_pitch_scale(shift_ratio);
    rubberband.process(input, false);
    rubberband.available() as usize;

    let mut output = vec![0f32; rubberband.available()];
    rubberband.retrieve(&mut output);
    output
}

#[test]
fn test_m() {
    let sample_rate = 44100;
    let duration_secs = 5;
    let frequency = 440.0;
    let shift_ratio = 1.5;

    let wave = sine_wave(frequency, duration_secs, sample_rate);
    let shifted_wave = pitch_shift(&wave, sample_rate, shift_ratio);

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter::create("pitch_shifted_wave.wav", spec).unwrap();
    for sample in shifted_wave {
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();
}
