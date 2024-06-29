use rand::SeedableRng;
use rand::Rng;

/// Represents different modulation modes for harmonic modulation.
#[derive(Debug, Clone)]
pub enum ModulationMode {
    /// Sine wave modulation.
    ///
    /// # Fields
    ///
    /// * `rate` - Defines the frequency of the sine wave.
    /// * `depth` - Defines the amplitude of the sine wave.
    /// * `offset` - Value to shift the wave horizontally.
    Sine { rate: f32, depth: f32, offset: f32 },

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
    /// * `rate` - Defines the rate of increase or decrease.
    Linear { rate: f32 },

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

    /// Quadratic modulation.
    ///
    /// # Fields
    ///
    /// * `a` - Coefficient for the quadratic term (base_value^2).
    /// * `b` - Coefficient for the linear term (base_value).
    /// * `c` - Constant term.
    Quadratic { a: f32, b: f32, c: f32 },
}

impl ModulationMode {
    /// Computes the modulation value at a given time.
    ///
    /// # Arguments
    ///
    /// * `time` - The time at which to compute the modulation value.
    ///
    /// # Returns
    ///
    /// The computed modulation value.
    pub fn compute(&self, time: f32) -> f32 {
        match self {
            ModulationMode::Sine { rate, depth, offset } => {
                depth * (rate * time + offset).sin()
            },
            ModulationMode::Linear { rate } => {
                rate * time
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
            },
            ModulationMode::Quadratic { a, b, c } => {
                let base_value = time; // Use time as the base value
                a * base_value.powi(2) + b * base_value + c
            },
        }
    }
}

/// Computes the modulation value for a list of modulators at a given time.
///
/// # Arguments
///
/// * `modulators` - A slice of `ModulationMode` enums to be applied.
/// * `time` - The time at which to compute the modulation value.
///
/// # Returns
///
/// A vector of computed modulation values.
pub fn compute_modulators(modulators: &[ModulationMode], time: f32) -> Vec<f32> {
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
        let modulator = ModulationMode::Sine { rate: 2.0, depth: 0.5, offset: 0.1 };
        let result = modulator.compute(1.0);
        println!("Computed value: {}", result);

        // Example of using one-liner pattern matching
        let mode = ModulationMode::Sine { rate: 2.0, depth: 0.5, offset: 0.1 };

        if let ModulationMode::Sine { rate, depth, offset } = mode {
            println!("Sine: rate={}, depth={}, offset={}", rate, depth, offset);
        }

        match mode {
            ModulationMode::Sine { rate, depth, offset } => {
                println!("Sine: rate={}, depth={}, offset={}", rate, depth, offset);
            },
            _ => {}
        }

        // Example of computing multiple modulators
        let modulators = vec![
            ModulationMode::Sine { rate: 2.0, depth: 0.5, offset: 0.1 },
            ModulationMode::Linear { rate: 1.0 },
            ModulationMode::Exponential { rate: 1.0, depth: 0.5, offset: 0.0 },
        ];
        let results = compute_modulators(&modulators, 1.0);
        for (i, result) in results.iter().enumerate() {
            println!("Modulator {}: {}", i, result);
        }
    }
}
