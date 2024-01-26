pub struct SynthConfig {
    pub sample_rate: u32,
    pub min_frequency: f32,
    pub max_frequency: f32,
    pub amplitude_scaling: f32,
    pub phase_offset: f32,
    pub tuning_offset_hz: f32,
    pub cps: f32
}

impl SynthConfig {
    pub fn new(sample_rate: u32, min_frequency: f32, max_frequency: f32, amplitude_scaling: f32, phase_offset: f32, tuning_offset_hz: f32, cps: f32) -> SynthConfig {
        SynthConfig { 
            sample_rate,
            min_frequency,
            max_frequency,
            amplitude_scaling,
            phase_offset,
            tuning_offset_hz,
            cps
        }
    }
}
