use crate::color;
use crate::render;
use crate::song;
use crate::synth;

mod jukebox {
    pub use crate::song::*;
    pub use crate::midi::*;
    use crate::synth;

    pub fn process_parts(parts: &HashMap<Spec, Vec<Vec<Midi>>>, cps: f32) -> Vec<(Spec, Vec<(Duration, f32, f32)>)> {
        parts.iter().map(|(spec, midi_vecs)| {
            let mote_vec = midi_vecs.iter().flat_map(|midi_vec| {
                midi_vec.iter().map(|&(duration, note, amplitude)| {
                    let frequency = midi_note_to_frequency(note as f32);
                    let amplitude_mapped = map_amplitude(amplitude as u8);
                    let adjusted_duration = duration / cps;

                    (adjusted_duration, frequency, amplitude_mapped)
                })
            }).collect::<Vec<(Duration, f32, f32)>>();

            (spec.clone(), mote_vec)
        }).collect()
    }

    pub fn transform_to_sample_buffers(cps:f32, motes: &Vec<(Duration, f32, f32)>) -> Vec<synth::SampleBuffer> {
        motes.iter().map(|&(duration, frequency, amplitude)| {
            synth::samp_ugen(44100, cps, amplitude, synth::silly_sine, duration, frequency)
        }).collect()
    }

    pub fn transform_to_sample_pairs(cps:f32, motes: &Vec<(Duration, f32, f32)>) -> Vec<(f32, synth::SampleBuffer)> {
        motes.iter().map(|&(duration, frequency, amplitude)| {
            (frequency, synth::samp_ugen(44100, cps, amplitude, synth::silly_sine, duration, frequency))
        }).collect()
    }
}

pub fn test_song_x_files() {
    use song::x_files::TRACK;

    let cps = TRACK.conf.cps;
    let processed_parts = jukebox::process_parts(&TRACK.composition.parts, cps);
    let mut buffs:Vec<Vec<synth::SampleBuffer>> = Vec::new();

    for (spec, motes) in processed_parts {
        buffs.push(jukebox::transform_to_sample_buffers(cps, &motes))
    }

    let mixers:Vec<synth::SampleBuffer> = buffs.into_iter().map(|buff|
        buff.into_iter().flatten().collect()
    ).collect();

    match render::pad_and_mix_buffers(mixers) {
        Ok(signal) => {
            render::samples_f32(44100, &signal, "dev-audio/x_files.wav");
        },
        Err(err) => {
            println!("Problem while mixing buffers. Message: {}", err)
        }
    }
}


pub fn test_song_x_files_in_color() {
    use song::x_files::TRACK;

    let cps = TRACK.conf.cps;
    let processed_parts = jukebox::process_parts(&TRACK.composition.parts, cps);
    let mut pair_buffs:Vec<Vec<(f32, synth::SampleBuffer)>> = Vec::new();

    for (spec, motes) in processed_parts {
        pair_buffs.push(jukebox::transform_to_sample_pairs(cps, &motes))
    }

    let file_names:Vec<&str> = vec!["fundamentals/convolve/m-12/saw/saw_76_hz_44100.wav"];
    
    
    let mut mixers:Vec<synth::SampleBuffer> = Vec::new();
    
    for melody in pair_buffs.iter() {
        let mut signal:Vec<f32> = Vec::new();

        for (freq, notebuff) in melody {
            match color::with_samples(*freq, &notebuff, &file_names) {
                Ok(result) => {
                    signal.extend(result);
                },
                Err(msg) => {
                    println!("Error while running test: {}", msg)
                }
            }
        };
        mixers.push(signal);
    }

    match render::pad_and_mix_buffers(mixers) {
        Ok(signal) => {
            render::samples_f32(44100, &signal, "dev-audio/x_files_color_1.wav");
        },
        Err(err) => {
            println!("Problem while mixing buffers. Message: {}", err)
        }
    }

}

#[test]
pub fn test_song_x_files_in_color2() {
    use song::x_files::TRACK;

    let cps = TRACK.conf.cps;
    let processed_parts = jukebox::process_parts(&TRACK.composition.parts, cps);
    let mut pair_buffs:Vec<Vec<(f32, synth::SampleBuffer)>> = Vec::new();

    for (spec, motes) in processed_parts {
        pair_buffs.push(jukebox::transform_to_sample_pairs(cps, &motes))
    }

    let file_names:Vec<&str> = vec!["fundamentals/convolve/m-3/saw_1_hz_44100.wav", "fundamentals/convolve/m-3/saw_1_hz_44100.wav"];
    
    
    let mut mixers:Vec<synth::SampleBuffer> = Vec::new();
    
    for melody in pair_buffs.iter() {
        let mut signal:Vec<f32> = Vec::new();

        for (freq, notebuff) in melody {
            match color::with_samples(*freq, &notebuff, &file_names) {
                Ok(result) => {
                    signal.extend(result);
                },
                Err(msg) => {
                    println!("Error while running test: {}", msg)
                }
            }
        };
        mixers.push(signal);
    }

    match render::pad_and_mix_buffers(mixers) {
        Ok(signal) => {
            render::samples_f32(44100, &signal, "dev-audio/x_files_color_2.wav");
        },
        Err(err) => {
            println!("Problem while mixing buffers. Message: {}", err)
        }
    }
}


#[test]
pub fn test_song_x_files_rand_each_colors() {
    use song::x_files::TRACK;
    use rand::Rng; 

    let cps = TRACK.conf.cps;
    let processed_parts = jukebox::process_parts(&TRACK.composition.parts, cps);
    let mut pair_buffs:Vec<Vec<(f32, synth::SampleBuffer)>> = Vec::new();

    for (spec, motes) in processed_parts {
        pair_buffs.push(jukebox::transform_to_sample_pairs(cps, &motes))
    }

    let file_names:Vec<&str> = vec![
        "fundamentals/convolve/m-3/saw/saw_1_hz_44100.wav", 
        "fundamentals/convolve/m-3/saw/saw_2_hz_44100.wav",
        "fundamentals/convolve/m-3/saw/saw_3_hz_44100.wav",
        "fundamentals/convolve/m-3/saw/saw_4_hz_44100.wav",
        "fundamentals/convolve/m-3/saw/saw_5_hz_44100.wav",
        "fundamentals/convolve/m-3/saw/saw_6_hz_44100.wav",
    ];
    
    
    let mut mixers:Vec<synth::SampleBuffer> = Vec::new();
    
    for melody in pair_buffs.iter() {
        let mut signal:Vec<f32> = Vec::new();

        for (freq, notebuff) in melody {
            let random_index = rand::thread_rng().gen_range(0..file_names.len());
            let selected_color = vec![file_names[random_index]];

            match color::with_samples(*freq, &notebuff, &selected_color) {
                Ok(result) => {
                    signal.extend(result);
                },
                Err(msg) => {
                    println!("Error while running test: {}", msg)
                }
            }
        };
        mixers.push(signal);
    }

    match render::pad_and_mix_buffers(mixers) {
        Ok(signal) => {
            render::samples_f32(44100, &signal, "dev-audio/x_files_colors_saw_rand.wav");
        },
        Err(err) => {
            println!("Problem while mixing buffers. Message: {}", err)
        }
    }
}



#[test]
pub fn test_song_x_files_match_color_tri() {
    use song::x_files::TRACK;
    use rand::Rng; 

    let cps = TRACK.conf.cps;
    let processed_parts = jukebox::process_parts(&TRACK.composition.parts, cps);
    let mut pair_buffs:Vec<Vec<(f32, synth::SampleBuffer)>> = Vec::new();

    for (spec, motes) in processed_parts {
        pair_buffs.push(jukebox::transform_to_sample_pairs(cps, &motes))
    }
     

    let file_opts = 10..3500;
    
    let mut mixers:Vec<synth::SampleBuffer> = Vec::new();
    
    for melody in pair_buffs.iter() {
        let mut signal:Vec<f32> = Vec::new();

        for (freq, notebuff) in melody {
            let rounded_hz = freq.floor() as usize;
            let selected_color = format!("fundamentals/convolve/m-3/tri/tri_{}_hz_44100.wav", rounded_hz);

            match color::with_samples(*freq, &notebuff, &vec![&selected_color]) {
                Ok(result) => {
                    signal.extend(result);
                },
                Err(msg) => {
                    println!("Error while running test: {}", msg)
                }
            }
        };
        mixers.push(signal);
    }

    match render::pad_and_mix_buffers(mixers) {
        Ok(signal) => {
            render::samples_f32(44100, &signal, "dev-audio/x_files_colors_tri_match.wav");
        },
        Err(err) => {
            println!("Problem while mixing buffers. Message: {}", err)
        }
    }
}



#[test]
pub fn test_song_x_files_blend_color_tri() {
    use song::x_files::TRACK;
    use rand::Rng; 

    let cps = TRACK.conf.cps;
    let processed_parts = jukebox::process_parts(&TRACK.composition.parts, cps);
    let mut pair_buffs:Vec<Vec<(f32, synth::SampleBuffer)>> = Vec::new();

    for (spec, motes) in processed_parts {
        pair_buffs.push(jukebox::transform_to_sample_pairs(cps, &motes))
    }
     

    let file_opts = 10..3500;
    
    let mut mixers:Vec<synth::SampleBuffer> = Vec::new();
    
    for melody in pair_buffs.iter() {
        let mut signal:Vec<f32> = Vec::new();

        for (freq, notebuff) in melody {
            let mut random_index = rand::thread_rng().gen_range(file_opts.clone());
            let rounded_hz = freq.floor() as usize;
            let random_color1 = format!("fundamentals/convolve/m-3/tri/tri_{}_hz_44100.wav", random_index);

            random_index = rand::thread_rng().gen_range(file_opts.clone());
            let random_color2 = format!("fundamentals/convolve/m-3/tri/tri_{}_hz_44100.wav", random_index);
            let matching_color = format!("fundamentals/convolve/m-3/tri/tri_{}_hz_44100.wav", rounded_hz);

            match color::with_samples(*freq, &notebuff, &vec![&random_color1, &random_color2]) {
                Ok(result) => {
                    signal.extend(result);
                },
                Err(msg) => {
                    println!("Error while running test: {}", msg)
                }
            }
        };
        mixers.push(signal);
    }

    match render::pad_and_mix_buffers(mixers) {
        Ok(signal) => {
            render::samples_f32(44100, &signal, "dev-audio/x_files_colors_tri_blend.wav");
        },
        Err(err) => {
            println!("Problem while mixing buffers. Message: {}", err)
        }
    }
}


#[test]
pub fn test_song_x_files_rand_each_colors_tri() {
    use song::x_files::TRACK;
    use rand::Rng; 

    let cps = TRACK.conf.cps;
    let processed_parts = jukebox::process_parts(&TRACK.composition.parts, cps);
    let mut pair_buffs:Vec<Vec<(f32, synth::SampleBuffer)>> = Vec::new();

    for (spec, motes) in processed_parts {
        pair_buffs.push(jukebox::transform_to_sample_pairs(cps, &motes))
    }
     

    let file_opts = 10..3500;
    
    let mut mixers:Vec<synth::SampleBuffer> = Vec::new();
    
    for melody in pair_buffs.iter() {
        let mut signal:Vec<f32> = Vec::new();

        for (freq, notebuff) in melody {
            let random_hz = rand::thread_rng().gen_range(file_opts.to_owned());
            let selected_color = format!("fundamentals/convolve/m-3/tri/tri_{}_hz_44100.wav", random_hz);

            match color::with_samples(*freq, &notebuff, &vec![&selected_color]) {
                Ok(result) => {
                    signal.extend(result);
                },
                Err(msg) => {
                    println!("Error while running test: {}", msg)
                }
            }
        };
        mixers.push(signal);
    }

    match render::pad_and_mix_buffers(mixers) {
        Ok(signal) => {
            render::samples_f32(44100, &signal, "dev-audio/x_files_colors_tri_rand.wav");
        },
        Err(err) => {
            println!("Problem while mixing buffers. Message: {}", err)
        }
    }
}


#[test]
pub fn test_song_x_files_rand_each_colors_postfx_filter() {
    use song::x_files::TRACK;
    use rand::Rng; 

    let cps = TRACK.conf.cps;
    let processed_parts = jukebox::process_parts(&TRACK.composition.parts, cps);
    let mut pair_buffs:Vec<Vec<(f32, synth::SampleBuffer)>> = Vec::new();

    for (spec, motes) in processed_parts {
        pair_buffs.push(jukebox::transform_to_sample_pairs(cps, &motes))
    }

    let file_names:Vec<&str> = vec![
        "fundamentals/convolve/m-3/saw/saw_1_hz_44100.wav", 
        "fundamentals/convolve/m-3/saw/saw_2_hz_44100.wav",
        "fundamentals/convolve/m-3/saw/saw_3_hz_44100.wav",
        "fundamentals/convolve/m-3/saw/saw_4_hz_44100.wav",
        "fundamentals/convolve/m-3/saw/saw_5_hz_44100.wav",
        "fundamentals/convolve/m-3/saw/saw_6_hz_44100.wav",
    ];
    
    
    let mut mixers:Vec<synth::SampleBuffer> = Vec::new();
    
    for melody in pair_buffs.iter() {
        let mut signal:Vec<f32> = Vec::new();

        for (freq, notebuff) in melody {
            let random_index = rand::thread_rng().gen_range(0..file_names.len());
            let selected_color = vec![file_names[random_index]];

            match color::with_samples(*freq, &notebuff, &selected_color) {
                Ok(result) => {
                    signal.extend(result);
                },
                Err(msg) => {
                    println!("Error while running test: {}", msg)
                }
            }
        };
        mixers.push(signal);
    }

    match render::pad_and_mix_buffers(mixers) {
        Ok(signal) => {
            let fff = 120.0;
            match color::with_samples(fff, &signal, &vec!["fundamentals/convolve/m-3/saw/saw_1_hz_44100.wav"]) {
                Ok(signal) => {
                    render::samples_f32(44100, &signal, &format!("dev-audio/x_files_colors_saw_rand_with_postfx_{}.wav",fff));
                },
                Err(msg) => {
                    println!("Problem while running postrender fx {}", msg)

                }
            }
        },
        Err(err) => {
            println!("Problem while mixing buffers. Message: {}", err)
        }
    }

}


#[test]
fn test_song() {
    test_song_x_files();
    test_song_x_files_in_color()
}
