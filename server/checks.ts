import _ from 'lodash'
import * as facts from './facts'

type Check = (x:any) => boolean
type CheckWith = (y:any) => (x: any) => boolean

export function posInt(x:any) { return  typeof x == 'number' && !isNaN(x) && x % 1 == 0 && x > 0 }
export function posNum(x:any) { return  typeof x == 'number' && !isNaN(x)&& x > 0 }

export function int(x:any) { return typeof x == 'number' && !isNaN(x) && x % 1 == 0 }
export function q(x:any) { return typeof x == 'number' && !isNaN(x) && (x == 0 || x == 1)
}
function range(x:any) { return typeof x == "number" && x <= 1 || x >= 0 }
function string(x:any) { return typeof x == 'string' }
function any(x:any) { return true } 
function isArray(x:any) { return Array.isArray(x) }
function isObject(x:any) { return typeof x == 'object' &&!isArray(x) }
function isEven(x:any) { return int(x) && x % 2 == 0 }
function isOdd(x:any) { return int(x) && x % 2 == 1 }
function maybe(matcher:Check): (x:any) => boolean { return (x:any) => typeof x === "undefined" || matcher(x) }
export let posIntOr0: Check = x => posInt(x) || x == 0
export let oneOf: CheckWith = ys => x => ys.some((y:any) => _.isEqual(x, y))
export let intMoreThan:CheckWith = (y:number) => (x:any) => int(x) && x > y


function mapOf(matcher:Function) {   
  return function matchObjectValue(x:any) {
    return typeof x === "object" 
    && Object.keys(x).length > 0 
    && Object.keys(x).every((k:string) => matcher(x[k]))
  }
}

export function listOf(matcher: (y:any) => boolean): (x:any) => boolean {
  return function matchArray(x:any):boolean {
    return isArray(x) && x.every((y:any) => matcher(y))
    
  }
}

export function functionOf(input:any, matcher:Function): (x:any) => boolean {
  return function matchFunction(x:any): boolean { 
    if (typeof x !== 'function') return false
    let res = x(...input)
    return matcher(res)
  }
}


export const api:any = {
  clientError(x:any) { return apiShapeOf('error')(x) && x.status >= 400 && x.status <= 500 },
  error(x:any) { return shapeOf('error')(x) },
  jobStatus(x:any) { return (["pending", "failed", "satisfied"]).includes(x) },
  serverError(x:any) { return shapeOf('error')(x) && x.status >= 500 },

}


export const apiShapes:any = {
  error(x:any) {
    error: is.string
  }
}


export const is:any = {
  audioHertz(x:any) {
    return posInt(x)
    && x >= 16
    && x <= 22000
  },
  array(x:any) { return isArray(x) },
  arc(x:any) { return shapeOf('arc')(x) },
  air(x:any) { return shapeOf('air')(x) },
  amp(x:any) { return range(x) },
  base(x:any) { return facts.bases.includes(x) },
  blurb(x:any) { return listOf(is.tone)(x) },
  center(x:any) { 
    return isArray(x) 
    && x.length == 2 
    && is.rotation(x[0])
    && is.q(x[1])
  },
  composition(x:any) { return shapeOf('composition')(x) },
  conf(x:any) { return shapeOf('conf')(x) },
  cpc(x:any) {
    return facts.cpcs.includes(x)
  },
  depth(x:any) {
    return int(x)
    && facts.depths.includes(x)
  },
  dimensions(x:any) {
    return shapeOf('dimensions')(x)
  },
  duty(x:any) {
    return is.ratio(x)
  },
  rotation(x:any): boolean { 
    return int(x) && facts.rotations.includes(x)
   },
  gamut(x:any):boolean { return listOf(is.monae)(x) },
  id(x:any) { return posInt(x) },
  monic(x:any) { return isOdd(x) },
  q(x:any) { return q(x) },
  register(x:any) { 
    return oneOf(facts.registers)(x) 
  },
  rhythm(x:any) {
    return functionOf([16], is.tala)(x)
  },
  duration(x:any) { return posInt(x) },
  volume(x:any) { return range(x) },
  volumes(x:any) { return isArray(x) && x.every(is.volume) },
  volumeOrVolumes(x:any) { 
    return is.volume(x)
    || (isArray(x) && x.every(is.volume))
  },
  freq(x:any) { return posNum(x) && x > 0.1 && x < 21000 },
  freqOrFreqs(x:any) { 
    return is.freq(x)
    || (isArray(x) && x.every(is.freq))
  },
  limit(x:any) {
      return int(x)
      && facts.limits.includes(x)
  },
  line(x:any) {
    return is.array(x)
    && x.every(is.mote)
  },
  motes(x:any) {
    return is.array(x)
    && x.every(is.line)
  },
  midi(x:any) {
    return isArray(x)
    && x.length == 3
    && posNum(x[0])
    && is.midival(x[1])
    && is.signedByte(x[2])
  },
  midinote(x:any) {
    return isArray(x)
    && is.step(x[0])
    && is.miditone(x[1])
  },
  miditone(x:any) {
    return isArray(x)
    && x.length == 3
    && is.register(x[0])
    && is.midival(x[1])
    && is.signedByte(x[2])
  },
  midival(x:any) {
    return posInt(x) && x < 122
  },
  moment(x:any) {
    return is.array(x)
    && x.length == 2
    && is.step(x[0])
    && is.place(x[1])
  },
  monae(x:any) {
    return isArray(x)
      && x.length == 3
      && is.rotation(x[0])
      && is.q(x[1])
      && is.monic(x[2])
  },
  mote(x:any) {
    return isArray(x)
    && posNum(x[0])
    && is.freq(x[1])
    && range(x[2])
  },
  moteMelody(x:any) {
    return isArray(x)
    && x.every((y:any) => listOf(is.mote)(y))
  },
  signedByte(x:any) { 
    return posIntOr0(x) && x <128
  },
  size(x:any) {
    return facts.sizes.includes(x)
  },
  sound(x:any) { return shapeOf('sound')(x) },
  sway(x:any) {
    return int(x)
    && facts.sways.includes(x)
  },
  tala(x:any) {
    return isArray(x) &&
    x.every((y:any) => is.step(y))
  },
  tone(x:any) {
    return is.array(x)
    && (x.length == 2)
    && is.register(x[0])
    && is.monae(x[1])
  }, 
  step(x:any) {
    return is.ratio(x)
  },
  voicing(x:any) {
    return isArray(x) 
    && posIntOr0(x[0])
    && isArray(x[1])
    && x[1].every(is.monae)
  },
  ratio(x:any) {
    return isArray(x) 
    && (x.length == 2)
    && posInt(x[0]) 
    && posInt(x[1])
  },
  oscType(x:any) {
    return facts.oscTypes.includes(x)
  },
  part(x:any) {
    return isObject(x)
    && shapeOf('part')(x)
  },
  perigee(x:any) {
    return is.array(x)
    && (x.length == 3)
    && is.limit(x[0])
    && is.depth(x[1])
    && is.sway(x[2])
  },
  place(x:any) {
    return is.array(x) 
    && is.center(x[0])
    && is.perigee(x[1])
  },
  progression(x:any) {
    return is.array(x)
    && x.every(is.moment)
  },
  score(x:any) { return shapeOf('score')(x) },
  midiscore(x:any) { return shapeOf('midiscore')(x) },
  origin(x:any) {
    return isArray(x)
      && x.length == 2
      && int(x[0])
      && q(x[1])
  }
}


export const shapes:any = {
  arc: {
    dimensions: is.dimensions,
    harmony: is.ratio,
    place: is.place,
  },
  composition: {
    conf: is.conf,
    parts: (y:any) => listOf(is.part)(y)
  },
  conf: {
    root: posNum,
    cps: posNum
  },
  dimensions: {
    base: is.base, 
    size: is.size, 
    cpc: is.cpc
  },
  euclidParam: {
    density: posInt,
    subdiv: int
  },
  powerParam: {
    base: intMoreThan(1),
    range: int,
  },
  rhythmParam: {
    type: oneOf(["power", "euclid"]),
    params: (x:any) => 
      shapeOf("powerParam")(x) 
      || shapeOf("euclidParam")(x),
  },
  part: {
    sound: is.sound, 
    motes: is.moteMelody
  },
  score: (x:any) => mapOf(listOf(is.note))(x),
  sound: {
    baseOsc: is.oscType,
    duty: is.duty,
    minFreq: is.audioHertz,
    maxFreq: is.audioHertz
  },
  midiscore: (x:any) => mapOf(listOf(is.midinote))(x)
}

export function errorsOf(shape:string, x:any): string[] {
  let msgs:string[]  = []
  let matcher = shapes[shape]
  for (let field in matcher) {
    if (!matcher[field](x[field])) {
      if (typeof x == 'undefined') { x = "undefined" } 
      if (x == null) { x = "null" } 
      msgs.push(`Failed typecheck for field <${field}> with value <${JSON.stringify(x[field])}>`)
    }
  }
  return msgs
}
  
export type XForm = {
  [label: string]: (x:string) => any
}

export default {
  is
}


export function shapeOf(selector:string): (x:any) => boolean {
  let schema:any = shapes[selector]
  return function matchObject(x:any): boolean {
    if (typeof x !== "object") return false
    let result:boolean = Object.keys(schema).every((k:string) => {
      return schema[k](x[k])
    })
    if (result) return true

    const errs = errorsOf(selector, x).join(', ')
    console.error(`Error matching shape for selector '${selector}'`)
    throw new Error(errs)
  }
}

export function apiShapeOf(selector:string): (x:any) => boolean {
  let schema:any = apiShapes[selector]
  return function matchObject(x:any): boolean {
    if (typeof x !== "object") return false
    let result:boolean = Object.keys(schema).every((k:string) => {
      return schema[k](x[k])
    })
    if (result) return true

    const errs = errorsOf(selector, x).join(', ')
    console.error(`Error matching shape for selector '${selector}'`)
    throw new Error(errs)
  }
}
