use rand::Rng;
use crate::{analysis::volume::db_to_amp, Visibility};


fn generate_offset_triangle_wave(
    offset_x_samples: f32,
    width_samples: f32,
    min_y: f32,
    max_y: f32,
    midpoint_y_progress: f32,
    n_samples: usize,
) -> Vec<f32> {
    let range_y = (max_y - min_y) / 2.0;
    let mut samples = Vec::with_capacity(n_samples);

    for i in 0..n_samples {
        let x = (i as f32 - offset_x_samples) / width_samples;

        samples.push(if x < 0.0 || x > 1.0 {
            min_y
        } else if x <= midpoint_y_progress {
            min_y + (2.0 * range_y * (x / midpoint_y_progress))
        } else {
            min_y + (2.0 * range_y * ((1.0 - x) / (1.0 - midpoint_y_progress)))
        })
    }

    samples
}


/// create an amplitude contour representing "organic" contour
/// where each layer is wider than the last
fn generate_layers(n_layers: usize, n_samples: usize, greatest_reduction:f32, least_reduction:f32) -> Vec<Vec<f32>> {
    let mut layers = Vec::new();
    let mut rng = rand::thread_rng();

    for i in 1..=n_layers {
        let layer_width = n_samples as f32 * ((i as f32 ) / n_layers as f32);
        let min_y = db_to_amp(greatest_reduction);
        let max_y = db_to_amp(least_reduction);
        let offset_x_samples = rng.gen_range(0.0..n_samples as f32) - layer_width / 2.0;
        let midpoint_y_progress = 4f32*(rng.gen::<f32>()-0.5f32).powi(2i32);

        let samples = generate_offset_triangle_wave(offset_x_samples, layer_width, min_y, max_y, midpoint_y_progress, n_samples);
        layers.push(samples);
    }

    layers
}

/// apply vector addition and then normalize all values by 1/L
fn sum_and_normalize_layers(layers: &[Vec<f32>]) -> Vec<f32> {
    if layers.is_empty() {
        return vec![];
    }

    let layer_length = layers[0].len();
    let n_layers = layers.len() as f32;
    let mut result = vec![0.0; layer_length];

    // Sum corresponding elements across all layers
    for layer in layers {
        for (index, &value) in layer.iter().enumerate() {
            result[index] += value;
        }
    }

    // Normalize by 1/n_layers
    result.iter_mut().for_each(|sum| *sum /= n_layers);
    result
}

/// Add layers and scale shape of output to be in the range of the output itself
fn sum_and_dynamic_normalize_layers(gain:f32, layers: &[Vec<f32>]) -> Vec<f32> {
    if layers.is_empty() {
        return vec![];
    }

    let layer_length = layers[0].len();
    let mut result = vec![0.0; layer_length];

    // Track the max value of each layer
    let max_values: Vec<f32> = layers.iter().map(|layer| *layer.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap()).collect();

    // Sum corresponding elements across all layers
    for layer in layers {
        for (index, &value) in layer.iter().enumerate() {
            result[index] += value;
        }
    }

    // Calculate the overall max and min of all individual layer peaks
    let max_peak_value = *max_values.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let min_peak_value = *max_values.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

    // Find the max value in the summed result
    let result_max = *result.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

    // Scale result to fit within the range of max and min peaks
    let peak_range = max_peak_value - min_peak_value;
    result.iter_mut().for_each(|value| *value = gain * (*value / result_max) * peak_range + min_peak_value);

    result
}


/// Create a varying amplitude layer providing subtle animation
pub fn gen_organic_amplitude(n_layers: usize, n_samples: usize, v:Visibility) -> Vec<f32> {
    let (min, max):(f32,f32) = match v {
        Visibility::Hidden => (-24f32, -18f32),
        Visibility::Background => (-18f32, -12f32),
        Visibility::Foreground => (-12f32, -9f32),
        Visibility::Visible => (-9f32, -6f32),
    };
    let gain = crate::presets::visibility_gain(v);
    let layers = generate_layers(n_layers, n_samples, min, max);
    sum_and_dynamic_normalize_layers(gain, &layers)
}