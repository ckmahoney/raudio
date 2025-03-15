use super::*;

/// mix of three different percs.
pub fn renderable<'render>(conf: &Conf, melody: &'render Melody<Note>, arf: &Arf) -> Renderable2<'render> {
  simple_stem(conf, melody, arf)
}

mod test_functional_compressor {

use std::f32::EPSILON;

use rustfft::num_traits::zero;

use super::*;
  use crate::analysis::{sampler::trim_tail_silence, tools::{count_energy, dev_audio_asset}};

  fn kick_arf() -> Arf {
    Arf {
      mode: Mode::Enharmonic,
      role: Role::Kick,
      register: 8,
      visibility: Visibility::Visible,
      energy: Energy::Medium,
      presence: Presence::Staccatto,
    }
  }

  fn get_stem_samples(role:crate::Role) -> Vec<f32> {
    let arf = kick_arf();
    let path = get_sample_path_by_index(&arf, 2, true).unwrap();
    let (stem_samples, sample_rate) = read_audio_file(&path).expect("Failed to read percussion sample");
    if stem_samples.len() == 1 {
      stem_samples[0].clone()
    } else if stem_samples.len() == 2{
      match crate::analysis::tools::downmix_stereo_to_mono(&stem_samples[0], &stem_samples[1]) {
        Ok(samples) => samples,
        Err(msg) => panic!("Error while getting stem samples: {}", msg)
      }
    } else {
      panic!("Unexpected number of channels for the sample: {}", stem_samples.len())
    }
  }


  /// attack for all below: 0.05
  /// release for all below: 0.1

  /// on snare 0
  /// ratio 8 
  /// no makeup gain
  ///  
  /// expander at thresh -10db is the maximal amount of punch
  /// expander at thresh -19db is an excellent snare
  /// 
  /// compressor at thresh -10db is not noticable
  /// compressor at thresh -19db is more clear, high ends feel more balance with lows

  
  /// on kick short
  /// ratio 8 
  /// no makeup gain
  ///  
  /// expander at thresh -10 to -23 becomes an impact
  /// expander midrange offers variations of the input sample tending towards impact
  /// expander at thresh -36db to -43 offers a punchier version of the input sample
  /// 
  /// compressor at thresh -10db is not very noticable
  /// compressor adds body and character up to around -25db  
  /// compressor past -25db cymbal begins to feel muted/ closed hat

  
  /// on kick
  /// ratio 8 
  /// no makeup gain
  ///  
  /// expander at thresh -43db is the minimum that tightens up the sound
  /// expander at thresh -19db is the maximal before result is too compressed 
  /// 
  /// compressor at thresh -10db is not very noticable
  /// compressor varies with similar intensity up through thresh -25db  
  /// compressor past -25db begins to feel highpassed

  #[test]
  fn test_g()  {
    let dbs = [-96f32, -84f32, -72f32, -60f32, -48f32, -36f32, -24f32, -12f32, 0f32];
    let dbs:Vec<f32> = (0..15).map(|x| (-6f32 * x as f32) as f32).collect();
    for db in dbs {
      println!("decibels: {} linear amplitude value: {} rms value: {}", db, db_to_amp(db), db_to_amp(db).powi(2i32));

    }

    
    for n_samples in [100, 200, 500, 1000] {
      let window_size_seconds = time::samples_to_seconds(n_samples);
      println!("Got duration: {}", window_size_seconds)
    }
  }
  
  #[test]
  fn test_iter_compressor_threshold() {
    let samples = get_stem_samples(Role::Kick);

    let thresholds:Vec<f32> = (0..12).map(|i| i as f32 * -3.0 - 10.0).collect::<Vec<f32>>();
    let attack_time = 0.05f32;
    let release_time = 0.1f32;
    let ratio = 8f32;
    render::engrave::samples(SR, &samples, &dev_audio_asset(&format!("original-{}.wav", Role::Kick)));

    for thresh in thresholds.iter() {
      let compressor_params = CompressorParams {
        threshold: *thresh,
        attack_time,
        release_time,
        ratio,
        ..Default::default()
      };

      match compressor(&samples, compressor_params, None) {
        Ok(result) => {
          let label = format!("kick-compressor-threshold-{}-attack-time-{}-release-time-{}-ratio-{}.wav", *thresh, attack_time, release_time, ratio);
          render::engrave::samples(SR, &result, &dev_audio_asset(&label))
        }
        Err(msg) => {
          panic!("Failure while running test: {}", msg)
        }
      }
    }
  }


  #[test]
  fn test_iter_expander_threshold() {
    let samples = get_stem_samples(Role::Kick);

    let thresholds:Vec<f32> = (0..12).map(|i| i as f32 * -3.0 - 10.0).collect::<Vec<f32>>();
    let attack_time = 0.05f32;
    let release_time = 0.1f32;
    let ratio = 8f32;
    render::engrave::samples(SR, &samples, &dev_audio_asset(&format!("original-{}.wav", Role::Kick)));
    
    for thresh in thresholds.iter() {
      let expander_params = ExpanderParams {
        threshold: *thresh,
        attack_time,
        release_time,
        ratio,
        ..Default::default()
      };

      match expander(&samples, expander_params, None) {
        Ok(result) => {

          let label = format!("kick-expander-threshold-{}-attack-time-{}-release-time-{}-ratio-{}.wav", *thresh, attack_time, release_time, ratio);
          println!("Samples at 48000..48200: {:?}", &samples[48000..48400]);
          render::engrave::samples(SR, &result, &dev_audio_asset(&label));


          // render::engrave::samples(SR, &result, &dev_audio_asset(&label));
          // let label = format!("kick-expander-threshold-{}-attack-time-{}-release-time-{}-ratio-{}-trimmed.wav", *thresh, attack_time, release_time, ratio);
          // let trimmed = trim_tail_silence(&samples, 0.002, 100);
        }
        Err(msg) => {
          panic!("Failure while running test: {}", msg)
        }
      }
    }
  }

  #[test]
  fn test_waveshaping() {
      let samples = get_stem_samples(Role::Kick);

      let attack_time = 0.05f32;
      let release_time = 0.1f32;
      let ratio = 8f32;

      let compressor_params = CompressorParams {
        threshold: -16f32,
        attack_time: attack_time*3f32,
        release_time,
        ratio,
        ..Default::default()
      };

      let expander_params = ExpanderParams {
        threshold: -53f32,
        attack_time,
        release_time: release_time / 2f32,
        ratio,
        ..Default::default()
      };
      render::engrave::samples(SR, &samples, &dev_audio_asset(&format!("original-{}.wav", Role::Kick)));

      let stage_1 = expander(&samples, expander_params, None).unwrap();
      let result = compressor(&stage_1, compressor_params, None).unwrap();
      let label = format!("kick-expander-then-compressor-threshold-{}.wav", 24);
      render::engrave::samples(SR, &result, &dev_audio_asset(&label));



      let stage_1 = compressor(&samples, compressor_params, None).unwrap();
      let result = expander(&stage_1, expander_params, None).unwrap();
      let label = format!("kick-compressor-then-expander-threshold-{}.wav", 24);
      render::engrave::samples(SR, &result, &dev_audio_asset(&label))
    }

    fn find_indexes<F>(vec: &[f32], callback: F) -> Vec<usize>
where
    F: Fn(f32) -> bool,
{
    vec.iter()
        .enumerate()
        .filter_map(|(i, &x)| if callback(x) { Some(i) } else { None })
        .collect()
}




  #[test]
  fn test_energy() {
      let samples = get_stem_samples(Role::Kick);

      let attack_time = 0.05f32;
      let release_time = 0.1f32;
      let ratio = 8f32;

      let compressor_params = CompressorParams {
        threshold: -16f32,
        attack_time: attack_time*3f32,
        release_time,
        ratio,
        ..Default::default()
      };

      let expander_params = ExpanderParams {
        threshold: -53f32,
        attack_time,
        release_time: release_time / 2f32,
        ratio,
        ..Default::default()
      };
      // if i use a rational window size compared to n_samples, then the index can be offset by that ratioanl expression for a silence detectedalgorithm
      
      // let n_bins = 256;
      // let window_size = samples.len() / n_bins;

      // let silence_detector = compute_rms(&samples, window_size);

      // let zero_indexes = find_indexes(&silence_detector, |x| x < std::f32::EPSILON);
      // let silence_window_seconds = 0.0125;
      // let index_interval_seconds = time::samples_to_seconds(window_size);
      // let n_contiguous_indices_required = (silence_window_seconds / index_interval_seconds).floor() as usize;
      // let kill_index = if n_contiguous_indices_required == 0 {
      //   // just get the first match
      //   zero_indexes[0]
      // } else {
      //   let mut accumulated_zeros = 0usize;
      //   let mut p:Option<usize> = None;
        
      //   // use the window 
      //   for (i, idx) in zero_indexes.iter().enumerate() {
      //     if accumulated_zeros == n_contiguous_indices_required {
      //       p = Some(*idx);
      //       break
      //     } else {
      //       if i == 0 || zero_indexes[i-1] == zero_indexes[i] - 1 {
      //         accumulated_zeros = accumulated_zeros + 1
      //       } else {
      //         accumulated_zeros = 0
      //       }
      //     };
      //   };

      //   p.unwrap()
      // }; 

      // let samples = samples[0..kill_index].to_vec();


      render::engrave::samples(SR, &samples, &dev_audio_asset(&format!("original-{}.wav", Role::Kick)));
      println!("Energy of original is {}", count_energy(&samples));
      let stage_1 = expander(&samples, expander_params, None).unwrap();
      println!("Energy of stage_1 is {}", count_energy(&stage_1));
      let result = compressor(&stage_1, compressor_params, None).unwrap();
      println!("Energy of stage_2 is {}", count_energy(&result));


    }
}