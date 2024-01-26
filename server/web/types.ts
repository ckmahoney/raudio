
export type Float = number
export type Range = number
export type NonZeroFloat = number

export type PosNonZeroFloat = number
export type PosInt = number 
export type ByteSigned = number // The values 0 to 127
export type Id = string

export type Rotation = -32 | -31 | -30 | -29 | -28 | -27 | -26 | -25 | -24 | -23 | -22 | -21 | -20 | -19 | -18 | -17 | -16 | -15 | -14 | -13 | -12 | -11 | -10 | -9 | -8 | -7 | -6 | -5 | -4 | -3 | -2 | -1 | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 30 | 31
export type HarmonicIndex = -7 | -6 | -5 | -4 | -3 | -2 | -1 | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7
export type Limit = 1 | 3 | 5 | 7 | 9
export type Monic = 1 | 3 | 5 | 7 | 9
export type Q = 0 | 1


export type Cps = PosNonZeroFloat
export type Duration = PosNonZeroFloat
export type Position = Float
export type Freq = PosNonZeroFloat
export type Root = PosNonZeroFloat
export type Amp = Range
export type Volume = Range
export type Vel = ByteSigned
export type MidiVal = PosInt
export type SignedByte = PosInt
export type Ratio = [ PosNonZeroFloat, PosNonZeroFloat]
export type Radian = number
export type Phase = Radian
export type Size = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7

export type Midi  = [ Duration, MidiVal, SignedByte ]
export type MidiNote = [ Step, MidiVal | MidiVal[], Vel ]

export type Step = Duration
export type Tala = Step[]
export type Role = 'kick' | 'perc' | 'hats' | 'bass' | 'chords' | 'lead';
export type Voice = PosInt;
export type Part = {
  role: Role;
  voice: Voice;
  span?: number;
};

export type Note = [Duration, MidiVal, Vel];



export type TuningParams = {
    monic: Monic 
    rotation: Rotation 
    q: Q
    root: Root
}

export type Height = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 

export type Thresh = {
    low: Height
    lowmid: Height;
    highmid: Height
    high: Height
  };
  
  export type SpectralPoint = [Ratio, Amp, Phase];

export type Stack<T> = Array<T>
export type Sequence<T> = Array<T>
export type SpectralLayer = Stack<SpectralPoint>
export type SpectralExpression = Sequence<SpectralLayer>
  
  export type WithMonae = (
    rotation: Rotation,
    q: Q,
    monic: Monic
  ) => SpectralPoint[]

  export type LayerFunction = (
    mix: number,
    lifespan: number
  ) => WithMonae;
  
  export type SpectralSynthParams = {
    thresh: Thresh
    samplesPerCycle: PosInt
  };
  
  export type SpectralSynthConfig = {
    params: SpectralSynthParams;
    layerFunctions: LayerFunction[];
  };
  