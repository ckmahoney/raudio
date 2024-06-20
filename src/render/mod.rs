pub mod blend; 
pub mod engrave;
pub mod ifft;
pub mod realize; 

use self::realize::normalize_channels;


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

pub fn norm_scale(buffer: &mut Vec<f32>, scale:f32) {
    if buffer.is_empty() {
        return;
    }

    let max_amplitude = buffer.iter().map(|&sample| sample.abs()).fold(0.0, f32::max);

    if max_amplitude != 0.0 {
        buffer.iter_mut().for_each(|sample| *sample /= scale * max_amplitude);
    }
}


pub fn amp_scale(buffer:&mut Vec<f32>, amp: f32) {
    buffer.iter_mut().for_each(|sample| *sample *= amp)
}

pub fn mix_and_normalize_buffers(buffers: Vec<Vec<f32>>) -> Result<Vec<f32>, &'static str> {
    realize::mix_buffers(&mut buffers.clone())
}

pub fn pad_and_mix_buffers(buffers: Vec<Vec<f32>>) -> Result<Vec<f32>, &'static str> {
    if buffers.is_empty() {
        return Ok(Vec::new());
    }

    let max_buffer_length = buffers.iter().map(|b| b.len()).max().unwrap_or(0);
    let padded_buffers = buffers.into_iter()
    .map(|buffer| {
        let mut padded = buffer;
        padded.resize(max_buffer_length, 0.0);
        padded
    })
    .collect();

    mix_and_normalize_buffers(padded_buffers)
}

use crate::time;
use crate::synth::{MF, NF, SR, SampleBuffer, pi, pi2};
use crate::types::render::{Melody};
use crate::druid::{Elementor, Element, ApplyAt, melody_frexer, inflect};
use crate::render::blend::{GlideLen};
use crate::phrasing::lifespan;
use crate::phrasing::contour;
use crate::types::timbre::{AmpContour,Arf};
use crate::types::synthesis::Note;

pub fn arf(cps:f32, contour:&AmpContour, melody:&Melody<Note>, synth:&Elementor, arf:Arf) -> SampleBuffer {
    let melody_frexd = melody_frexer(&melody, GlideLen::None, GlideLen::None);
    let mut channels:Vec<SampleBuffer> = Vec::with_capacity(melody.len());
    let mut seed:ThreadRng = thread_rng();
    
    // @art-choice: apply a visible dynamic amp mix for the synth as a whole
    let mod_amp_synth:f32 = 0.5f32 + 0.5 * seed.gen::<f32>();

    for (index, line_frexd) in melody_frexd.iter().enumerate() {
        let mut line_buff:SampleBuffer = Vec::new();
        let line = &melody[index];
        // @art-choice: apply a background dynamic amp for the melodies within as well
        let mod_amp_melody:f32 = 0.8f32 + 0.2 * seed.gen::<f32>();

        for (jindex, frex) in line_frexd.iter().enumerate() {
            let mod_amp_dither:f32 = 0.99 + 0.01 * seed.gen::<f32>();

            let dur = time::duration_to_cycles(line[jindex].0);
            let amp = line[jindex].2;
            let at = ApplyAt { frex: *frex, span: (cps, dur) };
            let weight_modulation:f32 = mod_amp_dither * mod_amp_melody * mod_amp_synth;
            let applied:Elementor = synth.iter().map(|(w, r)| (weight_modulation* *w * amp, *r)).collect();
            line_buff.append(&mut inflect(
                &frex, 
                &at, 
                &applied, 
                &arf.visibility,
                &arf.energy,
                &arf.presence
            ));
        }
        channels.push(line_buff)
    }

    match realize::mix_buffers(&mut channels) {
        Ok(mut mixdown) => {
            let cont = lifespan::mod_lifespan(
                mixdown.len()/2, 
                1f32, 
                &lifespan::select_lifespan(&contour), 
                1usize, 
                1f32
            );
            contour::apply_contour(&mut mixdown, &cont);
            mixdown
        },
        Err(msg) => panic!("Error while preparing mixdown: {}", msg)
    }

}
use rand;
use rand::{Rng, thread_rng};
use rand::rngs::ThreadRng;

