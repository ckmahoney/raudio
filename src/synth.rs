//! Generation of novel audio signals 
use itertools::Either;

use crate::convolve;
use crate::freq_forms;
use crate::render;
use crate::synth_config;
use crate::envelope::Envelope;
use crate::modulate;
use crate::mix;
use crate::phrase;
use crate::synth_config::SynthConfig;
pub enum HarmonicSelector {
    All,
    Odd,
    Even,
    Geometric(f32),
    Constant(f32),
}

impl HarmonicSelector {
    pub fn generate_harmonics(&self, start: usize, end: usize, offset: f32) -> Vec<f32> {
        match self {
            HarmonicSelector::All => (start..=end).map(|x| x as f32 + offset).collect(),
            HarmonicSelector::Odd => (start..=end).filter(|x| x % 2 != 0).map(|x| x as f32 + offset).collect(),
            HarmonicSelector::Even => (start..=end).filter(|x| x % 2 == 0).map(|x| x as f32 + offset).collect(),
            HarmonicSelector::Geometric(ratio) => (start..=end).map(|x| ratio.powi(x as i32) + offset).collect(),
            HarmonicSelector::Constant(value) => vec![*value + offset; end - start + 1],
        }
    }
}

// sample_rate, sample_num, frequency
type Ugen = fn(usize, usize, f32) -> f32;

pub type SampleBuffer = Vec<f32>;

pub struct RenderConfig {
    pub sample_rate: usize,
    pub amplitude_scaling: f32,
    pub cps: f32
}

pub fn silly_sine(sample_rate:usize, sample_num:usize, frequency:f32) -> f32 {
    let pi2 = std::f32::consts::PI * 2.0f32;
    let samples_per_period = (sample_rate as f32 / frequency) as f32;
    let sample_index = sample_num.rem_euclid(samples_per_period as usize) as f32;
    let phase:f32 = pi2 * (sample_index /samples_per_period);
    (pi2 * frequency + phase).sin()
}

// !! note this may truncate a partial sample for non-harmonic cps. 
// This eventually leads to frequency drift
// it is considerable to correct the drift by factoring the dt lost and total number of samples lost
pub fn sample_ugen(config:&RenderConfig, ugen:Ugen, duration:f32, freq:f32) -> SampleBuffer {
    let samples_per_cycle:f32 = config.sample_rate as f32 / config.cps;
    let n_samples = (samples_per_cycle * duration).floor() as usize;
    (0..n_samples-1).map({|i| 
            config.amplitude_scaling * ugen(config.sample_rate, i, freq)
    }).collect()
}

pub fn sample_period(config:&RenderConfig, ugen:Ugen, freq:f32) -> SampleBuffer {
    let samples_per_period = (config.sample_rate as f32 / freq) as usize;
    (0..samples_per_period-1).map({|i| 
        config.amplitude_scaling * ugen(config.sample_rate, i, freq)
    }).collect()
}

// !! note 
// this implementation may cause "sync" 
// in that the final copied period of a shorter-than-longest samples
// may not have a harmonic length to the longest samples. 
// therefore the final iterated loop will not copy the entire period, 
// resulting in sync. 
pub fn silly_sum_periods(config:&RenderConfig, freqs:&Vec<f32>, periods:&Vec<SampleBuffer>) -> SampleBuffer {
    
    let n_samples = periods.iter().map(|vec| vec.len()).max().unwrap_or(0);
    let mut result = vec![0.0; n_samples];

    for i in 0..n_samples-1 {
        for period in periods.to_owned() {
            let index:usize = i % period.len();
            result[i] += period[index];
        };
    };
    render::normalize(&mut result);
    render::amp_scale(&mut result, config.amplitude_scaling);
    result
}

pub fn silly_convolve_periods(periods: &[SampleBuffer]) -> SampleBuffer {
    if periods.is_empty() {
        return Vec::new();
    }
    let longest_length = periods.iter().map(Vec::len).max().unwrap_or_default();
    let initial_period = &periods[0];
    let convolved_result = periods.iter().skip(1).fold(initial_period.clone(), |acc, period| {
        convolve::full(&acc, period)
    });
    let mut resampled = convolve::resample(&convolved_result, longest_length);
    render::normalize(&mut resampled);
    resampled
}


#[test] 
fn test_sample_ugen() {
    let config = RenderConfig {
        sample_rate: 44100,
        amplitude_scaling: 1.0,
        cps: 1.0,
    };

    let frequencies = [100.0, 600.0, 3000.0];

    for &freq in &frequencies {
        let result = sample_ugen(&config, silly_sine, 8.0, freq);
        let filename = format!("dev-audio/test-sample-ugen-{}-hz.wav", freq);
        render::samples_f32(config.sample_rate, &result, &filename);
    }
}

#[test] 
fn test_sample_period() {
    let config = RenderConfig {
        sample_rate: 44100,
        amplitude_scaling: 1.0,
        cps: 1.0,
    };

    let frequencies = [1.0, 3.0, 5.0, 7.0, 9.0];

    for &freq in &frequencies {
        let result = sample_period(&config, silly_sine, freq);
        let filename = format!("dev-audio/test-sample-period-{}-hz.wav", freq);
        render::samples_f32(config.sample_rate, &result, &filename);
    }
}

#[test] 
fn test_silly_sum_periods() {
    let config = RenderConfig {
        sample_rate: 44100,
        amplitude_scaling: 1.0,
        cps: 1.0,
    };
    
    let n:usize = 9;
    let frequencies:Vec<f32> = (0..n).filter(|x| x % 2 != 0).map(|x| x as f32).collect();
    let periods:Vec<SampleBuffer> = frequencies.iter().map(|f| 
        sample_period(&config, silly_sine, *f)
    ).collect();

    for &freq in &frequencies {
        let result = silly_sum_periods(&config, &frequencies, &periods);
        let filename = format!("dev-audio/test-silly-sum-periods-odds-1thru{}.wav", n);
        render::samples_f32(config.sample_rate, &result, &filename);
    }
}



#[test] 
fn test_silly_convolve_periods() {
    let config = RenderConfig {
        sample_rate: 44100,
        amplitude_scaling: 1.0,
        cps: 1.0,
    };
    
    let n:usize = 101;
    let frequencies:Vec<f32> = (0..n).filter(|x| x % 2 != 0).map(|x| x as f32).collect();
    let periods:Vec<SampleBuffer> = frequencies.iter().map(|f| 
        sample_period(&config, silly_sine, *f)
    ).collect();

    for &freq in &frequencies {
        let result = silly_convolve_periods(&periods);
        let filename = format!("dev-audio/test-silly-convolve-periods-odds-1thru{}.wav", n);
        render::samples_f32(config.sample_rate, &result, &filename);
    }
}
























/*
    * notes on amplitude modulation of harmonics
    * teasted on freq=440.0
    * 
    * for these notes, there is no scaling other than the given modulation factors.
    * it is conventional to diminish the relative amplitude of harmonics as distance from origin increases
    * 
    * DYNAMIC VALUES
    * When harmonics each have unique amplitude modulation then 
    * the result is a blur of them all together
    * 
    * value (harmonic + n,n in (0, 10))
    *   - produces a chorus-like effect
    * 
    * CONSTANT VALUES 
    * When the harmonics each have the same amplitude modulation then 
    * it is extremely clear when they are all present together or not (for low n)
    * 
    * value in (1, 10)
    *   - produces highly visible filter sweep effect
    * value in (11, 25)
    *   - produce buzzy, almost noisy effect 
    * 
    * value in (50, 99)
    *   - similar to a pulse wave with some harmonics beginning to emerge 
    * 
    * value in (100, 150)
    *   - results in the perception of a different fundamental pitch
    * 
    * There appears to be a threshold where these effects loop,
    * 
    * given that the test is run in a power envelope over 8 cycles at 1cps
    * we know that the first 2 seconds has little upper harmonics 
    * 
    * it appears that on these subsequent "loops" of the first 
    * we get an increasingly enriched fundamental because of the 
    * rapidly amplitude modulated upper harmonics
    * even though they are not yet mixed in at full volume, their rapid changes
    * are immenently visible
    * 
    * DIFFERENTIAL VALUES
    * 
    * Here we let the amplitude be modulated with respect to the ratio modulated by a function of ratio
    * 
    * r * sqrt(r)
    *   - more clear visiblity of higher ratios than lower ratios 
    * 
    * r * r  / 2
    *   - exhibits properties of dynamic modulation (chorus effect)
    *   - more clear visiblity of higher ratios than lower ratios 
    * 
    * 
    * r * r 
    *   - exhibits properties of constant modulation (unison filter sweep)
    *   - exhibits properties of dynamic modulation (chorus effect)
    * 
    * r * r + r 
    *   - exhibits the dynamic moudlation (chorus effect)
    *   - a little bit of perceived amp mod 
    *   - and some noise 
    * 
    * r * r * r 
    *   - new distinct tone, highly "metallic"
    * 
    * r * r * r * r 
    *   - wow is this what magic sounds like?
    * 
    * r * r * r * r * r 
    *   - the chimes of cthulu rise 
*/

#[test]
fn test_render_floot() {
    let config = synth_config::SynthConfig::new(44100, 20.0, 20000.0, 0.1, 0.0, 0.0, 1.2);
    let freq: f32 = 440.0;
    let max_freq = (config.sample_rate/2) as f32 / freq;
    let n_monics = max_freq.floor() as i8;
    let monics: Vec<i8> = (1..n_monics+1).collect();
    let buffers:Vec<Vec<f32>> = monics.into_iter().map(|r| {
        let duration = 16.0;
        let n = (config.sample_rate as f32 * duration).floor() as usize;
        let f = freq * r as f32;
        let mut buf = Vec::with_capacity(n);
        let mut amp_mod = Vec::with_capacity(n);

        // write the base signal and modulator for the monic
        for i in 0..n {
            buf.push(freq_forms::sine(&config, i as u32, f, None));

            // add a modulator 
            // modulators are designed to output values in (-1, 1)
            // modulators have the same lifetime as the note
            // so iterate at the same time as fundamental creation
            amp_mod.push(freq_forms::sine(&config, i as u32, r as f32, None) / (r as f32).powf(2.0));
        }

        if r > 2 {
            // add an envelope
            // enevelopes are designed to output values in (0, 1)
            let env = Envelope::new(2*config.sample_rate as usize, config.sample_rate, 1.0);
            let e = env.power(1.0, 2.0, true);

            match modulate::apply(&config, &buf, vec![e; 1]) {
                Ok(res) => {
                    buf = res;
                }
                Err(e) =>panic!("Bad envelope stuff {}", e)
            }
        }

        match modulate::apply(&config, &buf, vec![amp_mod; 1]) {
            Ok(res) => res,
            Err(e) => panic!("Bad modulation stuff {}", e)
        }
    }).collect();

    match render::mix_and_normalize_buffers(buffers) {
        Ok(mix) => {
            let label = "dev-audio/synth-floot-sample.wav";
            render::samples(&config, &mix, &label);
        }
        Err(e) => println!("Caught an error while mixing buffers: {}", e)
    }
}

/*
  Given a function of periodic unit time, 
  Return a list of samples scaled to the synthesizer
 */
fn sample_phrase_function(f:&phrase::PhraseMod, cps:f32, n:usize, sr:usize) -> Vec<f32> {
    let mut samples = Vec::<f32>::with_capacity(n);
    let seconds = n as f32 / sr as f32;
    let n_cycles = seconds * cps;
    let dt = n_cycles/n as f32;
    let mut t = 0.0f32;
    
    for i in 0..n-1 {
        samples.push(f(t));
        t += dt
    }
    samples
}

#[test]
fn test_render_phrases() {
    let g:phrase::Globe = phrase::Globe {
        dur: 128.0,
        origin: 1.05,
        q: Either::Left(0),
        cps: 1.2
    };
    
    let p = phrase::Phrase {
        cpc: 16,
        root: 1.05,
        q: Either::Left(0)
    };

    let oscs = phrase::all(&g, &p);
    let config = synth_config::SynthConfig::new(44100, 20.0, 20000.0, 0.1, 0.0, 0.0, g.cps);

    let curve = sample_phrase_function(&oscs[0], config.cps, 44100, 44100);

    let freq: f32 = 440.0;
    let max_freq = (config.sample_rate/2) as f32 / freq;
    let n_monics = max_freq.floor() as i8;
    let monics: Vec<i8> = (1..n_monics+1).collect();

    let buffers:Vec<Vec<f32>> = monics.into_iter().map(|r| {
        let duration = 16.0;
        let n = (config.sample_rate as f32 * duration).floor() as usize;
        let f = freq * r as f32;
        let mut buf = Vec::with_capacity(n);
        let mut amp_mod = Vec::with_capacity(n);

        // write the base signal and modulator for the monic
        for i in 0..n {
            buf.push(freq_forms::sine(&config, i as u32, f, None));

            // add a modulator 
            // modulators are designed to output values in (-1, 1)
            // modulators have the same lifetime as the note
            // so iterate at the same time as fundamental creation
            amp_mod.push(freq_forms::sine(&config, i as u32, r as f32, None) / (r as f32).powf(2.0));
        }

        if r > 2 {
            // add a phrasing envelope
            // envelopes are designed to output values in (0, 1)
            let o = (r - 2) as usize % oscs.len();
            let e = sample_phrase_function(&oscs[o], config.cps, n, 44100);

            match modulate::apply(&config, &buf, vec![e; 1]) {
                Ok(res) => {
                    buf = res;
                }
                Err(e) =>panic!("Bad envelope stuff {}", e)
            }
        }

        match modulate::apply(&config, &buf, vec![amp_mod; 1]) {
            Ok(res) => res,
            Err(e) => panic!("Bad modulation stuff {}", e)
        }
    }).collect();

    match render::mix_and_normalize_buffers(buffers) {
        Ok(mix) => {
            let label = "dev-audio/synth-floot-phrased-contour-sample.wav";
            render::samples(&config, &mix, &label);
        }
        Err(e) => println!("Caught an error while mixing buffers: {}", e)
    }
}


#[test]
fn test_render_with_time_fx() {
    let config = synth_config::SynthConfig::new(44100, 20.0, 20000.0, 0.1, 0.0, 0.0, 1.2);
    let freq: f32 = 440.0;
    let max_freq = (config.sample_rate/2) as f32 / freq;
    let n_monics = max_freq.floor() as i8;
    let monics: Vec<i8> = (1..n_monics+1).collect();
    let buffers:Vec<Vec<f32>> = monics.into_iter().map(|r| {
        let duration = 128.0;
        let n = (config.sample_rate as f32 * duration).floor() as usize;
        let f = freq * r as f32;
        let mut buf = Vec::with_capacity(n);
        let mut amp_mod = Vec::with_capacity(n);

        // write the base signal and modulator for the monic
        for i in 0..n {
            buf.push(freq_forms::sine(&config, i as u32, f, None));

            // add a modulator 
            // modulators have the same lifetime as the note
            // so iterate at the same time as fundamental creation
            let amod = (r as f32 * r as f32 * r as f32 * r as f32 * r as f32) as f32;
            amp_mod.push(freq_forms::sine(&config, i as u32, amod, None));
        }

        if r > 2 {
            // add an envelope
            let env = Envelope::new(2*config.sample_rate as usize, config.sample_rate, 1.0);
            let e = env.power(2.0, 2.0, true);

            match modulate::apply(&config, &buf, vec![e; 1]) {
                Ok(res) => {
                    buf = res;
                }
                Err(e) =>panic!("Bad envelope stuff {}", e)
            }
        }

        match modulate::apply(&config, &buf, vec![amp_mod; 1]) {
            Ok(res) => res,
            Err(e) => panic!("Bad modulation stuff {}", e)
        }
    }).collect();

    match render::mix_and_normalize_buffers(buffers) {
        Ok(mix) => {
            let label = "dev-audio/time-sample.wav";
            render::samples(&config, &mix, &label);
        }
        Err(e) => println!("Caught an error while mixing buffers: {}", e)
    }
}


#[test]
fn test_render_enharmonic() {
    let config = synth_config::SynthConfig::new(44100, 20.0, 20000.0, 0.1, 0.0, 0.0, 1.2);
    let freq: f32 = 440.0;
    let max_freq = (config.sample_rate/2) as f32 / freq;
    let n_monics = max_freq.floor() as i8;
    let monics: Vec<i8> = (1..n_monics+1).collect();
    let buffers:Vec<Vec<f32>> = monics.into_iter().map(|r| {
        let duration = 64.0;
        let n = (config.sample_rate as f32 * duration).floor() as usize;
        let mod_r = r as f32 + 0.6666f32.powf(r as f32);
        let f = freq * mod_r;
        let mut buf = Vec::with_capacity(n);
        let mut amp_mod = Vec::with_capacity(n);

        // write the base signal and modulator for the monic
        for i in 0..n {
            buf.push(freq_forms::sine(&config, i as u32, f, None));

            // add a modulator 
            // modulators have the same lifetime as the note
            // so iterate at the same time as fundamental creation
            amp_mod.push(freq_forms::sine(&config, i as u32, 1.0, None));
        }

        if r > 2 {
            // add an envelope
            let env = Envelope::new(2*config.sample_rate as usize, config.sample_rate, 1.0);
            let e = env.power(2.0, 2.0, true);

            match modulate::apply(&config, &buf, vec![e; 1]) {
                Ok(res) => {
                    buf = res;
                }
                Err(e) =>panic!("Bad envelope stuff {}", e)
            }
        }

        match modulate::apply(&config, &buf, vec![amp_mod; 1]) {
            Ok(res) => res,
            Err(e) => panic!("Bad modulation stuff {}", e)
        }
    }).collect();

    match render::mix_and_normalize_buffers(buffers) {
        Ok(mix) => {
            let label = "dev-audio/enharmonic-sample.wav";
            render::samples(&config, &mix, &label);
        }
        Err(e) => println!("Caught an error while mixing buffers: {}", e)
    }
}


