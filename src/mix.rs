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
        return Err("Mixers cannot be longer than the primary signal");
    }

    let mut modulated_signal = signal.clone();

    for modulator in modulators {
        for (i, mod_sample) in modulator.into_iter().enumerate().take(buffer_length) {
            modulated_signal[i] += mod_sample;
        }
    }

    normalize(&mut modulated_signal);

    Ok(modulated_signal.to_vec())
}
