use super::compute::ModulationMode;

/// Parameters for amplitude modulation effects.
#[derive(Debug, Clone)]
pub struct AmplitudeModParams {
    pub rate: f32,
    pub depth: f32,
    pub offset: f32,
}

/// Parameters for frequency modulation effects.
#[derive(Debug, Clone)]
pub struct FrequencyModParams {
    pub rate: f32,
    pub offset: f32,
}

/// Parameters for phase modulation effects.
#[derive(Debug, Clone)]
pub struct PhaseModParams {
    pub rate: f32,
    pub depth: f32,
    pub offset: f32,
}

/// Different modulation effects that can be applied to an audio signal.
#[derive(Debug, Clone)]
pub enum ModulationEffect {
    Tremelo(AmplitudeModParams),
    Vibrato(PhaseModParams),
    Noise(PhaseModParams),
    Chorus(PhaseModParams),
    Glide(FrequencyModParams),
}

impl ModulationEffect {
    /// Applies the modulation effect at a given time to a base value.
    ///
    /// # Arguments
    ///
    /// * `time` - The time at which to apply the modulation effect.
    /// * `base_value` - The base value to be modulated.
    ///
    /// # Returns
    ///
    /// The modulated value.
    pub fn apply(&self, time: f32, base_value: f32) -> f32 {
        match self {
            ModulationEffect::Tremelo(params) => {
                let mode = ModulationMode::Sine {
                    rate: params.rate,
                    depth: params.depth,
                    offset: params.offset,
                };
                mode.compute(time) * base_value
            },
            ModulationEffect::Vibrato(params) => {
                let phase_modulated = params.depth * (params.rate * time + params.offset).sin();
                let mode = ModulationMode::Sine {
                    rate: 1.0,
                    depth: 1.0,
                    offset: phase_modulated,
                };
                mode.compute(time) + base_value
            },
            ModulationEffect::Noise(params) => {
                let mode = ModulationMode::Random { seed: params.offset as u64 };
                let random_value = mode.compute(time);
                let mode = ModulationMode::Sine {
                    rate: 1.0,
                    depth: 1.0,
                    offset: random_value,
                };
                mode.compute(time) + base_value
            },
            ModulationEffect::Chorus(params) => {
                let chorus_effect = params.depth * (params.rate * time + params.offset).sin();
                let mode = ModulationMode::Sine {
                    rate: 1.0,
                    depth: 1.0,
                    offset: chorus_effect,
                };
                mode.compute(time) + base_value
            },
            ModulationEffect::Glide(params) => {
                let mode = ModulationMode::Sine {
                    rate: params.rate,
                    depth: 1.0,
                    offset: params.offset,
                };
                mode.compute(time) + base_value
            },
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

/// Struct to hold information about a combined preset.
#[derive(Debug, Clone)]
pub struct CombinedPreset {
    pub name: String,
    pub description: String,
    pub effects: Vec<ModulationEffect>,
    pub combine_fn: fn(f32, f32) -> f32,
}

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
/// * `base_value` - The base value to be modulated.
///
/// # Returns
///
/// The modulated value after applying all effects.
pub fn chain(effects: &[ModulationEffect], time: f32, base_value: f32) -> f32 {
    effects.iter().fold(base_value, |acc, effect| effect.apply(time, acc))
}

/// Example presets for modulation effects.
pub fn get_presets() -> Vec<SimplePreset> {
    vec![
        create_simple_preset!(
            "Warm Chorus",
            "A gentle chorus effect with slight phase variations.",
            ModulationEffect::Chorus(PhaseModParams { rate: 0.8, depth: 0.3, offset: 0.1 })
        ),
        create_simple_preset!(
            "Deep Chorus",
            "A deep chorus effect with noticeable phase variations.",
            ModulationEffect::Chorus(PhaseModParams { rate: 0.6, depth: 0.4, offset: 0.2 })
        ),
        create_simple_preset!(
            "Intense Chorus",
            "An intense chorus effect with significant phase variations.",
            ModulationEffect::Chorus(PhaseModParams { rate: 0.5, depth: 0.5, offset: 0.3 })
        ),
    ]
}

/// Applies a combined preset by using the provided combination function.
pub fn apply_combined_preset(preset: &CombinedPreset, time: f32, base_value: f32) -> f32 {
    preset.effects.iter().fold(base_value, |acc, effect| (preset.combine_fn)(acc, effect.apply(time, base_value)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn main() {
        let tremelo = ModulationEffect::Tremelo(AmplitudeModParams { rate: 2.0, depth: 0.5, offset: 0.1 });
        let vibrato = ModulationEffect::Vibrato(PhaseModParams { rate: 5.0, depth: 0.3, offset: 0.0 });

        let effects = vec![tremelo.clone(), vibrato.clone()];

        let result = chain(&effects, 1.0, 1.0);
        println!("Modulated value: {}", result);

        // Using simple presets
        let simple_presets = get_presets();
        for (i, preset) in simple_presets.iter().enumerate() {
            let result = chain(&vec![preset.effect.clone()], 1.0, 1.0);
            println!("Simple Preset {} - {} modulated value: {}", i + 1, preset.name, result);
        }

        // Using a combined preset
        let combined_preset = create_combined_preset!(
            "Combined Effects",
            "Combines tremelo and vibrato effects.",
            vec![tremelo, vibrato],
            |acc, effect| acc + effect // Example combination function
        );
        let result = apply_combined_preset(&combined_preset, 1.0, 1.0);
        println!("Combined Preset - {} modulated value: {}", combined_preset.name, result);
    }
}
