use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub type Rotation = i8; 
pub type Monic = i8;
pub type Melody<C> = Vec<Vec<C>>;
pub type Duration = f32;
pub type Freq = f32;
pub type Ampl = f32;
pub type Mote = (Duration, Freq, Ampl);
pub type Origin = (Rotation, Q);
pub type Monae = (Rotation, Q, Monic);
pub type ScoreEntry<C> = (ContribComp, Melody<C>);


#[derive(Debug, Serialize)]
pub struct ContribComp {
    pub base: BaseOsc,
    pub fill: Fill,
    pub mode: Mode,
    pub register: i32,
    pub role: Role,
    pub visibility: Visibility,
}

#[derive(Debug, Serialize)]
pub struct PlayerTrack<C> {
    pub conf: Conf,
    pub composition: Composition<C>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Conf {
    pub origin: &'static str,
    pub duration: i32,
    pub cps: f32,
    pub title: &'static str,
    pub transposition: i32,
}


#[derive(Debug, Serialize)]
pub struct Composition<C> {
    pub composition_id: i32,
    pub duration: i32,
    pub quality: &'static str,
    pub dimensions: Dimensions,
    pub parts: Vec<ScoreEntry<C>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Dimensions {
    pub size: i32,
    pub cpc: i32,
    pub base: i32,
}


#[derive(Debug, Serialize)]
pub enum Fill {
    Frame,
    Support,
    Focus
}

#[derive(Debug, Serialize)]
pub enum Visibility {
    Foreground,
    Visible,
    Background,
    Hidden,
}

#[derive(Debug, Serialize)]
pub enum Mode {
    Melodic,
    Enharmonic,
    Vagrant,
    Noise
}

#[derive(Debug, Serialize)]
pub enum BaseOsc {
    Sine,
    Square,
    Sawtooth,
    Triangle
}

#[derive(Debug, Serialize)]
pub enum Role {
    Kick,
    Perc,
    Hats,
    Bass,
    Chords,
    Lead
}


#[derive(Debug)]
pub enum Q {
    O,
    U
}


/// When in the ContribComp's melody does the filter activate
#[derive(Debug)]
pub enum FilterPoint {
    Constant,
    Mid, 
    Tail
}

/// How the filter goes  from point A to point B
#[derive(Debug)]
pub enum FilterMode {
    Linear,
    Logarithmic,
}

#[derive(Debug)]
pub enum Energy {
    Low,
    Medium,
    High
}

#[derive(Debug)]
pub enum Presence {
    Staccatto,
    Legato,
    Tenuto,
}

pub type Bandpass = (f32, f32);

/// Instructions for rendering a part in a composition
/// The "same" part can have many renditions. For example, using new bandpass settings remarkably affects how it is perceived
#[derive(Debug)]
pub struct ContribSound {
    pub filter: (FilterMode, FilterPoint, Bandpass),
    pub energy: Energy,
    pub presence: Presence,
    pub pan: f32
}

#[derive(Debug)]
pub struct InputScore {
    pub composition_id: i32,
    pub duration: f32,
    pub origin: Origin,
    pub progression: Vec<(i32, Origin)>,
    pub parts: HashMap<ContribComp, Vec<Vec<Monae>>>,
}
