use rand::SeedableRng;
use rand::Rng;

/// Represents different modulation modes for harmonic modulation.
#[derive(Debug, Clone)]
pub enum ModulationMode {
    /// Sine wave modulation.
    ///
    /// # Fields
    ///
    /// * `freq` - Defines the frequency of the sine wave.
    /// * `depth` - Defines the amplitude of the sine wave.
    /// * `offset` - Value to shift the wave horizontally.
    Sine { freq: f32, depth: f32, offset: f32 },

    /// Peak modulation.
    ///
    /// # Fields
    ///
    /// * `midpoint` - Determines where the peak occurs. Allows shapes from triangle
    /// (midpoint = 0.5) to sawtooth (midpoint = 1.0) and reverse triangle (midpoint = 0.0).
    Peak { midpoint: f32 },

    /// Linear modulation.
    ///
    /// # Fields
    ///
    /// * `slope` - Defines the rate of increase or decrease.
    Linear { slope: f32 },

    /// Pulse modulation.
    ///
    /// # Fields
    ///
    /// * `duty_cycle` - Determines the proportion of the cycle in which the wave is high (1.0).
    Pulse { duty_cycle: f32 },

    /// Exponential modulation.
    ///
    /// # Fields
    ///
    /// * `rate` - Defines the rate of growth or decay.
    /// * `depth` - Defines the amplitude.
    /// * `offset` - Value to shift the exponential function horizontally.
    Exponential { rate: f32, depth: f32, offset: f32 },

    /// Random modulation.
    ///
    /// # Fields
    ///
    /// * `seed` - Seed to initialize the random number generator for reproducibility.
    Random { seed: u64 },
}

impl ModulationMode {
    /// Computes the modulation value at a given time.
    ///
    /// # Arguments
    ///
    /// * `time` - The time at which to compute the modulation value.
    /// * `y` - The base value to be modulated.
    ///
    /// # Returns
    ///
    /// The computed modulation value.
    pub fn compute(&self, time: f32) -> f32 {
        match self {
            ModulationMode::Sine { freq, depth, offset } => {
                depth * (freq * time + offset).sin()
            },
            ModulationMode::Linear { slope } => {
                slope * time
            },
            ModulationMode::Exponential { rate, depth, offset } => {
                depth * (rate * time + offset).exp()
            },
            ModulationMode::Peak { midpoint } => {
                let phase = time % 1.0;
                if phase < *midpoint {
                    2.0 * (phase / midpoint) - 1.0
                } else {
                    1.0 - 2.0 * ((phase - midpoint) / (1.0 - midpoint))
                }
            },
            ModulationMode::Pulse { duty_cycle } => {
                if time % 1.0 < *duty_cycle { 1.0 } else { 0.0 }
            },
            ModulationMode::Random { seed } => {
                let mut rng = rand::rngs::StdRng::seed_from_u64(*seed);
                rng.gen::<f32>()
            }
        }
    }
}

/// Computes the modulation value for a list of modulators at a given time.
///
/// # Arguments
///
/// * `modulators` - A slice of `ModulationMode` enums to be applied.
/// * `time` - The time at which to compute the modulation value.
/// * `y` - The base value to be modulated.
///
/// # Returns
///
/// A vector of computed modulation values.
pub fn compute_modulators(modulators: &[ModulationMode], time: f32, y: f32) -> Vec<f32> {
    modulators.iter().map(|mode| mode.compute(time)).collect()
}

/// Creates a uniform list of modulation modes.
///
/// # Arguments
///
/// * `modulator` - The modulation mode to be replicated.
/// * `len` - The number of replicas.
///
/// # Returns
///
/// A vector of replicated modulation modes.
pub fn uniform_modulators(modulator: ModulationMode, len: usize) -> Vec<ModulationMode> {
    vec![modulator; len]
}

/// Iterates over a list of modulation modes.
///
/// # Arguments
///
/// * `modulators` - The list of modulation modes to be iterated over.
/// * `len` - The length of the resulting vector.
///
/// # Returns
///
/// A vector of modulation modes.
pub fn iterate_modulators(modulators: Vec<ModulationMode>, len: usize) -> Vec<ModulationMode> {
    (0..len)
        .map(|i| modulators[i % modulators.len()].clone())
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn main() {
        let modulator = ModulationMode::Sine { freq: 2.0, depth: 0.5, offset: 0.1 };
        let result = modulator.compute(1.0);
        println!("Computed value: {}", result);

        // Example of using one-liner pattern matching
        let mode = ModulationMode::Sine { freq: 2.0, depth: 0.5, offset: 0.1 };

        if let ModulationMode::Sine { freq, depth, offset } = mode {
            println!("Sine: freq={}, depth={}, offset={}", freq, depth, offset);
        }

        match mode {
            ModulationMode::Sine { freq, depth, offset } => {
                println!("Sine: freq={}, depth={}, offset={}", freq, depth, offset);
            },
            _ => {}
        }

        // Example of computing multiple modulators
        let modulators = vec![
            ModulationMode::Sine { freq: 2.0, depth: 0.5, offset: 0.1 },
            ModulationMode::Linear { slope: 1.0 },
            ModulationMode::Exponential { rate: 1.0, depth: 0.5, offset: 0.0 },
        ];
        let results = compute_modulators(&modulators, 1.0, 2.0);
        for (i, result) in results.iter().enumerate() {
            println!("Modulator {}: {}", i, result);
        }
    }
}
