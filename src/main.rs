#![allow(unused_parens)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(non_snake_case)]
#![allow(unused_must_use)]
#![allow(non_upper_case_globals)]

use crate::render::{Renderable, Renderable2};
use crate::synth::{SRf, SR};
use crate::types::render::*;
use crate::types::synthesis;
use crate::types::synthesis::*;
use crate::types::timbre;
use crate::types::timbre::*;
use rand::{self, rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};
use reverb::convolution::ReverbParams;
use std::env;
use std::process;

mod analysis;
pub use analysis::monic_theory;
mod demo;
mod druid;
mod fastmast;
mod files;
mod fm;
mod inp;
mod music;
mod phrasing;
mod presets;
mod render;
mod reverb;
mod synth;
pub use analysis::time;
mod types;

use presets::Preset;

fn main() {
  let args: Vec<String> = env::args().collect();

  if args.len() < 3 {
    eprintln!(
      r#"Usage:
  raudio playbook.json preset-name asset-dir asset-name
  raudio apply-danceability input.wav output.wav"#
    );
    process::exit(1);
  }

  match args[1].as_str() {
    "apply-danceability" => {
      if args.len() != 4 {
        eprintln!(r#"Usage: raudio apply-danceability input.wav output.wav"#);
        process::exit(1);
      }
      let input_path = &args[2];
      let output_path = &args[3];
      fastmast::apply_danceability(input_path, output_path);
    }
    _ => {
      // Default behavior: render_playbook
      if args.len() < 5 {
        eprintln!(r#"Usage: raudio playbook.json preset-name asset-dir asset-name"#);
        process::exit(1);
      }
      let file_path = &args[1];
      let preset_pack = &args[2];
      let out_dir = &args[3];
      let mixdown_name = &args[4];
      render_playbook(out_dir, preset_pack, file_path, mixdown_name);
    }
  }
}

// fn resample_audio(
//     audio: Vec<Vec<f32>>,
//     input_sample_rate: usize,
//     target_sample_rate: usize,
// ) -> Vec<Vec<f32>> {
//     // Calculate the resampling ratio
//     let resample_ratio = target_sample_rate as f64 / input_sample_rate as f64;

//     // Define SincInterpolationParameters
//     let sinc_params = SincInterpolationParameters {
//         sinc_len: 128,                  // Filter length
//         f_cutoff: 0.95,                 // Cutoff frequency
//         interpolation: SincInterpolationType::Cubic, // Interpolation type
//         oversampling_factor: 32,        // Oversampling factor
//         window: WindowFunction::BlackmanHarris2, // Window function
//     };

//     // Define chunk size
//     let chunk_size = 1024;
//     let max_resample_ratio_relative = 3.0;

//     // Create a SincFixedIn resampler
//     let mut resampler = SincFixedIn::<f32>::new(
//         resample_ratio,
//         max_resample_ratio_relative,
//         sinc_params,
//         chunk_size,
//         audio.len(), // Number of channels
//     )
//     .unwrap();

//     // Resample each channel independently
//     audio
//         .into_iter()
//         .map(|channel| {
//             resampler
//                 .process(&[channel], None)
//                 .unwrap() // The result is a Vec<Vec<f32>>; extract the first channel
//                 .remove(0)
//         })
//         .collect()
// }

fn parse_preset(s: &str) -> Option<Preset> {
  let src = s.to_lowercase();
  match src.as_str() {
    "hop" => Some(Preset::Hop),
    "valley" => Some(Preset::Valley),
    "mountain" => Some(Preset::Mountain),
    "bland" => Some(Preset::Bland),
    "bright" => Some(Preset::Bright),
    "fum" => Some(Preset::Fum),
    _ => None,
  }
}

#[cfg(test)]
mod test_parse {
  use super::*;

  #[test]
  fn test_parse_preset_valid_lowercase() {
    let res = parse_preset("bright");
    println!("Got it {:#?}", res)
  }
}

fn render_playbook(out_dir: &str, preset_pack: &str, playbook_path: &str, asset_name: &str) {
  use std::fs;
  use std::path::Path;
  let keep_stems = true;

  let preset = parse_preset(preset_pack);
  if preset.is_none() {
    panic!(
      "Must provide a valid preset pack. here are the options: hop, valley, mountain, bland, bright. You gave me '{}'.",
      preset_pack
    )
  }

  match inp::arg_parse::load_score_from_file(&playbook_path) {
    Ok(score) => {
      let out_path = render_score(score, preset.unwrap(), out_dir, asset_name, keep_stems);
      println!("{}", out_path)
    }
    Err(msg) => {
      panic!("Failed to open score: {}", msg)
    }
  }
}

pub fn complexity(v: &Visibility, e: &Energy, p: &Presence) -> f32 {
  let cv: f32 = match *v {
    Visibility::Hidden => 0f32,
    Visibility::Background => 0.333f32,
    Visibility::Foreground => 0.666f32,
    Visibility::Visible => 1f32,
  };

  let ce: f32 = match *e {
    Energy::Low => 0f32,
    Energy::Medium => 0.5f32,
    Energy::High => 1f32,
  };

  let cp: f32 = match *p {
    Presence::Staccatto => 0f32,
    Presence::Legato => 0.5f32,
    Presence::Tenuto => 1f32,
  };

  (cv + ce + cp) / 3f32
}

fn dimensions_to_cycles(dims: &Dimensions) -> f32 {
  ((dims.base as i32).pow(dims.size as u32) * dims.cpc as i32) as f32
}

fn score_duration_seconds(score: &DruidicScore) -> f32 {
  let len_cycles: f32 = dimensions_to_cycles(&score.dimensions);
  len_cycles / score.conf.cps
}

// /// Given a melody, Labelled Arfs, and a preset to splay,
// /// Render each labelled arf using the preset into destination_dir.
// pub fn render_arf(
//     destination_dir: &str,
//     root:f32,
//     cps: f32,
//     melody:&Melody<Note>,
//     arf: &Arf,
//     preset: Preset
//     ) {
//     let group_reverbs:Vec<ReverbParams> = vec![];
//     let keep_stems = Some(destination_dir);
//     let stems:Vec<Renderable2> = vec![
//         Preset::create_stem(cps, melody, arf, preset)
//     ];
//     render::combiner_with_reso(&Conf {cps, root}, &stems, &group_reverbs, keep_stems);
// }

pub fn render_score(score: DruidicScore, preset: Preset, out_dir: &str, asset_name: &str, keep_stems: bool) -> String {
  let mixdown_name = format!("{}/{}.wav", out_dir, asset_name);
  files::with_dir(&mixdown_name);
  let mut pre_mix_buffs: Vec<synth::SampleBuffer> = Vec::new();
  let mut rng: ThreadRng = rand::thread_rng();
  let mut stems: Vec<(Arf, Renderable2)> = Vec::with_capacity(score.parts.len());
  let mut stem_reverbs: Vec<ReverbParams> = Vec::with_capacity(score.parts.len());

  for (client_positioning, arf, melody) in &score.parts {
    let delays = inp::arg_xform::gen_delays(
      &mut rng,
      score.conf.cps,
      &client_positioning.echo,
      complexity(&arf.visibility, &arf.energy, &arf.presence),
    );
    let len_cycles = time::count_cycles(&melody[0]);
    let len_seconds = len_cycles / score.conf.cps;
    let convolution_layer = inp::arg_xform::gen_convolution_stem(
      &mut rng,
      arf,
      len_seconds/4f32,
      score.conf.cps,
      &client_positioning.distance,
      &client_positioning.enclosure,
    );
    let stem = Preset::create_stem(&score.conf, melody, arf, preset);
    stem_reverbs.push(convolution_layer);
    stems.push((arf.clone(), stem))
  }

  let len_seconds: f32 = score_duration_seconds(&score);

  // a single small convolution layer varying only by groupEnclosure
  let group_reverb: Vec<ReverbParams> = vec![inp::arg_xform::reverb_params(
    &mut rng,
    len_seconds/8f32,
    score.conf.cps,
    &Distance::Near,
    &score.groupEnclosure,
    0f32,
  )];
  let keeps = if keep_stems { Some(out_dir) } else { None };
  let keeps = None;
  let signal = render::combiner_with_reso2(&score.conf, &stems, &stem_reverbs, &group_reverb, keeps);
  render::engrave::samples(crate::synth::SR, &signal, &mixdown_name);
  mixdown_name
}

#[test]
fn test_render_playbook() {
  let filepath: &str = &format!("{}/demo/test_render_playbook", crate::demo::out_dir);
  render_playbook(
    filepath,
    "fum",
    "src/demo/playbooks/house_short_1.json",
    "house_short_1_fum",
  )
}
