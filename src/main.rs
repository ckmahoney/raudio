#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(non_snake_case)]
#![allow(unused_must_use)]
#![allow(non_upper_case_globals)]
use std::env;
use std::process;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::types::synthesis;
use crate::types::synthesis::*;
use crate::types::timbre;
use crate::types::timbre::*;
use crate::types::render::*;

mod arg_parse;
pub mod analysis;
pub mod synth_config;
pub mod convolve;
pub mod decor;
pub mod engrave;
pub mod envelope;
pub mod files;
pub mod midi;
pub mod mix;
pub mod modulate;
pub mod monic_theory;
pub mod phrasing;
pub mod preset;
pub mod presets;
pub mod render;
pub mod sequence;
pub mod song;
pub mod synth;
pub mod time;
pub mod types;

const audio_dir:&str = "renditions/early";


fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!(r#"Usage: raudio "/abs/to/playbook.json" "/abs/to/audio.wav""#);
        process::exit(1);
    }

    let file_path = &args[1];
    let out_path = &args[2];
    render_playbook(out_path, file_path);
}

fn render_playbook(out_path: &str, playbook_path: &str) {
    use std::path::Path;
    use std::fs;

    match arg_parse::load_score_from_file(&playbook_path) {
        Ok(score) => {
            match render_score(out_path, score) {
                Ok(_) => {
                    println!("{}", out_path)
                },
                Err(msg) => {
                    eprint!("Problem while writing {}", msg)
                }
            }
        },
        Err(msg) => {
            panic!("Failed to open score: {}", msg)
        }
    }
}


fn contrib_to_osc(contrib:&Contrib) -> timbre::BaseOsc {
    use timbre::BaseOsc::*;
    let mut rng = thread_rng(); 
    let opts:Vec<timbre::BaseOsc> = match contrib.role {
        Role::Kick => {
            vec![Bell]
        },
        Role::Perc => {
            vec![Noise]
        },
        Role::Hats => {
            vec![Bell, Noise]
        },
        Role::Bass => {
            vec![Sawtooth, Square, Sine]
        },
        Role::Chords => {
            vec![Poly, Square, Sine]
        },
        Role::Lead => {
            vec![Triangle, Square, Sine]
        },
    };
    opts.choose(&mut rng).unwrap().clone()
}

/// Given a part to render as Contrib,
/// create a spectral domain bandpass filter
/// error: should also factor the root of composition as well
fn contrib_to_bandpass(contrib:&Contrib) -> BandpassFilter {
    static min_supported_frequency:f32 = 22.0;
    static max_supported_frequency:f32 = 23998.0;

    let min_max = match contrib.visibility {
        Visibility::Hidden => {
            // strict bandfiltering 
            (2f32.powi(contrib.register as i32), 2f32.powi((contrib.register + 2u32) as i32))
        },
        Visibility::Background => {
            // loose bandfiltering 
            (2f32.powi(contrib.register as i32 - 1i32), 2f32.powi((contrib.register + 3u32) as i32))
        },
        Visibility::Foreground => {
            // some bandfiltering 
            (2f32.powi(contrib.register as i32 - 1i32), 2f32.powi((contrib.register + 4u32) as i32))
        },
        Visibility::Visible => {
            // functional bandfiltering 
            (min_supported_frequency, max_supported_frequency)
        }
    };
    (FilterMode::Linear, FilterPoint::Constant, min_max)
}

fn contrib_to_amod(contrib:&Contrib) -> preset::AmpMod {
    let amod: preset::AmpMod = match &contrib.ampex {
        (AmpLifespan::Pluck, AmpContour::Fade) => {
            preset::pluck::amod_tanh
        },
        (AmpLifespan::Drone, AmpContour::Fade) => {
            preset::drone::amod
        },  
        (a, AmpContour::Fade) => {
            panic!("Uimplemented amex for {:#?}", a)
        },
        (a, b) => {
            panic!("Completely unimplemented amex for {:#?} {:#?}", a, b)
        }
    };
    amod
}



pub fn render_score(filename:&str, score:Score) -> Result<(), core::fmt::Error> {
    files::with_dir(filename);
    let lens:Vec::<f32> = (&score.parts).iter()
    .map(|(_, melody)| 
        melody[0].iter()
        .fold(0f32, |acc, note| acc + time::dur(score.conf.cps, &note.0)) 
    ).collect();

    let mut phr = Phrasing { 
        cps: score.conf.cps,
        form: Timeframe {
            cycles: lens[0],
            p: 0f32,
            instance: 0
        },
        arc: Timeframe {
            cycles: lens[0],
            p: 0f32,
            instance: 0
        },
        line: Timeframe {
            cycles: lens[0],
            p: 0f32,
            instance: 0
        },
        note: Timeframe {
            cycles: -1.0,
            p: 0f32,
            instance: 0
        }
    };

    let mut pre_mix_buffs:Vec<synth::SampleBuffer> = Vec::new();
    for (contrib, melody) in &score.parts {
        let osc:BaseOsc = contrib_to_osc(&contrib);
        let sound = Sound {
            bandpass: contrib_to_bandpass(&contrib),
            energy: contrib.energy,
            presence : contrib.presence,
            pan: 0f32,
        };
        //@art-choice Create modulators in a diferent way
        let mbs:preset::SomeModulators = preset::SomeModulators {
            amp: Some(match contrib.ampex.0 {
                AmpLifespan::Drone => {
                    preset::drone::amod
                },
                AmpLifespan::Pluck => {
                    preset::pluck::amod_tanh
                },
                _ => {
                    preset::none::amod
                }   
            }),
            freq: None,
            phase: None,
        };
        for line in melody {
            let mut line_buff:synth::SampleBuffer = engrave::color_line(score.conf.cps, &line, &osc, &sound, &mut phr, &mbs)
                .into_iter()
                .flatten()
                .collect();
            pre_mix_buffs.push(line_buff)
        }
    }

    match render::pad_and_mix_buffers(pre_mix_buffs) {
        Ok(signal) => {
            render::samples_f32(44100, &signal, &filename);
            Ok(())
        },
        Err(msg) => {
            panic!("Failed to mix and render audio: {}", msg)
        }
    }
}




#[cfg(test)]
mod test_integration {
    static test_dir:&str = "dev-audio/integration";

    use crate::{synth, presets, phrasing, files, render};
    use crate::synth::SampleBuffer;
    use crate::presets::bell;
    use crate::phrasing::contour;
    use crate::types::timbre::{AmpContour, BandpassFilter, Energy, Presence, BaseOsc, Sound, FilterMode, Timeframe, Phrasing, Ampex};
    use crate::types::synthesis::{Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};

    static contours:[AmpContour; 2] = [
        AmpContour::Fade, 
        AmpContour::Surge
    ];
    static energies:[Energy; 3] = [
        Energy::Low, 
        Energy::Medium, 
        Energy::High
    ];
    static presences:[Presence; 3] = [
        Presence::Staccatto, 
        Presence::Legato, 
        Presence::Tenuto
    ];

    #[test]
    // Generate 16 samples of the same bell signal, contoured by the inspected contour.
    // 
    fn test_contour_bells() {
        let n_cycles = 4f32;
        let n_bells = 16;
        let cps = 1f32;
        let mut buff_dry:SampleBuffer = Vec::new();

        let mut phr = Phrasing {
            cps, 
            form: Timeframe {
                cycles: n_cycles,
                p: 0f32,
                instance: 0
            },
            arc: Timeframe {
                cycles: n_cycles,
                p: 0f32,
                instance: 0
            },
            line: Timeframe {
                cycles: n_cycles,
                p: 0f32,
                instance: 0
            },
            note: Timeframe {
                cycles:n_cycles,
                p: 0f32,
                instance: 0
            }
        };

        let sound = Sound {
            bandpass: (FilterMode::Linear, FilterPoint::Tail, (1f32, 24000f32)),
            energy: Energy::High,
            presence : Presence::Legato,
            pan: 0f32,
        };

        let note:Note = ( (n_cycles as i32, 1), (7, (0i8, 0i8, 1i8)), 1f32);
        let mgen = bell::Mgen::new(BaseOsc::Bell, sound);
        let coeffs = bell::gen_coefficients(1.0);
        let samples = mgen.inflect_bell(&coeffs, &note, &mut phr);
        for i in 0..n_bells {
            buff_dry.extend(&samples)
        }

        files::with_dir(test_dir);
        render::samples_f32(44100, &buff_dry, &format!("{}/test_contour_dry_bell_sample.wav", test_dir));
        for c in &contours {
            let test_name = format!("bell-contour-{:#?}-forward",c);
            let mut buff_wet = buff_dry.clone();
            let mod_amp = contour::gen_contour(buff_wet.len(), n_cycles, &c, false);
            contour::apply_contour(&mut buff_wet, &mod_amp);
            render::samples_f32(44100, &buff_wet, &format!("{}/test_contour_wet_{}_sample.wav", test_dir, test_name));

            let test_name = format!("bell-contour-{:#?}-reverse",c);
            let mut buff_wet = buff_dry.clone();
            let mod_amp = contour::gen_contour(buff_wet.len(), n_cycles, &c, true);
            contour::apply_contour(&mut buff_wet, &mod_amp);
            render::samples_f32(44100, &buff_wet, &format!("{}/test_contour_wet_{}_sample.wav", test_dir, test_name));
        }
    }

    #[test]
    fn test_enumerate_bells() {
        let n_cycles = 4f32;
        let n_bells = 4;
        let cps = 1f32;
        files::with_dir(test_dir);

        for filter_point in &phrasing::filter_points {
            for filter_mode in &phrasing::filter_modes {
                let coeffs = bell::gen_coefficients(1.0);

                let test_name = format!("bell-contour-{:#?}-point-{:#?}-mode",filter_point, filter_mode);
                let mut buff:SampleBuffer = Vec::new();

                let mut phr = Phrasing {
                    cps, 
                    form: Timeframe {
                        cycles: n_cycles,
                        p: 0f32,
                        instance: 0
                    },
                    arc: Timeframe {
                        cycles: n_cycles,
                        p: 0f32,
                        instance: 0
                    },
                    line: Timeframe {
                        cycles: n_cycles,
                        p: 0f32,
                        instance: 0
                    },
                    note: Timeframe {
                        cycles:n_cycles,
                        p: 0f32,
                        instance: 0
                    }
                };

                let sound = Sound {
                    bandpass: (*filter_mode, *filter_point, (1f32, 24000f32)),
                    energy: Energy::High,
                    presence : Presence::Legato,
                    pan: 0f32,
                };

                let note:Note = ( (n_cycles as i32, 1), (7, (0i8, 0i8, 1i8)), 1f32);
                let mgen = bell::Mgen::new(BaseOsc::Bell, sound);
                
                for i in 0..n_bells {
                    phr.line.p = i as f32 / n_bells as f32;
                    buff.append(&mut mgen.inflect_bell(&coeffs, &note, &mut phr))
                }


                render::samples_f32(44100, &buff, &format!("{}/test-bell-{}.wav", test_dir, test_name));
            }
        }
    }
}