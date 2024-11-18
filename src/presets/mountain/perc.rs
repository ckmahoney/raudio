use super::*;

/// Returns a `Stem3` for the percussion preset.
///
/// # Parameters
/// - `conf`: Configuration object for additional context.
/// - `melody`: Melody structure specifying note events for the stem.
/// - `arf`: Configuration for amplitude and visibility adjustments.
///
/// # Returns
/// A `Stem3` with configured sample buffers, amplitude expressions, and effect parameters.
pub fn stemmy<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
    // Dynamically retrieve a percussion sample file path
    let sample_path = get_sample_path(arf);

    // Read the audio sample from the retrieved path
    let (ref_samples, sample_rate) = read_audio_file(&sample_path).expect("Failed to read percussion sample");
    let gain = visibility_gain_sample(arf.visibility);
    let amp_expr = dynamics::gen_organic_amplitude(10, 2000, arf.visibility)
    .iter()
    .map(|v| v * gain)
    .collect();

    // Initialize effect parameters
    let mut delays_note = vec![];
    let mut reverbs_room = vec![];

    // Add delays and reverbs only when Visibility::Foreground
    if let Visibility::Foreground = arf.visibility {
        // Generate delay macros for the percussion stem
        let delay_macros = generate_delay_macros(arf.visibility, arf.energy, arf.presence);
        let mut rng = rand::thread_rng();
        delays_note = delay_macros
            .iter()
            .map(|mac| mac.gen(&mut rng, conf.cps))
            .collect();

        // Manually define reverb parameters for the percussion stem
        reverbs_room = vec![ReverbParams {
            mix: in_range(&mut rng, 0.005, 0.01),
            amp: in_range(&mut rng, db_to_amp(-45.0), db_to_amp(-30.0)), // Impulse amplitude
            dur: in_range(&mut rng, 0.1, 0.5),
            rate: in_range(&mut rng, 0.01, 0.05),
        }];
    }

    // Set lowpass cutoff frequency based on energy level
    let lowpass_cutoff = NFf;
    let ref_sample = ref_samples[0].to_owned();

    // Return the renderable sample
    Renderable2::Sample(
        (
            melody,
            ref_sample,
            amp_expr,
            lowpass_cutoff,
            delays_note,
            vec![], // No room-level delays for percussion
            vec![], // No note-level reverbs for percussion
            reverbs_room, // Room-level reverb for percussion
        )
    )
}

/// mix of three different percs. 
pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
    Renderable2::Mix(vec![
        (2f32/3f32, stemmy(conf, melody, arf)),
        (1f32/3f32, stemmy(conf, melody, arf)),
    ])
}

/// Generates a set of delay macros for percussion in house music.
///
/// # Parameters
/// - `visibility`: Controls gain level for delay feedback.
/// - `energy`: Influences delay density and feedback time.
/// - `presence`: Adjusts delay timing and spatialization.
///
/// # Returns
/// A vector of `DelayParamsMacro` instances.
fn generate_delay_macros(visibility: Visibility, energy: Energy, presence: Presence) -> Vec<DelayParamsMacro> {
    let delay_gain = match visibility {
        Visibility::Hidden => db_to_amp(-18.0),
        Visibility::Background => db_to_amp(-12.0),
        Visibility::Foreground => db_to_amp(-9.0),
        Visibility::Visible => db_to_amp(-6.0),
    };

    let delay_time = match energy {
        Energy::Low => vec![0.25, 0.5],
        Energy::Medium => vec![0.5, 0.75],
        Energy::High => vec![0.75, 1.0],
    };

    let pan_spread = match presence {
        Presence::Staccatto => vec![StereoField::LeftRight(0.8, 0.2)],
        Presence::Legato => vec![StereoField::Mono],
        Presence::Tenuto => vec![StereoField::LeftRight(0.5, 0.5)],
    };

    vec![
        DelayParamsMacro {
            gain: [delay_gain, delay_gain + 0.1],
            dtimes_cycles: delay_time,
            n_echoes: [3, 5],
            mix: [0.4, 0.6],
            pan: pan_spread,
            mecho: vec![MacroMotion::Forward],
            mgain: vec![MacroMotion::Constant],
            mpan: vec![MacroMotion::Constant],
            mmix: vec![MacroMotion::Constant],
        },
    ]
}
