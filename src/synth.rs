use crate::render;
use crate::types::*;

pub const pi: f32 = std::f32::consts::PI;
pub const pi2: f32 = pi * 2f32;
pub const pi_2: f32 = std::f32::consts::FRAC_PI_2;
pub const pi_4: f32 = std::f32::consts::FRAC_PI_4;
pub const e: f32 = std::f32::consts::E;
pub const epi: f32 = pi * std::f32::consts::E;

pub use crate::types::synthesis::{RangeBuffer, SampleBuffer};

pub const SR: usize = 48000;
pub const SRi: i32 = SR as i32;
pub const SRf: f32 = SR as f32;
pub const SRu: u32 = SR as u32;

// Nyquist Frequency: Maximum renderable frequency
pub const NF: usize = SR / 2;
pub const NFi: i32 = NF as i32;
pub const NFu: u32 = NF as u32;
pub const NFf: f32 = SR as f32 / 2f32;

// Minimum Frequency: Minimum supported application frequency
pub const MF: usize = 24;
pub const MFi: i32 = MF as i32;
pub const MFu: u32 = MF as u32;
pub const MFf: f32 = MF as f32;

// Aliases for Time Domain
pub const SECONDS_PER_SAMPLE: f32 = 1.0 / SRf; // Time duration of a single sample
pub const SAMPLES_PER_MILLISECOND: f32 = SRf / 1000.0; // Number of samples in a millisecond
pub const SAMPLES_PER_SECOND: f32 = SRf; // Alias for samples per second

pub fn MAX_POW_2i() -> i32 { NFf.log2() as i32 }
pub fn MAX_POW_2f() -> f32 { NFf.log2() }
pub fn MAX_POW_2u() -> u32 { NFf.log2() as u32 }
pub const MAX_REGISTER: i32 = 13;
pub const MIN_REGISTER: i32 = 4;

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
