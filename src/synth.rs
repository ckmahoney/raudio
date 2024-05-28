use crate::render;
use crate::types::*;

pub const pi:f32 = std::f32::consts::PI;
pub const pi2:f32 = pi*2f32;


pub const SR:usize = 44100;
// Maximum renderable frequency 
pub const NF:usize = SR/2;
pub const NFf:f32 = SR as f32/2f32;
/* The minimum supported frequency to render*/
pub const MF:usize = 24;
pub const MFf:f32 = 24f32;
pub const MAX_REGISTER:i32 = 15;
pub const MIN_REGISTER:i32 = 4;

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

/// Sample values in -1 to 1
pub type SampleBuffer = Vec<f32>;

/// Sample values in 0 to 1 
pub type RangeBuffer = Vec<f32>; 
pub struct EnvelopeBuffer {
    ys: RangeBuffer
}

impl EnvelopeBuffer {
    pub fn new(minCps:f32, maxCps:f32, samples:SampleBuffer) -> SampleBuffer {
        if samples.iter().any(|x| *x < 0f32) {
            panic!("Cannot create an envelope with negative values")
        }
        samples
    }
}

pub struct RenderConfig {
    pub sample_rate: usize,
    pub amplitude_scaling: f32,
    pub cps: f32
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