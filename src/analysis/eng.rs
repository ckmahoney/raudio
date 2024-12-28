// audio.rs
//
// This code uses crate::synth::{SR, SRf} for sample-rate constants. 
// The functions are refined implementations of typical audio processing stages.
// Unit tests are provided below.

use crate::synth::{SR, SRf};

#[derive(Debug, Clone, Copy)]
pub enum EnvelopeMethod {
    Peak,
    Rms,
}

pub fn envelope_follower(
    samples: &[f32],
    attack_time: f32,
    release_time: f32,
    hold_time: Option<f32>,
    method: Option<EnvelopeMethod>,
    pre_emphasis: Option<f32>,
) -> Vec<f32> {
    let envelope_method = method.unwrap_or(EnvelopeMethod::Peak);
    let hold_samps = (hold_time.unwrap_or(0.0) * SRf).round() as usize;
    let attack_coeff = time_to_coefficient(attack_time);
    let release_coeff = time_to_coefficient(release_time);

    let processed_samples = if let Some(cutoff_hz) = pre_emphasis {
        apply_highpass(samples, cutoff_hz)
    } else {
        samples.to_vec()
    };

    let mut env = Vec::with_capacity(processed_samples.len());
    let mut current_env = 0.0;
    let mut hold_counter = 0usize;

    for &sample in processed_samples.iter() {
        let val = match envelope_method {
            EnvelopeMethod::Peak => sample.abs(),
            EnvelopeMethod::Rms => (sample * sample).sqrt(),
        };

        if val > current_env {
            current_env = attack_coeff * (current_env - val) + val;
            hold_counter = 0;
        } else {
            if hold_counter < hold_samps {
                hold_counter += 1;
            } else {
                current_env = release_coeff * (current_env - val) + val;
            }
        }
        env.push(current_env);
    }
    env
}

pub fn compressor(
    samples: &[f32],
    envelope: &[f32],
    threshold: f32,
    ratio: f32,
    knee_width: Option<f32>,
    makeup_gain: Option<f32>,
) -> Vec<f32> {
    let knee = knee_width.unwrap_or(0.0).max(0.0);
    let mg = makeup_gain.unwrap_or(1.0);

    let mut output = Vec::with_capacity(samples.len());
    for (i, &sample) in samples.iter().enumerate() {
        let env_val = envelope[i];
        let compressed = if env_val <= threshold {
            sample
        } else {
            if knee > 0.0 {
                let half_knee = knee * 0.5;
                let lower_bound = threshold - half_knee;
                let upper_bound = threshold + half_knee;
                if env_val < lower_bound {
                    sample
                } else if env_val > upper_bound {
                    apply_compression(sample, threshold, ratio)
                } else {
                    let normalized = (env_val - lower_bound) / knee;
                    let interp_ratio = ratio + (1.0 - ratio) * (1.0 - normalized);
                    apply_compression(sample, threshold, interp_ratio)
                }
            } else {
                apply_compression(sample, threshold, ratio)
            }
        };
        output.push(compressed * mg);
    }
    output
}

pub fn transient_shaper(
    samples: &[f32],
    envelope: &[f32],
    transient_emphasis: f32,
    threshold: Option<f32>,
    attack_time: Option<f32>,
    release_time: Option<f32>,
) -> Vec<f32> {
    let atk = time_to_coefficient(attack_time.unwrap_or(0.01));
    let rel = time_to_coefficient(release_time.unwrap_or(0.05));
    let thr = threshold.unwrap_or(0.0);
    let mut output = Vec::with_capacity(samples.len());
    let mut local_env = 0.0;

    for (i, &sample) in samples.iter().enumerate() {
        let env_val = envelope[i];
        let diff = (env_val - local_env).max(0.0);

        if env_val > local_env {
            local_env = atk * (local_env - env_val) + env_val;
        } else {
            local_env = rel * (local_env - env_val) + env_val;
        }

        let factor = if env_val > thr {
            1.0 + transient_emphasis * diff
        } else {
            1.0
        };
        output.push(sample * factor);
    }
    output
}

pub fn soft_clipper(samples: &[f32], clip_threshold: f32) -> Vec<f32> {
    samples.iter().map(|&s| {
        let abs_s = s.abs();
        let sign = s.signum();
        if abs_s <= clip_threshold {
            s
        } else if abs_s <= 2.0 * clip_threshold {
            // Smoothly compress between threshold and 2*threshold
            sign * ((3.0 - (2.0 - abs_s / clip_threshold).powi(2)) * clip_threshold / 3.0)
        } else {
            // Hard limit beyond 2*threshold
            sign * clip_threshold
        }
    }).collect()
}

pub fn normalizer(samples: &[f32], target_max: f32) -> Vec<f32> {
    let mut max_val :f32 = 0.0;
    for &s in samples.iter() {
        max_val = max_val.max(s.abs());
    }
    if max_val <= 0.0 {
        return samples.to_vec();
    }
    let gain = target_max / max_val;
    samples.iter().map(|&s| s * gain).collect()
}

pub fn gate(samples: &[f32], threshold: f32) -> Vec<f32> {
    samples
        .iter()
        .map(|&s| if s.abs() <= threshold { 0.0 } else { s })
        .collect()
}

pub fn calculate_threshold(samples: &[f32], factor: f32, use_rms: bool) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    if use_rms {
        let sum_sq: f32 = samples.iter().map(|&x| x * x).sum();
        let rms = (sum_sq / samples.len() as f32).sqrt();
        rms * factor
    } else {
        let peak = samples
            .iter()
            .fold(0.0_f32, |acc, &x| acc.max(x.abs()));
        peak * factor
    }
}

pub fn deinterleave(samples: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let mut left = Vec::with_capacity(samples.len() / 2);
    let mut right = Vec::with_capacity(samples.len() / 2);
    for chunk in samples.chunks_exact(2) {
        left.push(chunk[0]);
        right.push(chunk[1]);
    }
    (left, right)
}

pub fn interleave(left: &[f32], right: &[f32]) -> Vec<f32> {
    assert_eq!(left.len(), right.len(), "Channel length mismatch.");
    let mut out = Vec::with_capacity(left.len() * 2);
    for i in 0..left.len() {
        out.push(left[i]);
        out.push(right[i]);
    }
    out
}

fn time_to_coefficient(time_sec: f32) -> f32 {
    if time_sec <= 0.0 {
        0.0
    } else {
        (-1.0 / (time_sec * SRf)).exp()
    }
}

fn apply_highpass(samples: &[f32], cutoff_hz: f32) -> Vec<f32> {
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
    let alpha = rc / (rc + 1.0 / SRf);
    let mut out = Vec::with_capacity(samples.len());
    let mut prev_in = 0.0;
    let mut prev_out = 0.0;
    for &input in samples {
        let filtered = alpha * (prev_out + input - prev_in);
        out.push(filtered);
        prev_out = filtered;
        prev_in = input;
    }
    out
}

fn apply_compression(sample: f32, threshold: f32, ratio: f32) -> f32 {
    let sign = sample.signum();
    let abs_s = sample.abs();
    if abs_s <= threshold {
        sample
    } else {
        let diff = abs_s - threshold;
        let compressed = threshold + diff / ratio;
        sign * compressed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_envelope_follower_peak() {
        let input = vec![0.0, 0.1, 0.2, 0.4, 0.2, 0.1, 0.0];
        let result = envelope_follower(&input, 0.01, 0.01, None, Some(EnvelopeMethod::Peak), None);
        // We expect a rising/falling envelope near the absolute values
        assert_eq!(result.len(), input.len());
        // Just check it doesn't overshoot drastically
        for (r, i) in result.iter().zip(input.iter()) {
            assert!(r >= &0.0 && r <= &(i.abs().max(0.4) + 0.1));
        }
    }

    #[test]
    fn test_compressor_hard_knee() {
        let input = vec![0.0, 0.5, 1.0, 1.5];
        let envelope = input.clone();
        let threshold = 1.0;
        let ratio = 2.0;
        let output = compressor(&input, &envelope, threshold, ratio, None, None);
        // Values above threshold should be compressed
        // e.g. sample=1.5 above threshold => 1.0 + (0.5/2.0)=1.25
        assert_eq!(output[0], 0.0);
        assert_eq!(output[1], 0.5);
        assert_eq!(output[2], 1.0);
        assert!((output[3] - 1.25).abs() < 1e-6);
    }

    #[test]
    fn test_soft_clipper() {
        let input = vec![0.0, 0.2, 0.8, 1.2, -1.4, 1.0, 2.5];
        let clip_thresh = 1.0;
        let out = soft_clipper(&input, clip_thresh);
        assert_eq!(out.len(), input.len());
        // Values <= threshold should be unchanged
        assert!((out[0] - 0.0).abs() < 1e-6);
        assert!((out[1] - 0.2).abs() < 1e-6);
        assert!((out[2] - 0.8).abs() < 1e-6);
        // Values > threshold should be softly clipped
        assert!((out[3] - 0.7866667).abs() < 1e-5); // 1.2 clipped to ~0.7867
        assert!((out[4] + 0.88).abs() < 1e-5);      // -1.4 clipped to ~-0.88
        assert!((out[5] - 1.0).abs() < 1e-6);
        // Values beyond 2*threshold should be hard clipped to threshold
        assert!((out[6] - 1.0).abs() < 1e-6);      // 2.5 clipped to 1.0
    }

    #[test]
    fn test_normalizer() {
        let input = vec![0.0, 0.2, -0.5, 0.8];
        let out = normalizer(&input, 1.0);
        let max_val = out.iter().fold(0.0_f32, |acc, &x| acc.max(x.abs()));
        assert!((max_val - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gate() {
        let input = vec![0.01, 0.2, 0.001, -0.009];
        let out = gate(&input, 0.01);
        // Values <= 0.01 in absolute value => 0
        assert_eq!(out.len(), input.len());
        assert_eq!(out[0], 0.0); // 0.01 <= 0.01 => 0.0
        assert_eq!(out[1], 0.2); // 0.2 > 0.01 => 0.2
        assert_eq!(out[2], 0.0); // 0.001 <= 0.01 => 0.0
        assert_eq!(out[3], 0.0); // -0.009 <= 0.01 => 0.0
    }

    #[test]
    fn test_calculate_threshold_peak() {
        let input = vec![0.1, 0.2, 0.9, -1.2];
        let th = calculate_threshold(&input, 0.5, false);
        // Peak is 1.2 => scaled by 0.5 => 0.6
        assert!((th - 0.6).abs() < 1e-6);
    }

    #[test]
    fn test_deinterleave_interleave() {
        let input_stereo = vec![0.1, 0.2, 0.3, 0.4, -0.5, -0.6];
        let (left, right) = deinterleave(&input_stereo);
        assert_eq!(left.len(), 3);
        assert_eq!(right.len(), 3);
        let reinterleaved = interleave(&left, &right);
        assert_eq!(reinterleaved, input_stereo);
    }
}
