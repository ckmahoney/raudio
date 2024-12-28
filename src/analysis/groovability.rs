use crate::render::engrave;
use crate::synth::SR;
use crate::analysis::sampler::AudioFormat;

/// Groove Optimizer: Enhances groove by transient emphasis, dynamic range control,
/// and rhythmic element prioritization.
///
/// # Arguments
/// * `input_signal` - The input audio signal as an `AudioFormat`.
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

    match input_signal {
        AudioFormat::Mono(samples) => {
            let threshold = calculate_threshold(&samples, rhythmic_threshold);
            let processed = process_channel(samples, transient_emphasis, dynamic_range, threshold, attack_time, release_time);
            AudioFormat::Mono(processed)
        }
        AudioFormat::Stereo(left, right) => {
            let left_threshold = calculate_threshold(&left, rhythmic_threshold);
            let right_threshold = calculate_threshold(&right, rhythmic_threshold);
            let processed_left = process_channel(left, transient_emphasis, dynamic_range, left_threshold, attack_time, release_time);
            let processed_right = process_channel(right, transient_emphasis, dynamic_range, right_threshold, attack_time, release_time);
            AudioFormat::Stereo(processed_left, processed_right)
        }
        AudioFormat::Interleaved(samples) => {
            let (left, right) = deinterleave(samples);
            let left_threshold = calculate_threshold(&left, rhythmic_threshold);
            let right_threshold = calculate_threshold(&right, rhythmic_threshold);
            let processed_left = process_channel(left, transient_emphasis, dynamic_range, left_threshold, attack_time, release_time);
            let processed_right = process_channel(right, transient_emphasis, dynamic_range, right_threshold, attack_time, release_time);
            let interleaved = interleave(processed_left, processed_right);
            AudioFormat::Interleaved(interleaved)
        }
    }
}

/// Process a single channel of audio with transient shaping and dynamic range control.
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

    // Improved normalization: Scale only if the max amplitude exceeds 1.0
    let max_amp = output_signal.iter().map(|s| s.abs()).fold(0.0, f32::max);
    if max_amp > 1.0 {
        for sample in &mut output_signal {
            *sample /= max_amp;
        }
    }

    output_signal
}

/// Calculate an adaptive threshold based on RMS.
fn calculate_threshold(samples: &Vec<f32>, rhythmic_threshold: f32) -> f32 {
    let rms = (samples.iter().map(|s| s.powi(2)).sum::<f32>() / samples.len() as f32).sqrt();
    rhythmic_threshold * rms
}

/// Deinterleave a stereo interleaved signal into left and right channels.
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

/// Interleave left and right channels into a single interleaved signal.
fn interleave(left: Vec<f32>, right: Vec<f32>) -> Vec<f32> {
    let mut interleaved = Vec::with_capacity(left.len() + right.len());
    for (l, r) in left.into_iter().zip(right.into_iter()) {
        interleaved.push(l);
        interleaved.push(r);
    }
    interleaved
}
