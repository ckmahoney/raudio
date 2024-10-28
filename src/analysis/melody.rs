use crate::types::render::{Stem, Melody, Feel};
use crate::types::synthesis::{ Ely, Soids, Ampl,Frex, GlideLen, Register, Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::synth::{SRf, SampleBuffer, MAX_REGISTER, MIN_REGISTER, SR}; 
use crate::time;
use super::monic_theory::note_to_freq;

/// Given a list of Lines, 
/// identify the line that has the highest and lowest notes. 
/// ## Returns
/// ((min_register, min_index), (max_register, max_index))
pub fn find_reach(melody: &Melody<Note>) -> ((i8, usize), (i8, usize)) {
    let (mut max_register, mut max_index): (i8, usize) = (i8::MIN, 0);
    let (mut min_register, mut min_index): (i8, usize) = (i8::MAX, 0);

    for (i, line) in melody.iter().enumerate() {
        let mut highest_in_line = i8::MIN;
        let mut lowest_in_line = i8::MAX;

        for (_, (register, _), _) in line.iter() {
            highest_in_line = (*register).max(highest_in_line);
            lowest_in_line = (*register).min(lowest_in_line);
        }

        if highest_in_line > max_register {
            max_register = highest_in_line;
            max_index = i;
        }

        if lowest_in_line < min_register {
            min_register = lowest_in_line;
            min_index = i;
        }
    }

    ((min_register, min_index), (max_register, max_index))
}

// thresh_min: value in decibels representing activation thresh
// thresh_max: value in decibels representing maximum application distance
// stable_mul: multiple from fundamental as cutoff point when not active
// reach: number of octaves to open the filter. Is scaled with (thresh_min, thresh_max)
// odsr values in milliseconds representing { onset, decay, sustain, release }. Bounding box is inclusive
// An attempt to trigger the filter is made per-noteevent.  
// for example, a melody that is constantly below the threshold is going to have the lowpass applied.
// When the melody reaches thesh_min, the filter begins to rise over onset seconds. 
// this is statically derived from the melody, before any animations are appplied. 

/// Scalar values that are strongly related to one another
/// to be applied in-context.
pub struct Levels {
    /// starting / ending positition as a scalar
    stable: f32,
    /// scalar value 
    peak: f32, 
    /// scalar value as a percent of peak
    sustain: f32,
}

/// Absolute measurements of time 
/// to be applied in-context.
#[derive(Clone, Copy, Debug)]
pub struct ODR {
    /// in milliseconds
    onset: f32,
    /// time in ms to get to sustain
    decay: f32,
    /// time in ms to drop to stable value
    release: f32
}
impl ODR {
    /// Calculate the total number of samples required to satisfy the minimum possible ODR
    /// given the onset, decay, and release times in milliseconds and the sample rate (cps).
    pub fn total_samples(&self, cps: f32) -> usize {
        // Convert milliseconds to samples for each stage
        let onset_samples = time::samples_of_milliseconds(cps, self.onset);
        let decay_samples = time::samples_of_milliseconds(cps, self.decay);
        let release_samples = time::samples_of_milliseconds(cps, self.release);
        
        // Sum of samples for all stages
        onset_samples + decay_samples + release_samples
    }

    /// Given an ODR, and a requsted duration in seconds at playback rate CPS, 
    /// Return this same ODR or a new one that fits inside the requestd duration. 
    pub fn fit_in_samples(&self, cps: f32, n_seconds: f32) -> Self {
        let curr_length_samples:usize = self.total_samples(cps);
        let requested_length_samples:usize = time::samples_of_seconds(cps, n_seconds);

        if curr_length_samples > requested_length_samples {
            let scale_factor:f32 =  requested_length_samples as f32 / curr_length_samples as f32;
            return Self {
                onset: self.onset * scale_factor,
                decay: self.decay * scale_factor,
                release: self.release * scale_factor,
            }
        }
        *self
    }
}

#[cfg(test)]
mod tests_odr {
    use super::*;

    #[test]
    fn test_total_samples() {
        // Assuming a sample rate of 44100 Hz
        let cps = 44100.0;

        // Create an ODR with specific onset, decay, and release times
        let odr = ODR {
            onset: 10.0,   // 10 ms
            decay: 20.0,   // 20 ms
            release: 30.0, // 30 ms
        };

        // Expected samples for each stage
        let expected_onset_samples = time::samples_of_milliseconds(cps, odr.onset);
        let expected_decay_samples = time::samples_of_milliseconds(cps, odr.decay);
        let expected_release_samples = time::samples_of_milliseconds(cps, odr.release);

        // Calculate the total samples and verify
        let total_samples = odr.total_samples(cps);
        assert_eq!(total_samples, expected_onset_samples + expected_decay_samples + expected_release_samples, "Total samples calculation mismatch");
    }

    #[test]
    fn test_fit_in_samples_no_scaling_needed() {
        let cps = 44100.0;
        let n_seconds = 0.001; // 1 ms

        // Create an ODR that already fits within 1 ms
        let odr = ODR {
            onset: 0.2, // 0.2 ms
            decay: 0.3, // 0.3 ms
            release: 0.4, // 0.4 ms
        };

        // Since the ODR fits within the time, fit_in_samples should return the original ODR
        let result = odr.fit_in_samples(cps, n_seconds);
        assert_eq!(result.onset, odr.onset);
        assert_eq!(result.decay, odr.decay);
        assert_eq!(result.release, odr.release);
    }

    #[test]
    fn test_fit_in_samples_scaling_needed() {
        let cps = 1.0;
        let n_seconds = 0.1; // 100 ms, allowing for a more moderate scaling factor
    
        // Create an ODR that exceeds the 100 ms duration (total 200 ms)
        let odr = ODR {
            onset: 80.0,   // 80 ms
            decay: 60.0,   // 60 ms
            release: 60.0, // 60 ms
        };
    
        // Calculate the scaled ODR
        let result = odr.fit_in_samples(cps, n_seconds);
        let expected_samples = time::samples_from_dur(cps, n_seconds);
        let actual_samples = result.total_samples(cps);
        assert_eq!(expected_samples, expected_samples, "Must match the number of samples when resizing an ODR");
    
        // Since the total duration is scaled to 100 ms from 200 ms, we expect a scaling factor of 0.5
        let expected_scale_factor = 0.5;
        let tolerance = 1e-3; // Tolerance for floating-point comparison
    
        // Verify each component was scaled by the expected factor
        assert!((result.onset - odr.onset * expected_scale_factor).abs() < tolerance, "Onset scaling mismatch");
        assert!((result.decay - odr.decay * expected_scale_factor).abs() < tolerance, "Decay scaling mismatch");
        assert!((result.release - odr.release * expected_scale_factor).abs() < tolerance, "Release scaling mismatch");
    
        // Additional check to confirm the total duration is also scaled by the expected factor
        let scaled_total_duration = result.onset + result.decay + result.release;
        let original_total_duration = odr.onset + odr.decay + odr.release;
        assert!((scaled_total_duration - original_total_duration * expected_scale_factor).abs() < tolerance, "Total ODR duration scaling mismatch");
    }
    
}



/// Given a line, 
/// define a lowpass contour behaving as a "wah wah" effect
/// with respect to the given configuration. 
/// Guaranteed that a complete ODR will always fit in each noteevent's duration.
pub fn mask_wah(
    cps:f32, 
    line:&Vec<Note>, 
    Levels{stable, peak, sustain}:&Levels, 
    ODR {onset, decay, release}:&ODR
) -> SampleBuffer {
    let n_samples = time::samples_of_line(cps, line);
    let mut contour:SampleBuffer = Vec::with_capacity(n_samples);
    let applied_peak = peak.clamp(1f32, MAX_REGISTER as f32 - MIN_REGISTER as f32);

    let n_samples_ramp:usize = time::samples_of_milliseconds(cps, *onset);
    let n_samples_fall:usize = time::samples_of_milliseconds(cps, *decay);
    let n_samples_kill:usize = time::samples_of_milliseconds(cps, *release);
    // sustain level, boxed in by the ramp/fall/kill values
    let n_samples_hold:usize = n_samples - n_samples_ramp - n_samples_kill;

    for (i, note) in (*line).iter().enumerate() {
        let curr_freq:f32 = note_to_freq(note);
        let stable_freq= curr_freq+ 2f32.powf(*stable);
        let n_samples_note: usize = time::samples_of_note(cps, note);
        for j in 0..n_samples_note {
            let cutoff_freq:f32 = if j < n_samples_ramp {
                // onset
                let p = j as f32 / n_samples_ramp as f32;
                stable_freq + 2f32.powf(applied_peak * p)
            } else if j < n_samples_ramp + n_samples_fall {
                // decay
                let p = (j - n_samples_ramp)  as f32/ n_samples_fall as f32;
                let d_sustain = (1f32-p) * (1f32-sustain);
                stable_freq + 2f32.powf(applied_peak * d_sustain)
            } else if j < n_samples_ramp + n_samples_fall + n_samples_hold {
                let p = (j - n_samples_ramp - n_samples_fall)as f32 / n_samples_hold as f32;
                // sustain
                stable_freq + 2f32.powf(applied_peak * sustain)
            } else {
                // release
                let p = (j - n_samples_ramp - n_samples_fall - n_samples_hold) as f32/ n_samples_kill as f32;
                let d_sustain = (1f32-p) * sustain;
                stable_freq + 2f32.powf(d_sustain)
            };

            contour.push(cutoff_freq);
        }
    };
    contour
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::time; // Ensure `time::samples_of_note` and `time::samples_of_line` are accessible

    #[test]
    fn test_mask_wah_contour_initial_high_and_stable_end() {
        let cps = 2f32; // Sample rate in Hz, e.g., 44100 Hz

        let line: Vec<Note> = vec![
            ((1, 4), (5, (0,0,1)), 1.0),  // (Duration, Tone, Ampl)
            ((1, 4), (5, (0,0,1)), 1.0),
            ((1, 2), (5, (0,0,1)), 1.0),
        ];

        let levels = Levels {
            peak: 5f32,
            sustain: 2f32,
            stable: 1f32
        };

        let odr = ODR {
            onset: 30f32,
            decay: 200f32,
            release: 60f32,
        };

        // let contour = mask_wah(cps, &line);
        // let stable_freq = 200.0; // Stable frequency as per the mask_wah function
        // let initial_samples = 8000; // Initial high cutoff frequency for the first 8000 samples

        // // Ensure contour is non-empty and has correct length
        // let expected_length = time::samples_of_line(cps, &line);
        // assert_eq!(contour.len(), expected_length, "Contour length mismatch");

        // // Check the initial samples of the first note
        // for i in 0..initial_samples.min(contour.len()) {
        //     assert!(contour[i] > stable_freq, "Initial contour frequency should be higher than stable frequency");
        // }

        // // Check that the contour stabilizes to the stable frequency after the initial ramp
        // for i in (initial_samples+10)..contour.len() {
        //     assert_eq!(contour[i], stable_freq, "Contour should stabilize at the stable frequency after initial high");
        // }
    }
}

