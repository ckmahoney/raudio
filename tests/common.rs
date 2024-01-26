const TEST_AUDIO_DIR: &str = "test-render";
use raudio_synth::synth_config::SynthConfig;

pub fn test_audio_name(config:&SynthConfig, label:&str) -> String {
    let name: String = format!("{}_sample-rate_{}_channels_{}", label, config.sample_rate, 1);
    format!("{}/{}.wav", TEST_AUDIO_DIR, name)
}


// Define a basic SynthConfig for testing
pub fn test_config() -> SynthConfig {
    SynthConfig {
        sample_rate: 44100,
        min_frequency: 20.0,
        max_frequency: 20000.0,
        amplitude_scaling: 1.0,
        phase_offset: 0.0,
        tuning_offset_hz: 0.0,
        cps: 1.0,
    }
}