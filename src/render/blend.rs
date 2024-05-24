use crate::synth::{SR, MFf, MF, NFf, NF, pi2, pi, SampleBuffer};
use crate::types::synthesis::{Bp,Range, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::types::timbre::{BandpassFilter, Energy, Presence, BaseOsc, Sound, FilterMode, Timeframe, Phrasing, Ampex};
use crate::types::render::{Span};
use crate::phrasing::contour::{Expr, Position, sample};
use crate::phrasing::ranger::{Druid, Mixer, Cocktail, mix};

#[derive(Clone)]
pub enum GlideLen {
    None,
    Quarter,
    Eigth,
    Sixteenth
}

/// Context window for a frequency. 
/// Second, Third, and Fourth entries describe the frequencies being navigated.
/// Middle entry is the current frequency to perform.
/// The first and final f32 are the previous/next frequency.
/// First and final entries describe how to glide
///
/// If a C Major chord is spelled as C, E, G and we wanted to arpeggiate the notes,
/// then an analogous Frex looks like (GlideLen::None, None, C, E, GlideLen::None)
/// and then for the second note, (GlideLen::None, C, E, G, GlideLen::None)
pub type Frex = (GlideLen, Freq, Freq, Freq, GlideLen);


/// Returns an amplitude identity, attenuation, or cancellation 
/// for the given frequency and bandpass settings
fn filter(p:f32, freq:f32, bandpass:&Bp) -> Range {
    let min_f = sample(&bandpass.0, p).max(MF as f32);
    let max_f = sample(&bandpass.1, p).min(NF as f32);
    if freq < min_f || freq > max_f {
        return 0f32
    } else {
      return 1f32  
    }
}


fn mix_or(default:f32, maybe_cocktail:&Option<Cocktail>, k:f32, x:f32, d:f32) -> f32 {
    if maybe_cocktail.is_some() {
        let cocktail = maybe_cocktail.clone().unwrap();
        mix(k, x, d, &cocktail)
    } else {
        default
    }
}


/// Generates an expressive signal for a note. 
/// 
/// ### Arguments
/// * `funds` Immediate frequency context. Provides insight to previous and upcoming note, often for glissando.
/// * `expr` Contour buffers for amplitude, frequency, and phase. Sampled based on this note's progres and applied to the summed result (time-series) signal.
/// * `span` Tuple of (cps, n_cycles, n_samples) 
/// * `bp` Bandpass filter buffers. First entry is a list of highpass values; second entry is a list of lowpass values.
/// * `multipliers` Frequencies for multiplying the curr frequency; to create a triangle wave, for example. Values must be in range of (0, NF/2]
/// * `druid` Optional callbacks to apply for modulating amp, freq,and phase on each multiplier (by index + 1 as k).
/// * `noise_thresh` Minimum allowed amplitude. Use -1 for an allpass. 
/// ### Returns
/// A samplebuffer representing audio data of the specified event.
pub fn genb(
    funds: Frex,
    expr: Expr,
    span: &Span,
    bp: &Bp,
    multipliers: &Vec<Freq>,
    druid:&Druid,
    noise_thresh: f32
) -> SampleBuffer {
    let (_, prev, freq, next, _) = funds;
    let (acont, fcont, pcont) = expr;
    let n_samples = crate::time::samples_of_cycles(span.0, span.1);
    let mut sig = vec![0f32; n_samples];

    
    for j in 0..n_samples {
        let p:Range = j as f32 / n_samples as f32;
        let t:f32 = j as f32 / SR as f32;

        // collect instantaneous modulation factors from the expression envelopes
        let am = sample(&acont, p);
        let fm = sample(&fcont, p);
        let pm = sample(&pcont, p);

        let mut v:f32 = 0f32;

        for (i, m) in multipliers.iter().enumerate() {
            let k = (i + 1) as f32;
            let frequency = m * fm * freq * mix_or(1f32, &druid[1], k, p, span.1);
            let amp = am * filter(p, frequency, bp) * mix_or(1f32, &druid[0], k, p, span.1);
            if amp != 0f32 {
                let phase = frequency * pi2 * t + pm + mix_or(0f32, &druid[2], k, p, span.1); 
                v += amp * phase.sin();
            }
        }

        if v.abs() > noise_thresh {
            sig[j] += v
        }
    }

    sig
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{files, phrasing};
    use crate::render::engrave;

    static TEST_DIR:&str = "dev-audio/blend";
    static druid:Druid = [None,None,None];

    #[test]
    fn test_multipliers_overtones() {
        let test_name = "genb-overs";
        let funds:Frex = (
            GlideLen::None, 400f32, 500f32, 600f32, GlideLen::None
        );
        let expr:Expr = (vec![1f32], vec![1f32], vec![0f32]);
        let span:Span = (1.5, 2.0);
        let bp:Bp = (vec![MFf], vec![NFf]);
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let noise_thresh = 0f32;

        let signal = genb(
            funds,
            expr,
            &span,
            &bp,
            &multipliers,
            &druid,
            noise_thresh
        );
        files::with_dir(TEST_DIR);
        let filename = format!("{}/{}.wav", TEST_DIR, test_name);
        engrave::samples(SR, &signal, &filename);
    }

    #[test]
    fn test_multipliers_undertones() {
        let test_name = "genb-unders";
        let funds:Frex = (
            GlideLen::None, 400f32, 500f32, 600f32, GlideLen::None
        );
        let expr:Expr = (vec![1f32], vec![1f32], vec![0f32]);
        let span:Span = (1.5, 2.0);
        let bp:Bp = (vec![MFf], vec![NFf]);
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| 1f32/x as f32).collect();
        let noise_thresh = 0f32;

        let signal = genb(
            funds,
            expr,
            &span,
            &bp,
            &multipliers,
            &druid,
            noise_thresh
        );
        files::with_dir(TEST_DIR);
        let filename = format!("{}/{}.wav", TEST_DIR, test_name);
        engrave::samples(SR, &signal, &filename);
    }


    #[test]
    fn test_bp_filters() {
        let test_name = "genb-overs-highpass-filter";
        let funds:Frex = (
            GlideLen::None, 400f32, 500f32, 600f32, GlideLen::None
        );
        let expr:Expr = (vec![1f32], vec![1f32], vec![0f32]);
        let span:Span = (1.5, 2.0);
        let bp:Bp = (vec![MFf], vec![NFf]);
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let noise_thresh = 0f32;

        let n_samples = crate::time::samples_of_cycles(span.0, span.1);
        let highpass_filter:Vec<f32> = (0..n_samples/4).map(|x|  x as f32).collect();
        let lowpass_filter = vec![NFf];
        let signal = genb(
            funds.clone(),
            expr.clone(),
            &span,
            &(highpass_filter, lowpass_filter),
            &multipliers,
            &druid,
            noise_thresh
        );

        files::with_dir(TEST_DIR);
        let filename = format!("{}/{}.wav", TEST_DIR, test_name);
        engrave::samples(SR, &signal, &filename);

        let test_name = "genb-overs-lowpass-filter";
        let bp:Bp = (vec![MFf], vec![NFf]);

        let highpass_filter = vec![MFf];
        let lowpass_filter = (0..n_samples/4).map(|x| (15000 - x) as f32).collect();

        let signal = genb(
            funds,
            expr,
            &span,
            &(highpass_filter, lowpass_filter),
            &multipliers,
            &druid,
            noise_thresh
        );
        files::with_dir(TEST_DIR);
        let filename = format!("{}/{}.wav", TEST_DIR, test_name);
        engrave::samples(SR, &signal, &filename);
    }

    fn small_f_modulator(cps:f32, n_samples:usize)-> SampleBuffer {
        let frequency = 1f32;
        (0..n_samples).map(|j| {
            let phase = frequency * pi2 * (j as f32 / n_samples as f32) / cps;
            1f32 + (phase.sin() / 10f32)
        }).collect()
    }

    #[test]
    fn test_fmod() {
        let test_name = "genb-overs-fmod";
        let funds:Frex = (
            GlideLen::None, 400f32, 500f32, 600f32, GlideLen::None
        );
        let span:Span = (1.5, 2.0);
        let bp:Bp = (vec![MFf], vec![NFf]);
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let noise_thresh = 0f32;
        let n_samples = crate::time::samples_of_cycles(span.0, span.1);
        let expr:Expr = (vec![1f32], small_f_modulator(span.0, n_samples), vec![0f32]);

        let signal = genb(
            funds.clone(),
            expr.clone(),
            &span,
            &bp,
            &multipliers,
            &druid,
            noise_thresh
        );

        files::with_dir(TEST_DIR);
        let filename = format!("{}/{}.wav", TEST_DIR, test_name);
        engrave::samples(SR, &signal, &filename);

    }

    fn small_p_modulator(cps:f32, n_samples:usize)-> SampleBuffer {
        use rand;
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let frequency = 1f32;
        (0..n_samples).map(|j| {
            let phase = (j as f32).sqrt().sqrt()     * frequency * pi2 * (j as f32 / n_samples as f32) / cps;
            let x:f32 =  rng.gen();
             pi2 * (x - 0.5f32) * 0.01f32 
        }).collect()
    }

    #[test]
    fn test_pmod() {
        let test_name = "genb-overs-pmod";
        let funds:Frex = (
            GlideLen::None, 400f32, 500f32, 600f32, GlideLen::None
        );
        let span:Span = (1.5, 2.0);
        let bp:Bp = (vec![MFf], vec![NFf]);
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let noise_thresh = 0f32;
        let n_samples = crate::time::samples_of_cycles(span.0, span.1);
        let expr:Expr = (vec![1f32], vec![1f32], small_p_modulator(span.0, n_samples));

        let signal = genb(
            funds.clone(),
            expr.clone(),
            &span,
            &bp,
            &multipliers,
            &druid,
            noise_thresh
        );

        files::with_dir(TEST_DIR);
        let filename = format!("{}/{}.wav", TEST_DIR, test_name);
        engrave::samples(SR, &signal, &filename);
    }



    #[test]
    fn test_noise_thresh() {
        let test_name = "genb-thresh";
        let funds:Frex = (
            GlideLen::None, 400f32, 500f32, 600f32, GlideLen::None
        );
        let span:Span = (1.5, 2.0);
        let bp:Bp = (vec![MFf], vec![NFf]);
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let n_samples = crate::time::samples_of_cycles(span.0, span.1);
        let expr:Expr = (vec![1f32], vec![1f32], vec![0f32]);
        let noise_thresh = 0.7f32;
        let signal = genb(
            funds.clone(),
            expr.clone(),
            &span,
            &bp,
            &multipliers,
            &druid,
            noise_thresh
        );

        files::with_dir(TEST_DIR);
        let filename = format!("{}/{}.wav", TEST_DIR, test_name);
        engrave::samples(SR, &signal, &filename);
    }

    #[test]
    fn test_druid_amp() {
        let test_name = "genb-druid-amp";
        let funds:Frex = (
            GlideLen::None, 400f32, 500f32, 600f32, GlideLen::None
        );
        let span:Span = (1.5, 2.0);
        let bp:Bp = (vec![MFf], vec![NFf]);
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let n_samples = crate::time::samples_of_cycles(span.0, span.1);
        let expr:Expr = (vec![1f32], vec![1f32], vec![0f32]);
        let noise_thresh = 0f32;
        let the_druid:Druid = [
            Some(phrasing::gen_cocktail(2)),
            None,
            None,
        ];
        let signal = genb(
            funds.clone(),
            expr.clone(),
            &span,
            &bp,
            &multipliers,
            &the_druid,
            noise_thresh
        );

        files::with_dir(TEST_DIR);
        let filename = format!("{}/{}.wav", TEST_DIR, test_name);
        engrave::samples(SR, &signal, &filename);
    }

    #[test]
    fn test_druid_freq() {
        let test_name = "genb-druid-freq";
        let funds:Frex = (
            GlideLen::None, 400f32, 500f32, 600f32, GlideLen::None
        );
        let span:Span = (1.5, 2.0);
        let bp:Bp = (vec![MFf], vec![NFf]);
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let n_samples = crate::time::samples_of_cycles(span.0, span.1);
        let expr:Expr = (vec![1f32], vec![1f32], vec![0f32]);
        let noise_thresh = 0f32;
        let the_druid:Druid = [
            None,
            Some(phrasing::gen_cocktail(2)),
            None,
        ];
        let signal = genb(
            funds.clone(),
            expr.clone(),
            &span,
            &bp,
            &multipliers,
            &the_druid,
            noise_thresh
        );

        files::with_dir(TEST_DIR);
        let filename = format!("{}/{}.wav", TEST_DIR, test_name);
        engrave::samples(SR, &signal, &filename);
    }


    #[test]
    fn test_druid_phase() {
        let test_name = "genb-druid-phase";
        let funds:Frex = (
            GlideLen::None, 400f32, 500f32, 600f32, GlideLen::None
        );
        let span:Span = (1.5, 2.0);
        let bp:Bp = (vec![MFf], vec![NFf]);
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let n_samples = crate::time::samples_of_cycles(span.0, span.1);
        let expr:Expr = (vec![1f32], vec![1f32], vec![0f32]);
        let noise_thresh = 0f32;
        let the_druid:Druid = [
            None,
            None,
            Some(phrasing::gen_cocktail(2)),
        ];
        let signal = genb(
            funds.clone(),
            expr.clone(),
            &span,
            &bp,
            &multipliers,
            &the_druid,
            noise_thresh
        );

        files::with_dir(TEST_DIR);
        let filename = format!("{}/{}.wav", TEST_DIR, test_name);
        engrave::samples(SR, &signal, &filename);
    }
}