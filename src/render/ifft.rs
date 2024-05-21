extern crate rustfft;
use rustfft::{FftPlanner, FftDirection};
use rustfft::num_complex::Complex;
use std::f32::consts::PI;

use crate::files;
use crate::render::engrave;

static TEST_DIR: &str = "dev-audio/ifft";

#[test]
fn main() {
    // Define the sample rate and the number of samples
    let sample_rate = 44100.0;
    let num_samples = 44100;

    // Define the frequencies and their corresponding amplitudes and phases
    let mut frequencies = [
        (440.0, 0.5, PI),    // 440 Hz, amplitude 0.5, phase PI
        (660.0, 1.0, PI / 4.0), // 660 Hz, amplitude 1.0, phase PI/4
        (880.0, 0.5, 0.0),   // 880 Hz, amplitude 0.5, phase 0
    ];

    // Adjust the amplitudes if necessary
    adjust_amplitudes(&mut frequencies);

    // Create the frequency bins
    let mut spectrum = create_spectrum(&frequencies, num_samples, sample_rate);

    // Perform the IFFT
    let signal = perform_ifft(&mut spectrum, num_samples);

    // Print the first few samples of the resulting signal
    for sample in signal.iter().take(10) {
        println!("{}", sample);
    }

    // Calculate the max value of the signal
    let max_value = signal.iter().fold(f32::MIN, |a, &b| a.max(b));
    println!("{} elements and max value {}", signal.len(), max_value);

    // Save the output to a file
    files::with_dir(TEST_DIR);
    let test_name = "poc";
    let filename = format!("{}/{}.wav", TEST_DIR, test_name);
    engrave::samples(sample_rate as usize, &signal, &filename);
}

fn adjust_amplitudes(frequencies: &mut [(f32, f32, f32)]) {
    let amplitude_sum: f32 = frequencies.iter().map(|&(_, amplitude, _)| amplitude).sum();
    if amplitude_sum > 1.0 {
        let scale_factor = 1.0 / amplitude_sum;
        for freq in frequencies.iter_mut() {
            freq.1 *= scale_factor;
        }
    }
}

fn create_spectrum(frequencies: &[(f32, f32, f32)], num_samples: usize, sample_rate: f32) -> Vec<Complex<f32>> {
    let mut spectrum = vec![Complex::new(0.0, 0.0); num_samples];
    for &(freq, amplitude, phase) in frequencies {
        let bin = (freq / sample_rate * num_samples as f32).round() as usize;
        if bin < num_samples {
            spectrum[bin] = Complex::new(amplitude * phase.cos(), amplitude * phase.sin());
            // Since FFT of a real signal is symmetric, we need to set the symmetric bin as well
            if bin != 0 {
                spectrum[num_samples - bin] = Complex::new(amplitude * phase.cos(), -amplitude * phase.sin());
            }
        }
    }
    spectrum
}

fn perform_ifft(spectrum: &mut [Complex<f32>], num_samples: usize) -> Vec<f32> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft(num_samples, FftDirection::Inverse);
    let mut spectrum_clone = spectrum.to_vec();
    fft.process(&mut spectrum_clone);

    // Normalize the resulting signal
    let signal: Vec<f32> = spectrum_clone.iter().map(|c| c.re).collect();

    // Find the maximum absolute value in the signal
    let max_abs_value = signal.iter().fold(0.0f32, |max, &val| max.max(val.abs()));

    // Normalize the signal to the range [-1, 1] if needed
    if max_abs_value > 1.0 {
        signal.iter().map(|&val| val / max_abs_value).collect()
    } else {
        signal
    }
}
