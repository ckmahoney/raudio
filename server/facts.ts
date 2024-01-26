import { OscType, Rotation,  Cpc, Size, Reach, Register, Q, Subdiv, Density,Base } from './types'

import _ from 'lodash'

//# parameters applied to equations
export const qs:Q[] 
  = [ 0, 1 ]

export const limits: number[] 
  = [ 1, 3, 5, 7, 9 ]

export const sways: number[] 
  = [ -1, 0, 1 ]

export const depths: number[] 
  = [ 0, 1, 2, 3 ]

export const sizes:Size[]
  = [ 0, 1, 2, 3, 4, 5 ]

export const bases:Base[]
  = [ 2, 3, 5 ]

export const registers: Register[] 
  = [ 4, 5, 6, 7, 8, 9, 10, 11, 12, 13 ]

  export const rotations
  = _.range(-32, 32) as Rotation[]

export const minRegister = Math.min(...registers) as Register
export const maxRegister = Math.max(...registers) as Register

export const reaches: Reach[]
  = [...new Array(maxRegister - minRegister)].map((_,i) => i ) as Reach[]

//# paramters applied to music generation

export const oscTypes:OscType[]
  = [ "sine", "pulse", "saw", "triangle", "kick", "perc", "hats"]


export const cpcs:Cpc[]
  = [ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12 ]

export const densities:Density[] 
  = [ 0, 1, 2 ]

export const subdivs:Subdiv[] 
  = [ 0, 1, 2, 3, 4, 5 ]
