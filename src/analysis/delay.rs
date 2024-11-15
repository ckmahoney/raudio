use crate::time;
use crate::MacroMotion;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;

use crate::analysis::{in_range, in_range_usize};

/// Represents the stereo position in a mix, indicating the panning of the audio signal.
/// Variants allow for mono or varying degrees of stereo positioning, either left, right,
/// or with distinct levels for both left and right channels.
#[derive(Copy, Clone, Debug)]
pub enum StereoField {
    /// Mono signal with equal amplitude in a single channel.
    Mono,
    /// Pans the signal to the left with specified amplitude.
    Left(f32),
    /// Pans the signal to the right with specified amplitude.
    Right(f32),
    /// Allows different amplitudes for the left and right channels.
    LeftRight(f32, f32),
}

/// Parameters for configuring a delay effect, including duration, echo count, and
/// gain and mix levels, along with stereo panning options.
///
/// `DelayParams` allows for setting up complex delay behaviors, including the number of echoes,
/// balance between original and delayed signal, and stereo positioning.
#[derive(Copy, Clone, Debug)]
pub struct DelayParams {
    /// Total length of the delay in seconds.
    pub len_seconds: f32,
    /// Number of echo artifacts to create within the delay.
    pub n_echoes: usize,
    /// Gain applied to each echo, controlling the decay of echo intensity.
    pub gain: f32,
    /// Mix level between the original and delayed signal (0.0 = fully dry, 1.0 = fully wet).
    pub mix: f32,
    /// Stereo positioning of the delayed signal.
    pub pan: StereoField,
}

/// Macro struct for generating `DelayParams` instances with specified ranges, motions,
/// and a list of delay rates as cycle ratios. This allows the selection of different delay times
/// based on the cycles-per-second (`cps`) input.
pub struct DelayParamsMacro {
  /// Range for gain values to be applied to each echo [min, max].
  pub gain: [f32; 2],
  /// List of delay ratios representing cycle multiples (e.g., 1/4, 1/8, 2),
  /// which are used to determine delay lengths in seconds.
  pub dtimes_cycles: Vec<f32>,
  /// Range for the number of echoes (artifacts) to generate per delay [min, max].
  pub n_echoes: [usize; 2],
  /// Range for the mix level, defining the balance between the original and delayed signal [min, max].
  pub mix: [f32; 2],
  /// Stereo panning positions to be applied to each echo.
  pub pan: Vec<StereoField>,

  /// Macro motions for controlling delay time offsets.
  pub mecho: Vec<MacroMotion>,
  /// Macro motions for controlling gain behavior.
  pub mgain: Vec<MacroMotion>,
  /// Macro motions for controlling pan behavior.
  pub mpan: Vec<MacroMotion>,
  /// Macro motions for controlling mix behavior.
  pub mmix: Vec<MacroMotion>,
}

impl DelayParamsMacro {
  /// Generates a new `DelayParams` instance by selecting a delay ratio from `dtimes_cycles`,
  /// and calculating the delay length in seconds based on `cps`.
  ///
  /// # Arguments
  /// - `rng`: A random number generator to sample values within ranges.
  /// - `cps`: Cycles per second rate, used to convert delay cycles to seconds.
  ///
  /// # Returns
  /// A `DelayParams` instance populated with values derived from `DelayParamsMacro`.
  pub fn gen(&self, rng: &mut ThreadRng, cps: f32) -> DelayParams {
      // Select a delay ratio from `dtimes_cycles` to use for the delay time.
      let delay_len_cycles = *self.dtimes_cycles.choose(rng).unwrap_or(&1.0);
      
      // Calculate the delay length in seconds based on the selected ratio and `cps`.
      let len_seconds = delay_len_cycles / cps;

      // Sample other parameters within their ranges.
      let gain = in_range(rng, self.gain[0], self.gain[1]);
      let n_echoes = in_range_usize(rng, self.n_echoes[0], self.n_echoes[1]);
      let mix = in_range(rng, self.mix[0], self.mix[1]);
      
      // Select a panning position randomly from the available options in `pan`.
      let pan = *self.pan.choose(rng).unwrap_or(&StereoField::Mono);

      DelayParams {
          len_seconds,
          n_echoes,
          gain,
          mix,
          pan,
      }
  }
}

pub fn is_passthrough(params: &DelayParams) -> bool {
  params.mix == 0f32 || params.len_seconds == 0f32 || params.gain == 0f32 || params.n_echoes == 0
}

pub static passthrough: DelayParams = DelayParams {
  mix: 0f32,
  len_seconds: 0f32,
  gain: 0f32,
  n_echoes: 0,
  pan: StereoField::Mono,
};

/// determine the amplitude coeffecieint for a delay replica index
#[inline]
pub fn gain(j: usize, replica: usize, params: &DelayParams) -> f32 {
  if replica == 0 || is_passthrough(params) {
    return 1f32;
  }
  let samples_per_echo: usize = time::samples_from_dur(1f32, params.len_seconds);
  let min_distance = samples_per_echo * replica;
  if j < min_distance {
    return 0f32;
  }

  params.mix * params.gain.powi(replica as i32)
}

/// Given a delay params, identify the new duration of the sample for a given context.
pub fn length(cps: f32, dur: f32, params: &DelayParams) -> usize {
  let samples_per_echo: usize = time::samples_from_dur(1f32, params.len_seconds);
  let max_distance = samples_per_echo * params.n_echoes;
  max_distance
}

mod test {
  use super::*;

  fn test_with_outlived_final_note() {
    let durs: Vec<f32> = vec![2f32, 3f32, 10f32, 2f32];
    let durs: Vec<f32> = vec![2f32, 3f32, 10f32, 10f32];
    let durs: Vec<f32> = vec![2f32, 3f32, 10f32, 11f32];

    let params = DelayParams {
      mix: 0.5f32,
      gain: 0.99,
      len_seconds: 1f32,
      n_echoes: 5,
      pan: StereoField::Mono,
    };

    let total_dur: f32 = durs.iter().sum();
  }
}
