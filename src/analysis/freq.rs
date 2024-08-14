use crate::synth::{pi, pi_4};

/// Interpolates between two frequency values `f1` and `f2` based on time `t` and contour `contour_factor`.
/// The interpolation is done in the logarithmic domain (base 2) to provide a smooth interpolation 
/// across octaves, making it particularly suitable for audio applications where frequency changes 
/// should be perceived linearly by the human ear.
/// 
/// # Parameters
/// 
/// - `f1`: The starting frequency value (in Hz).
/// - `f2`: The ending frequency value (in Hz).
/// - `t`: A value in the range [0, 1] representing the interpolation position between `f1` and `f2`.
/// - `contour_factor`: A modifier for the interpolation curve, where 0 results in a linear interpolation.
///   Positive values steepen the curve, while negative values flatten it.
/// 
/// # Returns
/// 
/// - A `f32` value representing the interpolated frequency at the given `t`.
/// 
/// # Examples
/// 
/// ```
/// let f1 = 440.0; // A4 note
/// let f2 = 880.0; // A5 note
/// let t = 0.5; // Halfway interpolation
/// let contour_factor = 0.0; // Linear interpolation
/// let interpolated_freq = interpolate_frequency(f1, f2, t, contour_factor);
/// assert!((interpolated_freq - 622.25).abs() < 0.01); // Should be approximately the geometric mean
/// ```
pub fn interpolate_frequency(f1: f32, f2: f32, t: f32, contour_factor: f32) -> f32 {
    // Soft clip the contour factor to avoid extreme results near the edges
    let contour_factor = contour_factor * 0.995_f32; 

    // Convert frequencies to the logarithmic domain (base 2)
    let c1 = f1.log2();
    let c2 = f2.log2();

    // Calculate interpolation factor with contour modifier
    let interp_factor = t.powf(f32::tan(std::f32::consts::FRAC_PI_4 * (contour_factor + 1.0_f32)));

    // Interpolate in the logarithmic domain and then convert back to linear frequency
    let v = c1 + (c2 - c1) * interp_factor;
    
    2.0_f32.powf(v)
}

pub fn render_checkpoints(
    checkpoints: &[(f32, f32, f32)],
    freq1: f32,
    freq2: f32,
    n_samples: usize,
) -> Vec<f32> {
    let nf = n_samples as f32;
    let max_distance = freq2 - freq1;

    (0..n_samples)
        .map(|i| {
            let t = i as f32 / nf;

            // Find the current checkpoint range
            let (prev_checkpoint, next_checkpoint) = checkpoints
                .iter()
                .zip(checkpoints.iter().skip(1))
                .find(|&(&(p1, _, _), &(p2, _, _))| t >= p1 && t <= p2)
                .unwrap_or_else(|| (checkpoints.last().unwrap(), checkpoints.last().unwrap()));

            let (p1, v1, contour1) = prev_checkpoint;
            let (p2, v2, contour2) = next_checkpoint;

            // Interpolation factor between the two checkpoints
            let segment_t = (t - p1) / (p2 - p1);

            // Linear interpolation of gain and contour between the two checkpoints
            let gain = v1 + (v2 - v1) * segment_t;
            let contour_factor = contour1 + (contour2 - contour1) * segment_t;

            let local_max = gain * max_distance;
            interpolate_frequency(freq1, freq1 + local_max, segment_t, contour_factor)
        })
        .collect()
}


/// Slices a portion of the signal between two points, `p1` and `p2`, and interpolates
/// the values to generate a new signal of specified length.
/// 
/// # Parameters
/// 
/// - `sig`: A reference to the original signal slice (`&[f32]`).
/// - `p1`: A floating-point value representing the starting point of the slice, where `0.0` corresponds to the start of the signal and `1.0` to the end.
/// - `p2`: A floating-point value representing the ending point of the slice, where `0.0` corresponds to the start of the signal and `1.0` to the end.
/// - `into_n_samples`: The number of samples to interpolate into the resulting signal.
/// 
/// # Returns
/// 
/// - A `Vec<f32>` containing the interpolated slice of the signal.
/// 
/// # Panics
/// 
/// - The function does not explicitly panic, but providing invalid input such as `p1` greater than `p2` may lead to unexpected results.
/// 
/// # Examples
/// 
/// ```
/// let original_signal = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
/// let sliced_signal = slice_signal(&original_signal, 0.2, 0.8, 100);
/// assert_eq!(sliced_signal.len(), 100);
/// assert!(sliced_signal[0] >= 1.0);
/// assert!(sliced_signal[sliced_signal.len()-1] <= 4.0);
/// ```
/// 
pub fn slice_signal(sig: &[f32], p1: f32, p2: f32, into_n_samples: usize) -> Vec<f32> {
    if into_n_samples == 0 {
        return Vec::new();
    }

    let mut sliced_signal = Vec::with_capacity(into_n_samples);

    // Calculate the indices in the original signal corresponding to p1 and p2
    let len = sig.len() as f32;
    let start_idx = (p1 * (len - 1.0)).floor() as usize;
    let end_idx = (p2 * (len - 1.0)).floor() as usize;

    // Ensure indices are within bounds
    let start_idx = start_idx.min(sig.len() - 1);
    let end_idx = end_idx.min(sig.len() - 1);

    // Interpolate values between start_idx and end_idx into the sliced_signal
    for i in 0..into_n_samples {
        let t = i as f32 / (into_n_samples - 1) as f32;
        let sample_pos = start_idx as f32 * (1.0 - t) + end_idx as f32 * t;

        // Calculate the lower and upper indices for interpolation
        let lower_idx = sample_pos.floor() as usize;
        let upper_idx = sample_pos.ceil() as usize;

        // Ensure indices are within bounds (optimization: avoid repeated bounds checks)
        let lower_idx = lower_idx.min(sig.len() - 1);
        let upper_idx = upper_idx.min(sig.len() - 1);

        // Linear interpolation
        let interp_value = if lower_idx == upper_idx {
            sig[lower_idx]
        } else {
            let lower_value = sig[lower_idx];
            let upper_value = sig[upper_idx];
            lower_value + (upper_value - lower_value) * (sample_pos - lower_idx as f32)
        };

        sliced_signal.push(interp_value);
    }

    sliced_signal
}



#[cfg(test)]
mod functional {
    use super::*;
    use core::f32;

    #[test]
    fn test_render_checkpoints_linear() {
        let checkpoints: Vec<(f32, f32, f32)> = vec![
            (0.0, 0.2, 0.0),
            (0.3, 0.5, 0.0),
            (0.7, 0.9, 0.0),
            (0.85, 0.9, 0.0),
            (1.0, 0.9, 0.0),
        ];

        let freq1: f32 = 400.0;
        let freq2: f32 = 2400.0;
        let n_samples = 1000;

        let signal = render_checkpoints(&checkpoints, freq1, freq2, n_samples);

        let tolerance: f32 = 0.00005f32;

        assert!(
            signal.iter().all(|&f| (f - freq1).abs() < tolerance || f >= freq1),
            "All entries in the resulting signal must be bound to the lower input frequency"
        );
        assert!(
            signal.iter().all(|&f| f <= freq2),
            "All entries in the resulting signal must be bound to the upper input frequency"
        );
    }
            
    #[test]
    fn test_render_checkpoints_contour_up() {
        let checkpoints: Vec<(f32, f32, f32)> = vec![
            (0.0, 0.2, 0.2),
            (0.3, 0.5, 0.4),
            (0.7, 0.9, 0.6),
            (0.85, 0.9, 0.8),
            (1.0, 0.9, 1.0),
        ];

        let freq1: f32 = 400.0;
        let freq2: f32 = 2400.0;
        let n_samples = 1000;

        let signal = render_checkpoints(&checkpoints, freq1, freq2, n_samples);

        let tolerance: f32 = 0.00005f32;

        assert!(
            signal.iter().all(|&f| (f - freq1).abs() < tolerance || f >= freq1),
            "All entries in the resulting signal must be bound to the lower input frequency"
        );
        assert!(
            signal.iter().all(|&f| f <= freq2),
            "All entries in the resulting signal must be bound to the upper input frequency"
        );
    }

    #[test]
    fn test_render_checkpoints_contour_down() {
        let checkpoints: Vec<(f32, f32, f32)> = vec![
            (0.0, 0.2, -0.2),
            (0.3, 0.5, -0.4),
            (0.7, 0.9, -0.6),
            (0.85, 0.9, -0.8),
            (1.0, 0.9, -1.0),
        ];

        let freq1: f32 = 400.0;
        let freq2: f32 = 2400.0;
        let n_samples = 1000;

        let signal = render_checkpoints(&checkpoints, freq1, freq2, n_samples);

        let tolerance: f32 = 0.00005f32;

        assert!(
            signal.iter().all(|&f| (f - freq1).abs() < tolerance || f >= freq1),
            "All entries in the resulting signal must be bound to the lower input frequency"
        );
        assert!(
            signal.iter().all(|&f| f <= freq2),
            "All entries in the resulting signal must be bound to the upper input frequency"
        );
    }
    use rand::Rng;
    #[test]
    /// Generates 5 checkpoints at random. v is randomized in [0,1] and contour is randomized in [-1,1]
    fn test_random_checkpoints() {
        let mut rng = rand::thread_rng();
    
        // Generate 5 random checkpoints with increasing `p` values in [0, 1]
        let mut checkpoints: Vec<(f32, f32, f32)> = (0..5)
            .map(|i| {
                let p = i as f32 / 4.0; // p values are evenly distributed between 0 and 1
                let v = rng.gen_range(0.0..=1.0); // Randomized v in [0, 1]
                let contour = rng.gen_range(-1.0..=1.0); // Randomized contour in [-1, 1]
                (p, v, contour)
            })
            .collect();
    
        let freq1: f32 = 400.0;
        let freq2: f32 = 2400.0;
        let n_samples = 1000;
    
        // Generate the signal using the random checkpoints
        let signal = render_checkpoints(&checkpoints, freq1, freq2, n_samples);
    
        let tolerance: f32 = 0.00005f32;
    
        // Perform assertions to ensure signal values are within expected frequency range
        assert!(
            signal.iter().all(|&f| (f - freq1).abs() < tolerance || f >= freq1),
            "All entries in the resulting signal must be bound to the lower input frequency"
        );
        assert!(
            signal.iter().all(|&f| f <= freq2),
            "All entries in the resulting signal must be bound to the upper input frequency"
        );
    
        // Print the generated checkpoints and the first few signal values for reference
        println!("Generated Checkpoints: {:#?}", checkpoints);
        println!("First 10 Signal Values: {:?}", &signal[..10]);
    }

}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_interpolation_no_contour() {
        let v1 = 1.0_f32;
        let v2 = 10.0_f32;
        let tolerance = 1e-6_f32;

        let t = 0.5_f32;
        let contour_factor = 0.0_f32;
        let expected = (v1 * v2).sqrt();
        let actual = interpolate_frequency(v1, v2, t, contour_factor);
        assert!((actual - expected).abs() < tolerance, "Logarithmic interpolation failed");

        let t = 1.0_f32;
        let actual = interpolate_frequency(v1, v2, t, contour_factor);
        assert!((v2 - actual).abs() < tolerance, "Should reach the second value at 100% time completion");

        let t = 0.0_f32;
        let actual = interpolate_frequency(v1, v2, t, contour_factor);
        assert!((actual - v1).abs() < tolerance, "Should start at the first value at 0% time completion");
    }

    #[test]
    fn test_interpolation_extreme_values() {
        let v1 = 0.0001_f32;
        let v2 = 10000.0_f32;
        let tolerance = 1e-6_f32;

        let t = 0.5_f32;
        let contour_factor = 0.0_f32;
        let expected = (v1 * v2).sqrt();
        let actual = interpolate_frequency(v1, v2, t, contour_factor);
        assert!((actual - expected).abs() < tolerance, "Interpolation with extreme values failed");
    }

    #[test]
    fn test_interpolation_edge_case_same_values() {
        let v1 = 440.0_f32;
        let v2 = 440.0_f32;
        let tolerance = 1e-6_f32;

        let t = 0.5_f32;
        let contour_factor = 0.0_f32;
        let expected = v1;
        let actual = interpolate_frequency(v1, v2, t, contour_factor);
        assert!((actual - expected).abs() < tolerance, "Interpolation with same values failed");
    }
}
