/// Synthesizers
/// aka Druid
/// 
/// From the four elementary components
/// Melodic, Enharmonic, Bell, and Noise
/// We create a collection of elementary arfutors.
/// 
/// Synthesizers are likely to feature one or two elements primarly 
/// and attenuate or skip the remaining elements.
/// 
/// Wild synths may feature all elements, shifting form from one to the next!
/// 
/// This model should be able to provide 95% of the sounds we want to use in music :)

pub mod melodic;
pub mod bell;
pub mod noise;

use crate::phrasing::ranger::{Weight, Modders};
use crate::phrasing::contour::{Expr, expr_none};
use crate::{time};
use crate::types::synthesis::{Range,Freq,Bp,Muls, Amps, Phases, Note};
use crate::types::timbre::{Mode, Arf, Energy,Presence, Visibility};
use crate::types::render::{Span};
use crate::render::blend::{GlideLen,Frex, blender};
use crate::render::realize::{mix_buffers};
use crate::synth::{MFf, NFf, SampleBuffer, pi, pi2, SR};
use crate::monic_theory::tone_to_freq;

/// # Element
/// 
/// `mode` The type of signal this represents.
/// `muls` A list of multipliers for a fundamental frequency. Minimum length 1.
/// `modders` Gentime dynamic modulation. A major arfutor for defining this Element's sound.
/// `expr` Carrier signal for amplitude, frequency, and phase for a Note. For example, a pluck has an amplitude envelope rapidly falling and a none phase.
/// `hplp` Bandpass filters for the signal.
/// `thresh` Gate and clip thresholds for distorting the signal. Applied to the summed signal (not per multiplier).
pub struct Element {
    pub mode:Mode,
    pub amps: Amps,
    pub muls: Muls,
    pub phss: Phases,
    pub modders:Modders,
    pub expr:Expr,
    pub hplp: Bp,
    pub thresh: (Range, Range)
}

/// # Druid
/// 
/// A collection of weighted arfutors for a syntehsizer.
/// 
/// Weights of all elements must equal 1. 
pub type Druid = Vec<(Weight, Element)>;
pub type Elementor = Vec<(Weight, fn (f32, &Visibility, &Energy, &Presence) -> Element)>;


/// convenience struct for naming computed values
pub struct ApplyAt {
    pub span:Span,
    pub frex:Frex
}

fn weighted_amps(weight:f32, amps:&Vec<f32>) -> Vec<f32> {
    if weight < 0f32 {
        panic!("Cannot negate amplitude")
    }
    amps.iter().map(|y| y * weight).collect()
}

pub fn inflect(frex:&Frex, at:&ApplyAt, synth:&Elementor, vis:&Visibility, energy:&Energy, presence:&Presence) -> SampleBuffer {
    let n_samples:usize = time::samples_of_dur(at.span.0, at.span.1);
    let druid:Druid = synth.iter().map(|(weight, elementor)| 
        (*weight, elementor(frex.2, vis, energy, presence))
    ).collect();

    let mut sigs:Vec<SampleBuffer> = druid.iter().map(|(weight, element)|
        blender(
            frex, 
            &element.expr, 
            &at.span, 
            &element.hplp, 
            &element.muls, 
            &weighted_amps(*weight, &element.amps),
            &element.phss,
            &element.modders, 
            element.thresh
        )
    ).collect();
    match mix_buffers(&mut sigs) {
        Ok(signal) => signal,
        Err(msg) => panic!("Error while inflecting druid: {}", msg)
    }
}

pub fn freq_frexer(line: &Vec<f32>, glide_from: GlideLen, glide_to: GlideLen) -> Vec<Frex> {
    let len = line.len();
    line.iter().enumerate().map(|(i, &freq)| {
        if i == 0 && (i + 1) == len {
            // single element line
            (GlideLen::None, None, freq, None, GlideLen::None)
        } else if i == 0 {
            (GlideLen::None, None, freq, Some(line[i + 1]), glide_to)
        } else if i == len - 1 {
            (glide_from, Some(line[i - 1]), freq, None, GlideLen::None)
        } else {
            (glide_from, Some(line[i - 1]), freq, Some(line[i + 1]), glide_to)
        }
    }).collect()
}

pub fn line_frexer(line:&Vec<Note>, glide_from:GlideLen, glide_to:GlideLen) -> Vec<Frex> {
    let len = line.len();
    line.iter().enumerate().map(|(i,(_, ref tone,_))|
        if i == 0 && (i + 1) == len {
            // single element line
            (GlideLen::None, None, tone_to_freq(tone), None, GlideLen::None)
        } else if i == 0 {
            (GlideLen::None, None, tone_to_freq(tone), Some(tone_to_freq(&line[i+1].1)), glide_to)
        } else if i == len - 1 {
            (glide_from, Some(tone_to_freq(&line[i-1].1)), tone_to_freq(tone), None, GlideLen::None)
        } else {
            (glide_from, Some(tone_to_freq(&line[i-1].1)), tone_to_freq(tone), Some(tone_to_freq(&line[i+1].1)), glide_to)
        }
    ).collect()
}


pub fn melody_frexer(melody:&Vec<Vec<Note>>, glide_from:GlideLen, glide_to:GlideLen) -> Vec<Vec<Frex>> {
    melody.iter().map(|line| line_frexer(&line, glide_from, glide_to)).collect()
}


#[cfg(test)]
mod test {
    use super::*;
    static cps:f32 = 1.7;
    static test_dir:&str = "dev-audio/druid";
    use crate::files;
    use crate::render::engrave;
    use crate::synth::{SR};

    fn test_vep() -> (Visibility, Energy, Presence) {
        let energy = Energy::Low;
        let presence = Presence::Staccatto;
        let visibility = Visibility::Visible;
        (visibility,energy,presence)
    }

    fn nearly_none_enharmonic(fund:f32, vis:&Visibility, energy:&Energy, presence:&Presence) -> Element {
        Element {
            mode: Mode::Enharmonic,
            muls: vec![1.0, 2.1, 5.3],
            amps: vec![1f32, 1f32, 1f32],
            phss: vec![pi2,pi2, pi2],
            modders: [None, None, None],
            expr: expr_none(),
            hplp: (vec![MFf], vec![NFf]),
            thresh: (0f32, 1f32)
        }
    }


    #[test]
    fn test_blend_single_element() {
        let test_name:&str = "blend-single-enharmonic";
        let freqs:Vec<f32> = vec![200f32, 250f32, 400f32, 350f32, 300f32];
        let durs:Vec<f32> = vec![1f32, 2f32, 1f32, 2f32, 2f32];
        let frexs = freq_frexer(&freqs, GlideLen::Sixteenth, GlideLen::Eigth);
        let mut signal:SampleBuffer = Vec::new();

        let elementor:Elementor = vec![
            (1f32, nearly_none_enharmonic)
        ];
        let (v,e,p) = test_vep();

        for (index, frex) in frexs.iter().enumerate() {
            let dur = durs[index];
            let at = ApplyAt { frex: *frex, span: (cps, dur) };
            signal.append(&mut inflect(&frex, &at, &elementor, &v,&e,&p));
        }
        files::with_dir(test_dir);
        let filename:String = format!("{}/{}.wav", test_dir, test_name);
        engrave::samples(SR, &signal, &filename);
    }
}


pub fn test_data() -> (Vec<f32>,Vec<f32>,Vec<Frex>) {
    let freqs:Vec<f32> = vec![200f32, 250f32, 400f32, 350f32, 300f32];
    let durs:Vec<f32> = vec![1f32, 2f32, 1f32, 2f32, 2f32];
    let frexs = freq_frexer(&freqs, GlideLen::Sixteenth, GlideLen::Eigth);
    (freqs, durs, frexs)
}
pub fn test_frex() -> (Vec<f32>,Vec<f32>,Vec<Frex>) {
    test_data()
}

pub fn test_vep() -> (Visibility, Energy, Presence) {
    let energy = Energy::Low;
    let presence = Presence::Staccatto;
    let visibility = Visibility::Visible;
    (visibility,energy,presence)
}

pub fn modders_none() -> Modders {
    [
        None,
        None,
        None
    ]
}