/// A synth snare from three components
use crate::synth::{pi, pi2, SampleBuffer};
use crate::types::synthesis::{Bandpass, Direction, Duration, FilterPoint, Freq, Monae, Mote, Note, Tone};
use crate::presets::{Ctx, Coords, AmpMod, PhaseMod,FreqMod, none, bell};
use crate::types::{Range, Radian};
use crate::types::timbre::{Sound2, BandpassFilter, Energy, Presence, BaseOsc, Sound, FilterMode, Timeframe, Phrasing, Ampex};
use crate::time;

static one:f32 = 1f32;

/// Constrains the range of ampgens to [0, 1] for all amods
fn amp_filter(y:f32) -> f32 {
    one - (one - (-one/(0.5f32 * y * y)).exp())
}


/// melodic: provides a tonal center 
/// enharmonic: provides a percussive quality 
/// noise: provides the "snare" buzzing
pub fn amod_melodic(xyz:&Coords, ctx:&Ctx, snd:&Sound2,phr:&Phrasing) -> Range {
    if xyz.i == 0 {
        return 1f32
    }
    let n_samples = time::samples_from_dur(xyz.cps, ctx.dur_seconds);
    let p = xyz.i as f32/ n_samples as f32;
    let y = one / (xyz.k as f32* p * p.sqrt());
    amp_filter(y)  
}


pub fn amod_enharmonic(xyz:&Coords, ctx:&Ctx, snd:&Sound2,phr:&Phrasing) -> Range {
    if xyz.i == 0 {
        return 1f32
    }
    let kf = xyz.k as f32;
    let n_samples = time::samples_from_dur(xyz.cps, ctx.dur_seconds);
    let p = xyz.i as f32/ n_samples as f32;
    let y = 0.1f32 * kf.sqrt()/(kf * kf); 
    amp_filter(y)  
}

pub fn amod_noise(xyz:&Coords, ctx:&Ctx, snd:&Sound2,phr:&Phrasing) -> Range {
    if xyz.i == 0 {
        return 1f32
    }
    let kf = xyz.k as f32;
    let n_samples = time::samples_from_dur(xyz.cps, ctx.dur_seconds);
    let p = xyz.i as f32/ n_samples as f32;
    let y = one / (one - (-0.75f32 * (p + one)).exp());
    // no amp_filter
    y - one
}

/// Create a bell tone contribution for this noteevent
pub fn freq_component_enharmonic(note:&Note, energy:&Energy, snd:&Sound2, phr:&mut Phrasing, coeffs:&Vec<bell::BellPartial>) -> SampleBuffer {
    let signal:Vec<f32> = Vec::new();
    let n_cycles = time::duration_to_cycles(note.0);
   
   println!("Using static bell coefficients");
    

    let sound = Sound {
        bandpass: snd.bandpass,
        energy:energy.to_owned(),
        presence : Presence::Staccatto,
        pan: 0f32,
    };

    let mgen = bell::Mgen {
        osc:BaseOsc::Bell,
        sound
    };
    mgen.inflect_bell(&coeffs, &note, phr)
}

use crate::render;

use super::bell::BellPartial;
pub fn render_line(line:&Vec<Note>, energy:&Energy, snd:&Sound2, phr:&mut Phrasing) -> SampleBuffer {
    let n_cycles = line.iter().fold(0f32, |acc, note| acc + time::duration_to_cycles(note.0));
    let ext = 1;
    phr.line.cycles = n_cycles;
    
    let mut buff:SampleBuffer = Vec::new();


    let coeffs:Vec<bell::BellPartial> = vec![
        (0.00055, 0.25),
        (0.0013, 0.5),
        (0.005, 1.0),
        (0.02, 2.11),
        (0.023, 2.7),
        (0.012, 4.2),
        (0.0071, 5.1)
    ].iter()
    // quick dity ampmod
    .map(|(w, f)| (w * 10f32, *f))
    .collect::<Vec<BellPartial>>();

    for (index, &note) in line.iter().enumerate() {
        //@bug not correct implementation of p. needs to be decided by accumulative position not index
        phr.line.p = render::realize::dur_to(&line, index) / n_cycles;

        let mut enharmonic = freq_component_enharmonic(&note, energy, snd, phr, &coeffs);

        buff.append(&mut enharmonic)
    }
    buff
}

