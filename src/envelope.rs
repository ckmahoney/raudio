use crate::types::synthesis::{Duration, Note, Monae, Tone};
use crate::types::render::{Melody};
use crate::synth::SampleBuffer;
use crate::time;

/// Given the current tempo and the number of cycles to span,
/// Create a -60dB to 0dB amplitude curve lasting k cycles.
pub fn exp_env_k_cycles_db_60_0(cps:f32, k:f32) -> Vec<f32> {
    let n_samples = time::samples_of_cycles(cps, k);
    let minDb = -60f32;
    let maxDb = 0f32;
    db_env_n(n_samples, minDb, maxDb)
}

/// Better for linear modulation of amplitude
pub fn db_to_amp(db:f32) -> f32 {
    10f32.powf(db/20f32)
}


/// Given the current tempo and the number of samples to span,
/// Create a ear-friendly (dB scaled) amplitude curve lasting n_samples.
pub fn db_env_n(n_samples:usize, a:f32, b:f32) -> Vec<f32> {
    let dDb = (b - a)/n_samples as f32;
    
    (0..n_samples).map(|i|
        
        db_to_amp(a + i as f32 * dDb)
    ).collect()
}

pub fn mix_envelope(env:&SampleBuffer, buf:&mut SampleBuffer, offset:usize) {
    let mut o = offset;
    let l1 = env.len();
    let l2 = buf.len();
    if l1 > l2 {
        panic!("Unable to mix envelopes with greater length than the target signal")
    }

    if o + l1 > l2 {
        if o + l1  > l2 + 10 {
            panic!("Offset out of bounds. Got env.len {} and buf.len {} with offset {}",l1, l2, o)
        } else {
            o = o + l1 - l2;
            if o > 2 {
                println!("using a very liberal allowance of {} sample offset", o)
            }
        }
    }

    for i in 0..l1 {
        buf[i + o] *= env[i]
    }
}

/// the syllabic portion of the envelope is the main body. It occupies 66% to 98% of the notes duration.
pub fn gen_env(cps:f32, note:&Note, offset_start:usize) -> SampleBuffer {
    let (d,_,_) = note;
    let total_samples = time::samples_of_duration(cps, d) - offset_start;

    //@art-choice use a dynamic allocation of body/tail
    //@art-choice let them overlap and use a window function to blend them
    let n_body = (1. * total_samples as f32) as usize;
    
    let mut ys = Vec::<f32>::new();

    let keyframes = (00f32, -20f32);
    let body:Vec::<f32> = db_env_n(n_body, keyframes.0, keyframes.1);

    ys.extend(body);
   
    ys
}

/// the syllabic portion of the envelope is the main body. It occupies 66% to 98% of the notes duration.
pub fn rnd_env(cps:f32, note:&Note, offset_start:usize) -> SampleBuffer {
    let (d,_,_) = note;
    let total_samples = time::samples_of_duration(cps, d) - offset_start;
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    //@art-choice use a dynamic allocation of body/tail
    //@art-choice let them overlap and use a window function to blend them
    let n_body = (1. * total_samples as f32) as usize;
    
    let mut ys = Vec::<f32>::new();
    let a = 0f32;
    let r:f32 = rng.gen();
    let b:f32 = -10f32 - (r * -60f32);

    let keyframes = (00f32, -20f32);
    let body:Vec::<f32> = db_env_n(n_body, a, b);

    ys.extend(body);
   
    ys
}



mod test_unit {
    use super::*;
    use crate::synth;
    use crate::render;
    use crate::files;
    use crate::song::happy_birthday;

    /// iterate early monics over sequential rotations in alternating spaces
    fn test_line(register:i8) -> Vec<Note> {
        let monics:Vec<i8> = vec![1, 3, 5, 7];
        let rotations:Vec<i8> = vec![-3,-2,-1,0,1,2,3];
        let qs:Vec<i8> = vec![0, 1];

        const dur:Duration = (1,1);
        const amp:f32 = 1.0;
        let mut mel:Vec<Note> = Vec::new();
        for r in &rotations {
            for m in &monics {
                for q in &qs {
                    let monae:Monae = (*r,*q, *m);
                    let tone:Tone = (register, monae);
                    mel.push((dur, tone, amp));
                }
            }
        }

        mel
    }

    fn test_overs(register:i8) -> Vec<Note> {
        let monics:Vec<i8> = vec![1, 3, 5, 7];
        let rotations:Vec<i8> = vec![-2,-1,0,1,2];
        let qs:Vec<i8> = vec![0];

        const dur:Duration = (1,1);
        const amp:f32 = 1.0;
        let mut mel:Vec<Note> = Vec::new();
        for r in &rotations {
            for m in &monics {
                for q in &qs {
                    let monae:Monae = (*r,*q, *m);
                    let tone:Tone = (register, monae);
                    mel.push((dur, tone, amp));
                }
            }
        }

        mel
    }

}


#[cfg(test)]
mod unit_test {
    use crate::synth;
    use crate::files;
    use crate::render;
    use crate::types::synthesis::{Duration, Note, Monae, Tone, Direction, FilterPoint};
    use crate::types::timbre::{Energy, Presence, Sound, FilterMode, BaseOsc, Timeframe, Phrasing};
    use crate::engrave::color_line;
    use crate::preset;
    use crate::time;
    
    fn test_unders(register:i8) -> Vec<Note> {
        let monics:Vec<i8> = vec![1, 3, 5, 7];
        let rotations:Vec<i8> = vec![-2,-1,0,1,2];
        let qs:Vec<i8> = vec![1];

        const dur:Duration = (1,1);
        const amp:f32 = 1.0;
        let mut mel:Vec<Note> = Vec::new();
        for r in &rotations {
            for m in &monics {
                for q in &qs {
                    let monae:Monae = (*r,*q, *m);
                    let tone:Tone = (register, monae);
                    mel.push((dur, tone, amp));
                }
            }
        }

        mel
    }

    #[test]
    fn test_one() {
        let cps = 1.8f32;
        let register= 7i8;
        let direction = Direction::Constant;
        let energy = Energy::Medium;
        let energy = Energy::Low;
        let presence = Presence::Staccatto;
        // let presence = Presence::Legato;
        // let presence = Presence::Tenuto;
        let name = "pluck";

        let line:Vec<Note> = test_unders(register);
        let form_length = line.iter().fold(0f32, |acc, &note| acc + time::duration_to_cycles(note.0));

        let mut phr = Phrasing { 
            form: Timeframe {
                cycles: form_length,
                p: 0f32,
                instance: 0
            },
            arc: Timeframe {
                cycles: form_length,
                p: 0f32,
                instance: 0
            },
            line: Timeframe {
                cycles: form_length,
                p: 0f32,
                instance: 0
            },
            note: Timeframe {
                cycles: -1.0,
                p: 0f32,
                instance: 0
            }
        };

        let sound = Sound {
            bandpass: (FilterMode::Linear, FilterPoint::Constant, (1f32, 24000f32)),
            energy: energy.clone(),
            presence : presence.clone(),
            pan: 0f32,
        };

        let mut buffs:Vec<Vec<synth::SampleBuffer>> = Vec::new();
        let dev_dir = "dev-audio/preset";

        let mbs = preset::SomeModulators {
            amp: None,
            freq: None,
            phase: None,
        };
        let notebufs = color_line(cps, &line, &BaseOsc::Sine, &sound, &mut phr, &mbs);
        buffs.push(notebufs);

        let mixers:Vec<synth::SampleBuffer> = buffs.into_iter().map(|buff|
            buff.into_iter().flatten().collect()
        ).collect();    

        files::with_dir(&dev_dir);
        let filename = format!("{}/test-oox-preset-{}-register-{}-direction-{:?}-energy-{:?}-presence-{:?}", dev_dir, name, register, direction, energy, presence);
        match render::pad_and_mix_buffers(mixers) {
            Ok(signal) => {
                render::samples_f32(44100, &signal, &filename);
            },
            Err(err) => {
                println!("Problem rendering file {}. Message: {}", filename, err)
            }
        }
        
    }

    #[test]
    fn test_enumerate_params() {
        
        let cps = 1.8f32;
        let registers:Vec<i8> = vec![5,7,9,11,13];
        let directions:Vec<Direction> = vec![Direction::Constant];
        let energies:Vec<Energy> = vec![Energy::High];
        let presences:Vec<Presence> = vec![Presence::Staccatto, Presence::Legato, Presence::Tenuto ];
        
        for register in &registers {
            for direction in &directions {
                for energy in &energies {
                    for presence in &presences {
                        // this test iterates over a range of rotations about 0
                        // use each rotation as a new arc
                        
                        let line:Vec<Note> = test_unders(*register);
                        let form_length = line.iter().fold(0f32, |acc, &note| acc + time::duration_to_cycles(note.0));

                        let mut phr = Phrasing { 
                            form: Timeframe {
                                cycles: form_length,
                                p: 0f32,
                                instance: 0
                            },
                            arc: Timeframe {
                                cycles: form_length,
                                p: 0f32,
                                instance: 0
                            },
                            line: Timeframe {
                                cycles: form_length,
                                p: 0f32,
                                instance: 0
                            },
                            note: Timeframe {
                                cycles: -1.0,
                                p: 0f32,
                                instance: 0
                            }
                        };

                        let sound = Sound {
                            bandpass: (FilterMode::Linear, FilterPoint::Constant, (1f32, 24000f32)),
                            energy: energy.clone(),
                            presence : presence.clone(),
                            pan: 0f32,
                        };

                        let mut buffs:Vec<Vec<synth::SampleBuffer>> = Vec::new();
                        let dev_dir = "dev-audio/color-controls";

                        let mbs = preset::SomeModulators {
                            amp: None,
                            freq: None,
                            phase: None,
                        };
                        let notebufs = color_line(cps, &line, &BaseOsc::Sine, &sound, &mut phr, &mbs);
                        buffs.push(notebufs);

                        let mixers:Vec<synth::SampleBuffer> = buffs.into_iter().map(|buff|
                            buff.into_iter().flatten().collect()
                        ).collect();    

                        files::with_dir(&dev_dir);
                        let filename = format!("{}/test-register-{}-direction-{:?}-energy-{:?}-presence-{:?}", dev_dir, *register, *direction, *energy, *presence);
                        match render::pad_and_mix_buffers(mixers) {
                            Ok(signal) => {
                                render::samples_f32(44100, &signal, &filename);
                            },
                            Err(err) => {
                                println!("Problem rendering file {}. Message: {}", filename, err)
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_enumerate_osc_shell() {
        
        let cps = 1.8f32;
        let registers:Vec<i8> = vec![5,7,9,11,13];
        let oscs:Vec<BaseOsc> = vec![BaseOsc::Square, BaseOsc::Sawtooth, BaseOsc::Triangle, BaseOsc::Sine];
        let directions:Vec<Direction> = vec![Direction::Constant];
        let energies:Vec<Energy> = vec![Energy::High, Energy::Medium];
        let presences:Vec<Presence> = vec![Presence::Legato, Presence::Staccatto, Presence::Tenuto];
        
        for osc in &oscs {
            for register in &registers {
                for direction in &directions {
                    for energy in &energies {
                        for presence in &presences {
                            // this test iterates over a range of rotations about 0
                            // use each rotation as a new arc
                            
                            let line:Vec<Note> = test_unders(*register);
                            let form_length = line.iter().fold(0f32, |acc, &note| acc + time::duration_to_cycles(note.0));

                            let mut phr = Phrasing { 
                                form: Timeframe {
                                    cycles: form_length,
                                    p: 0f32,
                                    instance: 0
                                },
                                arc: Timeframe {
                                    cycles: form_length,
                                    p: 0f32,
                                    instance: 0
                                },
                                line: Timeframe {
                                    cycles: form_length,
                                    p: 0f32,
                                    instance: 0
                                },
                                note: Timeframe {
                                    cycles: -1.0,
                                    p: 0f32,
                                    instance: 0
                                }
                            };

                            let sound = Sound {
                                bandpass: (FilterMode::Linear, FilterPoint::Constant, (1f32, 24000f32)),
                                energy: energy.clone(),
                                presence : presence.clone(),
                                pan: 0f32,
                            };

                            let mut buffs:Vec<Vec<synth::SampleBuffer>> = Vec::new();
                            let dev_dir = "dev-audio/osc-shell";

                            let mbs = preset::SomeModulators {
                                amp: None,
                                freq: None,
                                phase: None,
                            };
                            let notebufs = color_line(cps, &line, &osc, &sound, &mut phr, &mbs);
                            buffs.push(notebufs);

                            let mixers:Vec<synth::SampleBuffer> = buffs.into_iter().map(|buff|
                                buff.into_iter().flatten().collect()
                            ).collect();    

                            files::with_dir(&dev_dir);
                            let filename = format!("{}/test-osc-{:?}-register-{}-direction-{:?}-energy-{:?}-presence-{:?}", dev_dir, *osc, *register, *direction, *energy, *presence);
                            match render::pad_and_mix_buffers(mixers) {
                                Ok(signal) => {
                                    render::samples_f32(44100, &signal, &filename);
                                },
                                Err(err) => {
                                    println!("Problem rendering file {}. Message: {}", filename, err)
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_bandpass_filter() {
        
        let cps = 1.5f32;
        let registers:Vec<i8> = vec![6];
        let directions:Vec<Direction> = vec![Direction::Constant];
        let energies:Vec<Energy> = vec![Energy::High, Energy::Low];
        let presences:Vec<Presence> = vec![Presence::Tenuto ];
        
        for register in &registers {
            for direction in &directions {
                for energy in &energies {
                    for presence in &presences {
                        // this test iterates over a range of rotations about 0
                        // use each rotation as a new arc
                        
                        let line:Vec<Note> = test_unders(*register);
                        let form_length = line.iter().fold(0f32, |acc, &note| acc + time::duration_to_cycles(note.0));

                        let mut phr = Phrasing { 
                            form: Timeframe {
                                cycles: form_length,
                                p: 0f32,
                                instance: 0
                            },
                            arc: Timeframe {
                                cycles: form_length,
                                p: 0f32,
                                instance: 0
                            },
                            line: Timeframe {
                                cycles: form_length,
                                p: 0f32,
                                instance: 0
                            },
                            note: Timeframe {
                                cycles: -1.0,
                                p: 0f32,
                                instance: 0
                            }
                        };

                        let sound = Sound {
                            bandpass: (FilterMode::Linear, FilterPoint::Constant, (1000f32, 4000f32)),
                            energy: energy.clone(),
                            presence : presence.clone(),
                            pan: 0f32,
                        };

                        let mut buffs:Vec<Vec<synth::SampleBuffer>> = Vec::new();
                        let dev_dir = "dev-audio/bandpass";

                        let mbs = preset::SomeModulators {
                            amp: None,
                            freq: None,
                            phase: None,
                        };
                        let notebufs = color_line(cps, &line, &BaseOsc::Sine, &sound, &mut phr, &mbs);
                        buffs.push(notebufs);

                        let mixers:Vec<synth::SampleBuffer> = buffs.into_iter().map(|buff|
                            buff.into_iter().flatten().collect()
                        ).collect();    

                        files::with_dir(&dev_dir);
                        let filename = format!("{}/test-bandpass-1000..4000-register-{}-direction-{:?}-energy-{:?}-presence-{:?}", dev_dir, *register, *direction, *energy, *presence);
                        match render::pad_and_mix_buffers(mixers) {
                            Ok(signal) => {
                                render::samples_f32(44100, &signal, &filename);
                            },
                            Err(err) => {
                                println!("Problem rendering file {}. Message: {}", filename, err)
                            }
                        }
                    }
                }
            }
        }
    }
}