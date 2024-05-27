/// Passthrough modulators 
/// 
/// Applying these at gentime will have no result on the signal. 

use super::{Coords, Ctx, Sound,Sound2, Direction, Phrasing};

/// Frequency modulation in range of (0, 2.pow(ctx.extension))
pub fn fmod(xyz:&Coords, ctx:&Ctx, snd:&Sound, dir:&Direction, phr:&Phrasing) -> f32 {
    1f32
}

/// Amplitude modulation in range of [0, 1]
pub fn amod(xyz:&Coords, ctx:&Ctx, snd:&Sound, dir:&Direction, phr:&Phrasing) -> f32 {
    1f32
}

/// Phase modulation in range of (-infinity, infinity)
pub fn pmod(xyz:&Coords, ctx:&Ctx, snd:&Sound, dir:&Direction, phr:&Phrasing) -> f32 {
    0f32
}

/// Frequency modulation in range of (0, 2.pow(ctx.extension))
pub fn fmod2(xyz:&Coords, ctx:&Ctx, snd:&Sound2, phr:&Phrasing) -> f32 {
    1f32
}

/// Amplitude modulation in range of [0, 1]
pub fn amod2(xyz:&Coords, ctx:&Ctx, snd:&Sound2, phr:&Phrasing) -> f32 {
    1f32
}

/// Phase modulation in range of (-infinity, infinity)
pub fn pmod2(xyz:&Coords, ctx:&Ctx, snd:&Sound2, phr:&Phrasing) -> f32 {
    0f32
}


/// RANGER
pub fn amod3(k:f32, x:f32, d:f32) -> f32 {
    1f32
}


/// RANGER
pub fn fmod3(k:f32, x:f32, d:f32) -> f32 {
    1f32
}


/// RANGER
pub fn pmod3(k:f32, x:f32, d:f32) -> f32 {
    0f32
}