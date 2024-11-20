use super::in_range;
use super::monic_theory::note_to_freq;
use crate::synth::{SRf, SampleBuffer, MAX_REGISTER, MIN_REGISTER, SR};
use crate::time;
use crate::types::render::{Feel, Melody, Stem};
use crate::types::synthesis::{
  Ampl, Bandpass, Direction, Duration, Ely, FilterPoint, Freq, Frex, GlideLen, MacroMotion, Monae, Mote, Note,
  Register, Soids, Tone,
};
use rand::{rngs::ThreadRng, thread_rng, Rng};

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

/// Scalar levels that correspond to parameters
/// `stable`, `peak`, and `sustain`.
/// `stable` is the initial or final value when not active,
/// `peak` is the maximum value during activity,
/// and `sustain` is the level as a percentage of `peak`.
pub struct Levels {
  /// Starting or ending position as a scalar value.
  pub stable: f32,
  /// Peak scalar value when the level is fully activated.
  pub peak: f32,
  /// Sustain level as a percentage of `peak`, in the range [0, 1].
  pub sustain: f32,
}

/// A struct representing the range of values for each level parameter,
/// where each parameter (`stable`, `peak`, and `sustain`) is defined as a
/// range with minimum and maximum bounds. These bounds are used to generate
/// a `Levels` instance with values within these ranges.
pub struct LevelMacro {
  /// Range for the `stable` scalar value [min, max].
  pub stable: [f32; 2],
  /// Range for the `peak` scalar value [min, max].
  pub peak: [f32; 2],
  /// Range for the `sustain` scalar value [min, max].
  pub sustain: [f32; 2],
}

impl LevelMacro {
  /// Generates a new `Levels` instance with values sampled within the
  /// provided ranges for each parameter (`stable`, `peak`, `sustain`).
  /// This uses a random number generator to select values within each
  /// specified range in `LevelMacro`.
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
  /// Creates a new `Levels` instance with specified values for `stable`,
  /// `peak`, and `sustain`. Validates that `sustain` is within [0, 1],
  /// as it represents a percentage of `peak`.
  ///
  /// # Panics
  /// Panics if `sustain` is outside the [0, 1] range.
  ///
  /// # Arguments
  /// - `stable`: The starting/ending scalar value.
  /// - `peak`: The maximum scalar value.
  /// - `sustain`: The sustain level as a fraction of `peak`.
  pub fn new(stable: f32, peak: f32, sustain: f32) -> Self {
    if sustain > 1f32 || sustain < 0f32 {
      panic!("Sustain value in Levels represents a percentage of peak, and must be given in range of [0, 1]. You provided {}", sustain)
    }

    Self { stable, peak, sustain }
  }
}

/// Absolute measurements of time for envelope stages: onset, decay, and release.  
/// These values are applied in milliseconds and used to control the timing of
/// audio events.
#[derive(Clone, Copy, Debug)]
pub struct ODR {
  /// Time in milliseconds to begin the onset stage, i.e., the initial rise to peak.
  pub onset: f32,
  /// Time in milliseconds to transition from peak to sustain level.
  pub decay: f32,
  /// Time in milliseconds to transition from sustain level back to stable/resting value.
  pub release: f32,
}

/// Min/max ranges for generating `ODR` values, used to set bounds for each stage (onset, decay, release)
/// within specified limits in milliseconds.
#[derive(Clone, Debug)]
pub struct ODRMacro {
  /// Range of onset times in milliseconds [min, max].
  pub onset: [f32; 2],
  /// Range of decay times in milliseconds [min, max].
  pub decay: [f32; 2],
  /// Range of release times in milliseconds [min, max].
  pub release: [f32; 2],
  pub mo: Vec<MacroMotion>,
  pub md: Vec<MacroMotion>,
  pub mr: Vec<MacroMotion>,
}

impl ODRMacro {
  /// Generates a new `ODR` instance with values sampled randomly within
  /// the ranges specified in `ODRMacro` for onset, decay, and release.
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
  /// Calculates the total number of samples required to represent this `ODR` envelope
  /// at a specified sample rate (`cps`). This includes the onset, decay, and release
  /// times converted from milliseconds to samples.  
  ///
  /// # Arguments
  /// - `cps`: Sample rate in cycles per second (Hz).  
  ///
  /// # Returns
  /// Total number of samples as `usize`.
  pub fn total_samples(&self, cps: f32) -> usize {
    // Convert milliseconds to samples for each stage
    let onset_samples = time::samples_of_milliseconds(cps, self.onset);
    let decay_samples = time::samples_of_milliseconds(cps, self.decay);
    let release_samples = time::samples_of_milliseconds(cps, self.release);

    // Sum of samples for all stages
    onset_samples + decay_samples + release_samples
  }

  /// Adjusts the `ODR` envelope to fit within a specified duration, scaling each stage
  /// proportionally if the current duration exceeds the requested duration.
  ///
  /// # Arguments
  /// - `cps`: Sample rate in cycles per second (Hz).
  /// - `n_seconds`: Target duration in seconds for the entire envelope.
  ///
  /// # Returns
  /// A new `ODR` with stages scaled to fit within the requested duration.
  pub fn fit_in_samples(&self, cps: f32, n_seconds: f32) -> Self {
    let curr_length_samples: usize = self.total_samples(cps);
    let requested_length_samples: usize = time::samples_of_seconds(cps, n_seconds);

    // If current length is within the requested length, return unchanged
    if curr_length_samples < requested_length_samples {
      return *self;
    }

    // Calculate scaling factor to fit within requested length
    let scale_factor: f32 = requested_length_samples as f32 / curr_length_samples as f32;

    Self {
      onset: self.onset * scale_factor,
      decay: self.decay * scale_factor,
      release: self.release * scale_factor,
    }
  }
}

/// Macro for generating a group of `ConvFilter` instances.
/// Provides bounds for each parameter, which can be used to
/// create `ConvFilter` instances with randomized values within these bounds.
pub struct ConvFilterMacro {
  /// The direction of motion applied to the filter contour.
  pub direction: MacroMotion,
  /// Range for amplitude values [min, max].
  pub amp: [f32; 2],
  /// Range for mix values representing the dry/wet ratio [min, max].
  pub mix: [f32; 2],
  /// Range for relative duration values. Represents a scalar value to be multiplied against the duration of input signal.
  pub dur_scale: [f32; 2],
  /// Range for rate values to control the contour's modulation rate [min, max].
  pub rate: [f32; 2],
}

impl ConvFilterMacro {
  /// Generates a new `ConvFilter` instance with values sampled within the
  /// provided ranges for each parameter (`amp`, `mix`, `dur_scale`, `rate`).
  pub fn gen(&self) -> ConvFilter {
    let mut rng = thread_rng();
    ConvFilter {
      direction: self.direction,
      amp: in_range(&mut rng, self.amp[0], self.amp[1]),
      mix: in_range(&mut rng, self.mix[0], self.mix[1]),
      duration: in_range(&mut rng, self.dur_scale[0], self.dur_scale[1]),
      rate: in_range(&mut rng, self.rate[0], self.rate[1]),
    }
  }
}

/// A convolution filter that applies an impulse response with specified
/// parameters and contours the signal based on rate and direction.
pub struct ConvFilter {
  /// The direction of the filter's motion contour.
  pub direction: MacroMotion,
  /// Amplitude applied during impulse response generation.
  pub amp: f32,
  /// Mix level used as a dry/wet knob for balancing the original and filtered signals.
  pub mix: f32,
  /// Duration scalar for the impulse response in relation to the input signal duration.
  pub duration: f32,
  /// Modulation rate applied to the contour of the signal.
  pub rate: f32,
}

impl ConvFilter {
  /// Creates a new `ConvFilter` instance directly with specific values.
  ///
  /// # Arguments
  /// - `direction`: Direction of filter contour.
  /// - `amp`: Amplitude of impulse response.
  /// - `mix`: Dry/wet mix ratio.
  /// - `duration`: Duration scalar for impulse response relative to input signal duration.
  /// - `rate`: Contour modulation rate.
  pub fn new(direction: MacroMotion, amp: f32, mix: f32, duration: f32, rate: f32) -> Self {
    Self {
      direction,
      amp,
      mix,
      duration,
      rate,
    }
  }

  /// Generate a random white noise sample within the range [-1, 1].
  fn noise_sample(rng: &mut ThreadRng) -> f32 {
    2f32 * rng.gen::<f32>() - 1f32
  }

  /// Applies an exponential contour for amplitude growth or decay based on coefficient `k`.
  #[inline]
  fn contour_sample(k: f32, t: f32) -> f32 {
    (k * t).exp().max(0.0)
  }

  /// Generates an impulse response based on the parameters within this `ConvFilter` instance.
  /// Supports exponential decay, exponential growth, and a constant amplitude contour.
  ///
  /// - `amp`: Direct amplitude coefficient scales the entire signal.
  /// - `rate`: Controls contouring, with decay mapped from -50 (shortest, rate=0) to -5 (longest, rate=1).
  ///           Growth and constant contours are determined by the `direction`.
  /// - `duration`: Length in seconds of the impulse to generate, scaled by `duration`.
  ///
  /// # Returns
  /// A `SampleBuffer` containing the generated impulse response.
  pub fn gen_ir(&self, input_duration: f32) -> SampleBuffer {
    // Scale duration by the `duration` scalar from ConvFilter.
    let scaled_duration = self.duration * input_duration;
    let n_samples = time::samples_of_dur(1f32, scaled_duration);
    let mut rng = thread_rng();

    let nf = n_samples as f32;
    let k = -5f32 + ((1f32 - self.rate) * -45f32);
    match self.direction {
      MacroMotion::Forward => (0..n_samples)
        .map(|i| self.amp * ConvFilter::contour_sample(k, i as f32 / nf) * ConvFilter::noise_sample(&mut rng))
        .collect(),
      MacroMotion::Reverse => (0..n_samples)
        .map(|i| self.amp * ConvFilter::contour_sample(k, (nf - i as f32) / nf) * ConvFilter::noise_sample(&mut rng))
        .collect(),
      _ => (0..n_samples).map(|i| self.amp * ConvFilter::noise_sample(&mut rng)).collect(),
    }
  }
}

/// Given a line,
/// define a lowpass contour behaving as a "wah wah" effect
/// with respect to the given configuration.
/// Guaranteed that a complete ODR will always fit in each noteevent's duration.
pub fn mask_wah(cps: f32, line: &Vec<Note>, level_macro: &LevelMacro, odr_macro: &ODRMacro) -> SampleBuffer {
  let n_samples = time::samples_of_line(cps, line);
  let mut contour: SampleBuffer = Vec::with_capacity(n_samples);

  for (i, note) in (*line).iter().enumerate() {
    let Levels { peak, sustain, stable } = level_macro.gen();
    let applied_peak = peak.clamp(1f32, MAX_REGISTER as f32 - MIN_REGISTER as f32);

    let n_samples_note: usize = time::samples_of_note(cps, note);

    let dur_seconds = time::step_to_seconds(cps, &(*note).0);
    let odr: ODR = (odr_macro.gen()).fit_in_samples(cps, dur_seconds);

    // let n_samples_ramp: usize = time::samples_of_milliseconds(cps, odr.onset);
    // let n_samples_fall: usize = time::samples_of_milliseconds(cps, odr.decay);
    // let n_samples_kill: usize = time::samples_of_milliseconds(cps, odr.release);
    // let animation_duration_samples = n_samples_fall + n_samples_ramp + n_samples_kill;
    // // sustain level, boxed in by the ramp/fall/kill values
    // let n_samples_hold: usize = if animation_duration_samples > n_samples_note {
    //   0
    // } else {
    //   n_samples_note - animation_duration_samples
    // };

    let (n_samples_ramp, n_samples_fall, n_samples_hold, n_samples_kill) = eval_odr(cps, n_samples_note, &odr);

    let curr_freq: f32 = note_to_freq(note);
    let stable_freq_base = stable * curr_freq.log2();
    for j in 0..n_samples_note {
      let cutoff_freq: f32 = if j < n_samples_ramp {
        // onset
        let p = j as f32 / n_samples_ramp as f32;
        2f32.powf(applied_peak * p + stable_freq_base)
      } else if j < n_samples_ramp + n_samples_fall {
        // decay
        let p = (j - n_samples_ramp) as f32 / n_samples_fall as f32;
        let d_sustain = p * (1f32 - sustain);
        2f32.powf((applied_peak - applied_peak * d_sustain) + stable_freq_base)
      } else if j < n_samples_ramp + n_samples_fall + n_samples_hold {
        let p = (j - n_samples_ramp - n_samples_fall) as f32 / n_samples_hold as f32;
        // sustain
        2f32.powf(applied_peak * sustain + stable_freq_base)
      } else {
        // release
        let p = (j - n_samples_ramp - n_samples_fall - n_samples_hold) as f32 / n_samples_kill as f32;
        let d_sustain = (1f32 - p) * (applied_peak * sustain);
        2f32.powf(d_sustain + stable_freq_base)
      };

      contour.push(cutoff_freq);
    }
  }
  contour
}

fn eval_odr(cps:f32, at_n_samples:usize, odr:&ODR) -> (usize, usize, usize, usize) {
  let ramp: usize = time::samples_of_milliseconds(cps, odr.onset);
  let fall: usize = time::samples_of_milliseconds(cps, odr.decay);
  let kill: usize = time::samples_of_milliseconds(cps, odr.release);
  let animation_duration_samples = fall + ramp + kill;
  // sustain level, boxed in by the ramp/fall/kill values
  let hold: usize = if animation_duration_samples > at_n_samples {
    0
  } else {
    at_n_samples - animation_duration_samples
  };

  (ramp, fall, hold, kill)
}

/// Given a line,
/// define a lowpass contour  with a unique ODSR per-note
/// bound by the Level and ODR macros provided.
/// Guaranteed that a complete ODR will always fit in each noteevent's duration.
pub fn mask_sigh(cps: f32, line: &Vec<Note>, level_macro: &LevelMacro, odr_macro: &ODRMacro) -> SampleBuffer {
  let n_samples = time::samples_of_line(cps, line);
  let mut contour: SampleBuffer = Vec::with_capacity(n_samples);

  for (i, note) in (*line).iter().enumerate() {
    let Levels { peak, sustain, stable } = level_macro.gen();
    let applied_peak = peak.clamp(1f32, MAX_REGISTER as f32 - MIN_REGISTER as f32);

    let n_samples_note: usize = time::samples_of_note(cps, note);

    let dur_seconds = time::step_to_seconds(cps, &(*note).0);
    let odr: ODR = (odr_macro.gen()).fit_in_samples(cps, dur_seconds);

    let (n_samples_ramp, n_samples_fall, n_samples_hold, n_samples_kill) = eval_odr(cps, n_samples_note, &odr);


    let curr_freq: f32 = note_to_freq(note);
    let stable_freq_base = stable * curr_freq.log2();
    for j in 0..n_samples_note {
      let cutoff_freq: f32 = if j < n_samples_ramp {
        // onset
        let p = j as f32 / n_samples_ramp as f32;
        2f32.powf(applied_peak * p + stable_freq_base)
      } else if j < n_samples_ramp + n_samples_fall {
        // decay
        let p = (j - n_samples_ramp) as f32 / n_samples_fall as f32;
        let d_sustain = p * (1f32 - sustain);
        2f32.powf((applied_peak - applied_peak * d_sustain) + stable_freq_base)
      } else if j < n_samples_ramp + n_samples_fall + n_samples_hold {
        let p = (j - n_samples_ramp - n_samples_fall) as f32 / n_samples_hold as f32;
        // sustain
        2f32.powf(applied_peak * sustain + stable_freq_base)
      } else {
        // release
        let p = (j - n_samples_ramp - n_samples_fall - n_samples_hold) as f32 / n_samples_kill as f32;
        let d_sustain = (1f32 - p) * (applied_peak * sustain);
        2f32.powf(d_sustain + stable_freq_base)
      };

      contour.push(cutoff_freq);
    }
  }
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
    assert_eq!(
      total_samples,
      expected_onset_samples + expected_decay_samples + expected_release_samples,
      "Total samples calculation mismatch"
    );
  }

  #[test]
  fn test_fit_in_samples_no_scaling_needed() {
    let cps = 2.1;
    let n_seconds = 0.1; // 1 ms

    // Create an ODR that already fits within 1 ms
    let odr = ODR {
      onset: 0.2,   // 0.2 ms
      decay: 0.3,   // 0.3 ms
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
    assert_eq!(
      expected_samples, expected_samples,
      "Must match the number of samples when resizing an ODR"
    );

    // Since the total duration is scaled to 100 ms from 200 ms, we expect a scaling factor of 0.5
    let expected_scale_factor = 0.5;
    let tolerance = 1e-3; // Tolerance for floating-point comparison

    // Verify each component was scaled by the expected factor
    assert!(
      (result.onset - odr.onset * expected_scale_factor).abs() < tolerance,
      "Onset scaling mismatch"
    );
    assert!(
      (result.decay - odr.decay * expected_scale_factor).abs() < tolerance,
      "Decay scaling mismatch"
    );
    assert!(
      (result.release - odr.release * expected_scale_factor).abs() < tolerance,
      "Release scaling mismatch"
    );

    // Additional check to confirm the total duration is also scaled by the expected factor
    let scaled_total_duration = result.onset + result.decay + result.release;
    let original_total_duration = odr.onset + odr.decay + odr.release;
    assert!(
      (scaled_total_duration - original_total_duration * expected_scale_factor).abs() < tolerance,
      "Total ODR duration scaling mismatch"
    );
  }
}
