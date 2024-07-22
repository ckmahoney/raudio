use crate::synth::pi2;
use super::compute::ModulationMode;
pub use crate::types::synthesis::{Modifiers, ModifiersHolder, Dressing, ModulationEffect, AmplitudeModParams, FrequencyModParams, PhaseModParams};


/// Macro to create a simple preset.
#[macro_export]
macro_rules! create_simple_preset {
    ($name:expr, $description:expr, $effect:expr) => {
        SimplePreset {
            name: $name.to_string(),
            description: $description.to_string(),
            effect: $effect,
        }
    };
}
impl ModulationEffect {
    /// Applies the modulation effect at a given time to a base value.
    ///
    /// # Arguments
    ///
    /// * `time` - The time at which to apply the modulation effect.
    /// * `y` - The base value to be modulated.
    ///
    /// # Returns
    ///
    /// The modulated value.
    pub fn apply(&self, time: f32, y: f32) -> f32 {
        match self {
            ModulationEffect::Tremelo(params) => {
                // Tremelo modifies the amplitude of a sine wave.
                let mode = ModulationMode::Sine {
                    freq: params.freq,
                    depth: params.depth,
                    offset: params.offset,
                };
                mode.compute(time) * y
            },
            ModulationEffect::Vibrato(params) => {
                let theta = pi2 * params.rate * time;

                // Vibrato modifies the phase of a sine wave.
                let phase_modulated = params.depth * (theta + params.offset).sin();
                let mode = ModulationMode::Sine {
                    freq: 1.0,
                    depth: 1.0,
                    offset: phase_modulated,
                };
                mode.compute(time) + y
            },
            ModulationEffect::Noise(params) => {
                // Noise adds random variations to the phase of a sine wave.
                let mode = ModulationMode::Random { seed: params.offset as u64 };
                let random_value = mode.compute(time);
                let mode = ModulationMode::Sine {
                    freq: 1.0,
                    depth: 1.0,
                    offset: random_value,
                };
                mode.compute(time) + y
            },
            ModulationEffect::Chorus(params) => {
                // Chorus adds slight variations to the phase of a sine wave.
                let chorus_effect = params.depth * (params.rate * time + params.offset).sin();
                let mode = ModulationMode::Sine {
                    freq: 1.0,
                    depth: 1.0,
                    offset: chorus_effect,
                };
                mode.compute(time) + y
            },
            ModulationEffect::Sway(params) => {
                let mode = ModulationMode::Sine {
                    freq: params.rate / 2f32,
                    depth: 1f32,
                    offset: params.offset,
                };
                (mode.compute(time).abs() + 1f32) / 2f32
            }
            ModulationEffect::Warp(params) => {
                let mode = ModulationMode::Sine {
                    freq: params.rate,
                    depth: params.depth,
                    offset: params.offset,
                }; 
                let s = y.log2();
                let dy = s*2f32;
                let v = dy * mode.compute(time).abs();
                // println!("v {} dy {} ", v, dy);
                y + v

            }
        }
    }
}

/// Struct to hold information about a simple preset.
#[derive(Debug, Clone)]
pub struct SimplePreset {
    pub name: String,
    pub description: String,
    pub effect: ModulationEffect,
}

/// Macro to create a combined preset.
#[macro_export]
macro_rules! create_combined_preset {
    ($name:expr, $description:expr, $effects:expr, $combine_fn:expr) => {
        CombinedPreset {
            name: $name.to_string(),
            description: $description.to_string(),
            effects: $effects,
            combine_fn: $combine_fn,
        }
    };
}

/// Applies all effects in the chain to a base value at a given time.
///
/// # Arguments
///
/// * `time` - The time at which to apply the effects.
/// * `y` - The base value to be modulated.
///
/// # Returns
///
/// The modulated value after applying all effects.
pub fn chain(effects: &[ModulationEffect], time: f32, y: f32) -> f32 {
    effects.iter().fold(y, |acc, effect| effect.apply(time, acc))
}

impl Dressing {
    pub fn empty(len: usize) -> Self {
        Dressing {
            len,
            multipliers: vec![0f32; len],
            amplitudes: vec![0f32; len],
            offsets: vec![0f32; len],
        }
    }

    pub fn new(amplitudes: Vec<f32>, multipliers: Vec<f32>, offsets: Vec<f32>) -> Self {
        let len = amplitudes.len();
        if len != multipliers.len() || len != offsets.len() {
            panic!(
                "Input vectors must all have the same length. Got lengths: amplitudes: {}, multipliers: {}, offsets: {}",
                len, multipliers.len(), offsets.len()
            );
        }

        Dressing {
            len,
            amplitudes,
            multipliers,
            offsets,
        }
    }

    pub fn set_muls(&mut self, muls: Vec<f32>) {
        if muls.len() != self.len {
            panic!(
                "Unable to update multipliers. Requires a vector of length {} but got actual length {}",
                self.multipliers.len(),
                muls.len()
            );
        }
        self.multipliers = muls;
    }

    pub fn set_amplitudes(&mut self, amps: Vec<f32>) {
        if amps.len() != self.len {
            panic!(
                "Unable to update amplitudes. Requires a vector of length {} but got actual length {}",
                self.amplitudes.len(),
                amps.len()
            );
        }
        self.amplitudes = amps;
    }

    pub fn set_offsets(&mut self, offsets: Vec<f32>) {
        if offsets.len() != self.len {
            panic!(
                "Unable to update offsets. Requires a vector of length {} but got actual length {}",
                self.offsets.len(),
                offsets.len()
            );
        }
        self.offsets = offsets;
    }

    pub fn unit_amp(length: usize) -> Vec<f32> {
        vec![1f32; length]
    }

    pub fn unit_freq(length: usize) -> Vec<f32> {
        (1..=length).map(|i| i as f32).collect()
    }

    pub fn unit_offset(length: usize) -> Vec<f32> {
        vec![0f32; length]
    }

    pub fn normalize(&mut self) {
        let sum: f32 = self.amplitudes.iter().sum();
        if sum > 0.0 {
            self.amplitudes.iter_mut().for_each(|a| *a /= sum);
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "Dressing {{ len: {}, amplitudes: {:?}, multipliers: {:?}, offsets: {:?} }}",
            self.len, self.amplitudes, self.multipliers, self.offsets
        )
    }
}

/// Example presets for modulation effects.
pub fn get_presets() -> Vec<SimplePreset> {
    vec![
        create_simple_preset!(
            "Warm Chorus",
            "A gentle chorus effect with slight phase variations.",
            ModulationEffect::Chorus(PhaseModParams { rate: 1.0, depth: 0.3, offset: 0.1 })
        ),
        create_simple_preset!(
            "Deep Chorus",
            "A deep chorus effect with noticeable phase variations.",
            ModulationEffect::Chorus(PhaseModParams { rate: 1.0,depth: 0.4, offset: 0.2 })
        ),
        create_simple_preset!(
            "Intense Chorus",
            "An intense chorus effect with significant phase variations.",
            ModulationEffect::Chorus(PhaseModParams { rate: 1.0, depth: 0.5, offset: 0.3 })
        ),
    ]
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    /// Show that a tremelo effect can be applied to a melodic synth
    fn melodic_tremelo() {
        let a = AmplitudeModParams { freq: 2.0, depth: 0.5, offset: 0.1 };
        let b = PhaseModParams { rate: 1.0, depth: 0.3, offset: 0.0 };
        
        let tremelo = ModulationEffect::Tremelo(a);
        let vibrato = ModulationEffect::Vibrato(b);

        let effects = vec![tremelo.clone(), vibrato.clone()];

        let result = chain(&effects, 1.0, 1.0);
        println!("Modulated value: {}", result);

        // Using simple presets
        let simple_presets = get_presets();
        for (i, preset) in simple_presets.iter().enumerate() {
            let result = chain(&vec![preset.effect.clone()], 1.0, 1.0);
            println!("Simple Preset {} - {} modulated value: {}", i + 1, preset.name, result);
        }
    }

}

pub fn add(a: f32, b: f32) -> f32 {
    a + b
}
