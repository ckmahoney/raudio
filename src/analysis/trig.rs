use crate::druid::{Element};
    
/// Given vecs representing the amplitude and phase values for a constant reference frequency, 
/// Return the aggregate amplitude and phase value as constant values. 
pub fn merge_soids(
    amplitudes: &Vec<f32>,
    phases: &Vec<f32>
) -> (f32, f32) {
    let real_sum: f32 = amplitudes.iter()
        .zip(phases.iter())
        .map(|(&amp, &phi)| amp * phi.cos())
        .sum();

    let imag_sum: f32 = amplitudes.iter()
        .zip(phases.iter())
        .map(|(&amp, &phi)| amp * phi.sin())
        .sum();

    let resultant_amplitude = (real_sum.powi(2) + imag_sum.powi(2)).sqrt();
    let resultant_phase = imag_sum.atan2(real_sum);

    (resultant_amplitude, resultant_phase)
}

pub fn process_soids(sinusoids: (Vec<f32>, Vec<f32>, Vec<f32>)) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let (mut amps, mut muls, mut phis) = sinusoids;

    // Create a sorted index array based on multipliers
    let mut indices: Vec<usize> = (0..muls.len()).collect();
    indices.sort_by(|&a, &b| muls[a].partial_cmp(&muls[b]).unwrap());

    // Prepare the result vectors
    let mut result_amps = Vec::new();
    let mut result_muls = Vec::new();
    let mut result_phis = Vec::new();

    // Initialize first group
    let mut current_mul = muls[indices[0]];
    let mut grouped_amps = Vec::new();
    let mut grouped_phis = Vec::new();

    for &i in &indices {
        if muls[i] == current_mul {
            grouped_amps.push(amps[i]);
            grouped_phis.push(phis[i]);
        } else {
            let (resultant_amp, resultant_phi) = merge_soids(&grouped_amps, &grouped_phis);
            result_amps.push(resultant_amp);
            result_muls.push(current_mul);
            result_phis.push(resultant_phi);

            // Reset for the next group
            current_mul = muls[i];
            grouped_amps = vec![amps[i]];
            grouped_phis = vec![phis[i]];
        }
    }

    // Handle the last group
    if !grouped_amps.is_empty() {
        let (resultant_amp, resultant_phi) = merge_soids(&grouped_amps, &grouped_phis);
        result_amps.push(resultant_amp);
        result_muls.push(current_mul);
        result_phis.push(resultant_phi);
    }

    (result_amps, result_muls, result_phis)
}

pub fn el_to_soid(el:&Element) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    (el.amps.to_owned(), el.muls.to_owned(), el.phss.to_owned())
}

/// Flatten the input Elements into a single tuple of (amp, mul, offset) values as (Vec<f32>, Vec<f32>, Vec<f32>)
pub fn prepare_soids_input(
    input: Vec<(Vec<f32>, Vec<f32>, Vec<f32>)>
) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
    let mut amps_flattened = Vec::new();
    let mut muls_flattened = Vec::new();
    let mut phis_flattened = Vec::new();

    for (amps, muls, phis) in input {
        amps_flattened.extend(amps);
        muls_flattened.extend(muls);
        phis_flattened.extend(phis);
    }

    (amps_flattened, muls_flattened, phis_flattened)
}
#[test]
fn test() {
    let amplitudes = vec![1.0, 0.5, 0.8]; // unique amplitudes
    let phases = vec![0.0, std::f32::consts::PI, std::f32::consts::FRAC_PI_2]; // example phases

    let (resultant_amplitude, resultant_phase) = merge_soids(&amplitudes, &phases);

    println!("Resultant Amplitude: {}", resultant_amplitude);
    println!("Resultant Phase: {}", resultant_phase);
}