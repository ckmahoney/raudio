use crate::synth_config::SynthConfig;

fn normalize(signal: &mut Vec<f32>) {
    let max_amplitude = signal.iter().map(|&sample| sample.abs()).fold(0.0, f32::max);
    if max_amplitude != 0.0 && max_amplitude > 1.0 {
        signal.iter_mut().for_each(|sample| *sample /= max_amplitude);
    }
}


pub fn apply(config: &SynthConfig, signal: &Vec<f32>, modulators: Vec<Vec<f32>>) -> Result<Vec<f32>, &'static str> {
    let buffer_length = signal.len();

    if modulators.iter().any(|m| m.len() > buffer_length) {
        return Err("Modulators cannot be longer than the primary signal");
    }

    let mut modulated_signal = signal.clone();

    for modulator in modulators {
        for (i, mod_sample) in modulator.into_iter().enumerate().take(buffer_length) {
            modulated_signal[i] *= mod_sample;
        }
    }

    normalize(&mut modulated_signal);

    Ok(modulated_signal.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Define a basic config for testing
    fn test_config() -> SynthConfig {
        SynthConfig {
            sample_rate: 44100,
            min_frequency: 20.0,
            max_frequency: 20000.0,
            amplitude_scaling: 1.0,
            phase_offset: 0.0,
            tuning_offset_hz: 0.0,
            cps: 1.0,
        }
    }

    #[test]
    fn apply_with_no_modulators() {
        let config = test_config();
        let signal = vec![1.0, 2.0, 3.0];
        let modulators: Vec<Vec<f32>> = vec![];
        let result = apply(&config, &signal, modulators).unwrap();
        let mut expected = vec![1.0, 2.0, 3.0];
        normalize(&mut expected);
        assert_eq!(result, expected);
    }

    #[test]
    fn apply_with_modulators_same_length() {
        let config = test_config();
        let signal = vec![1.0, 2.0, 3.0];
        let modulators = vec![vec![0.5, 1.0, 1.5], vec![2.0, 2.0, 2.0]];
        let mut expected = vec![1.0 * 0.5 * 2.0, 2.0 * 1.0 * 2.0, 3.0 * 1.5 * 2.0];
        normalize(&mut expected);
        let result = apply(&config, &signal, modulators).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn apply_with_modulators_shorter_length() {
        let config = test_config();
        let signal = vec![1.0, 2.0, 3.0, 4.0];
        let modulators = vec![vec![0.5, 1.0, 1.5]];
        let mut expected = vec![1.0 * 0.5, 2.0 * 1.0, 3.0 * 1.5, 4.0];
        normalize(&mut expected);
        let result = apply(&config, &signal, modulators).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn apply_with_modulator_longer_than_signal() {
        let config = test_config();
        let signal = vec![1.0, 2.0];
        let modulators = vec![vec![0.5, 1.0, 1.5]];
        let result = apply(&config, &signal, modulators);
        assert!(result.is_err());
    }
}
