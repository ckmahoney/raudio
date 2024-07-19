use crate::synth::{SR,SampleBuffer, pi};
use crate::phrasing::contour::gen_contour;
use crate::types::timbre::AmpContour;
use rand::{Rng,thread_rng};

/// Produce an exponentially decaying noise sample 
fn noise_buffer(n_samples:usize) -> SampleBuffer {
    let mut rng = thread_rng();
    let contour = gen_contour(n_samples, 1f32, &AmpContour::Surge, true);
    (0..n_samples).map(|i| contour[i] * (2f32 * rng.gen::<f32>() - 1f32)).collect()
}

fn generate_major_chord(frequencies: &[f32], sample_rate: usize, duration: f32) -> SampleBuffer {
    let mut buffer = vec![0f32; (sample_rate as f32 * duration) as usize];

    for &freq in frequencies {
        for (i, sample) in buffer.iter_mut().enumerate() {
            let t = i as f32 / sample_rate as f32;
            let envelope = (1.0 - t / duration).max(0.0); // Simple linear decay
            *sample += (2.0 * pi * freq * t).sin() * envelope;
        }
    }

    buffer
}

fn main() {
    let sample_rate = 44100;
    let duration = 1.0; // 1 second
    let frequencies = [400.0, 500.0, 600.0];
    
    let major_chord = generate_major_chord(&frequencies, sample_rate, duration);

    // Example usage: Apply convolution reverb (apply function defined earlier)
    let reverb_len = 1000; // Example reverb length
    let wet_signal = apply(&major_chord, reverb_len);

    // For testing: Output the generated signals or visualize them as needed
    // ...
}

// Placeholder for the apply function (convolution reverb function)
fn apply(signal: &SampleBuffer, reverb_len: usize) -> SampleBuffer {
    let impulse_response = noise_buffer(reverb_len);
    let (sig, impulse_response) = pad_buffers(signal, &impulse_response);

    let mut wet = vec![0f32; sig.len() + impulse_response.len() - 1];

    for n in 0..wet.len() {
        for k in 0..sig.len() {
            if n >= k && n - k < impulse_response.len() {
                wet[n] += sig[k] * impulse_response[n - k];
            }
        }
    }

    wet
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

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_major_chord(frequencies: &[f32], sample_rate: usize, duration: f32) -> SampleBuffer {
        let mut buffer = vec![0f32; (sample_rate as f32 * duration) as usize];
    
        for &freq in frequencies {
            for (i, sample) in buffer.iter_mut().enumerate() {
                let t = i as f32 / sample_rate as f32;
                let envelope = (1.0f32 - t / duration).max(0.0); // Simple linear decay
                *sample += (2.0f32 * pi * freq * t).sin() * envelope;
            }
        }
    
        buffer
    }

    #[test]
    fn test_apply_convolution_reverb() {
        let sample_rate = 44100;
        let duration = 1.0; // 1 second
        let frequencies = [400.0, 500.0, 600.0];
        let reverb_len = 1000; // Example reverb length

        // Generate the major chord signal
        let major_chord = generate_major_chord(&frequencies, sample_rate, duration);

        // Apply convolution reverb to the generated signal
        let wet_signal = apply(&major_chord, reverb_len);

        // Assertions to verify the output
        assert_eq!(wet_signal.len(), major_chord.len() + reverb_len - 1);

        // Check that the wet signal is not all zeros
        let non_zero_samples: Vec<&f32> = wet_signal.iter().filter(|&&x| x != 0.0).collect();
        assert!(!non_zero_samples.is_empty(), "Wet signal should not be all zeros");

        // Optionally: Add more specific checks based on expected behavior
        // e.g., peak amplitude, decay characteristics, etc.
    }
}