pub mod kick;
pub mod snare;
pub mod hats;
pub mod bell;
pub mod noise;
pub mod none;
pub mod melodic;

use crate::types::synthesis::{Freq, Note, Direction};
use crate::types::timbre::{Sound, Sound2, Energy, Presence, Phrasing};
use crate::types::{Range, Radian};

pub type Modulator<T> = fn (xyz:&Coords, ctx:&Ctx, snd:&Sound2, phr:&Phrasing) -> T;
pub type DynModulator<T> = Box<dyn Fn (&Coords, &Ctx, &Sound2, &Phrasing) -> T>;

pub type AmpMod = Modulator<Range>;
pub type PhaseMod = Modulator<Radian>;
pub type FreqMod = Modulator<Freq>;

pub type DAmpMod = DynModulator<Range>;
pub type DPhaseMod = DynModulator<Radian>;
pub type DFreqMod = DynModulator<Freq>;

pub struct Modulators {
    pub amp: AmpMod,
    pub freq: FreqMod,
    pub phase: PhaseMod,
}

pub struct DModulators {
    pub amp: Option<DAmpMod>,
    pub freq: Option<DAmpMod>,
    pub phase: Option<DPhaseMod>,
}

/// 3-tuple enabling index-based (Vertical) modulation of a signal.
pub type VModulators = (
    Option<DAmpMod>,
    Option<DAmpMod>,
    Option<DPhaseMod>
);


pub struct Ctx {
    pub dur_seconds: f32,
    pub root: f32,
    pub extension: usize
}


pub struct Coords {
    pub cps: Freq,
    pub k: usize,
    pub i: usize
}

pub struct PartialModulators {
    pub amp: Option<AmpMod>,
    pub freq: Option<FreqMod>,
    pub phase: Option<PhaseMod>,
}

impl PartialModulators {
    pub fn dynamize(&self) -> DModulators {
        DModulators {
            freq: if self.freq.is_some() { Some(Box::new(self.freq.unwrap()))} else { None },
            phase: if self.phase.is_some() { Some(Box::new(self.phase.unwrap()))} else { None },
            amp: if self.amp.is_some() { Some(Box::new(self.amp.unwrap()))} else { None },
        }
    }

    pub fn mix(parts:Vec<&PartialModulators>) -> DModulators {
        let fs:Vec<FreqMod> = (&parts).into_iter().filter_map(|partial| partial.freq).collect();
        let ps:Vec<PhaseMod> = (&parts).into_iter().filter_map(|partial| partial.phase).collect();
        let aas:Vec<AmpMod> = (&parts).into_iter().filter_map(|partial| partial.amp).collect();

        let fmod:Option<DFreqMod> = if ps.len() == 0 {
            None
        } else if fs.len() == 1 {
            Some(Box::new(fs[0]))
        } else {
            Some(Box::new(move |xyz:&Coords, ctx:&Ctx, snd:&Sound2, phr:&Phrasing| -> f32 {
                fs.iter().fold(1f32, |v, fmod| 
                    v * (fmod)(xyz, ctx, snd, phr)
                )
            }))
        };

        let amod:Option<DAmpMod> = if ps.len() == 0 {
            None
        } else if aas.len() == 1 {
            Some(Box::new(aas[0]))
        } else {
            Some(Box::new(move |xyz:&Coords, ctx:&Ctx, snd:&Sound2, phr:&Phrasing| -> f32 {
                aas.iter().fold(1f32, |v, fmod| 
                    v * (fmod)(xyz, ctx, snd, phr)
                )
            }))
        };

        let pmod:Option<DPhaseMod> = if ps.len() == 0 {
            None
        } else if ps.len() == 1 {
            Some(Box::new(ps[0]))
        } else {
            Some(Box::new(move |xyz:&Coords, ctx:&Ctx, snd:&Sound2, phr:&Phrasing| -> f32 {
                ps.iter().fold(0f32, |v, fmod| 
                    v + (fmod)(xyz, ctx, snd, phr)
                )
            }))
        };

        DModulators {
            freq: fmod,
            amp: amod,
            phase: pmod
        }
    }
}