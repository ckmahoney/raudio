extern crate rustfft;
extern crate rand;

use crate::synth::SampleBuffer;
use rustfft::{FftPlanner, FftDirection};
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use std::f32::consts::PI;
use rand::Rng;
use crate::time;
use crate::files;
use crate::render::engrave;

static TEST_DIR: &str = "dev-audio/ifft";

/// Adjusts the amplitudes of the frequency components to ensure their sum does not exceed 1.
fn adjust_amplitudes(frequencies: &mut Vec<(f32, f32, f32)>) {
    let amplitude_sum: f32 = frequencies.iter().map(|&(_, amplitude, _)| amplitude).sum();
    if amplitude_sum > 1.0 {
        let scale_factor = 1.0 / amplitude_sum;
        for freq in frequencies.iter_mut() {
            freq.1 *= scale_factor;
        }
    }
}

/// Creates a frequency spectrum based on the provided frequencies, amplitudes, and phases.
fn create_spectrum(frequencies: &[(f32, f32, f32)], num_samples: usize, sample_rate: f32) -> Vec<Complex<f32>> {
    let mut spectrum = vec![Complex::new(0.0, 0.0); num_samples];
    for &(freq, amplitude, phase) in frequencies {
        let bin = (freq / sample_rate * num_samples as f32).round() as usize;
        if bin < num_samples {
            spectrum[bin] = Complex::new(amplitude * phase.cos(), amplitude * phase.sin());
            if bin != 0 {
                spectrum[num_samples - bin] = Complex::new(amplitude * phase.cos(), -amplitude * phase.sin());
            }
        }
    }
    spectrum
}

/// Performs the inverse FFT on the provided spectrum and normalizes the resulting signal.
fn do_ifft(spectrum: &mut [Complex<f32>], num_samples: usize) -> Vec<f32> {
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft(num_samples, FftDirection::Inverse);
    let mut scratch = vec![Complex::zero(); fft.get_inplace_scratch_len()];
    fft.process_with_scratch(spectrum, &mut scratch);

    let signal: Vec<f32> = spectrum.iter().map(|c| c.re).collect();
    let max_abs_value = signal.iter().fold(0.0f32, |max, &val| max.max(val.abs()));

    if max_abs_value > 1.0 {
        signal.iter().map(|&val| val / max_abs_value).collect()
    } else {
        signal
    }
}


/// Performs the inverse FFT on the provided spectrum and normalizes the resulting signal.
pub fn ifft(sinus: &mut [(f32, f32, f32)], sample_rate:usize, n_samples: usize) -> SampleBuffer {
    let mut spectrum = create_spectrum(sinus, n_samples, sample_rate as f32);
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft(n_samples, FftDirection::Inverse);
    let mut scratch = vec![Complex::zero(); fft.get_inplace_scratch_len()];
    fft.process_with_scratch(&mut spectrum, &mut scratch);

    let signal: Vec<f32> = spectrum.iter().map(|c| c.re).collect();
    let max_abs_value = signal.iter().fold(0.0f32, |max, &val| max.max(val.abs()));

    if max_abs_value > 1.0 {
        signal.iter().map(|&val| val / max_abs_value).collect()
    } else {
        signal
    }
}

/// Manually sums sines based on the provided frequencies, amplitudes, and phases.
fn manual_sum_of_sines(frequencies: &[(f32, f32, f32)], num_samples: usize, sample_rate: f32) -> Vec<f32> {
    let mut signal = vec![0.0; num_samples];
    let dt = 1.0 / sample_rate;

    for i in 0..num_samples {
        let t = i as f32 * dt;
        for &(freq, amplitude, phase) in frequencies {
            signal[i] += amplitude * (2.0 * PI * freq * t + phase).cos();
        }
    }

    let max_abs_value = signal.iter().fold(0.0f32, |max, &val| max.max(val.abs()));
    if max_abs_value > 1.0 {
        signal.iter().map(|&val| val / max_abs_value).collect()
    } else {
        signal
    }
}

/// Generates random test frequencies, amplitudes, and phases.
fn gen_test_freqs(n: usize) -> Vec<(f32, f32, f32)> {
    let mut rng = rand::thread_rng();
    let mut frequencies = Vec::with_capacity(n);
    for _ in 0..n {
        let freq = rng.gen_range(20.0..20000.0);
        let amplitude = rng.gen_range(0.0..1.0);
        let phase = rng.gen_range(0.0..(2.0 * PI));
        frequencies.push((freq, amplitude, phase));
    }
    frequencies
}

use crate::synth::SR;

#[cfg(test)]
mod test {
    use super::*;
    use crate::files;

    #[test]
    fn test_applied() {
        let mut freqs:Vec<(f32,f32,f32)> = vec![
            (400f32, 1f32, 0f32),  
            (500f32, 1f32, 0f32),  
            (600f32, 1f32, 0f32),  
        ];
        files::with_dir(TEST_DIR);
        let filename = format!("{}/major-chord.wav", TEST_DIR);

        let signal = ifft(&mut freqs, SR, SR * 4);
        engrave::samples(SR as usize, &signal, &filename);

        let mut freqs:Vec<(f32,f32,f32)> = vec![
            (600f32, 1f32, 0f32),  
            (400f32, 1f32, 0f32),  
            (240f32, 1f32, 0f32),  
        ];
        let filename = format!("{}/minor-chord.wav", TEST_DIR);

        let signal = ifft(&mut freqs, SR, SR * 4);
        engrave::samples(SR as usize, &signal, &filename);

        let mut freqs:Vec<(f32,f32,f32)> = vec![
            (400f32, 1f32, 0f32),  
            (500f32, 1f32, 0f32),  
            (600f32, 1f32, 0f32),  
            (400f32, 1f32, 0f32),  
            (240f32, 1f32, 0f32),  
        ];
        let filename = format!("{}/monic-chord.wav", TEST_DIR);

        let signal = ifft(&mut freqs, SR, SR * 4);
        engrave::samples(SR as usize, &signal, &filename);
    }

    #[test]
    fn test_benchmark() {
        let sample_rate = 44100.0;
        let num_samples = 44100;
    
        let mut frequencies = gen_test_freqs(1000);
        adjust_amplitudes(&mut frequencies);
    
        let mut spectrum = create_spectrum(&mut frequencies.clone(), num_samples, sample_rate);
        let filename = format!("{}/benchmark-{}.wav", TEST_DIR, spectrum.len());

        let (signal, ifft_duration) = time::measure(|| {
            do_ifft(&mut spectrum.clone(), num_samples)
        });
        engrave::samples(sample_rate as usize, &signal, &filename);

        let (_, sum_duration) = time::measure(|| {
            manual_sum_of_sines(&mut frequencies, num_samples, sample_rate)
        });

        println!("Manual sum of sines duration: {:?}", sum_duration);
    }

    
    
}
