use crate::render;
use crate::types::*;



pub struct Renderable {
    pub composition_id: i32,
    pub samples: Vec<SampleBuffer>
}


impl Renderable {
    pub fn render(self, mix_polypohonics:bool) -> SampleBuffer {
        let buf:Vec<f32> = Vec::new();

        if mix_polypohonics {
            buf
        } else {
            buf
        }
    }
}

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
pub type Ugen = fn(usize, usize, f32) -> f32;

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
    (0..n_samples).map({|i| 
            config.amplitude_scaling * ugen(config.sample_rate, i, freq)
    }).collect()
}

// !! note this may truncate a partial sample for non-harmonic cps. 
// This eventually leads to frequency drift
// it is considerable to correct the drift by factoring the dt lost and total number of samples lost
pub fn samp_ugen(sample_rate:usize, cps:f32, amp:f32, ugen:Ugen, duration:f32, freq:f32) -> SampleBuffer {
    let samples_per_cycle:f32 = sample_rate as f32 / cps;
    let n_samples = (samples_per_cycle * duration).floor() as usize;
    (0..n_samples).map({|i| 
            amp * ugen(sample_rate, i, freq)
    }).collect()
}

pub fn sample_period(config:&RenderConfig, ugen:Ugen, freq:f32) -> SampleBuffer {
    let samples_per_period = (config.sample_rate as f32 / freq) as usize;
    (0..samples_per_period).map({|i| 
        config.amplitude_scaling * ugen(config.sample_rate, i, freq)
    }).collect()
}

pub fn sample_scale_period(config:&RenderConfig, ugen:Ugen, freq:f32, amp:f32) -> SampleBuffer {
    let samples_per_period = (config.sample_rate as f32 / freq) as usize;
    (0..samples_per_period).map({|i| 
        amp * ugen(config.sample_rate, i, freq)
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


#[cfg(test)]
mod test {
    use super::*;
    use crate::files;


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
        let frequencies:Vec<f32> = (1..n).filter(|x| x % 2 != 0).map(|x| x as f32).collect();
        let periods:Vec<SampleBuffer> = frequencies.iter().map(|f| 
            sample_period(&config, silly_sine, *f)
        ).collect();

        let result = silly_sum_periods(&config, &frequencies, &periods);
        let filename = format!("dev-audio/test-silly-sum-periods-odds-1thru{}.wav", n);
        render::samples_f32(config.sample_rate, &result, &filename);
    }

    fn get_freqs() -> std::ops::Range::<usize> {
        let min_f:usize = 10;
        let max_f:usize = 3500;
        min_f..max_f
    }

    const DEST_FUNDAMENTAL:&str = "fundamentals";

    #[test] 
    fn test_prerender_fundamental_sums() {
        let mut dir = DEST_FUNDAMENTAL.to_owned();
        files::ensure_directory_exists(&dir);
        dir.push_str(&"/sum/all");
        files::ensure_directory_exists(&dir);

        let config = RenderConfig {
            sample_rate: 44100,
            amplitude_scaling: 1.0,
            cps: 1.0,
        };
        let max_monics = 1024;

        let basis = get_freqs();
        let offset = 0.5; 
        for fundamental in basis {
            let n:usize = max_monics.min((config.sample_rate/2) / fundamental as usize);

            let monics:Vec<f32> = (1..n+1).map(|x| x as f32).collect();
            let periods:Vec<SampleBuffer> = monics.iter().map(|f| 
                sample_period(&config, silly_sine, *f)
            ).collect();

            let result = silly_sum_periods(&config, &monics, &periods);
            let filename = format!("{}/sum/all/all_{}_{}.wav", DEST_FUNDAMENTAL, fundamental, config.sample_rate);
            render::samples_f32(config.sample_rate, &result, &filename);
            std::mem::drop(result);

            let fundamental2 = fundamental as f32 + offset;
            let n2:usize = (config.sample_rate as f32 / fundamental2) as usize;
            let monics2:Vec<f32> = (1..n+1).map(|x| x as f32).collect();
            let periods2:Vec<SampleBuffer> = monics2.iter().map(|f| 
                sample_period(&config, silly_sine, *f)
            ).collect();
            let result2 = silly_sum_periods(&config, &monics2, &periods);
            let filename2 = format!("{}/sum/all/all_{}_{}.wav", DEST_FUNDAMENTAL, fundamental2, config.sample_rate);
            render::samples_f32(config.sample_rate, &result2, &filename2);
            std::mem::drop(result2);
        }
    }

    #[test] 
    fn test_prerender_fundamental_sums_saw() {
        let mut dir = DEST_FUNDAMENTAL.to_owned();
        files::ensure_directory_exists(&dir);
        dir.push_str(&"/sum/saw");
        files::ensure_directory_exists(&dir);
        
        files::ensure_directory_exists(DEST_FUNDAMENTAL);
        let config = RenderConfig {
            sample_rate: 44100,
            amplitude_scaling: 1.0,
            cps: 1.0,
        };

        let basis = get_freqs();
        let offset = 0.5; 
        let max_monics = 1024;
        for fundamental in basis {
            let n:usize = max_monics.min((config.sample_rate/2) / fundamental as usize);
            // use all freqs for saw
            let monics:Vec<f32> = (1..n+1).map(|x| x as f32).collect();

            let mut periods:Vec<SampleBuffer> = monics.iter().enumerate().map({|(i,f)| 
                sample_scale_period(&config, silly_sine, *f as f32, 1.0/(i+1) as f32)
            })
            .collect(); 

            periods.iter_mut().enumerate()
            .for_each(|(i, period)|
                render::amp_scale(&mut *period, 1.0/(i+1) as f32)
            );

            let result = silly_sum_periods(&config, &monics, &periods);
            let filename = format!("{}/sum/saw/saw_{}_hz_{}.wav", DEST_FUNDAMENTAL, fundamental, config.sample_rate);
            render::samples_f32(config.sample_rate, &result, &filename);
            std::mem::drop(periods);
            std::mem::drop(result);

            let fundamental2 = fundamental as f32 + offset;
            let n2:usize = (config.sample_rate as f32 / fundamental2) as usize;
            let monics2:Vec<f32> = (1..n+1).map(|x| x as f32).collect();
            let mut periods2:Vec<SampleBuffer> = monics2.iter().enumerate().map(|(i,f)| 
                sample_scale_period(&config, silly_sine, *f as f32, 1.0/(i+1) as f32)
            ).collect();

            periods2.iter_mut().enumerate()
            .for_each(|(i, period)|
                render::amp_scale(&mut *period, 1.0/(i+1) as f32)
            );
            let result2 = silly_sum_periods(&config, &monics2, &periods2);
            let filename2 = format!("{}/sum/saw/saw_{}_hz_{}.wav", DEST_FUNDAMENTAL, fundamental2, config.sample_rate);
            render::samples_f32(config.sample_rate, &result2, &filename2);
            std::mem::drop(periods2);
            std::mem::drop(result2);
        }
    }
}
/*
// notes from /home/naltroc/synthony-serivce/wendy/src/synth.rs
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