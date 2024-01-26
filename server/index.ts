import type { Res, Id, CreateResponse, UpdateResponse } from './api.types'

import { PrismaClient } from '../node_modules/.prisma/client'
import Pri from '../node_modules/.prisma/client'

import express from 'express' 
import * as check from './checks'
import * as log from './log'
import * as configuration from './conf'

import * as task from './sc-task'
import { v4 as uuidv4 } from 'uuid';

type Err = {code:number,error:string}
type R<T> = Err|T

const prisma = new PrismaClient()
const APP_PORT = process.env.APP_PORT
  ? parseInt(process.env.APP_PORT)
  : 5000

const app = express()
const MOUNT_PATH = "/mnt/v0.9.2/"

app.use(express.json({limit: '50mb'}))
app.use(express.static("out"))

function recUri(name:string) {
  return MOUNT_PATH + name
}

app.get("/download/performance/:performanceId", async (req:any, res:Res<string>) => {
  let performanceId = parseInt(req.params.performanceId)
  let format = req.params.format || "mp3"

  if (!check.is.id(performanceId)) {
    return res.status(400).send({error:"Must provide a performanceId in the path param"})
  }

  let recordingDoc = await prisma.recording.findFirst({
    where: {
      format,
      performanceId
    }
  })
  
  if (!recordingDoc) {
    return res.status(400).send({error:"No recording with that format found for performance " + performanceId})
  }

  recordingDoc = recordingDoc as Pri.Recording
  const filepath = configuration.address.wendy + "/" + recordingDoc.name + "." + format
  return res.send(filepath)
})

app.post("/render/performance/:performanceId", async (req:any, res:Res<CreateResponse<any>>) => {
  let performanceId = parseInt(req.params.performanceId)
  
  if (!check.is.id(performanceId)) {
    return res.status(400).send({error:"Must provide a performanceId in the path param"})
  }
  if (!check.is.composition(req.body)) {
    return res.status(400).send({error:"Must provide a composition in the request body"})
  }
  try {
    const { conf, parts } = req.body
    if (!parts.every(check.is.part)) {
      return res.status(400).send({error: "Must provide a list of wendy formatted parts in the 'parts' field"})
    }

    if (!check.is.conf(conf)) {
      return req.status(400).send({error: "Must provide a conf in the req body"})
    }
    let filename = uuidv4().toString()
    let inputFile
    try {
      inputFile = task.makeTmpTemplate(req.body) as string
    } catch (e) {
      log.error("Error writing template to disk")
      log.state(e)
      res.status(500).send({error:"bad writing"})
    }

    const jobDoc = await prisma.job.create({
      data:  {
        inputFile,
        outFile: filename + ".mp3",
        status: "pending"
      }
    })

    const taskDoc = await prisma.renderTask.create({
      data:  {
        name:filename,
        performanceId,
        jobId: jobDoc.id
      }
    })

    // startJob(filename, inputFile, performanceId, jobDoc)
    return res.status(201).send({
      created: {
        resource: "task-async-render" ,
        id: taskDoc.id
      }
    })
  } catch (e) {
    log.error("Unexpected error creating wendy job")
    log.state(e)
  }
  return res.status(500).send({error:"Unexpected server error while creating job-wendy-render-raw"})
})

app.get("/job/:id", async (req:any, res:Res<Pri.RenderRaw>) => {
  const id = parseInt(req.params.id)
  if (!check.is.id(id)) {
    return res.status(400).send({error:"It must be an int in the path param"})
  }

  let jobDoc = await prisma.renderRaw.findFirst({
    where: {id}
  })

  if (!jobDoc) {
    return res.status(404).send({error: "No job found for id " + id})
  }

  return res.send(jobDoc)
})

app.patch("/job/:id", async (req:any, res:Res<UpdateResponse<"job-wendy-render-raw">>) => {
  const id = parseInt(req.params.id)
  if (!check.is.id(id)) {
    return res.status(400).send({error:"It must be an int in the path param"})
  }

  let jobDoc = await prisma.renderRaw.findFirst({
    where: {id}
  })

  if (!jobDoc) {
    return res.status(404).send({error: "No job found for id " + id})
  }
  try {
    let values:any = {
      updatedAt: new Date()
    }
    if (check.api.jobStatus(req.body.status)) {
      values["status"] = req.body.status
    }

    jobDoc = await prisma.renderRaw.update({
      where: {id},
      data:  values
    })

    return res.send({
      updated: {
        resource: "job-wendy-render-raw" ,
        id: jobDoc.id
      }
    })
  } catch (e) {
    log.error("Unexpected error creating wendy job")
    log.state(e)
  }
  return res.status(500).send({error:"Unexpected server error while creating job-wendy-render-raw"})
})

app.delete("/job/:id", async (req:any, res:Res<true>) => {
  const id = parseInt(req.params.id)
  if (!check.is.id(id)) {
  log.debug("bad id", id)
  return res.status(400).send({error:"It must be an int in the path param"})
  }

  let jobDoc = await prisma.renderRaw.findFirst({
    where: {id}
  })
  log.debug("found job doc for id", id, jobDoc)

  if (!jobDoc) {
    return res.status(404).send({error: "No job found for id " + id})
  }

  try {
    await prisma.renderRaw.delete({
      where: {id}
    })
  } catch(e) {
    log.error("Unexpected error deleting a job")
    log.state(e)
    res.status(500).send({error: "Unexpected error while removing a job record"})
  }
  res.status(204).send(true)
})

//@ts-ignore
app.post('/smoke', (_: Request, res: Response) => {
//@ts-ignore
  res.send("puff")
})

app.listen(APP_PORT, () => {
  log.state("Ready to take job requests on port " + APP_PORT)
})

const actions = {
  "createPerformanceJob":{},
  "deleteJob": {},
  "getJob": {},
  "getPerformanceURL": {},
  "renderPerformance":{},
  "transcodeFile": {},
  "updateJob": {}

}