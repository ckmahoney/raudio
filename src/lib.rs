#![allow(dead_code)]
#![allow(unused_variables)]

pub mod canvas;
pub mod color;
pub mod convolve;
pub mod engrave;
pub mod envelope;
pub mod files;
pub mod freq_forms;
pub mod gen;
pub mod midi;
pub mod mix;
pub mod modulate;
pub mod phrase;
pub mod render;
pub mod sequence;
pub mod song;
pub mod synth;
pub mod synth_config;
pub mod time_forms;

pub fn sum_periods(config: &synth::RenderConfig, selector: &synth::HarmonicSelector, start: usize, max_harmonic: usize, offset: f32) -> synth::SampleBuffer {
    let frequencies = selector.generate_harmonics(start, max_harmonic, offset);
    let periods: Vec<synth::SampleBuffer> = frequencies.iter().map(|f| 
        synth::sample_period(config, synth::silly_sine, *f)
    ).collect();

    synth::silly_sum_periods(config, &frequencies, &periods)
}

pub fn convolve_periods(config: &synth::RenderConfig, selector: &synth::HarmonicSelector, start: usize, max_harmonic: usize, offset: f32) -> synth::SampleBuffer {
    let frequencies = selector.generate_harmonics(start, max_harmonic, offset);
    let periods: Vec<synth::SampleBuffer> = frequencies.iter().map(|f| 
        synth::sample_period(config, synth::silly_sine, *f)
    ).collect();

    synth::silly_convolve_periods(&periods)
}
