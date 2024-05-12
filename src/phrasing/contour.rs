use crate::phrasing::AmpModulation;
use crate::synth::{pi,pi2,SampleBuffer};
use crate::types::timbre::AmpContour;


/// Given an index i in a sample buffer representing n_cycles,
/// Produce amplitude modulation for a long form lifespan 
/// Uses 1 and 0 as min and max values. May start and end on either.
pub fn mod_contour(n_samples:usize, n_cycles:f32, contour:&AmpContour, rev:bool) -> AmpModulation {
    use AmpContour::*;
    let mut modulator:AmpModulation = vec![0f32; n_samples];

    let range = modulator.iter_mut().enumerate();

    match contour {
        Fade => {
            for (i, sample) in range {
                let p:f32 = i as f32 / n_samples as f32;
                *sample = 1f32 - p
            }
        },
        _ => {
            panic!("Not implemented for contour")
        }
    };

    if rev { modulator.reverse() };
    modulator
}


/// Given an audio signal and an amplitude modulation signal,
/// Adjust the amplitude of the signal by the given shape.
fn apply_contour(signal: &mut SampleBuffer, contour:&AmpModulation) {
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
mod test {
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