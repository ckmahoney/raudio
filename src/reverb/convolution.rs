use crate::synth::{SR,SampleBuffer, pi};
use crate::phrasing::contour::gen_contour;
use crate::types::timbre::AmpContour;
use rand::{Rng,thread_rng};

/// Produce an exponentially decaying noise sample 
fn noise_buffer(amp:f32, n_samples:usize) -> SampleBuffer {
    let mut rng = thread_rng();
    let contour = gen_contour(n_samples, 1f32, &AmpContour::Surge, true);
    (0..n_samples).map(|i| amp * contour[i] * (2f32 * rng.gen::<f32>() - 1f32)).collect()
}

fn apply1(sig: &SampleBuffer, reverb_len: usize) -> SampleBuffer {
    let impulse_response = noise_buffer(0.2f32, reverb_len);
    let mut wet = vec![0f32; sig.len() + impulse_response.len() - 1];

    for n in 0..wet.len() {
        for k in 0..sig.len() {
            if n >= k && (n - k) < impulse_response.len() {
                wet[n] += sig[k] * impulse_response[n - k];
            }
        }
    }

    wet
}
use rustfft::{FftPlanner, num_complex::Complex};
fn apply(sig: &SampleBuffer, reverb_len: usize) -> SampleBuffer {
    let mut impulse_response = generate_pink_noise(0.2f32, reverb_len);
// normalize(&mut impulse_response);  
    let n = sig.len() + impulse_response.len() - 1;
    
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    let ifft = planner.plan_fft_inverse(n);

    let mut sig_padded: Vec<Complex<f32>> = sig.iter().cloned().map(|s| Complex::new(s, 0.0)).collect();
    sig_padded.resize(n, Complex::new(0.0, 0.0));

    let mut ir_padded: Vec<Complex<f32>> = impulse_response.iter().cloned().map(|s| Complex::new(s, 0.0)).collect();
    ir_padded.resize(n, Complex::new(0.0, 0.0));

    fft.process(&mut sig_padded);
    fft.process(&mut ir_padded);

    let mut result = vec![Complex::new(0.0, 0.0); n];
    for i in 0..n {
        result[i] = sig_padded[i] * ir_padded[i];
    }

    ifft.process(&mut result);

    result.iter().map(|c| c.re / n as f32).collect()
}
fn generate_violet_noise(amp:f32, length: usize) -> SampleBuffer {
    let white_noise = generate_white_noise(length);
    let mut violet_noise = vec![0.0; length];

    for i in 1..length {
        violet_noise[i] = amp*(white_noise[i] - white_noise[i - 1]);
    }

    violet_noise
}

fn generate_white_noise(length: usize) -> SampleBuffer {
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen_range(-1.0..1.0)).collect()
}
fn generate_pink_noise(amp: f32, length: usize) -> SampleBuffer {
    let mut rng = rand::thread_rng();
    let num_rows = 16;
    let mut rows = vec![0.0; num_rows];
    let mut pink_noise = vec![0.0; length];

    for i in 0..length {
        let row = rng.gen_range(0..num_rows);
        rows[row] = rng.gen_range(-1.0..1.0);
        
        let running_sum: f32 = rows.iter().sum();
        pink_noise[i] = amp * running_sum / num_rows as f32;
    }

    pink_noise
}
fn pad_buffers(signal: &SampleBuffer, impulse_response: &SampleBuffer) -> (SampleBuffer, SampleBuffer) {
    let mut padded_signal = signal.clone();
    let mut padded_ir = impulse_response.clone();

    if impulse_response.len() < signal.len() {
        padded_ir.append(&mut vec![0f32; signal.len() - impulse_response.len()]);
    } else if signal.len() < impulse_response.len() {
        padded_signal.append(&mut vec![0f32; impulse_response.len() - signal.len()]);
    }

    (padded_signal, padded_ir)
}
fn normalize(buffer: &mut [f32]) {
    let max_val = buffer.iter().cloned().fold(f32::NEG_INFINITY, f32::max).abs();
    if max_val > 0.0 {
        for sample in buffer.iter_mut() {
            *sample /= max_val;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::{self, with_dir};
    use crate::render::engrave;
    static TEST_GROUP:&str = "reverb-convolution";
    fn out_dir() -> String { format!("dev-audio/{}", TEST_GROUP) }
    fn setup() {
        files::with_dir(&out_dir())
    }   

    fn generate_major_chord(frequencies: &[f32], sample_rate: usize, duration: f32) -> SampleBuffer {
        let n_samples = (sample_rate as f32 * duration) as usize;
        let mut buffer = vec![0f32; n_samples];
        let n = n_samples as f32;
        for &freq in frequencies {
            for (i, sample) in buffer.iter_mut().enumerate() {
                let t = i as f32 / n;
                let envelope = 0.2 * (-5.0 * t).exp().max(0.0); 
                *sample += (2.0f32 * pi * freq * t).sin() * envelope;
            }
        }
    
        buffer
    }

    fn gen_signal() -> SampleBuffer {
        let duration = 1.0; 
        let frequencies = [400.0, 500.0, 600.0];
        let samples = generate_major_chord(&frequencies, SR, duration);
        let filename = format!("{}/dry.wav", &out_dir());

        engrave::samples(SR, &samples, &filename);
        samples

    }

    #[test]
    fn test_apply_convolution_reverb_spring() {
        setup();
        let test_name = "springverb";
        let reverb_len = 1000; 

        let major_chord = gen_signal();
        let wet_signal = apply(&major_chord, reverb_len);
        assert_eq!(wet_signal.len(), major_chord.len() + reverb_len - 1);
        let non_zero_samples: Vec<&f32> = wet_signal.iter().filter(|&&x| x != 0.0).collect();
        assert!(!non_zero_samples.is_empty(), "Wet signal should not be all zeros");

        let filename2 = format!("{}/wet-{}.wav", &out_dir(), test_name);
        engrave::samples(SR, &wet_signal, &filename2);
    }

    #[test]
    fn test_apply_convolution_reverb_samelen() {
        setup();
        let test_name = "sameverb";

        let major_chord = gen_signal();
        let reverb_len = major_chord.len(); 
        let wet_signal = apply(&major_chord, reverb_len);
        assert_eq!(wet_signal.len(), major_chord.len() + reverb_len - 1);
        let non_zero_samples: Vec<&f32> = wet_signal.iter().filter(|&&x| x != 0.0).collect();
        assert!(!non_zero_samples.is_empty(), "Wet signal should not be all zeros");

        let filename2 = format!("{}/wet-{}.wav", &out_dir(), test_name);
        engrave::samples(SR, &wet_signal, &filename2);
    }
    #[test]
    fn test_apply_convolution_reverb_SRlen() {
        setup();
        let test_name = "longverb";

        let major_chord = gen_signal();
        let reverb_len = SR*8;
        let wet_signal = apply(&major_chord, reverb_len);
        assert_eq!(wet_signal.len(), major_chord.len() + reverb_len - 1);
        let non_zero_samples: Vec<&f32> = wet_signal.iter().filter(|&&x| x != 0.0).collect();
        assert!(!non_zero_samples.is_empty(), "Wet signal should not be all zeros");

        let filename2 = format!("{}/wet-{}.wav", &out_dir(), test_name);
        engrave::samples(SR, &wet_signal, &filename2);
    }
}