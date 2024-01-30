use crate::song;

mod jukebox {
    pub use crate::song::*;
    pub use crate::midi::*;

    pub fn process_parts(parts: &HashMap<Spec, Vec<Vec<Midi>>>, cps: f32) -> Vec<(Spec, Vec<(Duration, f64, f64)>)> {
        parts.iter().map(|(spec, midi_vecs)| {
            let mote_vec = midi_vecs.iter().flat_map(|midi_vec| {
                midi_vec.iter().map(|&(duration, note, amplitude)| {
                    let frequency = midi_note_to_frequency(note as f64);
                    let amplitude_mapped = map_amplitude(amplitude as u8);
                    let adjusted_duration = duration / cps;

                    (adjusted_duration, frequency, amplitude_mapped)
                })
            }).collect::<Vec<(Duration, f64, f64)>>();

            (spec.clone(), mote_vec)
        }).collect()
    }
}


pub fn test_song_with_x_files() {
    use song::x_files::TRACK;

    let cps = TRACK.conf.cps;
    let processed_parts = jukebox::process_parts(&TRACK.composition.parts, cps);

    for (spec, motes) in processed_parts {
        println!("Spec: {:?}", spec);
        for (duration, frequency, amplitude) in motes {
            println!("  Duration: {}, Frequency: {:.2} Hz, Amplitude: {:.2}", duration, frequency, amplitude);
        }
    }
}

#[test]
fn test_song() {
    test_song_with_x_files()
}
