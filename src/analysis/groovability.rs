use rand::Rng;
use rand::seq::SliceRandom;
use crate::render::engrave;
use crate::synth::SR;
use crate::analysis::sampler::{ AudioFormat};

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
    input_signal: AudioFormat,
    transient_emphasis: f32,
    dynamic_range: f32,
    rhythmic_threshold: f32,
    time_constant: f32,
    ) -> AudioFormat {
let sr_factor = (time_constant / 1000.0) * SR as f32;
let attack_time = sr_factor;
let release_time = sr_factor;
let threshold = rhythmic_threshold * 0.5; // normalized threshold

match input_signal {
    AudioFormat::Mono(samples) => {
        let processed = process_channel(samples, transient_emphasis, dynamic_range, threshold, attack_time, release_time);
        AudioFormat::Mono(processed)
    }
    AudioFormat::Stereo(left, right) => {
        let processed_left = process_channel(left, transient_emphasis, dynamic_range, threshold, attack_time, release_time);
        let processed_right = process_channel(right, transient_emphasis, dynamic_range, threshold, attack_time, release_time);
        AudioFormat::Stereo(processed_left, processed_right)
    }
    AudioFormat::Interleaved(samples) => {
        let (left, right) = deinterleave(samples);
        let processed_left = process_channel(left, transient_emphasis, dynamic_range, threshold, attack_time, release_time);
        let processed_right = process_channel(right, transient_emphasis, dynamic_range, threshold, attack_time, release_time);
        let interleaved = interleave(processed_left, processed_right);
        AudioFormat::Interleaved(interleaved)
    }
}
}

fn process_channel(
samples: Vec<f32>,
transient_emphasis: f32,
dynamic_range: f32,
threshold: f32,
attack_time: f32,
release_time: f32,
) -> Vec<f32> {
let mut envelope = 0.0f32;
let mut peak_level = 0.0f32;
let mut output_signal = vec![0.0; samples.len()];

for (i, &sample) in samples.iter().enumerate() {
    let abs_sample = sample.abs();

    // Envelope follower
    if abs_sample > envelope {
        envelope += (abs_sample - envelope) / attack_time;
    } else {
        envelope += (abs_sample - envelope) / release_time;
    }
    peak_level = peak_level.max(abs_sample);

    // Dynamic gain calculations
    let gain = if envelope > threshold {
        1.0 + transient_emphasis * (envelope - threshold)
    } else {
        1.0 - (1.0 - dynamic_range) * threshold
    };
    let adaptive_gain = 1.0 + (transient_emphasis * 0.5) * (peak_level - envelope);

    // Combine gains
    let mut processed_sample = sample * gain * adaptive_gain;

    // Gentle soft-clip
    let clip_threshold = 0.95;
    if processed_sample > clip_threshold {
        let overshoot = processed_sample - clip_threshold;
        processed_sample = clip_threshold + overshoot.tanh() * 0.05;
    } else if processed_sample < -clip_threshold {
        let overshoot = processed_sample + clip_threshold;
        processed_sample = -clip_threshold + overshoot.tanh() * -0.05;
    }

    output_signal[i] = processed_sample;
}

// Improved normalization
let max_amp = output_signal.iter().map(|s| s.abs()).fold(0.0, f32::max);
if max_amp > 1.0 {
    for sample in &mut output_signal {
        *sample /= max_amp;
    }
}

output_signal
}

fn deinterleave(samples: Vec<f32>) -> (Vec<f32>, Vec<f32>) {
let mut left = Vec::with_capacity(samples.len() / 2);
let mut right = Vec::with_capacity(samples.len() / 2);

for chunk in samples.chunks(2) {
    if let [l, r] = chunk {
        left.push(*l);
        right.push(*r);
    }
}

(left, right)
}

fn interleave(left: Vec<f32>, right: Vec<f32>) -> Vec<f32> {
let mut interleaved = Vec::with_capacity(left.len() + right.len());
for (l, r) in left.into_iter().zip(right.into_iter()) {
    interleaved.push(l);
    interleaved.push(r);
}
interleaved
}


// /// Test module for generating signals and evaluating the groove optimizer
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::render::engrave::samples;
//     use rand::Rng;
//     use std::f32::consts::PI;

//     /// Generate a sine wave with an exponential decay envelope and optional chorus
//     fn generate_chorus_sine_impulse(freq: f32, duration: f32, sr: usize) -> Vec<f32> {
//         let mut signal = vec![0.0; (sr as f32 * duration) as usize];
//         let detune_offsets = [-0.4, -0.2, 0.0, 0.2, 0.4]; // Detune by multiples of 0.2 Hz

//         for &offset in &detune_offsets {
//             for t in 0..signal.len() {
//                 let time = t as f32 / sr as f32;
//                 let env = (-5.0 * time).exp();
//                 signal[t] += (2.0 * PI * (freq + offset) * time).sin() * env;
//             }
//         }

//         signal
//     }

//     /// Generate a noise burst with sinc-like decay
//     fn generate_noise_burst(duration: f32, sr: usize) -> Vec<f32> {
//         let mut rng = rand::thread_rng();
//         let mut signal = vec![];
//         for t in 0..(sr as f32 * duration) as usize {
//             let time = t as f32 / sr as f32;
//             let env = (-5.0 * time).exp();
//             signal.push(rng.gen_range(-1.0..1.0) * env);
//         }
//         signal
//     }

//     /// Generate a rhythmic signal with a random pattern of sine and noise bursts
//     /// Includes melodies adjusted to registers 6 to 8
//     fn generate_test_signal(register: u8, sr: usize) -> (Vec<f32>, Vec<f32>) {
//         let notes: Vec<f32> = vec![100.0, 200.0, 300.0, 500.0, 700.0, 250.0, 333.0];
//         let mut rng = rand::thread_rng();
//         let mut melody_track = vec![];
//         let mut noise_track = vec![];

//         for _ in 0..16 {
//             let root = *notes.choose(&mut rng).unwrap();
//             let scaled_freq = root * 2f32.powi(register as i32 - 7);
//             let duration = *[1.0, 2.0, 0.5].choose(&mut rng).unwrap();

//             if rng.gen_bool(0.5) {
//                 melody_track.extend(generate_chorus_sine_impulse(scaled_freq, duration, sr));
//             } else {
//                 noise_track.extend(generate_noise_burst(duration, sr));
//             }
//         }

//         (melody_track, noise_track)
//     }

//     /// Sum multiple tracks into a single buffer
//     fn sum_tracks(track1: &[f32], track2: &[f32]) -> Vec<f32> {
//         let len = track1.len().min(track2.len());
//         (0..len).map(|i| track1[i] + track2[i]).collect()
//     }

//     /// Repeat a sample for a given number of cycles
//     fn repeat_sample(sample: Vec<f32>, k_cycles: usize, cps: f32) -> Vec<f32> {
//         let mut repeated = vec![];
//         let cycle_len = (SR as f32 / cps) as usize;

//         for _ in 0..k_cycles {
//             repeated.extend(sample.iter().cycle().take(cycle_len));
//         }

//         repeated
//     }

//     #[test]
//     fn test_groove_optimizer() {
//         let register = 7; // Use registers 6, 7, or 8 for melodies
//         let sr = SR;
//         let cps = 2.0;

//         // Generate separate tracks for melody and noise
//         let (melody_track, noise_track) = generate_test_signal(register, sr);

//         // Mix tracks into a single signal
//         let mixed_signal = sum_tracks(&melody_track, &noise_track);

//         // Apply groove optimization
//         let optimized_signal = groove_optimizer(
//             mixed_signal.clone(),
//             0.8,  // Transient emphasis
//             0.6,  // Dynamic range control
//             0.5,  // Rhythmic threshold
//             50.0, // Time constant (ms)
//         );

//         let repeated_signal = repeat_sample(optimized_signal.clone(), 4, cps);

//         // Write both original and optimized signals to disk for evaluation
//         samples(sr, &melody_track, "dev-audio/test-melody-track.wav");
//         samples(sr, &noise_track, "dev-audio/test-noise-track.wav");
//         samples(sr, &mixed_signal, "dev-audio/test-mixed-signal.wav");
//         samples(sr, &optimized_signal, "dev-audio/test-optimized-signal.wav");
//         samples(sr, &repeated_signal, "dev-audio/test-repeated-optimized-signal.wav");

//         println!("Test signals written to disk for evaluation.");
//     }
// }
