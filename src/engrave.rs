use crate::types::synthesis::*;
use crate::types::render::*;

use crate::song;
use crate::midi;
use crate::midi::Midi;
use crate::synth;


fn midi_to_mote(cps:f32, (duration, note, amplitude):&Midi) -> Mote {
    let frequency = midi::note_to_frequency(*note as f32);
    let amp = midi::map_amplitude(*amplitude as f32);
    let dur = duration / cps;

    (dur, frequency, amp)
}

pub fn midi_mel_to_mote(cps:f32, mel:&Vec<Midi>) -> Vec<Mote> {
    mel.into_iter().map(|&mid| midi_to_mote(cps, &mid)).collect()
}

/// Given a list of score part, create a list of motes. 
pub fn midi_entry_to_motes(cps:f32, entry:ScoreEntry<Midi>) -> Melody<Mote> {
    let midi_mels = entry.1;
    midi_mels.into_iter().map(|midi_mel| midi_mel_to_mote(cps, &midi_mel)).collect()
}


pub fn process_midi_parts(parts: Vec::<ScoreEntry<Midi>>, cps: f32) -> Vec<Melody<Mote>> {
    parts.into_iter().map(|entry|
        midi_entry_to_motes(cps, entry)
    ).collect()
}

pub fn transform_to_sample_buffers(cps:f32, motes: &Vec<Mote>) -> Vec<synth::SampleBuffer> {
    motes.iter().map(|&(duration, frequency, amplitude)| {
        synth::samp_ugen(44100, cps, amplitude, synth::silly_sine, duration, frequency)
    }).collect()
}

pub fn transform_to_sample_pairs(cps:f32, motes: &Vec<Mote>) -> Vec<(f32, synth::SampleBuffer)> {
    motes.iter().map(|&(duration, frequency, amplitude)| {
        (frequency, synth::samp_ugen(44100, cps, amplitude, synth::silly_sine, duration, frequency))
    }).collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::song::x_files;
    use crate::render; 
    use crate::files;
    #[test]
    fn test_song_x_files() {
        let track = x_files::get_track();
        let cps = track.conf.cps;
        let processed_parts = process_midi_parts(track.composition.parts, cps);
        let mut buffs:Vec<Vec<synth::SampleBuffer>> = Vec::new();

        for mote_mels in processed_parts {
            for mel_mote in mote_mels {
                buffs.push(transform_to_sample_buffers(cps, &mel_mote))
            }
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