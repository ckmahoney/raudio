//! Optimized generation of popular waveforms using vector rotation
//! important, this does not work as intented. The contents here are the output of ChatGPT trying to do it.
//! I appreciate the attempt.
extern crate num_complex;
use num_complex::Complex;
use crate::synth_config::SynthConfig;

struct Harmonic {
    amplitude: f64,
    phase_offset: f64,
    envelope: Box<dyn Fn(f64) -> f64>, // Dynamic envelope function
}

impl Harmonic {
    fn new(amplitude: f64, phase_offset: f64, envelope: Box<dyn Fn(f64) -> f64>) -> Self {
        Harmonic { amplitude, phase_offset, envelope }
    }
}

pub struct WaveformGenerator {
    sample_rate: f64,
    frequency: f64,
    harmonics: Vec<Harmonic>,
    multipliers: Vec<Complex<f64>>,
    values: Vec<Complex<f64>>,
    time: f64,
}

impl WaveformGenerator {
    fn new(sample_rate: f64, frequency: f64, num_harmonics: usize) -> Self {
        let mut harmonics = Vec::with_capacity(num_harmonics);
        let mut multipliers = Vec::with_capacity(num_harmonics);
        let mut values = Vec::with_capacity(num_harmonics);

        for _ in 0..num_harmonics {
            harmonics.push(Harmonic::new(0.0, 0.0, Box::new(|_| 1.0)));
            multipliers.push(Complex::new(0.0, 0.0));
            values.push(Complex::new(1.0, 0.0));
        }

        let mut generator = WaveformGenerator {
            sample_rate,
            frequency,
            harmonics,
            multipliers,
            values,
            time: 0.0,
        };
        generator.update_frequency(frequency);
        generator
    }

    fn update_frequency(&mut self, frequency: f64) {
        self.frequency = frequency;
        for i in 0..self.harmonics.len() {
            let harmonic_freq = frequency * (i as f64 + 1.0);
            let phase_increment = 2.0 * std::f64::consts::PI * harmonic_freq / self.sample_rate;
            self.multipliers[i] = Complex::new(phase_increment.cos(), phase_increment.sin());
        }
    }

    fn set_harmonic(&mut self, index: usize, amplitude: f64, phase_offset: f64, envelope: Box<dyn Fn(f64) -> f64>) {
        if index < self.harmonics.len() {
            self.harmonics[index] = Harmonic::new(amplitude, phase_offset, envelope);
        }
    }

    pub fn next_sample(&mut self) -> f64 {
        let mut sample = 0.0;
        for i in 0..self.harmonics.len() {
            let envelope_value = (self.harmonics[i].envelope)(self.time);
            self.values[i] *= self.multipliers[i] * Complex::new(envelope_value, 0.0);
            sample += self.values[i].im * self.harmonics[i].amplitude;
        }
        self.time += 1.0 / self.sample_rate;
        sample
    }
}
pub fn sine_wave_generator(config: &SynthConfig, freq: f32) -> WaveformGenerator {
    let num_harmonics = 1; // Only the fundamental frequency
    let mut generator = WaveformGenerator::new(config.sample_rate as f64, freq as f64, num_harmonics);
    generator.set_harmonic(0, 1.0, 0.0, Box::new(|_| 1.0)); // Fundamental frequency with amplitude 1
    generator
}

pub fn square_wave_generator(config: &SynthConfig, freq: f32) -> WaveformGenerator {
    let nyquist = config.sample_rate as f32 / 2.0;
    let max_harmonic = (nyquist / freq).floor() as usize;
    let num_harmonics = max_harmonic / 2; // Considering only odd harmonics
    let mut generator = WaveformGenerator::new(config.sample_rate as f64, freq as f64, num_harmonics);

    for n in (1..=max_harmonic).step_by(2) {
        let amplitude = 1.0 / n as f64;
        generator.set_harmonic(n / 2, amplitude, 0.0, Box::new(move |_| amplitude));
    }

    generator
}

pub fn sawtooth_wave_generator(config: &SynthConfig, freq: f32) -> WaveformGenerator {
    let nyquist = config.sample_rate as f32 / 2.0;
    let max_harmonic = (nyquist / freq).floor() as usize;
    let mut generator = WaveformGenerator::new(config.sample_rate as f64, freq as f64, max_harmonic);

    for n in 1..=max_harmonic {
        let amplitude = 1.0 / n as f64;
        generator.set_harmonic(n - 1, amplitude, 0.0, Box::new(move |_| amplitude));
    }

    generator
}

pub fn triangle_wave_generator(config: &SynthConfig, freq: f32) -> WaveformGenerator {
    let nyquist = config.sample_rate as f32 / 2.0;
    let max_harmonic = (nyquist / freq).floor() as usize;
    let num_harmonics = max_harmonic / 2;
    let mut generator = WaveformGenerator::new(config.sample_rate as f64, freq as f64, num_harmonics);

    for n in (1..=max_harmonic).step_by(2) {
        let amplitude = 1.0 / (n as f64).powi(2);
        generator.set_harmonic(n / 2, amplitude, 0.0, Box::new(move |_| amplitude));
    }

    generator
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveform_generator() {
        let sample_rate = 44100.0;
        let frequency = 440.0;
        let num_harmonics = 64;
        let mut generator = WaveformGenerator::new(sample_rate, frequency, num_harmonics);

        for i in 0..num_harmonics {
            let amplitude = 1.0 / (i as f64 + 1.0);
            let phase_offset = 0.0;
            let envelope = Box::new(move |_| amplitude);
            generator.set_harmonic(i, amplitude, phase_offset, envelope);
        }

        for _ in 0..sample_rate as i32 {
            let sample = generator.next_sample();
            assert!(sample >= -1.0 && sample <= 1.0, "Sample out of range");
        }
    }
}