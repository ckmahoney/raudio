import type { Composition, Template, Part,Sound, SCPart, OscType } from './types'

import fs from 'fs'
import child_process from 'node:child_process'
import path from 'path'

import * as log from './log'
import { v4 as uuidv4 } from 'uuid';
const TMP_DIR = "/tmp"
function randFrom(arr:any[]) {
    return arr[Math.floor((Math.random()*arr.length))];
  }
  
function pickSample(type:"kick"|"perc"|"hats") {
    let kind = randFrom(["long", "short"])
    let place = path.resolve(`./samples/${type}/${kind}`)
    let files = fs.readdirSync(place)
    return place + "/" + randFrom(files)
}
  
function updateTemplate(composition:Composition):Template {
    let parts = composition.parts.map((p:Part) => {
        let type = p.sound.baseOsc
        let next:any = {...p.sound}
        delete next.baseOsc
        if ((["kick", "perc", "hats"]).includes(type)) {
            next["filepath"] = pickSample(type as "kick"|"perc"|"hats")
        } else {
            if (type == "noise") {
                type = "pulse"
            }
            next["osc"] = type
        }
        return {...p, sound:next}
    })
    return {conf: composition.conf, parts}
}

function cleanupTmpTemplate(filepath:string) {
    if (!filepath.endsWith(".json")) {
        log.error("Attempted to delete a non tmp file at path " + filepath)
        return;
    }
    try {
        fs.rmSync(filepath)
    } catch (e) {
        log.error("Error attempting to remove file")
        log.state({filepath})
        log.state(e)
    }
}
function cleanupHiResFile(filepath:string) {
    if (!filepath.endsWith(".aiff")) {
        log.error("Attempted to delete a non aiff file at path " + filepath)
        return;
    }
    try {
        fs.rmSync(filepath)
    } catch (e) {
        log.error("Error attempting to remove file")
        log.state({filepath})
        log.state(e)
    }
}

export function makeTmpTemplate(composition:Composition):string|{error:string} {
    const filepath = path.resolve(TMP_DIR, uuidv4() + ".json")
    let updated = updateTemplate(composition)
    try {
        fs.writeFileSync(filepath, JSON.stringify(updated))
    } catch (e) {
        log.error("Unexpected error saving template")
        log.state({filepath, updated})
        return {error: "Unable to create template"}
    }
    return filepath
}

function checkIsSuccess(output: any[]) {
    const err = parseErrorOutput(output)
    if (err) { throw new Error(err) }

    if (output.length ==0 ) {
        throw new Error("No results to read")
    }
  
    const delim = "xsynx"
    const data = output.map((x: { toString: () => any })=>x.toString())
    const lines = data.filter((val: string | string[],i: any) => {
      return val.includes(delim)
    })
    const msg = lines[0].split(delim)[1]
    return msg == "completed"
}
  

function hasError(msg:string):boolean {
    const search = "xsynxerror"
    return msg.includes(search)
}

function parseErrorOutput(output: any[]) {
    const search = "xsynxerror"
    const data = output.map((x: { toString: () => any })=>x.toString())
    const lines = data.filter((val: string | string[],i: any) => {
      return val.includes(search)
    })
    const msg = lines.map((l: string) => l.split(search)[1]).join(", ")
    return msg
}


async function convertAudio(src:string, to:string):Promise<number|string> {
    const search = path.extname(src)
    const outPath = src.replace(search, to)
    let args = [src, outPath]
    const proc = child_process.spawn("lame", args)  
    log.state("sclang ", args)
    return new Promise((resolve, reject) => {
        // proc.stderr.on("data", (data: { toString: () => string }) => { log.state("lame error: " + data.toString()) })
        proc.stdout.on("data", (data: { toString: () => string }) => { log.state("lame info: " + data.toString()) })
        proc.on('close', (code: number) => {
            if (code == 0) {
                resolve(outPath)
            } else {
                log.error("Unexpected exit code from lame")
                log.state(code)
                reject(code)
            }
        })
        const maxRunTime = 30
        setTimeout(function timeoutProc() {
            if (proc.connected) {
                log.state(`No response from LAME transcoder and exeeded the ${maxRunTime} time limit. Closing it now. LAME pid: ${proc.pid}`)
                proc.kill(9)
            }
        }, 1000 * maxRunTime)
    })
}

export async function main(filepath:string, outpath:string):Promise<string> {
    // note, this is relative to the entry point at "RUNTIME", not "COMPILETIME"
    const script = path.resolve(__dirname, "./nrt.sc")
    let args = [script, filepath, outpath]
    let buff:any[] =[]
    let completed = false
    log.state("sclang ", args)

    const proc = child_process.spawn("sclang", args)

    proc.on('error', (e:any) => {
        log.state("Failed to start a subprocess")
        log.error(e)
        return 
    })

    return new Promise((resolve, reject) => {
        function res(x:any) {
            if (!completed) { completed = true; resolve(x) }
            else { log.state("Attempted to resolve a completed promise")}
        }
        function rej(x:any) {
            if (!completed) { completed = true; reject(x) }
            else { log.state("Attempted to reject a completed promise")}
        }

        proc.stdout.on('data', (data: any) => {
            // log.state("sclang any data ", data.toString())
            buff.push(data)
        })
        proc.stderr.on('data', (data: any) => {
            // log.state("sclang err data ", data.toString())
            if (hasError(data.toString())) {
                log.error("reporting error from sclang")
                log.state(data.toString())
            }
        })

        proc.on('close', async (code: number) => {
            // log.state("closed sclang",code)
            if (code == 0 || code == null) {
                const results = checkIsSuccess(buff)
                let mp3Path:any
                try { 
                    log.state("mp3 conversion on " + outpath)
                    mp3Path = await convertAudio(outpath, ".mp3")
                } catch (transcodingError) {
                    log.error("Unexpected error during conversion")
                    log.state(transcodingError)
                    rej("Unexpected lame error")
                } finally {
                    log.state("removing temporary assets")
                    cleanupTmpTemplate(filepath)
                    cleanupHiResFile(outpath)
                }
                res(mp3Path)
            } else {
                const err = parseErrorOutput(buff)
                rej(err)
            }
        })

        const maxRunTime = 30
        setTimeout(function timeoutProc() {
            if (proc.connected) {
                log.state(`No response from process, exeeded the ${maxRunTime} time limit. Closing it now. id: ${proc.pid}`)
                proc.kill(9)
            }
        }, 1000 * maxRunTime)
    })
}