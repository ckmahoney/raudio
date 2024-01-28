use crate::synth_config::SynthConfig;

pub type Ugen = fn(&SynthConfig, u32, f32, Option<f32>) -> f32;

pub fn rescale(samples: &[f32], original_freq: f32, target_freq: f32) -> Vec<f32> {
    let ratio = original_freq / target_freq;
    let new_length = (samples.len() as f32 * ratio) as usize;
    let mut resampled = Vec::with_capacity(new_length);

    for i in 0..new_length {
        let float_idx = i as f32 / ratio;
        let idx = float_idx as usize;
        let next_idx = if idx + 1 < samples.len() { idx + 1 } else { idx };
        
        // Linear interpolation
        let sample = if idx != next_idx {
            let fraction = float_idx.fract();
            samples[idx] * (1.0 - fraction) + samples[next_idx] * fraction
        } else {
            samples[idx]
        };

        resampled.push(sample);
    }

    resampled
}

pub fn normalize(buffer: &mut Vec<f32>) {
    if buffer.is_empty() {
        return;
    }

    let max_amplitude = buffer.iter().map(|&sample| sample.abs()).fold(0.0, f32::max);

    if max_amplitude != 0.0 {
        buffer.iter_mut().for_each(|sample| *sample /= max_amplitude);
    }
}


pub fn amp_scale(buffer:&mut Vec<f32>, amp: f32) {
    buffer.iter_mut().for_each(|sample| *sample *= amp)
}

pub fn mix_and_normalize_buffers(buffers: Vec<Vec<f32>>) -> Result<Vec<f32>, &'static str> {
    if buffers.is_empty() {
        return Ok(Vec::new());
    }

    let buffer_length = buffers.first().unwrap().len();
    if buffers.iter().any(|b| b.len() != buffer_length) {
        return Err("Buffers do not have the same length");
    }

    let mut mixed_buffer = vec![0.0; buffer_length];

    for buffer in buffers {
        for (i, sample) in buffer.into_iter().enumerate() {
            mixed_buffer[i] += sample;
        }
    }

    let max_amplitude = mixed_buffer.iter().map(|&sample| sample.abs()).fold(0.0, f32::max);

    if max_amplitude != 0.0 && max_amplitude > 1.0 {
        mixed_buffer.iter_mut().for_each(|sample| *sample /= max_amplitude);
    }

    Ok(mixed_buffer)
}


pub fn render_ugen(config: &SynthConfig, ugen: &Ugen, filename: &str) -> String {
    let dur_cycles = 4;
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: config.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(filename, spec).unwrap();
    let mut ts: Vec<u32> = Vec::new();

    for i in 0..(dur_cycles * config.sample_rate) { 
        ts.push(i) 
    };

    let sequence = render2(config, ts, config.sample_rate, ugen, 440.0, 1.0);
    for sample in sequence {
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();
    String::from("done")
}


pub fn samples(config: &SynthConfig, samples: &Vec<f32>, filename: &str) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: config.sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(filename, spec).unwrap();
    for &sample in samples {
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();
}


pub fn samples_f32(sample_rate:usize, samples: &Vec<f32>, filename: &str) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: sample_rate as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(filename, spec).unwrap();
    for &sample in samples {
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();
}

pub fn render2(config: &SynthConfig, ts: Vec<u32>, sr:u32, ugen: &Ugen, freq: f32, amp: f32) -> Vec<f32> {
    let mut samples: Vec<f32> = Vec::new();
    for t in ts {
        let sample = amp * ugen(config, t, freq, Some(0.5));
        samples.push(sample);
    }
    samples
}

