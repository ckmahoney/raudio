pub type Duration = f32;
pub type MidiVal = i32;
pub type SignedByte = i8;

pub type Midi = (Duration, MidiVal, SignedByte);



pub fn midi_note_to_frequency(note: f64) -> f64 {
    440.0 * 2.0_f64.powf((note - 69.0) / 12.0)
}

pub fn map_amplitude(amplitude: u8) -> f64 {
    amplitude as f64 / 127.0
}
