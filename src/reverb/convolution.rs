use crate::synth::{SR,SampleBuffer, pi};
use crate::phrasing::contour::gen_contour;
use crate::time;
use crate::types::timbre::AmpContour;
use rustfft::{FftPlanner, num_complex::Complex};
use rand::{Rng,thread_rng, rngs::ThreadRng};

#[derive(Copy,Clone)]
pub enum Cube {
    Room,
    Hall,
    Vast 
}

#[derive(Copy,Clone)]
pub struct ReverbProfile {
    cube: Cube
}

#[derive(Copy,Clone)]
/// 
/// amp: The impulse amplitude coefficient
/// dur: Length in seconds for the impulse to live
/// rate: Decay rate for impulse. 
pub struct ReverbParams {
    pub mix: f32,
    pub amp: f32, 
    pub dur: f32,
    pub rate: f32
}

// total energy on interval [0,1] decreases by 0.5 
// for every 5x. 5dx=0.5dy

/// Produce an exponentially decaying noise sample 
fn noise_buffer(amp:f32, n_samples:usize) -> SampleBuffer {
    let mut rng = thread_rng();
    let contour = gen_contour(n_samples, 1f32, &AmpContour::Surge, true);
    (0..n_samples).map(|i| amp * contour[i] * (2f32 * rng.gen::<f32>() - 1f32)).collect()
}

#[inline]
/// equal power white noise sample generator
fn noise_sample(rng:&mut ThreadRng) -> f32 {
    2f32 * rng.gen::<f32>() - 1f32
}

#[inline]
/// natural exponential growth or decay
fn contour_sample(k:f32, t:f32) -> f32 {
    (k * t).exp().max(0.0)
}

/// Produce an exponentially decaying white noise sample 
/// amp: direct amplitude coeffecient to scale the entire signal
/// rate: standard range value mapping into decay rates from -50 (shortest, rate=0) to -5 (longest, rate=1)
/// dur: length in seconds of impulse to generate
fn gen_impulse(amp:f32, rate:f32, dur:f32) -> SampleBuffer {
    let n_samples = time::samples_of_dur(1f32, dur);
    let mut rng = thread_rng();
    let k = -50f32 + (rate * (50f32 - 5f32));
    let nf = n_samples as f32;
    (0..n_samples).map(|i| 
        amp *  contour_sample(k, i as f32 / nf) * noise_sample(&mut rng)
    ).collect()
}


fn apply(sig: &SampleBuffer, reverb_len: usize) -> SampleBuffer {
    let impulse_response = noise_buffer(0.005f32, reverb_len);
    let n = sig.len() + impulse_response.len() - 1;
    
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

    result.iter().map(|c| c.re / n as f32).collect()
}


/// Applies convolution with a noise buffer
/// onto a given signal. Intended for reverb
pub fn of(sig: &SampleBuffer, params: &ReverbParams) -> SampleBuffer {
    let impulse_response = gen_impulse(params.amp, params.rate, params.dur);
    let n = sig.len() + impulse_response.len() - 1;
    
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);
    let ifft = planner.plan_fft_inverse(n); 

    let mut sig_padded: Vec<Complex<f32>> = sig.iter().cloned().map(|s| Complex::new(s, 0.0)).collect();
    sig_padded.resize(n, Complex::new(0.0, 0.0));

    let mut ir_padded: Vec<Complex<f32>> = impulse_response.iter().cloned().map(|s| Complex::new(s, 0.0)).collect();
    ir_padded.resize(n, Complex::new(0.0, 0.0));

    fft.process(&mut sig_padded);
    fft.process(&mut ir_padded);

    let mut          result = vec![Complex::new(0.0, 0.0); n];
    for i in 0..n {
        result[i] = sig_padded[i] * ir_padded[i];
    }

    ifft.process(&mut result);

    // Normalize the result by n and create the wet signal
    let wet_signal: SampleBuffer = result.iter().map(|c| c.re / n as f32).collect();

    // Mix dry and wet signals
    let mut mixed_signal: SampleBuffer = vec![0.0; sig.len()];
    for i in 0..sig.len() {
        mixed_signal[i] = (1.0 - params.mix) * sig[i] + params.mix * wet_signal[i];
    }

    mixed_signal
}


/// Mix dry and wet signals using Fourier transform for fast convolution.
/// `sig` - input signal buffer
/// `dur_seconds` - duration of reverb in seconds
/// `gain` - gain for the reverb
/// `wet` - mix ratio for wet signal (0.0 = fully dry, 1.0 = fully wet)
fn mix(sig: &SampleBuffer, dur_seconds: f32, gain: f32, wet: f32) -> SampleBuffer {
    /*
     * 
     * observations:
     * 
     * context, covolution of complete sequenced melody and H(t) = white noise filter with exponential decay k=-5 length=8seconds 
     * when applied to the "chords" (lower register) part in the x files demo,
     * this behaved more like a saturation effect than a blur effect. 
     * It did not sound at all like reverb. It did sound thicker and way richer. 
     * 
     * 
     */
    let reverb_len = time::samples_of_dur(1f32, dur_seconds);
    let impulse_response = noise_buffer(gain, reverb_len);
    let n = sig.len() + impulse_response.len() - 1;

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

    let wet_signal: SampleBuffer = result.iter().map(|c| c.re / n as f32).collect();
    let mut mixed_signal: SampleBuffer = vec![0.0; sig.len()];
    for i in 0..sig.len() {
        mixed_signal[i] = (1.0 - wet) * sig[i] + wet * wet_signal[i];
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::files::{self, with_dir};
    use crate::render::engrave;
    static TEST_GROUP:&str = "reverb-convolution";
    fn out_dir() -> String { format!("dev-audio/{}", TEST_GROUP) }
    fn setup() {
        files::with_dir(&out_dir())
    }

    fn generate_major_chord(frequencies: &[f32], sample_rate: usize, duration: f32) -> SampleBuffer {
        let n_samples = (sample_rate as f32 * duration) as usize;
        let mut buffer = vec![0f32; n_samples];
        let n = n_samples as f32;
        for &freq in frequencies {
            for (i, sample) in buffer.iter_mut().enumerate() {
                let t = i as f32 / n;
                let envelope = 0.2 * (-5.0 * t).exp().max(0.0); 
                *sample += (2.0f32 * pi * freq * t).sin() * envelope;
            }
        }
    
        buffer
    }

    fn gen_signal() -> SampleBuffer {
        let duration = 1.0; 
        let frequencies = [400.0, 500.0, 600.0];
        let samples = generate_major_chord(&frequencies, SR, duration);
        let filename = format!("{}/dry.wav", &out_dir());

        engrave::samples(SR, &samples, &filename);
        samples

    }

    #[test]
    fn test_apply_convolution_reverb_spring() {
        setup();
        let test_name = "springverb";
        let reverb_len = 1000; 

        let major_chord = gen_signal();
        let wet_signal = apply(&major_chord, reverb_len);
        assert_eq!(wet_signal.len(), major_chord.len() + reverb_len - 1);
        let non_zero_samples: Vec<&f32> = wet_signal.iter().filter(|&&x| x != 0.0).collect();
        assert!(!non_zero_samples.is_empty(), "Wet signal should not be all zeros");

        let filename2 = format!("{}/wet-{}.wav", &out_dir(), test_name);
        engrave::samples(SR, &wet_signal, &filename2);
    }

    #[test]
    fn test_apply_convolution_reverb_samelen() {
        setup();
        let test_name = "sameverb";

        let major_chord = gen_signal();
        let reverb_len = major_chord.len(); 
        let wet_signal = apply(&major_chord, reverb_len);
        assert_eq!(wet_signal.len(), major_chord.len() + reverb_len - 1);
        let non_zero_samples: Vec<&f32> = wet_signal.iter().filter(|&&x| x != 0.0).collect();
        assert!(!non_zero_samples.is_empty(), "Wet signal should not be all zeros");

        let filename2 = format!("{}/wet-{}.wav", &out_dir(), test_name);
        engrave::samples(SR, &wet_signal, &filename2);
    }
    #[test]
    fn test_apply_convolution_reverb_SRlen() {
        setup();
        let test_name = "longverb";

        let major_chord = gen_signal();   
        let reverb_len = SR*8;
        let wet_signal = apply(&major_chord, reverb_len);
        assert_eq!(wet_signal.len(), major_chord.len() + reverb_len - 1);
        let non_zero_samples: Vec<&f32> = wet_signal.iter().filter(|&&x| x != 0.0).collect();
        assert!(!non_zero_samples.is_empty(), "Wet signal should not be all zeros");

        let filename2 = format!("{}/wet-{}.wav", &out_dir(), test_name);
        engrave::samples(SR, &wet_signal, &filename2);
    }
}