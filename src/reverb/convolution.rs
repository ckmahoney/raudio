use crate::druid::{soid_fx, soids as some_soids};
use crate::phrasing::contour::gen_contour;
use crate::synth::{pi, SampleBuffer, SR};
use crate::time;
use crate::types::timbre::AmpContour;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use rustfft::{num_complex::Complex, FftPlanner};

#[derive(Copy, Clone)]
pub enum Cube {
  Room,
  Hall,
  Vast,
}

#[derive(Copy, Clone)]
pub struct ReverbProfile {
  cube: Cube,
}

#[derive(Copy, Clone, Debug)]
///
/// amp: The impulse amplitude coefficient
/// dur: Length in seconds for the impulse to live
/// rate: Decay rate for impulse.
pub struct ReverbParams {
  pub mix: f32,
  pub amp: f32,
  pub dur: f32,
  pub rate: f32,
}
#[inline]
/// equal power white noise sample generator
fn noise_sample(rng: &mut ThreadRng) -> f32 {
  2f32 * rng.gen::<f32>() - 1f32
}

#[inline]
/// natural exponential growth or decay
fn contour_sample(k: f32, t: f32) -> f32 {
  (k * t).exp().max(0.0)
}

/// Produce an exponentially decaying white noise sample  
/// amp: direct amplitude coeffecient to scale the entire signal  
/// rate: standard range value mapping into decay rates from -50 (shortest, rate=0) to -5 (longest, rate=1)  
/// dur: length in seconds of impulse to generate  
fn gen_impulse(amp: f32, rate: f32, dur: f32) -> SampleBuffer {
  let n_samples = time::samples_of_dur(1f32, dur);
  let mut rng = thread_rng();
  let k = -50f32 + (rate * (50f32 - 5f32));
  let nf = n_samples as f32;
  (0..n_samples).map(|i| amp * contour_sample(k, i as f32 / nf) * noise_sample(&mut rng)).collect()
}

/// Applies convolution with a noise buffer
/// onto a given signal. Here it genereates an impulse response to produce a reverberation effect.
pub fn of(sig: &SampleBuffer, params: &ReverbParams) -> SampleBuffer {
  let impulse_response = gen_impulse(params.amp, params.rate, params.dur);
  let n = sig.len() + impulse_response.len();

  let mut planner = FftPlanner::new();
  let fft = planner.plan_fft_forward(n);
  let ifft = planner.plan_fft_inverse(n);

  let mut sig_padded: Vec<Complex<f32>> = sig.iter().cloned().map(|s| Complex::new(s, 0.0)).collect();
  sig_padded.resize(n, Complex::new(0.0, 0.0));

  let mut ir_padded: Vec<Complex<f32>> = impulse_response.iter().cloned().map(|s| Complex::new(s, 0.0)).collect();
  ir_padded.resize(n, Complex::new(0.0, 0.0));

  fft.process(&mut sig_padded);
  fft.process(&mut ir_padded);

  let mut result = vec![Complex::new(0.0, 0.0); n];
  for i in 0..n {
    result[i] = sig_padded[i] * ir_padded[i];
  }

  ifft.process(&mut result);

  // Normalize the result by n and create the wet signal
  let wet_signal: SampleBuffer = result.iter().map(|c| c.re / n as f32).collect();

  // Mix dry and wet signals
  let mut mixed_signal: SampleBuffer = vec![0.0; n];
  for i in 0..sig.len() {
    mixed_signal[i] = (1.0 - params.mix) * sig[i] + params.mix * wet_signal[i];
  }

  mixed_signal
}

fn pad_buffers(signal: &SampleBuffer, impulse_response: &SampleBuffer) -> (SampleBuffer, SampleBuffer) {
  let mut padded_signal = signal.clone();
  let mut padded_ir = impulse_response.clone();

  if impulse_response.len() < signal.len() {
    padded_ir.append(&mut vec![0f32; signal.len() - impulse_response.len()]);
  } else if signal.len() < impulse_response.len() {
    padded_signal.append(&mut vec![0f32; impulse_response.len() - signal.len()]);
  }

  (padded_signal, padded_ir)
}
