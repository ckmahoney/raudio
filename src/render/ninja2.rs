//! # Additive Synthesis Engine
//! 
//! This module provides an implementation of an additive synthesis engine
//! with support for modulation and automation of parameters such as amplitude,
//! frequency, and phase over time. The `ninja` function is the main entry point
//! for generating a synthesized audio buffer.
//! 
//! ## Types
//! 
//! - `Frex`: Represents the frequency context as a tuple of floats and optional floats.
//! - `Expr`: Holds expression contours for amplitude, frequency, and phase, along with automation envelopes.
//! - `Span`: Represents cycles and cycles per second as a tuple of floats.
//! - `Bp`: Bandpass filter settings (structure to be defined as needed).
//! - `ModulationValues`: Holds vectors for multipliers, amplifiers, and phases.
//! - `Modders`: Optional transformers for amplitude, frequency, and phase.
//! - `ModulationParams`: Enum for modulation parameters.
//! - `SampleBuffer`: Alias for a vector of floats representing the output audio buffer.

/// Represents the frequency context.
type Frex = (f32, Option<f32>, f32, Option<f32>, f32);

/// Holds expression contours for amplitude, frequency, and phase, along with automation envelopes.
struct Expr {
    amplitude: Vec<f32>,
    frequency: Vec<f32>,
    phase: Vec<f32>,
    amplitude_automation: Option<AutomationEnvelope>,
    frequency_automation: Option<AutomationEnvelope>,
    phase_automation: Option<AutomationEnvelope>,
}

/// Represents cycles and cycles per second.
type Span = (f32, f32);

/// Bandpass filter settings.
struct Bp {
    // Specific fields and types to be defined based on the bandpass filter implementation.
}

/// Holds vectors for multipliers, amplifiers, and phases.


/// amp tremelo, amp contour 
pub type AmodRanger = fn(&ModulationParams, usize, f32, f32, Option<f32>, Option<f32>, Option<f32>) -> f32;
/// pitch vibrato, equal power noise, chorus
/// want to try one that uses PWM or Triangle modulation on phase  
pub type PmodRanger = fn(&ModulationParams, usize, f32, f32, Option<f32>, Option<f32>, Option<f32>) -> f32;
/// pitch glide, hoover
pub type FmodRanger = fn(&ModulationParams, usize, f32, f32, f32, Option<f32>, Option<f32>) -> f32;

/// Wrapper for modders to distinguish between types.
pub enum Ranger {
    AmodPmod(AmodPmodRanger),
    Fmod(FmodRanger),
}

/// Updated WRangers type to accommodate different ranger types.
pub type WRangers = Vec<(f32, Ranger)>;

/// Modders type remains unchanged.
pub type Modders = [Option<WRangers>; 3];

/// Enum for modulation parameters.
enum ModulationParams {
    PWM { rate:f32, width: f32 },
    Sin { rate: f32 },
    Vibrato { coeffs: [f32; 3], range: f32, offset_center: f32 },
    Phase { rate: f32, noise: f32 },
}

/// Alias for a vector of floats representing the output audio buffer.
type SampleBuffer = Vec<f32>;

/// Automation Envelope for dynamic parameter changes over time.
struct AutomationEnvelope {
    start_time: f32,
    end_time: f32,
    start_value: f32,
    end_value: f32,
}

impl AutomationEnvelope {
    /// Gets the interpolated value at the given current time.
    fn get_value(&self, current_time: f32) -> f32 {
        let t = (current_time - self.start_time) / (self.end_time - self.start_time);
        self.start_value + t * (self.end_value - self.start_value)
    }
}

/// Samples the value at the given progress (p) from the vector.
fn sample(values: &Vec<f32>, p: f32) -> f32 {
    let index = (p * values.len() as f32).min(values.len() as f32 - 1.0) as usize;
    values[index]
}

/// Placeholder for the bandpass filter logic.
fn filter(_p: f32, _frequency: f32, _bp: &Bp) -> f32 {
    1.0
}

/// Mixes the modulation or returns the default value if modulation is not present.
fn mix_or(
    default: f32,
    modders: &Option<WRangers>,
    params: Option<&ModulationParams>,
    k: usize,
    p: f32,
    d: f32,
    cur: f32,
    prev: Option<f32>,
    next: Option<f32>,
    context: ModulationContext
) -> f32 {
    if let Some(modders) = modders {
        let mut result = 0.0;
        for (weight, ranger) in modders {
            if let Some(params) = params {
                result += weight * match ranger {
                    Ranger::AmodPmod(func) => func(params, k, p, d, context.velocity, context.harmonic_index, context.timbre),
                    Ranger::Fmod(func) => func(params, k, p, d, cur, prev, next),
                };
            }
        }
        result
    } else {
        default
    }
}

/// Main synthesis function for generating the audio buffer.
/// 
/// # Parameters
/// 
/// - `frex`: Frequency context.
/// - `expr`: Expression contours and automation envelopes.
/// - `span`: Cycles and cycles per second.
/// - `bp`: Bandpass filter settings.
/// - `values`: Multipliers, amplifiers, and phases.
/// - `modders`: Modulation functions.
/// - `params`: Modulation parameters.
/// - `thresh`: Tuple of gate and clip thresholds.
/// - `context`: Additional modulation context parameters.
/// 
/// # Returns
/// 
/// - `SampleBuffer`: The synthesized audio buffer.
pub fn ninja(
    frex: &Frex,
    expr: &Expr,
    span: &Span,
    bp: &Bp,
    values: &ModulationValues,
    modders: &Modders,
    params: &ModulationParamSet,
    thresh: (f32, f32),
    context: ModulationContext
) -> SampleBuffer {
    let (glide_from, maybe_prev, freq, maybe_next, glide_to) = frex;
    let n_samples = crate::time::samples_of_cycles(span.0, span.1);
    let mut sig = vec![0f32; n_samples];

    for j in 0..n_samples {
        let p: f32 = j as f32 / n_samples as f32;
        let t: f32 = j as f32 / 44100.0; // Assuming sample rate of 44100 Hz

        let mut am = sample(&expr.amplitude, p);
        let mut fm = sample(&expr.frequency, p);
        let mut pm = sample(&expr.phase, p);

        if let Some(amp_auto) = &expr.amplitude_automation {
            am *= amp_auto.get_value(t);
        }
        if let Some(freq_auto) = &expr.frequency_automation {
            fm *= freq_auto.get_value(t);
        }
        if let Some(phase_auto) = &expr.phase_automation {
            pm += phase_auto.get_value(t);
        }

        let mut v: f32 = 0f32;

        for (i, &m) in values.multipliers.iter().enumerate() {
            let k = i + 1;
            let aaa = mix_or(
                1f32,
                &modders[1],
                params[1].as_ref(),
                k,
                p,
                span.1,
                freq,
                maybe_prev,
                maybe_next,
                context
            );
            let frequency = m * fm * freq * aaa;
            let amplifier = values.amplifiers.get(i).cloned().unwrap_or(0.0);

            if amplifier > 0f32 {
                let amp = amplifier * am * filter(p, frequency, bp) * mix_or(
                    1f32,
                    &modders[0],
                    params[0].as_ref(),
                    k,
                    p,
                    span.1,
                    freq,
                    maybe_prev,
                    maybe_next,
                    context
                );
                if amp != 0f32 {
                    let phase = (frequency * std::f32::consts::PI * 2.0 * t)
                        + values.phases.get(i).cloned().unwrap_or(0.0)
                        + pm
                        + mix_or(
                            0f32,
                            &modders[2],
                            params[2].as_ref(),
                            k,
                            p,
                            span.1,
                            freq,
                            maybe_prev,
                            maybe_next,
                            context
                        );
                    v += amp * phase.sin();
                }
            }
        }

        let (gate_thresh, clip_thresh) = thresh;
        if v.abs() > clip_thresh {
            let sign: f32 = if v > 0f32 { 1f32 } else { -1f32 };
            sig[j] += sign * clip_thresh;
        }
        if v.abs() >= gate_thresh {
            sig[j] += v;
        }
    }

    sig
}

/// Contexts for amplitude and phase modulation.
struct ModulationContext {
    velocity: Option<f32>,
    harmonic_index: Option<f32>,
    timbre: Option<f32>,
}

/// Time-related utility functions.
mod crate {
    pub mod time {
        /// Calculates the number of samples based on the given cycles and cycles per second (CPS).
        pub fn samples_of_cycles(cycles: f32, cps: f32) -> usize {
            (cycles * cps * 44100.0) as usize```rust
// ninja.rs

//! # Additive Synthesis Engine
//! 
//! This module provides an implementation of an additive synthesis engine
//! with support for modulation and automation of parameters such as amplitude,
//! frequency, and phase over time. The `ninja` function is the main entry point
//! for generating a synthesized audio buffer.
//! 
//! ## Types
//! 
//! - `Frex`: Represents the frequency context as a tuple of floats and optional floats.
//! - `Expr`: Holds expression contours for amplitude, frequency, and phase, along with automation envelopes.
//! - `Span`: Represents cycles and cycles per second as a tuple of floats.
//! - `Bp`: Bandpass filter settings (structure to be defined as needed).
//! - `ModulationValues`: Holds vectors for multipliers, amplifiers, and phases.
//! - `Modders`: Optional transformers for amplitude, frequency, and phase.
//! - `ModulationParams`: Enum for modulation parameters.
//! - `SampleBuffer`: Alias for a vector of floats representing the output audio buffer.

/// Represents the frequency context.
type Frex = (f32, Option<f32>, f32, Option<f32>, f32);

/// Holds expression contours for amplitude, frequency, and phase, along with automation envelopes.
struct Expr {
    amplitude: Vec<f32>,
    frequency: Vec<f32>,
    phase: Vec<f32>,
    amplitude_automation: Option<AutomationEnvelope>,
    frequency_automation: Option<AutomationEnvelope>,
    phase_automation: Option<AutomationEnvelope>,
}

/// Represents cycles and cycles per second.
type Span = (f32, f32);

/// Bandpass filter settings.
struct Bp {
    // Specific fields and types to be defined based on the bandpass filter implementation.
}

/// Holds vectors for multipliers, amplifiers, and phases.
struct ModulationValues {
    multipliers: Vec<f32>,
    amplifiers: Vec<f32>,
    phases: Vec<f32>,
}

/// Ranger function type for amplitude and phase modulation with contextual parameters.
pub type AmodPmodRanger = fn(&ModulationParams, usize, f32, f32, Option<f32>, Option<f32>, Option<f32>) -> f32;

/// Ranger function type for frequency modulation (with frequency context).
pub type FmodRanger = fn(&ModulationParams, usize, f32, f32, f32, Option<f32>, Option<f32>) -> f32;

/// Wrapper for modders to distinguish between types.
pub enum Ranger {
    AmodPmod(AmodPmodRanger),
    Fmod(FmodRanger),
}

/// Updated WRangers type to accommodate different ranger types.
pub type WRangers = Vec<(f32, Ranger)>;

/// Modders type remains unchanged.
pub type Modders = [Option<WRangers>; 3];

/// Enum for modulation parameters.
enum ModulationParams {
    PWM { width: f32 },
    Sin { base_rate: f32 },
    Vibrato { poly_coeffs: [f32; 3], range: f32, offset_center: f32 },
    Phase { base_rate: f32 },
}

/// Alias for a vector of floats representing the output audio buffer.
type SampleBuffer = Vec<f32>;

/// Automation Envelope for dynamic parameter changes over time.
struct AutomationEnvelope {
    start_time: f32,
    end_time: f32,
    start_value: f32,
    end_value: f32,
}

impl AutomationEnvelope {
    /// Gets the interpolated value at the given current time.
    fn get_value(&self, current_time: f32) -> f32 {
        let t = (current_time - self.start_time) / (self.end_time - self.start_time);
        self.start_value + t * (self.end_value - self.start_value)
    }
}

/// Samples the value at the given progress (p) from the vector.
fn sample(values: &Vec<f32>, p: f32) -> f32 {
    let index = (p * values.len() as f32).min(values.len() as f32 - 1.0) as usize;
    values[index]
}

/// Placeholder for the bandpass filter logic.
fn filter(_p: f32, _frequency: f32, _bp: &Bp) -> f32 {
    1.0
}

/// Mixes the modulation or returns the default value if modulation is not present.
fn mix_or(
    default: f32,
    modders: &Option<WRangers>,
    params: Option<&ModulationParams>,
    k: usize,
    p: f32,
    d: f32,
    cur: f32,
    prev: Option<f32>,
    next: Option<f32>,
    context: ModulationContext
) -> f32 {
    if let Some(modders) = modders {
        let mut result = 0.0;
        for (weight, ranger) in modders {
            if let Some(params) = params {
                result += weight * match ranger {
                    Ranger::AmodPmod(func) => func(params, k, p, d, context.velocity, context.harmonic_index, context.timbre),
                    Ranger::Fmod(func) => func(params, k, p, d
