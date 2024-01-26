
export type Q = 0 | 1

export type Center = [Rotation, Q]

export type Size
 = -4
 | -3
 | -2
 | -1
 | 0
 | 1
 | 2
 | 3
 | 4
 | 5
 | 6

export type Division = Size


export type Motion 
  = 0
  | 1
  | 2
  | 3
  | 4 
  | 5
  | 6

export type Cpc 
  = 1
  | 2
  | 3
  | 4 
  | 5
  | 6
  | 7
  | 8
  | 9 
  | 10
  | 11 
  | 12

export type Beat
  = 'kick' 
  | 'perc' 
  | 'hats' 
  

export type Inst
  = 'lead' 
  | 'bass' 
  | 'chords'

export type Role 
  = Beat
  | Inst


export type OscType
  = "sine" 
  | "pulse" 
  | "saw" 
  | "triangle"
  | "noise"
  | "kick"
  | "perc"
  | "hats"

type Label = string 
export type Conf = {
  cps: number
  root: number
}

export type Rotation = -32 | -31 | -30 | -29 | -28 | -27 | -26 | -25 | -24 | -23 | -22 | -21 | -20 | -19 | -18 | -17 | -16 | -15 | -14 | -13 | -12 | -11 | -10 | -9 | -8 | -7 | -6 | -5 | -4 | -3 | -2 | -1 | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 30 | 31
export type HarmonicIndex = -7 | -6 | -5 | -4 | -3 | -2 | -1 | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7
export type Limit = 1 | 3 | 5 | 7 | 9
export type Monic = 1 | 3 | 5 | 7 | 9
export type Reach = 0 | 1 | 2 | 3
export type Sway
  = -1 | 0 | 1
export type Depth
  = 0 | 1 | 2 | 3 | 4 | 5


export type Voice = 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 
export type Register = 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 

export type Root = number
export type Float = number
export type Range = number
export type NonZeroFloat = number

export type PosNonZeroFloat = number
export type PosInt = number 

export type Ratio = [ PosNonZeroFloat, PosNonZeroFloat ] 
export type Step = Ratio
export type Tala = Step[]

export type Cps = PosNonZeroFloat
export type Duration = PosNonZeroFloat
export type Position = Float
export type Freq = PosNonZeroFloat
export type Amp = Range

export type Subdiv
 = 0 | 1 | 2 | 3 | 4 | 5

export type Density
 = "any" | 0 | 1 | 2

 export type Base = number // the base in an exponential function call

 export type Sound = {
  baseOsc: OscType
  duty: Ratio
  minFreq: number
  maxFreq: number
}

export type SCSound = {
  oscType?: OscType
  filepath?: string
  duty: Ratio
  minFreq: number
  maxFreq: number
}

export type Mote = [ Duration, Freq, Amp]

export type Part = {sound: Sound, motes: Array<Mote[]> }
export type SCPart = {sound: SCSound, motes: Array<Mote[]> }
export type Composition = {
  conf: Conf 
  parts: Array<Part>
}

export type Template = {
  conf: Conf
  parts: Array<SCPart>
}