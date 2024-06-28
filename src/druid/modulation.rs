use rand::SeedableRng;
use rand::Rng;

#[derive(Debug)]
struct Dressing {
    len: usize,
    pub multipliers: Vec<f32>,
    pub amplitudes: Vec<f32>,
    pub offsets: Vec<f32>,
}

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

    fn normalize(&mut self) {
        let sum: f32 = self.amplitudes.iter().sum();
        if sum > 0.0 {
            self.amplitudes.iter_mut().for_each(|a| *a /= sum);
        }
    }

    fn to_string(&self) -> String {
        format!(
            "Dressing {{ len: {}, amplitudes: {:?}, multipliers: {:?}, offsets: {:?} }}",
            self.len, self.amplitudes, self.multipliers, self.offsets
        )
    }
}

#[derive(Debug, Clone)]
struct HarmonicModulator(Option<(ModulationMode, ModulationParams)>);

#[derive(Debug, Clone)]
enum ModulationParams {
    Amplitude { rate: f32, depth: f32, offset: Option<f32> },
    Frequency { rate: f32, offset: Option<f32> },
    Phase { rate: f32, depth: f32, offset: f32 },
}

#[derive(Debug, Clone)]
enum ModulationMode {
    Sine,
    Peak { midpoint: f32 },
    Linear,
    Pulse { duty_cycle: f32 },
    Exponential,
    Random { seed: Option<u64> },
    Quadratic { a: f32, b: f32, c: f32 },
}

impl ModulationParams {
    fn default_offset() -> f32 {
        0.0
    }

    fn at(&self, time: f32) -> f32 {
        match self {
            ModulationParams::Amplitude { rate, depth, offset } => {
                let offset = offset.unwrap_or(Self::default_offset());
                depth * ModulationParams::apply_mode(rate * time + offset, &ModulationMode::Sine)
            },
            ModulationParams::Frequency { rate, offset } => {
                let offset = offset.unwrap_or(Self::default_offset());
                ModulationParams::apply_mode(rate * time + offset, &ModulationMode::Sine)
            },
            ModulationParams::Phase { rate, depth, offset } => {
                depth * ModulationParams::apply_mode(rate * time + offset, &ModulationMode::Sine)
            },
        }
    }

    fn apply_mode(value: f32, mode: &ModulationMode) -> f32 {
        match mode {
            ModulationMode::Sine => value.sin(),
            ModulationMode::Peak { midpoint } => {
                let phase = value % 1.0;
                if phase < *midpoint {
                    2.0 * (phase / midpoint) - 1.0
                } else {
                    1.0 - 2.0 * ((phase - midpoint) / (1.0 - midpoint))
                }
            },
            ModulationMode::Linear => value,
            ModulationMode::Pulse { duty_cycle } => {
                if value % 1.0 < *duty_cycle { 1.0 } else { 0.0 }
            },
            ModulationMode::Exponential => value.exp(),
            ModulationMode::Random { seed } => {
                if let Some(seed) = seed {
                    let mut rng = rand::rngs::StdRng::seed_from_u64(*seed);
                    rng.gen::<f32>()
                } else {
                    rand::random::<f32>()
                }
            },
            ModulationMode::Quadratic { a, b, c } => a * value.powi(2) + b * value + c,
        }
    }
}

impl HarmonicModulator {
    fn new(mode: Option<ModulationMode>, params: Option<ModulationParams>) -> Self {
        HarmonicModulator(mode.zip(params))
    }

    fn uniform(modulator: HarmonicModulator, dressing: &Dressing) -> Vec<HarmonicModulator> {
        vec![modulator; dressing.len]
    }

    fn iterate(modulators: Vec<HarmonicModulator>, dressing: &Dressing) -> Vec<HarmonicModulator> {
        (0..dressing.len)
            .map(|i| modulators[i % modulators.len()].clone())
            .collect()
    }

    fn modulate(&self, time: f32, base_value: f32, modulation_type: &str) -> f32 {
        if let Some((mode, params)) = &self.0 {
            let modulated_value = match mode {
                ModulationMode::Sine => (params.at(time)).sin(),
                ModulationMode::Peak { midpoint } => {
                    let phase = (params.at(time)) % 1.0;
                    if phase < *midpoint {
                        2.0 * (phase / midpoint) - 1.0
                    } else {
                        1.0 - 2.0 * ((phase - midpoint) / (1.0 - midpoint))
                    }
                },
                ModulationMode::Linear => params.at(time),
                ModulationMode::Pulse { duty_cycle } => {
                    if (params.at(time)) % 1.0 < *duty_cycle { 1.0 } else { 0.0 }
                },
                ModulationMode::Exponential => (params.at(time)).exp(),
                ModulationMode::Random { seed } => {
                    if let Some(seed) = seed {
                        let mut rng = rand::rngs::StdRng::seed_from_u64(*seed);
                        rng.gen::<f32>()
                    } else {
                        rand::random::<f32>()
                    }
                },
                ModulationMode::Quadratic { a, b, c } => a * (params.at(time)).powi(2) + b * params.at(time) + c,
            };

            match modulation_type {
                "amplitude" => modulated_value * base_value,
                "frequency" => modulated_value + base_value,
                "phase" => modulated_value + base_value,
                _ => base_value,
            }
        } else {
            base_value
        }
    }
}

#[derive(Debug, Clone)]
enum ModulationEffect {
    Tremelo(ModulationParams::Amplitude),
    Vibrato(ModulationParams::Phase),
    Noise(ModulationParams::Phase),
    Chorus(ModulationParams::Phase),
    Glide(ModulationParams::Frequency),
}

impl ModulationEffect {
    fn apply(&self, time: f32, base_value: f32) -> f32 {
        match self {
            ModulationEffect::Tremelo(params) => params.at(time) * base_value,
            ModulationEffect::Vibrato(params) => params.at(time) + base_value,
            ModulationEffect::Noise(params) => params.at(time) + base_value,
            ModulationEffect::Chorus(params) => params.at(time) + base_value,
            ModulationEffect::Glide(params) => params.at(time) + base_value,
        }
    }
}

#[derive(Debug, Clone)]
struct ModulationChain {
    effects: Vec<ModulationEffect>,
}

impl ModulationChain {
    fn new() -> Self {
        ModulationChain {
            effects: Vec::new(),
        }
    }

    fn add_effect(&mut self, effect: ModulationEffect) {
        self.effects.push(effect);
    }

    fn remove_effect(&mut self, index: usize) {
        if index < self.effects.len() {
            self.effects.remove(index);
        }
    }

    fn apply(&self, time: f32, base_value: f32) -> f32 {
        self.effects.iter().fold(base_value, |acc, effect| effect.apply(time, acc))
    }
}

impl HarmonicModulator {
    fn apply_effects(&self, time: f32, base_value: f32, chain: &ModulationChain) -> f32 {
        chain.apply(time, base_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dressing() {
        let amps = vec![0.5, 0.8, 1.0];
        let muls = vec![1.0, 2.0, 3.0];
        let offsets = vec![0.0, 0.1, 0.2];

        let mut dressing = Dressing::new(amps, muls, offsets);
        println!("{:?}", dressing);

        dressing.normalize();
        println!("Normalized: {:?}", dressing);

        let new_muls = vec![1.1, 2.1, 3.1];
        dressing.set_muls(new_muls);
        println!("Updated Multipliers: {:?}", dressing.multipliers);
    }

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

    #[test]
    fn test_harmonic_modulator() {
        let dressing = Dressing::new(vec![0.5, 0.8, 1.0], vec![1.0, 2.0, 3.0], vec![0.0, 0.1, 0.2]);

        let sine_modulator = HarmonicModulator::new(
            Some(ModulationMode::Sine),
            Some(ModulationParams::Amplitude { rate: 2.0, depth: 0.5, offset: Some(0.1) }),
        );

        let modulators = HarmonicModulator::uniform(sine_modulator.clone(), &dressing);
        assert_eq!(modulators.len(), dressing.len);
        for modulator in modulators {
            println!("{:?}", modulator);
        }

        let alt_modulators = HarmonicModulator::iterate(vec![
            HarmonicModulator::new(
                Some(ModulationMode::Sine),
                Some(ModulationParams::Amplitude { rate: 2.0, depth: 0.5, offset: Some(0.1) }),
            ),
            HarmonicModulator::new(
                Some(ModulationMode::Peak { midpoint: 0.5 }),
                Some(ModulationParams::Frequency { rate: 5.0, offset: None }),
            ),
        ], &dressing);
        
        assert_eq!(alt_modulators.len(), dressing.len);
        for (i, modulator) in alt_modulators.iter().enumerate() {
            println!("Harmonic {}: {:?}", i, modulator);
        }
    }

    #[test]
    fn test_modulation_chain() {
        let mut chain = ModulationChain::new();
        chain.add_effect(ModulationEffect::Tremelo(ModulationParams::Amplitude { rate: 2.0, depth: 0.5, offset: Some(0.1) }));
        chain.add_effect(ModulationEffect::Vibrato(ModulationParams::Phase { rate: 5.0, depth: 0.3, offset: 0.0 }));

        let base_value = 1.0;
        let time = 0.5;

        let result = chain.apply(time, base_value);
        println!("Result: {}", result);

        assert!(result.abs() <= 1.0);
    }
}

fn main() {
    // Initialize Dressing
    let amps = vec![0.5, 0.8, 1.0];
    let muls = vec![1.0, 2.0, 3.0];
    let offsets = vec![0.0, 0.1, 0.2];

    let mut dressing = Dressing::new(amps, muls, offsets);
    dressing.normalize();
    println!("Normalized: {:?}", dressing);

    // Initialize Harmonic Modulators
    let sine_modulator = HarmonicModulator::new(
        Some(ModulationMode::Sine),
        Some(ModulationParams::Amplitude { rate: 2.0, depth: 0.5, offset: Some(0.1) }),
    );

    let modulators = HarmonicModulator::uniform(sine_modulator.clone(), &dressing);
    for modulator in modulators.iter() {
        let mut chain = ModulationChain::new();
        chain.add_effect(ModulationEffect::Tremelo(ModulationParams::Amplitude { rate: 2.0, depth: 0.5, offset: Some(0.1) }));
        chain.add_effect(ModulationEffect::Vibrato(ModulationParams::Phase { rate: 5.0, depth: 0.3, offset: 0.0 }));
        let modulated_value = modulator.apply_effects(0.5, 1.0, &chain);
        println!("Modulated Value: {}", modulated_value);
    }
}
