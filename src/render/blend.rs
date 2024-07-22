use crate::synth::{SR, MFf, MF, NFf, NF, pi2, pi, SampleBuffer};
use crate::types::synthesis::{Clippers, GlideLen, Frex, Bp,Range, Direction, Duration, FilterPoint, Radian, Freq, Monae, Mote, Note, Tone};
use crate::types::timbre::{BandpassFilter, Energy, Presence, BaseOsc, Sound, FilterMode, Timeframe, Phrasing};
use crate::types::render::{Span};
use crate::phrasing::contour::{Expr, Position, sample};
use crate::phrasing::ranger::{Ranger, Modders, Mixer, WRangers, mix, example_options};



/// Returns an amplitude identity or cancellation value
/// for the given frequency and bandpass settings
/// 
/// idea: enable attenuation by providing conventional Q settings wrt equalization/filtering.
/// That is, Ratio Q for how wide the attenuation reaches and Mod Q for how much to attenuate.
fn filter(p:f32, freq:f32, bandpass:&Bp) -> Range {
    let min_f = sample(&bandpass.0, p).max(MF as f32);
    let max_f = sample(&bandpass.1, p).min(NF as f32);
    if freq < min_f || freq > max_f {
        return 0f32
    } else {
      return 1f32  
    }
}


/// Given a cocktail, apply it at (k,x,d) iff it exists 
/// Otherwise apply the default value.
fn mix_or(default:f32, maybe_cocktail:&Option<WRangers>, k:usize, x:f32, d:f32) -> f32 {
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
/// * `multipliers` Frequencies for multiplying the curr frequency. For example, integer harmonics or bell partials. Values must be in range of (0, NF/2]
/// * `amplifiers` Amplitudes for each multiplier. Values must be in the range of [0, 1].
/// * `rangers` Optional callbacks to apply for modulating amp, freq,and phase on each multiplier (by index + 1 as k).
/// * `gate_thresh` Minimum allowed amplitude. Use 0 for an allpass. 
/// * `clip_thresh` Maximum allowed amplitude, truncating larger values to `clip_thresh`. Use 1 for an allpass. 
/// 
/// 
/// ### Returns
/// A samplebuffer representing audio data of the specified event.
pub fn blender(
    frex: &Frex,
    expr: &Expr,
    span: &Span,
    bp: &Bp,
    multipliers: &Vec<Freq>,
    amplifiers: &Vec<Range>,
    phases: &Vec<Radian>,
    modders:&Modders,
    thresh: (f32, f32)
) -> SampleBuffer {
    let (glide_from, maybe_prev, freq, maybe_next, glide_to) = frex;
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
            let k = i + 1;
            let aaa = mix_or(1f32, &modders[1], k, p, span.1);
            let frequency = m * fm * freq * aaa;
            let amplifier = amplifiers[i];
            if amplifier > 0f32 {
                let amp = amplifier * am * filter(p, frequency, bp) * mix_or(1f32, &modders[0], k, p, span.1);
                if amp != 0f32 {
                    let phase = (frequency * pi2 * t) 
                        + phases[i]
                        + pm 
                        + mix_or(0f32, &modders[2], k, p, span.1); 
                    v += amp * phase.sin();
                }
            }
        }

        let (gate_thresh, clip_thresh) = thresh;

        if v.abs() > clip_thresh {
            let sign:f32 = if v > 0f32 { 1f32 } else { -1f32 };
            sig[j] += sign * clip_thresh    
        }

        if v.abs() >= gate_thresh {
            sig[j] += v
        }

    }

    sig
}

pub fn gen_cocktail(n:usize)-> Vec<Mixer> {
    use rand;
    use rand::Rng;
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();

    if n > example_options.len() {
        panic!("Requested more rangers than are available. Repeating the same ranger is the same as boosting its weight.")
    }

    let weights:Vec<f32> = if n == 1usize {
        vec![1f32]
    } else {
        let init = rng.gen();
        let mut ws = vec![init];
        for i in 0..(n-1) {
            let rem = 1f32 - ws.iter().sum::<f32>();
            let next = if i == (n-2) { rem } else {
                rng.gen::<f32>() * rem
            };
            ws.push(next) 
        }

        ws
    };

    let mut opts = example_options.to_vec();
    opts.shuffle(&mut rng);
    let rangers:Vec<Ranger> = opts.to_vec().iter().cloned().take(n).collect();   
    weights.into_iter().zip(rangers.into_iter()).collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{files, phrasing};
    use crate::render::engrave;

    static TEST_DIR:&str = "dev-audio/blend";
    static modders:Modders = [None,None,None];

    fn test_frex() -> Frex {
        (
            GlideLen::None, Some(400f32), 500f32, Some(600f32), GlideLen::None
        )
    }

    fn test_expr() -> Expr {
        (vec![1f32], vec![1f32], vec![0f32])
    }

    fn test_span() -> Span {
        (1.5, 2.0)
    }

    fn test_bp() -> Bp {
        (vec![MFf], vec![NFf])
    }

    fn test_thresh() -> Clippers {
        (0f32, 1f32)
    }

    fn write_test_asset(signal:&SampleBuffer, test_name:&str) {
        files::with_dir(TEST_DIR);
        let filename = format!("{}/{}.wav", TEST_DIR, test_name);
        engrave::samples(SR, &signal, &filename);
    } 

    #[test]
    fn test_multipliers_overtones() {
        let test_name = "blender-overs";
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let amplifiers:Vec<f32> = vec![1f32; multipliers.len()];
        let phases:Vec<f32> = vec![pi2; multipliers.len()];

        let signal = blender(
            &test_frex(),
            &test_expr(),
            &test_span(),
            &test_bp(),
            &multipliers,
            &amplifiers,
            &phases,
            &modders,
            test_thresh()
        );
        write_test_asset(&signal, &test_name)
    }

    #[test]
    fn test_multipliers_undertones() {
        let test_name = "blender-unders";
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| 1f32/x as f32).collect();
        let amplifiers:Vec<f32> = vec![1f32; multipliers.len()];
        let phases:Vec<f32> = vec![pi2; multipliers.len()];

        let signal = blender(
            &test_frex(),
            &test_expr(),
            &test_span(),
            &test_bp(),
            &multipliers,
            &amplifiers,
            &phases,
            &modders,
            test_thresh()
        );
        write_test_asset(&signal, &test_name)
    }


    #[test]
    fn test_bp_filters() {
        let test_name = "blender-overs-highpass-filter";
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let amplifiers:Vec<f32> = vec![1f32; multipliers.len()];
        let phases:Vec<f32> = vec![pi2; multipliers.len()];
        let span = test_span();

        let n_samples = crate::time::samples_of_cycles(span.0, span.1);
        let highpass_filter:Vec<f32> = (0..n_samples).map(|x| NFf * (x as f32/n_samples as f32)).collect();
        let lowpass_filter = vec![NFf];
        let signal = blender(
            &test_frex(),
            &test_expr(),
            &test_span(),
            &(highpass_filter, lowpass_filter),
            &multipliers,
            &amplifiers,
            &phases,
            &modders,
            test_thresh()
        );
        write_test_asset(&signal, &test_name);


        let test_name = "blender-overs-lowpass-filter";
        let highpass_filter = vec![MFf];
        let lowpass_filter = (0..n_samples).map(|x| NFf * (x as f32/n_samples as f32)).collect();

        let signal = blender(
            &test_frex(),
            &test_expr(),
            &test_span(),
            &(highpass_filter, lowpass_filter),
            &multipliers,
            &amplifiers,
            &phases,
            &modders,
            test_thresh()
        );
        write_test_asset(&signal, &test_name);
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
        let test_name = "blender-expr-fmod";
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let amplifiers:Vec<f32> = vec![1f32; multipliers.len()];
        let phases:Vec<f32> = vec![pi2; multipliers.len()];
        let span = test_span();
        let n_samples = crate::time::samples_of_cycles(span.0, span.1);
        let expr:Expr = (vec![1f32], small_f_modulator(span.0, n_samples), vec![0f32]);

        let signal = blender(
            &test_frex(),
            &expr,
            &test_span(),
            &test_bp(),
            &multipliers,
            &amplifiers,
            &phases,
            &modders,
            test_thresh()
        );
        write_test_asset(&signal, &test_name)
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
        let test_name = "blender-expr-pmod";
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let amplifiers:Vec<f32> = vec![1f32; multipliers.len()];
        let phases:Vec<f32> = vec![pi2; multipliers.len()];
        let span = test_span();
        let span = test_span();
        let n_samples = crate::time::samples_of_cycles(span.0, span.1);
        let expr:Expr = (vec![1f32], vec![1f32], small_p_modulator(span.0, n_samples));

        let signal = blender(
            &test_frex(),
            &expr,
            &test_span(),
            &test_bp(),
            &multipliers,
            &amplifiers,
            &phases,
            &modders,
            test_thresh()
        );
        write_test_asset(&signal, &test_name)
    }


    #[test]
    fn test_gate_thresh() {
        let test_name = "blender-thresh";

        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let amplifiers:Vec<f32> = vec![1f32; multipliers.len()];
        let phases:Vec<f32> = vec![pi2; multipliers.len()];
        let span = test_span();
        let thresh = (0.3f32, 0.7f32);
        let signal = blender(
            &test_frex(),
            &test_expr(),
            &test_span(),
            &test_bp(),
            &multipliers,
            &amplifiers,
            &phases,
            &modders,
            thresh
        );

        write_test_asset(&signal, &test_name)
    }

    #[test]
    fn test_modders_amp() {
        let test_name = "blender-modders-amp";
        
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let amplifiers:Vec<f32> = vec![1f32; multipliers.len()];
        let phases:Vec<f32> = vec![pi2; multipliers.len()];
        let the_modders:Modders = [
            Some(gen_cocktail(2)),
            None,
            None,
        ];
        let signal = blender(
            &test_frex(),
            &test_expr(),
            &test_span(),
            &test_bp(),
            &multipliers,
            &amplifiers,
            &phases,
            &the_modders,
            test_thresh()
        );

        write_test_asset(&signal, &test_name)
    }


    #[test]
    fn test_modders_freq() {
        let test_name = "blender-modders-freq";
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let amplifiers:Vec<f32> = vec![1f32; multipliers.len()];
        let phases:Vec<f32> = vec![pi2; multipliers.len()];
        let the_modders:Modders = [
            None,
            Some(gen_cocktail(2)),
            None,
        ];
        let signal = blender(
            &test_frex(),
            &test_expr(),
            &test_span(),
            &test_bp(),
            &multipliers,
            &amplifiers,
            &phases,
            &the_modders,
            test_thresh()
        );

        write_test_asset(&signal, &test_name)
    }


    #[test]
    fn test_modders_phase() {
        let test_name = "blender-modders-phase";
        let multipliers:Vec<f32> = (1..15).step_by(2).map(|x| x as f32).collect();
        let amplifiers:Vec<f32> = vec![1f32; multipliers.len()];
        let phases:Vec<f32> = vec![pi2; multipliers.len()];
        let the_modders:Modders = [
            None,
            None,
            Some(gen_cocktail(2)),
        ];
        let signal = blender(
            &test_frex(),
            &test_expr(),
            &test_span(),
            &test_bp(),
            &multipliers,
            &amplifiers,
            &phases,
            &the_modders,
            test_thresh()
        );

        write_test_asset(&signal, &test_name)
    }


    const MONICS: [usize; 59] = [
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
        21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59
    ];

    const DOMAIN: [f32; 48000] = {
        let mut array = [0.0; 48000];
        let mut i = 0;
        while i < 48000 {
            array[i] = i as f32 / 48000.0;
            i += 1;
        }
        array
    };

    #[test]
    fn test_gen_mixer() {
        let n:usize = 3;
        let d = 1f32;
        let min = 0f32;
        let max = 1f32;

        let mixers = gen_cocktail(n);
        for k in MONICS {
            let kf = k as f32;
            let mut has_value = false;
            let mut not_one = false;
            for x in DOMAIN {
                let y = mix(k, x, d, &mixers);
                if y > 0f32 && !has_value {
                    has_value = true
                };
                if y < 1f32 && !not_one {
                    not_one = true
                };
                assert!(y >= min, "Mixing rangers must not produce values below {}", min);
                assert!(y <= max, "Mixing rangers must not produce values above {}", max);
            }
            assert!(has_value, "Mixing rangers must not be 0 valued over its domain");
            assert!(not_one, "Mixing rangers must not be 1 valued over its domain");
        }
    }

}