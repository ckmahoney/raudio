// use rustfft::FftPlanner;
// use rustfft::num_complex::Complex;
// use rustfft::num_traits::Zero;

// fn zero_pad(signal: &[f32], length: usize) -> Vec<Complex<f32>> {
//     let mut padded_signal = vec![Complex::zero(); length];
//     for (i, &sample) in signal.iter().enumerate() {
//         padded_signal[i] = Complex::new(sample, 0.0);
//     }
//     padded_signal
// }

// fn fft_convolution(signal: &[f32], impulse: &[f32]) -> Vec<f32> {
//     let output_length = signal.len() + impulse.len() - 1;
//     let padded_signal = zero_pad(signal, output_length);
//     let padded_impulse = zero_pad(impulse, output_length);

//     let mut planner = FftPlanner::new(false);
//     let mut fft_signal = padded_signal.clone();
//     let fft_plan_signal = planner.plan_fft(output_length);
//     fft_plan_signal.process(&mut fft_signal);
//     let mut fft_impulse = padded_impulse.clone();
//     let fft_plan_impulse = planner.plan_fft(output_length);
//     fft_plan_impulse.process(&mut fft_impulse);

//     let mut fft_result: Vec<Complex<f32>> = fft_signal.iter()
//         .zip(fft_impulse.iter())
//         .map(|(x, y)| *x * *y)
//         .collect();

//     let mut planner = FftPlanner::new(true);
//     planner.plan_fft(output_length).process(&mut fft_result);

//     fft_result.iter().map(|x| x.re / output_length as f32).collect()
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_fft_convolution() {
//         let signal = vec![1.0, 2.0, 3.0];
//         let impulse = vec![0.5, 0.25];
//         let expected_result = vec![0.5, 1.25, 2.25, 0.75];
//         assert_eq!(fft_convolution(&signal, &impulse), expected_result);
//     }
// }

// fn fft_convolution_same_length(signal: &[f32], impulse: &[f32]) -> Vec<f32> {
//     let output_length = signal.len() + impulse.len() - 1;
//     let padded_signal = zero_pad(signal, output_length);
//     let padded_impulse = zero_pad(impulse, output_length);

//     let mut planner = FftPlanner::new(false);
//     let mut fft_signal = padded_signal.clone();
//     let fft_plan_signal = planner.plan_fft(output_length);
//     fft_plan_signal.process(&mut fft_signal);
//     let mut fft_impulse = padded_impulse.clone();
//     let fft_plan_impulse = planner.plan_fft(output_length);
//     fft_plan_impulse.process(&mut fft_impulse);

//     let mut fft_result: Vec<Complex<f32>> = fft_signal.iter()
//         .zip(fft_impulse.iter())
//         .map(|(x, y)| *x * *y)
//         .collect();

//     let mut planner = FftPlanner::new(true);
//     planner.plan_fft(output_length).process(&mut fft_result);

//     // Calculate the start and end indices for slicing the FFT result to match the original signal length
//     let start_index = (impulse.len() - 1) / 2;
//     let end_index = start_index + signal.len();
//     fft_result[start_index..end_index].iter().map(|x| x.re / output_length as f32).collect()
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_fft_convolution_same_length() {
//         let signal = vec![1.0, 2.0, 3.0, 4.0];
//         let impulse = vec![0.5, 0.25];
//         let expected_result = vec![1.25, 2.25, 3.25, 2.0]; // Example expected output, adjust as needed
//         assert_eq!(fft_convolution_same_length(&signal, &impulse), expected_result);
//     }
// }
