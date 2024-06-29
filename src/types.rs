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
    pub type Amps = Vec<Range>;
    pub type Phases = Vec<Radian>;
    pub type Muls = Vec<Freq>;
    
    pub type Radian = f32;
    pub type Range = f32;

    /// Sample values in -1 to 1
    pub type SampleBuffer = Vec<f32>;

    /// Sample values in 0 to 1 
    pub type RangeBuffer = Vec<f32>; 


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
    /// Contour vector for Highpass, Lowpass frequencies.
    /// 
    /// Minimum number of entries is 1 and can be the Minimum Frequency (MF) or Nyquist Frequency (NF).
    /// Can have any number of entries and the highpass/lowpass vecs can have different lenghts. Gentime filter will interpolate
    pub type Bp = (Vec<f32>, Vec<f32>);


    #[derive(Clone,Copy)]
    pub enum GlideLen {
        None,
        Quarter,
        Eigth,
        Sixteenth
    }


    /// Context window for a frequency in series of frequencies, as in a melody. 
    /// 
    /// - f32,f32,f32 Second, Third, and Fourth entries describe the frequencies being navigated.
    /// - f32 Centermost entry is the current frequency to perform.
    /// 
    /// The first and final f32 are the previous/next frequency.
    /// First and final GlideLen describe how to glide, 
    /// where the first GlideLen pairs with the "prev" frquency and the final GlideLen pairs with the "next" frquency.
    /// 
    /// This allows us to glide into a note from its predecessor,
    /// and glide out of a note into its upcoming note,
    /// Or perform no glide either way.
    ///
    /// If a C Major chord is spelled as C, E, G and we wanted to arpeggiate the notes,
    /// then an analogous Frex looks like (GlideLen::None, None, C, E, GlideLen::None)
    /// and then for the second note, (GlideLen::None, C, E, G, GlideLen::None)
    /// 
    /// as of May 25 2024 the glide modulation logic is yet to be implemented in the ugen
    pub type Frex = (GlideLen, Option<Freq>, Freq, Option<Freq>, GlideLen);

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
    pub type Cps = f32;
    pub type Span = (Cps, NCycles);
    pub type Duration = f32;
    pub type MidiVal = i32;
    pub type SignedByte = i8;

    pub type Midi = (Duration, MidiVal, SignedByte);

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Marker {
        tag: String,
        start: f32,
        end:f32, 
        instance:usize
    }

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
    pub struct DruidicScore {
        pub conf: Conf,
        pub dimensions: Dimensions,
        pub parts: Vec<DruidicScoreEntry<synthesis::Note>>,
        pub markers:Vec<Marker>
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Conf {
        pub cps: f32,
        pub root: f32
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Template {
        conf: Conf,
        dimensions:Dimensions,
        parts: Vec<timbre::Arf>,
    }

    pub type Melody<C> = Vec<Vec<C>>;
    pub type ScoreEntry<C> = (timbre::Arf, Melody<C>);
    pub type DruidicScoreEntry<C> = (timbre::AmpContour, timbre::Arf, Melody<C>);
    pub type Part = (timbre::Arf, Melody<synthesis::Monae>);
    pub type Entry = (timbre::Arf, Melody<synthesis::Note>);
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

    pub type BandpassFilter = (FilterMode, synthesis::FilterPoint, synthesis::Bandpass);

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
        Fall,
        Snap,
        Spring,
        Pluck,
        Bloom,
        Burst,
        Pad,
        Drone,
    }

    #[derive(Debug, Serialize)]
    pub enum MicroLifespan {
        Pop,
        Chiff,
        Click
    }
    
    #[derive(Debug, Serialize)]
    pub enum AmpContour {
        Fade,
        Throb,
        Surge,
        Chops,
        Flutter,
    }

    /// - mode and role are the primary selectors for mebn druid
    /// - register defines how much harmonic space is available
    /// - visibility affects overall amplitude
    /// - energy defines how much harmonic content is present
    /// - presence is an envelope selector
    #[derive(Debug, Serialize, Deserialize, Copy, Clone)]
    pub struct Arf {
        pub mode: Mode,
        pub register: i8,
        pub role: Role,
        pub visibility: Visibility,
        pub energy: Energy,
        pub presence: Presence,
    }

    #[derive(Debug, Serialize, Copy, Clone)]
    pub enum Visibility {
        Foreground,
        Visible,
        Background,
        Hidden,
    }

    #[derive(Debug, Serialize, Copy, Clone)]
    pub enum Mode {
        Melodic,
        Enharmonic,
        Vagrant,
        Bell,
        Noise
    }

    #[derive(Debug, Serialize, Copy, Clone)]
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
