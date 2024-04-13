pub type Duration = f32;
pub type MidiVal = i32;
pub type SignedByte = i8;

pub type Midi = (Duration, MidiVal, SignedByte);



pub fn note_to_frequency(note: f32) -> f32 {
    440.0 * 2.0_f32.powf((note - 69.0) / 12.0)
}

pub fn map_amplitude(amplitude: u8) -> f32 {
    amplitude as f32 / 127.0
}
