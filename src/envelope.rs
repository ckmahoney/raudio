use crate::types::synthesis::Note;
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
    // let n_body = (0.9 * total_samples as f32) as usize;
    let n_body = (1. * total_samples as f32) as usize;
    // let n_tail = total_samples - n_body;
    
    let mut ys = Vec::<f32>::new();

    let keyframes = (00f32, -20f32, -60f32);
    let body:Vec::<f32> = db_env_n(n_body, keyframes.0, keyframes.1);
    // let tail:Vec::<f32> = db_env_n(n_tail, keyframes.1, keyframes.2);

    ys.extend(body);
    // ys.extend(tail.clone());
    // if ys.len() != total_samples {
    //     println!("total_samples {} ys.len {}",  total_samples , ys.len());
    //     let x = total_samples - ys.len();
    //     println!("Expected to produce {} samples and actually got {}. Filling in the gap with {} tail value", total_samples, ys.len(), x);
    //     let fill = vec![tail[tail.len()-1]; x];
    //     ys.extend(fill);
    // }
    ys

}