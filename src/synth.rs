/// This module provides the settings for the application's synthesis engine.
/// It includes definitions for Sample Rate, Minimum Frequency, Minimum Decibel Value,
/// Maximum Decibel value.
///
/// The module also offers convenient aliases for standard constants at f32 precision.
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

pub fn MAX_POW_2i() -> i32 {
  NFf.log2() as i32
}
pub fn MAX_POW_2f() -> f32 {
  NFf.log2()
}
pub fn MAX_POW_2u() -> u32 {
  NFf.log2() as u32
}
pub const MAX_REGISTER: i32 = 13;
pub const MIN_REGISTER: i32 = 4;

/// Global static values in decibels
///
/// The total supported dynamic range is MAX_DB-MIN_DB
/// however, musically we have a more conservative THRESH_NOISE_DB
///
/// Presets will output amplitude envelopes bound by THRESH_NOISE_DB
/// Ingested samples, applied reverb and delay, and resampling may produce
/// artifacts under MIN_DB. These will be gated.
///
/// Signal values in (-72, -60) DB are left in tact.
pub const MIN_DB: f32 = -80f32;
pub const MAX_DB: f32 = 0f32;
pub const THRESH_NOISE_DB: f32 = -72f32;
pub const DYNAMIC_RANGE_DB: f32 = MAX_DB - MIN_DB;
