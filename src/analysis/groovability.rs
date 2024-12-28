use rand::Rng;
use rand::seq::SliceRandom;
use crate::render::engrave;
use crate::synth::SR;

/// Groove Optimizer: Enhances groove by transient emphasis, dynamic range control, 
/// and rhythmic element prioritization.
/// 
/// # Arguments
/// * `input_signal` - The input audio signal as a vector of samples.
/// * `transient_emphasis` - Boost transients, 0.0 to 1.0.
/// * `dynamic_range` - Compression/expansion balance, 0.0 to 1.0.
/// * `rhythmic_threshold` - Threshold for rhythmic elements, 0.0 to 1.0.
/// * `time_constant` - Attack/release time for processing in ms.
/// 
/// # Returns
/// A new signal optimized for danceability.
pub fn groove_optimizer(
    input_signal: Vec<f32>,
    transient_emphasis: f32,
    dynamic_range: f32,
    rhythmic_threshold: f32,
    time_constant: f32,
) -> Vec<f32> {
    let mut output_signal = vec![0.0; input_signal.len()];

    // Basic parameters for transient shaping and compression
    let attack_time = (time_constant / 1000.0) * SR as f32;
    let release_time = (time_constant / 1000.0) * SR as f32;
    let threshold = rhythmic_threshold * 0.5; // Normalize threshold for beats

    // Process the input signal
    let mut envelope = 0.0;
    for (i, &sample) in input_signal.iter().enumerate() {
        // Transient emphasis: simple peak detection
        let abs_sample = sample.abs();
        if abs_sample > envelope {
            envelope += (abs_sample - envelope) / attack_time;
        } else {
            envelope += (abs_sample - envelope) / release_time;
        }

        // Dynamic range control
        let gain = if envelope > threshold {
            1.0 + transient_emphasis * (envelope - threshold)
        } else {
            1.0 - (1.0 - dynamic_range) * threshold
        };

        // Apply gain to signal
        output_signal[i] = sample * gain;
    }

    output_signal
}

/// Test module for generating signals and evaluating the groove optimizer
#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::engrave::samples;
    use rand::Rng;
    use std::f32::consts::PI;
    const gain:f32 = 0.1f32;

    /// Generate a sine wave with an exponential decay envelope and optional chorus
    fn generate_chorus_sine_impulse(freq: f32, duration: f32, sr: usize) -> Vec<f32> {
        let mut rng = rand::thread_rng();
        let amplitude_scaler = 0.1 * rng.gen_range(1.01..1.05); // Scale amplitude by 0.3 and add random variation
        let mut signal = vec![0.0; (sr as f32 * duration) as usize];
        let detune_offsets = [-0.4, -0.2, 0.0, 0.2, 0.4]; // Detune by multiples of 0.2 Hz

        for &offset in &detune_offsets {
            for t in 0..signal.len() {
                let time = t as f32 / sr as f32;
                let env = (-5.0 * time).exp()  *gain;
                signal[t] += (2.0 * PI * (freq + offset) * time).sin() * env * amplitude_scaler;
            }
        }

        signal
    }

    /// Generate a kick-drum-like noise burst using a lowpass-filtered noise
    fn generate_kick_like_burst(freq: f32, duration: f32, sr: usize) -> Vec<f32> {
        let mut rng = rand::thread_rng();
        let mut signal = vec![];
        let cutoff = freq; // Cutoff frequency for the lowpass effect

        for t in 0..(sr as f32 * duration) as usize {
            let time = t as f32 / sr as f32;
            let env = (-5.0 * time).exp(); // Exponential decay
            let raw_noise = rng.gen_range(-1.0..1.0);
            let lowpassed = raw_noise * (1.0 - (time * cutoff).sin().abs()); // Simple lowpass effect
            signal.push(lowpassed * env *gain);
        }

        signal
    }


    /// Generate a noise burst with sinc-like decay
    fn generate_noise_burst(duration: f32, sr: usize) -> Vec<f32> {
        let mut rng = rand::thread_rng();
        let amplitude_scaler = 0.3 * rng.gen_range(1.01..1.05); // Scale amplitude by 0.3 and add random variation
        let mut signal = vec![];
        for t in 0..(sr as f32 * duration) as usize {
            let time = t as f32 / sr as f32;
            let env = (-5.0 * time).exp() * gain;
            signal.push(rng.gen_range(-1.0..1.0) * env * amplitude_scaler);
        }
        signal
    }


    /// Generate a rhythmic signal with a random pattern of sine and noise bursts
    /// Includes melodies adjusted to registers 6 to 8
    fn generate_test_signal(register: u8, sr: usize) -> (Vec<f32>, Vec<f32>) {
        let notes: Vec<f32> = vec![100.0, 200.0, 300.0, 500.0, 700.0, 250.0, 333.0];
        let mut rng = rand::thread_rng();
        let mut melody_track = vec![];
        let mut noise_track = vec![];

        for _ in 0..16 {
            let root = *notes.choose(&mut rng).unwrap();
            let scaled_freq = root * 2f32.powi(register as i32 );
            let duration = *[1.0, 2.0, 0.5].choose(&mut rng).unwrap();

            if rng.gen_bool(0.5) {
                melody_track.extend(generate_chorus_sine_impulse(scaled_freq, duration, sr));
            } else {
                noise_track.extend(generate_kick_like_burst(scaled_freq, duration, sr));
            }
        }

        (melody_track, noise_track)
    }

    /// Sum multiple tracks into a single buffer
    fn sum_tracks(track1: &[f32], track2: &[f32]) -> Vec<f32> {
        let len = track1.len().min(track2.len());
        (0..len).map(|i| track1[i] + track2[i]).collect()
    }

    /// Repeat a sample for a given number of cycles
    fn repeat_sample(sample: Vec<f32>, k_cycles: usize, cps: f32) -> Vec<f32> {
        let mut repeated = vec![];
        let cycle_len = (SR as f32 / cps) as usize;

        for _ in 0..k_cycles {
            repeated.extend(sample.iter().cycle().take(cycle_len));
        }

        repeated
    }

    #[test]
    fn test_groove_optimizer() {
        let register = 7; // Use registers 6, 7, or 8 for melodies
        let sr = SR;
        let cps = 2.0;

        // Generate separate tracks for melody and noise
        let (melody_track, noise_track) = generate_test_signal(register, sr);

        // Mix tracks into a single signal
        let mixed_signal = sum_tracks(&melody_track, &noise_track);

        // Apply groove optimization
        let optimized_signal = groove_optimizer(
            mixed_signal.clone(),
            0.8,  // Transient emphasis
            0.6,  // Dynamic range control
            0.5,  // Rhythmic threshold
            50.0, // Time constant (ms)
        );

        let repeated_signal = repeat_sample(optimized_signal.clone(), 4, cps);

        // Write both original and optimized signals to disk for evaluation
        samples(sr, &melody_track, "dev-audio/test-melody-track.wav");
        samples(sr, &noise_track, "dev-audio/test-noise-track.wav");
        samples(sr, &mixed_signal, "dev-audio/test-mixed-signal.wav");
        samples(sr, &optimized_signal, "dev-audio/test-optimized-signal.wav");
        samples(sr, &repeated_signal, "dev-audio/test-repeated-optimized-signal.wav");

        println!("Test signals written to disk for evaluation.");
    }
}
