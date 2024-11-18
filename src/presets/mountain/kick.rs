use std::os::unix::thread;

use super::*;

/// Returns a `Stem3` for the kick preset.
///
/// # Parameters
/// - `conf`: Configuration object for additional context.
/// - `melody`: Melody structure specifying note events for the stem.
/// - `arf`: Configuration for amplitude and visibility adjustments.
///
/// # Returns
/// A `Stem3` with configured sample buffers, amplitude expressions, and effect parameters.
pub fn stemmy<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  // Dynamically retrieve a kick sample file path
  let sample_path = get_sample_path(arf);
    
    // Read the audio sample from the retrieved path
    let (ref_samples, sample_rate) = read_audio_file(&sample_path).expect("Failed to read kick sample");

        let gain = visibility_gain_sample(arf.visibility);
        let amp_expr = dynamics::gen_organic_amplitude(10, 2000, arf.visibility)
        .iter()
        .map(|v| v * gain)
        .collect();

        let mut rng = thread_rng();
        let delays_note = vec![];
        let delays_room = vec![];
        let reverbs_note = vec![
            ReverbParams {
                mix: in_range(&mut rng, 0.05, 0.12), 
                amp: in_range(&mut rng, db_to_amp(-45.0), db_to_amp(-32.0)),
                dur: in_range(&mut rng, 0.005, 0.1), 
                rate: in_range(&mut rng, 0.4, 0.8),
            }
        ];
        let reverbs_room = vec![
            ReverbParams {
                mix: in_range(&mut rng, 0.01, 0.05), 
                amp: in_range(&mut rng, db_to_amp(-50.0), db_to_amp(-45.0)),
                dur: in_range(&mut rng, 0.05, 0.1), 
                rate: in_range(&mut rng, 0.05, 0.1),
            }
        ];

    // Configure lowpass cutoff frequency based on energy level
    let lowpass_cutoff = match arf.energy {
        Energy::Low => NFf / 6f32,
        Energy::Medium => NFf / 4f32,
        Energy::High => NFf,
    };
    let lowpass_cutoff = NFf;
    let ref_sample = ref_samples[0].to_owned();

    Renderable2::Sample((
        melody,
        ref_sample,
        amp_expr,
        lowpass_cutoff,
        delays_note,
        delays_room,
        reverbs_note,
        reverbs_room,
    ))
}

pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
    Renderable2::Mix(vec![
        (0.4, stemmy(conf, melody, arf)),
        (0.2, crate::presets::hill::kick::renderable(conf, melody, arf)),
        (0.3, stemmy(conf, melody, arf)),
    ])
}