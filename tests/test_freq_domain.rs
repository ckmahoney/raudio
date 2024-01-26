mod common;

use raudio_synth::render::Ugen;
use std::collections::HashMap;

#[test]
fn test_write_freq_forms() {
    let config = &common::test_config();
    let mut shapes_map: HashMap<String, Ugen> = HashMap::new();
    shapes_map.insert(String::from("sawtooth"), raudio_synth::freq_forms::sawtooth);
    shapes_map.insert(String::from("triangle"), raudio_synth::freq_forms::triangle);
    shapes_map.insert(String::from("sine"), raudio_synth::freq_forms::sine);

    for (name, func) in &shapes_map {
        let label = common::test_audio_name(&config, &format!("time_form_{}", name));
        let filename = raudio_synth::render::render_ugen(&config, func, &label);
        println!("Completed writing test waveform {}", filename);

    }
}
