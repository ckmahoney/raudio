use std::collections::HashMap;

pub type Rotation = i8; 
pub type Monic = i8;
#[derive(Debug)]
pub enum Q {
    O,
    U
}
pub type Origin = (Rotation, Q);
pub type Monae = (Rotation, Q, Monic);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Spec {
    pub role: &'static str,
    pub register: i32,
    pub fill: &'static str,
    pub spec_type: &'static str,
}

/// When in the Spec's melody does the filter activate
#[derive(Debug)]
pub enum FilterPoint {
    Constant,
    Mid, 
    Tail
}

/// How does the filter get from point A to point B
#[derive(Debug)]
pub enum FilterMode {
    Linear,
    Logarithmic,
}

#[derive(Debug)]
pub enum ContribEnergy {
    Low,
    Medium,
    High
}

#[derive(Debug)]
pub enum ContribPresence {
    Staccatto,
    Legato,
    Tenuto,
}


pub type Bandpass = (f32, f32);

/// Instructions for rendering a part in a composition
/// The "same" part can have many renditions. For example, using new bandpass settings remarkably affects how it is perceived
#[derive(Debug)]
pub struct SegmentSoundSpec {
    pub filter: (FilterMode, FilterPoint, Bandpass, Bandpass),
    pub energy: ContribEnergy,
    pub presence: ContribPresence,
    pub pan: f32
}


#[derive(Debug, Clone)]
pub struct Conf {
    pub origin: &'static str,
    pub duration: i32,
    pub cps: f32,
    pub title: &'static str,
    pub transposition: i32,
}

#[derive(Debug)]
pub struct InputScore {
    pub composition_id: i32,
    pub duration: f32,
    pub origin: Origin,
    pub progression: Vec<(i32, Origin)>,
    pub parts: HashMap<Spec, Vec<Vec<Monae>>>,
}