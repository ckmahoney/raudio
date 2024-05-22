use crate::phrasing::AmpModulation;
use crate::synth::{pi,pi2,SampleBuffer};
use crate::types::timbre::AmpContour;
use crate::types::synthesis::{Range, Radian,Freq};

const x:AmpContour = AmpContour::Fade;

pub type Cont<T> = Vec<T>;
pub type AmpCont = Cont<Range>;
pub type FreqCont = Cont<Freq>;
pub type PhaseCont = Cont<Radian>;

pub type Expr = (AmpCont, FreqCont, PhaseCont);

pub struct Timeframe {
    pub n_cycles: f32,
    pub p: Range,
}
pub struct Position {
    cps: f32,
    note: Timeframe,
    line: Timeframe,
    arc: Timeframe,
    form: Timeframe,
}

/// Given an index i in a sample buffer representing n_cycles,
/// Produce amplitude modulation for a long form lifespan 
/// Uses 1 and 0 as min and max values. May start and end on either.
/// "Default" description is a forward contour (not reverse) starting at 0 going to 1.
pub fn gen_contour(n_samples:usize, n_cycles:f32, contour:&AmpContour, reverse:bool) -> AmpModulation {
    use AmpContour::*;
    let mut modulator:AmpModulation = vec![0f32; n_samples];

    let range = modulator.iter_mut().enumerate();

    let n = n_samples as f32;

    match contour {
        // linear rise and fall
        Fade => {
            if reverse {
                for (i, sample) in range {
                    let p = i as f32 / n;
                    *sample = 1f32 - p
                }
            } else {
                for (i, sample) in range {
                    let p = (n - i as f32) / n;
                    *sample = 1f32 - p
                }
            }
        },
        Surge => {
            let power:i32 = 5;
            fn boost(p:f32) -> f32 {
                (p/5f32)*(1f32-p).sqrt()
            }
            // rise and fall pow5 + logx boost
            if reverse {    
                for (i, sample) in range {
                    let t = i as f32 / n;
                    *sample = -(t - 1f32).powi(power) + boost(1f32-t);
                }
            } else {
                for (i, sample) in range {
                    let t = i as f32 / n;
                    *sample = t.powi(power) + boost(t);
                }
            }
        }
        _ => {
            panic!("Not implemented for contour")
        }
    };

    modulator
}

pub fn sample(contour:&AmpModulation, p:f32) -> f32 {
    let index = (p * contour.len() as f32) as usize;
    contour[index]
}


/// Given an audio signal and an amplitude modulation signal,
/// Adjust the amplitude of the signal by the given shape.
pub fn apply_contour(signal: &mut SampleBuffer, contour:&AmpModulation) {
    let l = signal.len() as f32;
    let l2 = contour.len() as f32;
    for i in 0..signal.len() {
        let p = i as f32 / l;

        let position = p * l2;
        let index = position as usize;
        let rem = position.fract();

        let amp_mod: f32 = if rem == 0.0 { 
            contour[index]
        } else {
            let v1 = contour[index];
            let v2 = if index + 1 < contour.len() { contour[index + 1] } else { v1 }; 
            v1 * (1.0 - rem) + v2 * rem
        };

        signal[i] = signal[i] * amp_mod;
    }
}


#[cfg(test)]
mod test_gen_contour {

    use super::*;
    static n_samples:usize = 48000 * 1;
    static n_cycles:f32 = 2f32;
    static err_margin:f32 = 0.0001;

    #[test]
    fn test_contour_fade() {
        let amp_mod = gen_contour(n_samples, n_cycles, &AmpContour::Fade, false);
        assert_eq!(amp_mod.len(), n_samples, "Must produce a contour of the requested duration in cycles");
        assert_eq!(amp_mod[0], 0f32, "Linear fade must start at 0");

        let f:f32 = amp_mod[amp_mod.len()-1];
        assert!((1f32 - f) < err_margin, "Linear fade must end near 1 when reversed");

        let amp_mod = gen_contour(n_samples, n_cycles, &AmpContour::Fade, true);
        assert_eq!(amp_mod.len(), n_samples, "Must produce a reversed contour of the requested duration in cycles");
        
        let f:f32 = amp_mod[0];
        assert!(f == 1f32, "Must start with 1 when using reverse linear fade");
        let v:f32 = amp_mod[amp_mod.len()-1];
        assert!(v <= err_margin, "Must have a final value near 0");
    }


    #[test]
    fn test_contour_surge() {
        let amp_mod = gen_contour(n_samples, n_cycles, &AmpContour::Surge, false);
        assert_eq!(amp_mod.len(), n_samples, "Must produce a contour of the requested duration in cycles");
        assert_eq!(amp_mod[0], 0f32, "Polynomial surge must start at 1. Found {:#?}", &amp_mod[0..10]);

        let f:f32 = amp_mod[amp_mod.len()-1];
        assert!((1f32 - f) <= err_margin, "Must have a final value near 1");

        let amp_mod = gen_contour(n_samples, n_cycles, &AmpContour::Surge, true);
        assert_eq!(amp_mod.len(), n_samples, "Must produce a reversed contour of the requested duration in cycles");
        
        let f:f32 = amp_mod[0];
        assert!((1f32 - f) <= err_margin, "Must start near 1 when using reverse linear surge");
        let v:f32 = amp_mod[amp_mod.len()-1];
        assert!(v < err_margin, "Linear surge must end near 0 when reversed");
    }
}

#[cfg(test)]
mod test_apply_contour {
    use super::*;
    use crate::analysis;
    static min_allowed_mod:f32 = 0f32;
    static max_allowed_mod:f32 = 0f32;

    #[test]
    fn test_apply_contour() {
        let mut signal:SampleBuffer = vec![1.0f32; 10];
        let contour:AmpModulation = vec![
            0f32,
            0.1f32,
            0.2f32,
            0.3f32,
            0.4f32,
            0.5f32,
            0.6f32,
            0.7f32,
            0.8f32,
            0.9f32
        ];

        apply_contour(&mut signal, &contour);
        assert_eq!(signal, contour, "When provided a unit input vector, must shape the input to contour exactly")
    }


    #[test]
    fn test_apply_contour_inner_interpolation() {
        let mut signal:SampleBuffer = vec![1.0f32; 10];
        let contour:AmpModulation = vec![
            0f32,
            1f32
        ];
        let expected:SampleBuffer = vec![
            0f32,
            0.2f32,
            0.4f32,
            0.6f32,
            0.8f32,
            1f32,
            1f32,
            1f32,
            1f32,
            1f32
        ];

        apply_contour(&mut signal, &contour);
        assert_eq!(signal, expected, "When provided a contour shorter than the input vector, must use linear interpolation to extrapolate values")
    }


    #[test]
    fn test_apply_contour_outter_interpolation() {
        let mut signal:SampleBuffer = vec![1.0f32; 10];
        let contour: AmpModulation = vec![
            0.00f32,
            0.05f32,
            0.10f32,
            0.15f32,
            0.20f32,
            0.25f32,
            0.30f32,
            0.35f32,
            0.40f32,
            0.45f32,
            0.50f32,
            0.55f32,
            0.60f32,
            0.65f32,
            0.70f32,
            0.75f32,
            0.80f32,
            0.85f32,
            0.90f32,
            0.95f32
        ];
        let expected:SampleBuffer = vec![
            0f32,
            0.1f32,
            0.2f32,
            0.3f32,
            0.4f32,
            0.5f32,
            0.6f32,
            0.7f32,
            0.8f32,
            0.9f32
        ];

        apply_contour(&mut signal, &contour);
        assert_eq!(signal, expected, "When provided a contour longer than the input vector, must use the nearest relative point in values")
    }
}