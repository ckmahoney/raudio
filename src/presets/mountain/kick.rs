use super::*;

/// Returns a `Stem3` for the kick preset.
///
/// # Parameters
/// - `arf`: Configuration for amplitude and visibility adjustments.
/// - `melody`: Melody structure specifying note events for the stem.
///
/// # Returns
/// A `Stem3` with configured sample buffers, amplitude expressions, and effect parameters.
pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {

  // Directory and sample file configuration for the kick preset
  let sample_path = "audio-samples/kick/kick-0.wav"; // Replace with logic for selecting sample variants
  let (ref_samples, sample_rate) = read_audio_file(sample_path).expect("Failed to read kick sample");

  let amp_expr = vec![visibility_gain(arf.visibility)]; // Adjust amplitude dynamically

  // Placeholder effect parameters
  let delays_note = vec![];
  let delays_room = vec![];
  let reverbs_note = vec![];
  let reverbs_room = vec![];

  let lowpass_cutoff = match arf.energy {
    Energy::Low => NFf/6f32,
    Energy::Medium => NFf/4f32,
    Energy::High => NFf,
  };

  Renderable2::Sample(
    (
      melody,
      ref_samples[0].to_owned(),
      amp_expr,
      lowpass_cutoff,
      delays_note,
      delays_room,
      reverbs_note,
      reverbs_room,
    )
  )
}