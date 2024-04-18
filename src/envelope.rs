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
fn db_to_amp(db:f32) -> f32 {
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
    let total_samples = time::samples_of_dur(cps, d) - offset_start;

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
    let total_samples = time::samples_of_dur(cps, d) - offset_start;
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
    use crate::engrave::transform_to_monic_buffers;
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

    #[test]
    fn test_constant_env() {
        let track = happy_birthday::get_track();
        let cps = track.conf.cps;
        let mut buffs:Vec<Vec<synth::SampleBuffer>> = Vec::new();
            let melody:Vec<Note> = test_overs(7);
            let notebufs = transform_to_monic_buffers(cps, &melody);
            buffs.push(notebufs);

        let mixers:Vec<synth::SampleBuffer> = buffs.into_iter().map(|buff|
            buff.into_iter().flatten().collect()
        ).collect();

        files::with_dir("dev-audio");
        match render::pad_and_mix_buffers(mixers) {
            Ok(signal) => {
                render::samples_f32(44100, &signal, "dev-audio/test-constant-env.wav");
            },
            Err(err) => {
                println!("Problem while mixing buffers. Message: {}", err)
            }
        }
    }
}
