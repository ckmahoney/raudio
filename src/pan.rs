pub fn left_scaling_factor(pan: f64) -> f64 {
    let pi_over_four = std::f64::consts::PI / 4.0;
    let theta = (pan + 1.0) * pi_over_four; // Scaling pan from -1 to 1 to the range of 0 to PI/2
    (theta.cos() * 0.5 + 0.5).sqrt()
}

pub fn right_scaling_factor(pan: f64) -> f64 {
    let pi_over_four = std::f64::consts::PI / 4.0;
    let theta = (pan + 1.0) * pi_over_four; // Scaling pan from -1 to 1 to the range of 0 to PI/2
    (theta.sin() * 0.5 + 0.5).sqrt()
}

pub fn get_pan_modulation_amplitudes(pan: f64) -> (f64, f64) {
    let left_amp = left_scaling_factor(pan);
    let right_amp = right_scaling_factor(pan);

    (left_amp, right_amp)
}

pub struct AudioChannel {
    buffer: Vec<f64>,
    buffer_position: usize,
    delay_samples: usize,
}

impl AudioChannel {
    pub fn new(max_delay_ms: usize, sample_rate: usize) -> Self {
        let max_delay_samples = max_delay_ms * sample_rate / 1000;
        AudioChannel {
            buffer: vec![0.0; max_delay_samples],
            buffer_position: 0,
            delay_samples: 0,
        }
    }

    pub fn set_delay(&mut self, delay_ms: usize, sample_rate: usize) {
        self.delay_samples = delay_ms * sample_rate / 1000;
    }

    pub fn process_sample(&mut self, input_sample: f64) -> f64 {
        let output_sample = self.buffer[self.buffer_position];
        self.buffer[self.buffer_position] = input_sample;
        self.buffer_position = (self.buffer_position + 1) % self.buffer.len();
        output_sample
    }
}

// Example of usage within an audio processing context
pub fn process_audio_signal(input_sample: f64, pan: f64, left_channel: &mut AudioChannel, right_channel: &mut AudioChannel) -> (f64, f64) {
    let (left_amp, right_amp) = get_pan_modulation_amplitudes(pan);
    let left_output = left_channel.process_sample(input_sample * left_amp);
    let right_output = right_channel.process_sample(input_sample * right_amp);

    (left_output, right_output)
}