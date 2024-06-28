/// amplitude modulation has rate and depth
/// phase modulation has rate and depth and offset
/// frequency modulation has rate 
/// params are defined at the start of a melody.
/// they are passed to the blender to be mixed with the envelope and dressing values.

struct Dressing {
    len: usize,
    pub multipliers: Vec<f32>,
    pub amplitudes: Vec<f32>,
    pub offsets: Vec<f32>,
}

/// Instance of arguments for additive synthesizer
impl Dressing {
    fn empty(len: usize) -> Self {
        Dressing {
            len,
            multipliers: vec![0f32; len],
            amplitudes: vec![0f32; len],
            offsets: vec![0f32; len],
        }
    }

    fn new(amplitudes: Vec<f32>, multipliers: Vec<f32>, offsets: Vec<f32>) -> Self {
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

    fn set_muls(&mut self, muls: Vec<f32>) {
        if muls.len() != self.len {
            panic!(
                "Unable to update multipliers. Requires a vector of length {} but got actual length {}",
                self.multipliers.len(),
                muls.len()
            );
        }
        self.multipliers = muls;
    }

    fn set_amplitudes(&mut self, amps: Vec<f32>) {
        if amps.len() != self.len {
            panic!(
                "Unable to update amplitudes. Requires a vector of length {} but got actual length {}",
                self.amplitudes.len(),
                amps.len()
            );
        }
        self.amplitudes = amps;
    }

    fn set_offsets(&mut self, offsets: Vec<f32>) {
        if offsets.len() != self.len {
            panic!(
                "Unable to update offsets. Requires a vector of length {} but got actual length {}",
                self.offsets.len(),
                offsets.len()
            );
        }
        self.offsets = offsets;
    }

    fn unit_amp(length: usize) -> Vec<f32> {
        vec![1f32; length]
    }

    fn unit_freq(length: usize) -> Vec<f32> {
        (1..=length).map(|i| i as f32).collect()
    }

    fn unit_offset(length: usize) -> Vec<f32> {
        vec![0f32; length]
    }

    fn to_string(&self) -> String {
        format!(
            "Dressing {{ len: {}, amplitudes: {:?}, multipliers: {:?}, offsets: {:?} }}",
            self.len, self.amplitudes, self.multipliers, self.offsets
        )
    }

    fn normalize(&mut self) {
        let sum: f32 = self.amplitudes.iter().sum();
        if sum > 0.0 {
            self.amplitudes.iter_mut().for_each(|a| *a /= sum);
        }
    }

}
enum ModulationParams {
    Amplitude { rate: f32, depth: f32, offset: Option<f32> },
    Frequency { rate: f32, offset: Option<f32> },
    Phase { rate: f32, depth: f32, offset: f32 },
}

impl ModulationParams {
    fn default_offset() -> f32 {
        0.0
    }

    /// Computes the modulation value at a given time.
    ///
    /// # Arguments
    ///
    /// * `time` - The current time, which affects the modulation calculation.
    ///
    /// # Returns
    ///
    /// The modulated value at the given time.
    fn at(&self, time: f32) -> f32 {
        match self {
            ModulationParams::Amplitude { rate, depth, offset } => {
                let offset = offset.unwrap_or(Self::default_offset());
                depth * (rate * time + offset).sin()
            },
            ModulationParams::Frequency { rate, offset } => {
                let offset = offset.unwrap_or(Self::default_offset());
                (rate * time + offset).sin()
            },
            ModulationParams::Phase { rate, depth, offset } => {
                depth * (rate * time + offset)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modulation_params() {
        let amp_mod = ModulationParams::Amplitude { rate: 2.0, depth: 0.5, offset: Some(0.1) };
        let freq_mod = ModulationParams::Frequency { rate: 5.0, offset: None };
        let phase_mod = ModulationParams::Phase { rate: 0.5, depth: 0.3, offset: 0.0 };

        let time = 0.5; // Current time in seconds

        let modulated_amplitude = amp_mod.at(time);
        let modulated_frequency = freq_mod.at(time);
        let modulated_phase = phase_mod.at(time);

        println!("Modulated Amplitude: {}", modulated_amplitude);
        println!("Modulated Frequency: {}", modulated_frequency);
        println!("Modulated Phase: {}", modulated_phase);

        // Add assertions to verify the correctness of the results
        assert!(modulated_amplitude.abs() <= 0.5); // amplitude modulation depth is 0.5
        assert!(modulated_frequency.abs() <= 1.0); // frequency modulation should be within -1.0 to 1.0
        assert!(modulated_phase.abs() <= 0.3);     // phase modulation depth is 0.3
    }
}


// enum ModulationEffect {
//     Tremelo(ModulationParams::Amplitude),
//     Vibrato(ModulationParams::Phase),
//     Noise(ModulationParams::Phase),
//     Chorus(ModulationParams::Phase),
//     Glide(ModulationParams::Frequency)
// }

enum BaseBand {
    Sine(f32),
    Peak(f32),
    Linear(f32),
    Pulse(f32)
}

type AmplitudeModulation = fn(&ModulationParams, usize, f32, f32, Option<f32>, Option<f32>, Option<f32>) -> f32;