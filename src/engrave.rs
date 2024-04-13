use crate::files;
use crate::song::*;
use crate::midi;
use crate::midi::{Duration, Midi};
use crate::synth;

pub fn process_parts(parts: &HashMap<Spec, Vec<Vec<Midi>>>, cps: f32) -> Vec<(Spec, Vec<(Duration, f32, f32)>)> {
    parts.iter().map(|(spec, midi_vecs)| {
        let mote_vec = midi_vecs.iter().flat_map(|midi_vec| {
            midi_vec.iter().map(|&(duration, note, amplitude)| {
                let frequency = midi::note_to_frequency(note as f32);
                let amplitude_mapped = midi::map_amplitude(amplitude as u8);
                let adjusted_duration = duration / cps;

                (adjusted_duration, frequency, amplitude_mapped)
            })
        }).collect::<Vec<(Duration, f32, f32)>>();

        (spec.clone(), mote_vec)
    }).collect()
}

pub fn transform_to_sample_buffers(cps:f32, motes: &Vec<(Duration, f32, f32)>) -> Vec<synth::SampleBuffer> {
    motes.iter().map(|&(duration, frequency, amplitude)| {
        synth::samp_ugen(44100, cps, amplitude, synth::silly_sine, duration, frequency)
    }).collect()
}

pub fn transform_to_sample_pairs(cps:f32, motes: &Vec<(Duration, f32, f32)>) -> Vec<(f32, synth::SampleBuffer)> {
    motes.iter().map(|&(duration, frequency, amplitude)| {
        (frequency, synth::samp_ugen(44100, cps, amplitude, synth::silly_sine, duration, frequency))
    }).collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::song::x_files::TRACK;
    use crate::render; 

    #[test]
    fn test_song_x_files() {

        let cps = TRACK.conf.cps;
        let processed_parts = process_parts(&TRACK.composition.parts, cps);
        let mut buffs:Vec<Vec<synth::SampleBuffer>> = Vec::new();

        for (spec, motes) in processed_parts {
            buffs.push(transform_to_sample_buffers(cps, &motes))
        }

        let mixers:Vec<synth::SampleBuffer> = buffs.into_iter().map(|buff|
            buff.into_iter().flatten().collect()
        ).collect();

        files::with_dir("dev-audio");
        match render::pad_and_mix_buffers(mixers) {
            Ok(signal) => {
                render::samples_f32(44100, &signal, "dev-audio/x_files.wav");
            },
            Err(err) => {
                println!("Problem while mixing buffers. Message: {}", err)
            }
        }
    }

}