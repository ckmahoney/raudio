use crate::synth::{SampleBuffer};

/// Better for linear modulation of amplitude
pub fn db_to_amp(db:f32) -> f32 {
    10f32.powf(db/20f32)
}

fn amplitude_to_db(amplitude: f32) -> f32 {
    20.0 * amplitude.log10()
}
    

/// Identify the RMS value of a signal slice.
/// Intended for short form slices or samples
pub fn rms(signal: &[f32]) -> f32 {
    if signal.is_empty() {
        return 0.0;
    }
    let sum: f32 = signal.iter().map(|&x| x * x).sum();
    (sum / signal.len() as f32).sqrt()
}

/// Produce a valid root mean square on available data for a given window_len. 
/// Intended for long form time varying singles.
pub fn root_mean_squared(signal: &SampleBuffer, window_len: usize) -> SampleBuffer {
    let mut result = vec![0f32; signal.len()];
    for i in 0..signal.len() {  
        let k = if i + window_len > signal.len() {
            signal.len() - i
        } else {
            window_len
        };

        result[i] = rms(&signal[i..i+k]);
    }
    result
}

pub fn is_monotonically_increasing_rms(signal: &SampleBuffer, sample_rate: usize) -> bool {
    let bucket_size = sample_rate; // Assuming sample_rate gives the number of samples per bucket
    let mut last_rms = 0.0;
    let mut is_first = true;

    // Handle full buckets and the possible last partial bucket
    let num_buckets = (signal.len() + bucket_size - 1) / bucket_size; // Ensures last partial bucket is included

    for t in 0..num_buckets {
        let start = t * bucket_size;
        let end = usize::min(start + bucket_size, signal.len()); // Ensures we don't go out of bounds
        let bucket = &signal[start..end];
        
        if !bucket.is_empty() { // Check to ensure non-empty bucket
            let r = rms(&bucket);

            // Check for monotonically increasing
            if is_first {
                is_first = false;
            } else if r < last_rms {
                return false;
            }
            last_rms = r;
        }
    }
    
    true
}

#[cfg(test)]
mod test {
    use super::*;
    use rustfft::num_complex::Complex;
    static n_samples:usize = 48000 * 4;
    static i:Complex<f32> = Complex::new(0.0, 1.0);

    #[test]
    fn test_increase_happy_path_valid() {
        // create a constantly growing absolute value signal
        let signal:SampleBuffer = (0..n_samples).enumerate().map(|(index, _)| 
            i.powi(2i32 + (2i32*index as i32) ).re * index as f32 / n_samples as f32 
        ).collect();

        let result = is_monotonically_increasing_rms(&signal, signal.len()/10);
        assert!(result, "Must be able to measure that a constantly growing signal is growing")
    }   

    #[test]
    fn test_increase_happy_path_invalid() {
        // create a constantly falling absolute value signal
        let signal:SampleBuffer = (0..n_samples).enumerate().map(|(index, _)| 
        i.powi(2i32 + (2i32*index as i32) ).re * (n_samples - index) as f32 / n_samples as f32 
        ).collect();

        let result = is_monotonically_increasing_rms(&signal, signal.len()/10);
        assert!(result == false, "Must be able to measure that a constantly growing signal is growing")
    }   


}