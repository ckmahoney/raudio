use std::collections::HashMap;

pub type Range = f32;
pub type Radian = f32;
pub mod synthesis {
  use super::render;
  use super::timbre;
  use crate::analysis::in_range;
  use crate::druid::melodic;
  use crate::phrasing::ranger::{KnobMods, KnobMods2};
  use crate::synth::{MFf, NFf};
  use serde::{Deserialize, Serialize};

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
  /// A floating point value value in [0, 1]
  pub type Range = f32;
  pub type Clippers = (f32, f32);

  /// Sample values in -1 to 1
  pub type SampleBuffer = Vec<f32>;

  /// Sample values in 0 to 1
  pub type RangeBuffer = Vec<f32>;

  /// When in the melody does the filter activate
  #[derive(Debug, Clone, Copy)]
  pub enum FilterPoint {
    Constant,
    Head,
    Tail,
  }
  #[derive(Clone, Copy, Debug)]
  pub enum Direction {
    Constant,
    Rising,
    Falling,
    Brownian,
  }

  pub type Bandpass = (f32, f32);
  /// Contour vector for Highpass, Lowpass frequencies.
  ///
  /// Minimum number of entries is 1 and can be the Minimum Frequency (MF) or Nyquist Frequency (NF).
  /// Can have any number of entries and the highpass/lowpass vecs can have different lenghts. Gentime filter will interpolate
  pub type Bp = (Vec<f32>, Vec<f32>);

  // region of frequencies that are amplified more than other regions
  // applied as a gain reduction to out-of-range frequencies
  #[derive(Clone, Debug)]
  pub struct BoostGroup {
    pub bp: Bp,
    // amount to attenuate out of range values
    // (applied as a decrease to all out of range values by inverse)
    pub att: f32,
    // db per octave
    pub rolloff: f32,
    // intensity of rolloff
    pub q: f32,
  }

  impl BoostGroup {
    pub fn static_width(min_freq: f32, max_freq: f32, att: f32, rolloff: f32, q: f32) -> Self {
      Self {
        bp: (vec![min_freq], vec![max_freq]),
        att,
        rolloff,
        q,
      }
    }
  }

  #[derive(Copy, Clone, Debug)]
  pub enum MacroMotion {
    // static values
    Min,
    Mean,
    Max,
    Constant,
    // animated values
    Forward,
    Reverse,
    Random,
  }

  #[derive(Copy, Clone, Debug)]
  pub struct BoostGroupMacro {
    // min/max freqs. is a static value for macro application
    pub bandpass: [f32; 2],
    // min/max range of allowed frequencies in octaves
    pub bandwidth: [f32; 2],
    // amount to attenuate out of range values
    // (applied as a decrease to all out of range values by inverse)
    pub att: [f32; 2],
    // db per octave
    pub rolloff: [f32; 2],
    // intensity of rolloff
    pub q: [f32; 2],
    pub motion: MacroMotion,
  }

  impl BoostGroupMacro {
    pub fn gen(&self) -> BoostGroup {
      let mut rng = rand::thread_rng();
      let min_freq = in_range(&mut rng, self.bandpass[0], self.bandpass[1]);
      let width = in_range(&mut rng, self.bandwidth[0], self.bandwidth[1]);
      BoostGroup {
        bp: (vec![min_freq], vec![min_freq + 2f32.powf(width)]),
        att: in_range(&mut rng, self.att[0], self.att[1]),
        rolloff: in_range(&mut rng, self.rolloff[0], self.rolloff[1]),
        q: in_range(&mut rng, self.q[0], self.q[1]),
      }
    }
  }

  /// Tuple representing the (highpass contour, lowpass contour, boost_groups)
  pub type Bp2 = (Vec<f32>, Vec<f32>, Vec<BoostGroupMacro>);

  pub fn bp2_unit() -> Bp2 {
    (vec![MFf], vec![NFf], vec![])
  }
  #[derive(Clone, Copy)]
  pub enum GlideLen {
    None,
    Quarter,
    Eigth,
    Sixteenth,
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

  /// Parameters for amplitude modulation effects.
  /// Is passed as direct amplitude mutation of audio signal's constituent sine component.
  /// This is the final modulator in the chain of Expr, KnobMod, and Modders.
  #[derive(Debug, Clone, Copy)]
  pub struct AmplitudeModParams {
    pub freq: f32,
    pub depth: f32,
    pub offset: f32,
  }

  /// Parameters for frequency modulation effects.
  /// Is passed as direct frequency mutation of audio signal's constituent sine component.
  /// This is the final modulator in the chain of Expr, KnobMod, and Modders.
  #[derive(Debug, Clone, Copy)]
  pub struct FrequencyModParams {
    pub rate: f32,
    pub offset: f32,
  }

  /// Parameters for phase modulation effects.
  /// Is passed as direct phase mutation of audio signal's constituent sine component.
  /// This is the final modulator in the chain of Expr, KnobMod, and Modders.
  #[derive(Debug, Clone, Copy)]
  pub struct PhaseModParams {
    pub rate: f32,
    pub depth: f32,
    pub offset: f32,
  }

  /// Convient name aliasing for common audio processing effects
  #[derive(Debug, Clone, Copy)]
  pub enum ModulationEffect {
    Tremelo(AmplitudeModParams),
    Vibrato(PhaseModParams),
    Noise(PhaseModParams),
    Chorus(PhaseModParams),
    Sway(FrequencyModParams),
    Warp(PhaseModParams),
  }

  /// Collection of optional additive modulations for a signal.
  /// Entries in the form of (amp, freq, offset, time) modulation vectors.
  /// Use a 0 length entry to skip modulation of that parameter.
  ///
  /// Amp must output value in [0,1]
  /// Freq must output value in (0, Nf/f)
  /// Offset must output a value in [0, 2pi]
  /// Time must output a value in (0, Nf/t)
  pub type Modifiers<'render> = (
    &'render Vec<ModulationEffect>,
    &'render Vec<ModulationEffect>,
    &'render Vec<ModulationEffect>,
    &'render Vec<ModulationEffect>,
  );
  pub type Ms<'render> = &'render (
    Vec<ModulationEffect>,
    Vec<ModulationEffect>,
    Vec<ModulationEffect>,
    Vec<ModulationEffect>,
  );

  pub type ModifiersHolder = (
    Vec<ModulationEffect>,
    Vec<ModulationEffect>,
    Vec<ModulationEffect>,
    Vec<ModulationEffect>,
  );
  pub struct ModBox;
  impl ModBox {
    pub fn unit() -> ModifiersHolder {
      (
        vec![], // amplitude
        vec![], // frequency
        vec![], // phase
        vec![], // time (cps)
      )
    }
  }

  pub type Soids = (Vec<f32>, Vec<f32>, Vec<f32>);

  pub struct Ely {
    pub soids: (Vec<f32>, Vec<f32>, Vec<f32>),
    pub modders: ModifiersHolder,
    pub knob_mods: KnobMods,
  }

  impl Ely {
    pub fn new(soids: Soids, modders: ModifiersHolder, knob_mods: KnobMods) -> Self {
      Ely {
        soids,
        modders,
        knob_mods,
      }
    }
    pub fn from_soids(amps: Vec<f32>, muls: Vec<f32>, phis: Vec<f32>) -> Self {
      Ely {
        soids: (amps, muls, phis),
        modders: ModBox::unit(),
        knob_mods: KnobMods(vec![], vec![], vec![]),
      }
    }

    pub fn push_amod(self, amod: ModulationEffect) -> Self {
      let mut amods = self.modders.1;
      amods.push(amod);
      let modders = (self.modders.0, amods, self.modders.2, self.modders.3);
      Ely { modders, ..self }
    }
  }
}

pub mod render {
  use super::synthesis::{self, *};
  use super::timbre;
  use crate::analysis::delay::DelayParams;
  use crate::druid::melodic::{soids_sawtooth, soids_square, soids_triangle};
  use crate::phrasing::contour::{Expr, Expr2};
  use crate::phrasing::ranger::{KnobMods, KnobMods2};
  use serde::{Deserialize, Serialize};

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
    end: f32,
    instance: usize,
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
    pub markers: Vec<Marker>,
    pub groupEnclosure: timbre::Enclosure,
  }

  #[derive(Debug, Serialize, Deserialize)]
  pub struct Conf {
    pub cps: f32,
    pub root: f32,
  }

  #[derive(Debug, Serialize, Deserialize)]
  pub struct Template {
    conf: Conf,
    dimensions: Dimensions,
    parts: Vec<timbre::Arf>,
  }

  pub type Melody<C> = Vec<Vec<C>>;
  pub type ScoreEntry<C> = (timbre::Arf, Melody<C>);
  pub type DruidicScoreEntry<C> = (timbre::ClientPositioning, timbre::Arf, Melody<C>);
  pub type Part = (timbre::Arf, Melody<Monae>);
  pub type Entry = (timbre::Arf, Melody<Note>);

  #[derive(Clone, Debug)]
  pub struct Feel {
    pub bp: Bp,
    pub modifiers: ModifiersHolder,
    pub clippers: Clippers,
  }

  use rand::seq::SliceRandom;
  use rand::{thread_rng, Rng};

  impl Feel {
    pub fn unit() -> Self {
      Feel {
        bp: (vec![crate::synth::MFf], vec![crate::synth::NFf]),
        modifiers: ModBox::unit(),
        clippers: (0f32, 1f32),
      }
    }

    pub fn select(arf: &timbre::Arf) -> Self {
      use timbre::AmpLifespan;
      use timbre::Role::*;

      let mut rng = rand::thread_rng();

      let bp_reg_low = arf.register as f32;
      const MAX_REGISTER: usize = 14;
      let cap: f32 = if MAX_REGISTER - arf.register as usize <= 1 {
        1f32
      } else {
        let max = MAX_REGISTER as f32 - arf.register as f32;
        max
          * match arf.energy {
            timbre::Energy::Low => 0.5f32,
            timbre::Energy::Medium => 0.7f32,
            timbre::Energy::High => 1f32,
          }
      };
      let bp_reg_high = bp_reg_low + cap;
      let n_segments: usize = 3
        * match arf.visibility {
          timbre::Visibility::Hidden => 1,
          timbre::Visibility::Background => rand::thread_rng().gen_range(2..=3),
          timbre::Visibility::Visible => rand::thread_rng().gen_range(3..=5),
          timbre::Visibility::Foreground => rand::thread_rng().gen_range(4..=7),
        };
      use crate::inp::arg_xform::gen_bp_contour;
      // arbitrary number of samples for the filter contour
      let resolution = 10000;
      let bp: Bp = match arf.visibility {
        timbre::Visibility::Hidden => {
          let highpass = gen_bp_contour(
            n_segments,
            2f32.powf(bp_reg_low),
            2f32.powf(bp_reg_low - 1f32),
            resolution,
          );
          let lowpass = gen_bp_contour(
            n_segments,
            2f32.powf(bp_reg_high - 2f32),
            2f32.powf(bp_reg_high),
            resolution,
          );
          (highpass, lowpass)
        }
        timbre::Visibility::Background => {
          let highpass = gen_bp_contour(
            n_segments,
            2f32.powf(bp_reg_low - 1f32),
            2f32.powf(bp_reg_low),
            resolution,
          );
          let lowpass = vec![crate::synth::NFf];
          (highpass, lowpass)
        }
        timbre::Visibility::Foreground => {
          let highpass = vec![crate::synth::MFf];
          let lowpass = gen_bp_contour(
            n_segments,
            2f32.powf(bp_reg_high - 1f32),
            2f32.powf(bp_reg_high + 1f32),
            resolution,
          );
          (highpass, lowpass)
        }
        timbre::Visibility::Visible => {
          let highpass = gen_bp_contour(
            n_segments,
            2f32.powf(bp_reg_low - 1f32),
            2f32.powf(bp_reg_low + 1f32),
            resolution,
          );
          let lowpass = gen_bp_contour(
            n_segments,
            2f32.powf(bp_reg_high - 1f32),
            2f32.powf(bp_reg_high + 1f32),
            resolution,
          );
          (highpass, lowpass)
        }
      };
      Feel { bp, ..Self::unit() }
    }

    pub fn with_expr(expr: Expr) -> Self {
      Feel { ..Feel::unit() }
    }

    pub fn with_modifiers(self, modifiers: ModifiersHolder) -> Self {
      Feel { modifiers, ..self }
    }
  }

  /// Applied parameters to create a SampleBuffer
  pub type Stem<'render> = (
    &'render Melody<synthesis::Note>,
    Soids,
    Expr,
    Feel,
    KnobMods,
    Vec<crate::analysis::delay::DelayParams>,
  );

  /// Applied parameters to create a SampleBuffer
  /// Tuple represents
  /// (soids, expresion envelopes, bandpass envelopes, dynamic knobs, delay1 (per noteevent), delay2 (total line)), reverb1 (per noteevent), reverb2 (total line))
  pub type Stem2<'render> = (
    &'render Melody<synthesis::Note>,
    Soids,
    Expr,
    Bp2,
    KnobMods2,
    Vec<crate::analysis::delay::DelayParams>,
    Vec<crate::analysis::delay::DelayParams>,
    Vec<crate::reverb::convolution::ReverbParams>,
    Vec<crate::reverb::convolution::ReverbParams>,
  );

  /// SampleBuffer from input samples (like perc)
  /// Tuple represents
  /// (melody, reference sample buffer, amplitude contour, lowpass_cutoff_freq, delay1 (per noteevent), delay2 (total line)), reverb1 (per noteevent), reverb2 (total line))
  pub type Stem3<'render> = (
    &'render Melody<synthesis::Note>,
    SampleBuffer,
    Vec<Range>,
    f32,
    Vec<crate::analysis::delay::DelayParams>,
    Vec<crate::analysis::delay::DelayParams>,
    Vec<crate::reverb::convolution::ReverbParams>,
    Vec<crate::reverb::convolution::ReverbParams>,
  );

  use crate::{presets, AmpContour};
}

pub mod timbre {
  use super::render;
  use super::synthesis;
  use crate::analysis::delay::DelayParams;
  use crate::phrasing::contour::Position;
  use crate::reverb::convolution::ReverbParams;
  use serde::{Deserialize, Serialize};
  use std::fmt;

  pub struct DelayLine;
  impl DelayLine {
    pub fn unit() -> Vec<DelayParams> {
      vec![]
    }
  }

  #[derive(Debug)]
  /// Signal offsets and reverberations to apply to a part
  pub struct SpaceEffects {
    pub delays: Vec<DelayParams>,
    pub reverbs: Vec<ReverbParams>,
    pub gain: f32,
  }
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
    All,
  }

  pub type BandpassFilter = (FilterMode, synthesis::FilterPoint, synthesis::Bandpass);

  #[derive(Debug)]
  pub struct Sound {
    pub bandpass: BandpassFilter,
    pub energy: Energy,
    pub presence: Presence,
    pub pan: f32,
  }

  #[derive(Debug)]
  pub struct Sound2 {
    pub bandpass: BandpassFilter,
    pub extension: usize,
  }

  #[derive(Debug)]
  pub struct Timeframe {
    pub cycles: f32,
    pub p: super::Range,
    pub instance: usize,
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
    pub note: Timeframe,
  }

  #[derive(Debug, Deserialize, Serialize, Copy, Clone)]
  #[serde(rename_all = "kebab-case")]
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
    Click,
  }

  #[derive(Debug, Deserialize, Serialize, Copy, Clone)]
  #[serde(rename_all = "kebab-case")]
  pub enum AmpContour {
    Fade,
    Throb,
    Surge,
    Chops,
    Flutter,
  }

  #[derive(Debug, Deserialize, Serialize, Copy, Clone)]
  #[serde(rename_all = "kebab-case")]
  pub enum Distance {
    Adjacent,
    Near,
    Far,
  }

  #[derive(Debug, Deserialize, Serialize, Copy, Clone)]
  #[serde(rename_all = "kebab-case")]
  pub enum Enclosure {
    Spring,
    Room,
    Hall,
    Vast,
  }

  #[derive(Debug, Deserialize, Serialize, Copy, Clone)]
  #[serde(rename_all = "kebab-case")]
  pub enum Echo {
    None,
    Slapback,
    Trailing,
    Bouncy,
  }

  #[derive(Debug, Deserialize, Serialize, Copy, Clone)]
  #[serde(rename_all = "kebab-case")]
  pub struct ClientPositioning {
    pub echo: Echo,
    pub enclosure: Enclosure,
    pub distance: Distance,
  }

  /// High level description for audio effect generation.
  ///
  /// ### Arguments
  /// - `contour`: Impression of the mean amplitude envelope
  /// - `distance`: Amplitude, reverb, and delay macro. Farther away means quiter with more dispersion whereas closer is louder with more slapback and less dispersion.
  /// - `echoes`: When provided, a style of delay effect to emphasize a part.
  /// - `complexity`: Range in [0,1] describing the richness of sound. 0 represents a plain wave (for example, a sine wave) and 1 a more complex version of that wave (saturated). Has a similar effect on delay and reverb parameters generation.
  #[derive(Debug, Deserialize, Serialize, Copy, Clone)]
  #[serde(rename_all = "kebab-case")]
  pub struct Positioning {
    pub distance: Distance,
    pub echo: Echo,
    pub complexity: f32,
  }

  /// ### Arguments
  /// - `mode`
  /// - `register` defines how much harmonic space is available
  /// - `visibility` affects overall amplitude
  /// - `energy` defines how much harmonic content is present
  /// - `presence` is an envelope selector
  #[derive(Debug, Serialize, Deserialize, Copy, Clone)]
  pub struct Arf {
    pub mode: Mode,
    pub register: i8,
    pub role: Role,
    pub visibility: Visibility,
    pub energy: Energy,
    pub presence: Presence,
  }

  #[derive(Debug, Deserialize, Serialize, Copy, Clone)]
  #[serde(rename_all = "kebab-case")]
  pub enum Visibility {
    Foreground,
    Visible,
    Background,
    Hidden,
  }

  impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      let output = match self {
        Visibility::Hidden => "hidden",
        Visibility::Foreground => "foreground",
        Visibility::Background => "background",
        Visibility::Visible => "visible",
      };
      write!(f, "{}", output)
    }
  }

  #[derive(Debug, Deserialize, Serialize, Copy, Clone)]
  #[serde(rename_all = "kebab-case")]
  pub enum Mode {
    Melodic,
    Enharmonic,
    Vagrant,
    Bell,
    Noise,
  }

  #[derive(Debug, Deserialize, Serialize, Copy, Clone)]
  #[serde(rename_all = "kebab-case")]
  pub enum Role {
    Kick,
    Perc,
    Hats,
    Bass,
    Chords,
    Lead,
  }

  impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      let output = match self {
        Role::Kick => "kick",
        Role::Perc => "perc",
        Role::Hats => "hats",
        Role::Bass => "bass",
        Role::Chords => "chords",
        Role::Lead => "lead",
      };
      write!(f, "{}", output)
    }
  }

  #[derive(Debug, Deserialize, Serialize, Copy, Clone)]
  #[serde(rename_all = "kebab-case")]
  pub enum Energy {
    Low,
    Medium,
    High,
  }

  impl fmt::Display for Energy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      let output = match self {
        Energy::Low => "low",
        Energy::Medium => "medium",
        Energy::High => "high",
      };
      write!(f, "{}", output)
    }
  }

  #[derive(Debug, Deserialize, Serialize, Copy, Clone)]
  #[serde(rename_all = "kebab-case")]
  pub enum Presence {
    Staccatto,
    Legato,
    Tenuto,
  }

  impl fmt::Display for Presence {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      let output = match self {
        Presence::Staccatto => "staccatto",
        Presence::Legato => "legato",
        Presence::Tenuto => "tenuto",
      };
      write!(f, "{}", output)
    }
  }
}
