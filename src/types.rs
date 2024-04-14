use std::collections::HashMap;

pub mod synthesis {
    use serde::{Deserialize, Serialize};
    use super::timbre;
    use super::render;

    pub type Rotation = i8; #[derive(Debug, Serialize)]
    pub enum Q {
        O,
        U
    }
    pub type Monic = i8;
    pub type Origin = (Rotation, Q);
    pub type Monae = (Rotation, Q, Monic);
    pub type Duration = f32;
    pub type Freq = f32;
    pub type Ampl = f32;
    pub type Mote = (Duration, Freq, Ampl);

    /// When in the melody does the filter activate
    #[derive(Debug)]
    pub enum FilterPoint {
        Constant,
        Mid, 
        Tail
    }

    /// How the filter goes from point A to point B
    #[derive(Debug)]
    pub enum FilterMode {
        Linear,
        Logarithmic,
    }

    
    pub type Bandpass = (f32, f32);
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

    #[derive(Debug, Serialize, Deserialize)]
    pub struct PlayerTrack<C> {
        pub conf: Conf,
        pub duration: f32,
        pub dimensions: Dimensions,
        pub parts: Vec<ScoreEntry<C>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Conf {
        pub cps: f32,
        pub root: f32,
        pub cube: timbre::Cube,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Composition {
        pub duration: f32,
        pub dimensions: Dimensions,
        pub parts: Vec<Part>,
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
}

pub mod timbre {
    use super::render;
    use super::synthesis;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize)] // requires custom serde Deserialize
    pub enum BaseOsc {
        Sine,
        Square,
        Sawtooth,
        Triangle
    }

    #[derive(Debug)]
    pub struct Sound {
        pub bandpass: (synthesis::FilterMode, synthesis::FilterPoint, synthesis::Bandpass),
        pub energy: Energy,
        pub presence: Presence,
        pub pan: f32
    }


    #[derive(Debug, Serialize, Deserialize)]
    pub struct Contrib {
        pub mode: Mode,
        pub register: i32,
        pub role: Role,
        pub visibility: Visibility,
    }

    #[derive(Debug, Serialize)]
    pub enum Cube {
        Room,
        Hall, 
        Vast
    }

    // #[derive(Debug, Serialize)]
    // pub enum Fill {
    //     Frame,
    //     Support,
    //     Focus
    // }

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


    #[derive(Debug, Serialize)]
    pub enum Energy {
        Low,
        Medium,
        High
    }

    #[derive(Debug, Serialize)]
    pub enum Presence {
        Staccatto,
        Legato,
        Tenuto,
    }
}
