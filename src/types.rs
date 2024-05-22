use std::collections::HashMap;

pub type Range = f32;
pub type Radian = f32;

pub mod synthesis {
    use serde::{Deserialize, Serialize};
    use super::timbre;
    use super::render;

    pub type Rotation = i8;
    pub type Q = i8;
    pub type Ratio = (i32, i32);
    pub type Place = (Rotation, Q);
    pub type Monic = i8;
    pub type Register = i8;
    pub type Origin = (Rotation, Q);
    pub type Monae = (Rotation, Q, Monic);
    pub type Tone = (Register, Monae);
    pub type Duration = Ratio;
    pub type Dur = f32;
    pub type Freq = f32;
    pub type Ampl = f32;
    pub type Mote = (Dur, Freq, Ampl);
    pub type Note = (Duration, Tone, Ampl);
    pub type Progression = Vec<(Duration, Place)>;
    
    pub type Radian = f32;
    pub type Range = f32;

    /// When in the melody does the filter activate
    #[derive(Debug, Clone, Copy)]
    pub enum FilterPoint {
        Constant,
        Head, 
        Tail
    }
    #[derive(Clone, Copy, Debug)]
    pub enum Direction {
        Constant,
        Rising,
        Falling,
        Brownian
    }

    
    pub type Bandpass = (f32, f32);
    /// Contour vector for Highpass, Lowpass frequencies
    pub type Bp = (Vec<f32>, Vec<f32>);
}

pub mod render {
    use serde::{Deserialize, Serialize};
    use super::synthesis;
    use super::timbre;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Dimensions {
        pub size: i8,
        pub cpc: i16,
        pub base: i8,
    }
    pub type NCycles = f32;
    pub type NSamples = usize;
    pub type Span = (NCycles, NSamples);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PlayerTrack<C> {
        pub conf: Conf,
        pub duration: f32,
        pub dimensions: Dimensions,
        pub parts: Vec<ScoreEntry<C>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Score {
        pub conf: Conf,
        pub dimensions: Dimensions,
        pub parts: Vec<ScoreEntry<synthesis::Note>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Conf {
        pub cps: f32,
        pub root: f32,
        pub cube: timbre::Cube,
    }
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Template {
        conf: Conf,
        dimensions:Dimensions,
        parts: Vec<timbre::Contrib>,
    }

    pub type Melody<C> = Vec<Vec<C>>;
    pub type ScoreEntry<C> = (timbre::Contrib, Melody<C>);
    pub type Part = (timbre::Contrib, Melody<synthesis::Monae>);
    pub type Entry = (timbre::Contrib, Melody<synthesis::Note>);
}

pub mod timbre {
    use super::render;
    use super::synthesis;
    use serde::{Deserialize, Serialize};


    /// How the filter goes from point A to point B
    #[derive(Debug, Clone, Copy)]
    pub enum FilterMode {
        Linear,
        Logarithmic,
    }

    #[derive(Debug, Serialize, Clone)] // requires custom serde Deserialize
    pub enum MelodicOsc {
        Sine,
        Square,
        Sawtooth,
        Triangle,
        All
    }

    #[derive(Debug, Serialize, Clone)] // requires custom serde Deserialize
    pub enum BaseOsc {
        Sine,
        Square,
        Sawtooth,
        Triangle,
        Poly,
        Bell,
        Noise,
        All
    }

    pub type BandpassFilter =  (FilterMode, synthesis::FilterPoint, synthesis::Bandpass);

    #[derive(Debug)]
    pub struct Sound {
        pub bandpass:BandpassFilter,
        pub energy: Energy,
        pub presence: Presence,
        pub pan: f32
    }
    #[derive(Debug)]
    pub struct Sound2 {
        pub bandpass:BandpassFilter,
        pub extension: usize
    }

    #[derive(Debug)]
    pub struct Timeframe {
        pub cycles: f32,
        pub p: super::Range,
        pub instance: usize
    }

    #[derive(Debug)]
    pub struct Phrasing {
        pub cps: f32,
        /// Current position wrt the complete Composition
        pub form: Timeframe,
        /// Current position wrt the current Arc
        pub arc: Timeframe,
        /// Current position wrt the current Line in an Arc
        pub line: Timeframe,
        /// Current position wrt the current Note in a Line
        pub note: Timeframe
    }

    #[derive(Debug, Serialize)]
    pub enum AmpLifespan {
        Spring,
        Pluck,
        Bloom,
        Pad,
        Drone,
    }
    
    #[derive(Debug, Serialize)]
    pub enum AmpContour {
        Fade,
        Throb,
        Surge,
        Chops,
        Flutter,
    }

    pub type Ampex = (AmpLifespan, AmpContour);


    #[derive(Debug, Serialize, Deserialize)]
    pub struct Contrib {
        pub mode: Mode,
        pub register: u32,
        pub role: Role,
        pub visibility: Visibility,
        pub energy: Energy,
        pub presence: Presence,
        pub ampex: (AmpLifespan, AmpContour),
    }

    #[derive(Debug, Serialize)]
    pub enum Cube {
        Room,
        Hall, 
        Vast
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
    pub enum Role {
        Kick,
        Perc,
        Hats,
        Bass,
        Chords,
        Lead
    }

    #[derive(Debug, Serialize, Copy, Clone)]
    pub enum Energy {
        Low,
        Medium,
        High
    }

    #[derive(Debug, Serialize, Copy, Clone)]
    pub enum Presence {
        Staccatto,
        Legato,
        Tenuto,
    }
}
