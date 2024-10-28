use super::in_range;
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
    pub stable: f32,
    /// scalar value 
    pub peak: f32, 
    /// scalar value as a percent of peak
    pub sustain: f32,
}

pub struct LevelMacro {
    pub stable: [f32;2],
    pub peak: [f32;2],
    pub sustain: [f32;2],
}

impl LevelMacro {
    pub fn gen(&self) -> Levels {
        let mut rng = rand::thread_rng();
        Levels {
            stable: in_range(&mut rng, self.stable[0], self.stable[1]),
            peak: in_range(&mut rng, self.peak[0], self.peak[1]),
            sustain: in_range(&mut rng, self.sustain[0], self.sustain[1]),
        }
    }
}

impl Levels {
    pub fn new(stable:f32, peak:f32, sustain:f32) -> Self {
        if sustain > 1f32 || sustain < 0f32 {
            panic!("Sustain value in Levels represents a percentage of peak, and must be given in range of [0, 1]. You provided {}", sustain)
        }

        Self {
            stable,
            peak,
            sustain
        }
    }
}

/// Absolute measurements of time 
/// to be applied in-context.
#[derive(Clone, Copy, Debug)]
pub struct ODR {
    /// in milliseconds
    pub onset: f32,
    /// time in ms to get to sustain
    pub decay: f32,
    /// time in ms to drop to stable value
    pub release: f32
}


/// Min/max values for generating an ODR
/// to be applied in-context.
#[derive(Clone, Copy, Debug)]
pub struct ODRMacro {
    /// in milliseconds
    pub onset: [f32;2],
    /// time in ms to get to sustain
    pub decay: [f32;2],
    /// time in ms to drop to stable value
    pub release: [f32;2]
}


impl ODRMacro {
    pub fn gen(&self) -> ODR {
        let mut rng = rand::thread_rng();
        ODR {
            onset: in_range(&mut rng, self.onset[0], self.onset[1]),
            decay: in_range(&mut rng, self.decay[0], self.decay[1]),
            release: in_range(&mut rng, self.release[0], self.release[1]),
        }
    }
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
        if curr_length_samples < requested_length_samples {
            return *self
        }

        let scale_factor:f32 =  requested_length_samples as f32 / curr_length_samples as f32;

        Self {
            onset: self.onset * scale_factor,
            decay: self.decay * scale_factor,
            release: self.release * scale_factor,
        }
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
    base_odr:&ODR
) -> SampleBuffer {
    let n_samples = time::samples_of_line(cps, line);
    let mut contour:SampleBuffer = Vec::with_capacity(n_samples);
    let applied_peak = peak.clamp(1f32, MAX_REGISTER as f32 - MIN_REGISTER as f32);


    for (i, note) in (*line).iter().enumerate() {
        let n_samples_note: usize = time::samples_of_note(cps, note);

        let dur_seconds = time::step_to_seconds(cps, &(*note).0);
        let odr:ODR = base_odr.fit_in_samples(cps, dur_seconds);

        let n_samples_ramp:usize = time::samples_of_milliseconds(cps, odr.onset);
        let n_samples_fall:usize = time::samples_of_milliseconds(cps, odr.decay);
        let n_samples_kill:usize = time::samples_of_milliseconds(cps, odr.release);
        // sustain level, boxed in by the ramp/fall/kill values
        let n_samples_hold:usize = n_samples_note - (n_samples_fall + n_samples_ramp + n_samples_kill);

        let curr_freq:f32 = note_to_freq(note);
        let stable_freq_base = *stable * curr_freq.log2();
        for j in 0..n_samples_note {
            let cutoff_freq:f32 = if j < n_samples_ramp {
                // onset
                let p = j as f32 / n_samples_ramp as f32;
                2f32.powf(applied_peak * p + stable_freq_base)
            } else if j < n_samples_ramp + n_samples_fall {
                // decay
                let p = (j - n_samples_ramp)  as f32/ n_samples_fall as f32;
                let d_sustain = p * (1f32-sustain);
                2f32.powf((applied_peak - applied_peak * d_sustain) + stable_freq_base)
            } else if j < n_samples_ramp + n_samples_fall + n_samples_hold {
                let p = (j - n_samples_ramp - n_samples_fall)as f32 / n_samples_hold as f32;
                // sustain
                2f32.powf(applied_peak * sustain + stable_freq_base)
            } else {
                // release
                let p = (j - n_samples_ramp - n_samples_fall - n_samples_hold) as f32/ n_samples_kill as f32;
                let d_sustain = (1f32-p) * (applied_peak * sustain);
                2f32.powf(d_sustain + stable_freq_base)
            };

            contour.push(cutoff_freq);
        }
    };
    contour
}




/// Given a line, 
/// define a lowpass contour behaving as an "epic brakcore emotional pad" effect
/// with a unique ODSR per-note
/// bound by the Level and ODR macros provided.
/// Guaranteed that a complete ODR will always fit in each noteevent's duration.
pub fn mask_sighwah(
    cps:f32, 
    line:&Vec<Note>, 
    level_macro:&LevelMacro, 
    odr_macro:&ODRMacro
) -> SampleBuffer {
    let n_samples = time::samples_of_line(cps, line);
    let mut contour:SampleBuffer = Vec::with_capacity(n_samples);

    for (i, note) in (*line).iter().enumerate() {
        let Levels {peak, sustain, stable} = level_macro.gen();
        let applied_peak = peak.clamp(1f32, MAX_REGISTER as f32 - MIN_REGISTER as f32);

        let n_samples_note: usize = time::samples_of_note(cps, note);

        let dur_seconds = time::step_to_seconds(cps, &(*note).0);
        let odr:ODR = (odr_macro.gen()).fit_in_samples(cps, dur_seconds);

        let n_samples_ramp:usize = time::samples_of_milliseconds(cps, odr.onset);
        let n_samples_fall:usize = time::samples_of_milliseconds(cps, odr.decay);
        let n_samples_kill:usize = time::samples_of_milliseconds(cps, odr.release);
        // sustain level, boxed in by the ramp/fall/kill values
        let n_samples_hold:usize = n_samples_note - (n_samples_fall + n_samples_ramp + n_samples_kill);

        let curr_freq:f32 = note_to_freq(note);
        let stable_freq_base = stable * curr_freq.log2();
        for j in 0..n_samples_note {
            let cutoff_freq:f32 = if j < n_samples_ramp {
                // onset
                let p = j as f32 / n_samples_ramp as f32;
                2f32.powf(applied_peak * p + stable_freq_base)
            } else if j < n_samples_ramp + n_samples_fall {
                // decay
                let p = (j - n_samples_ramp)  as f32/ n_samples_fall as f32;
                let d_sustain = p * (1f32-sustain);
                2f32.powf((applied_peak - applied_peak * d_sustain) + stable_freq_base)
            } else if j < n_samples_ramp + n_samples_fall + n_samples_hold {
                let p = (j - n_samples_ramp - n_samples_fall)as f32 / n_samples_hold as f32;
                // sustain
                2f32.powf(applied_peak * sustain + stable_freq_base)
            } else {
                // release
                let p = (j - n_samples_ramp - n_samples_fall - n_samples_hold) as f32/ n_samples_kill as f32;
                let d_sustain = (1f32-p) * (applied_peak * sustain);
                2f32.powf(d_sustain + stable_freq_base)
            };

            contour.push(cutoff_freq);
        }
    };
    contour
}


#[cfg(test)]
mod tests_odr {
    use super::*;

    #[test]
    fn test_total_samples() {
        let cps = 2.1;

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
        let cps = 2.1;
        let n_seconds = 0.1; // 1 ms

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
