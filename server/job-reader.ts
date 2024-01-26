import type { Composition, Part } from './types'

import { PrismaClient } from '../node_modules/.prisma/client'
import Pri from '../node_modules/.prisma/client'

import path from 'path'
import axios from 'axios'
import express from 'express' 
import * as check from './checks'
import * as log from './log'
import * as configuration from './conf'

import * as task from './sc-task'
import { v4 as uuidv4 } from 'uuid';

const prisma = new PrismaClient()


const OUT_DIR = "/mnt/v0.9.2"

function recUri(name:string) {
  return OUT_DIR + "/" + name
}


const spawnState:any = {
  jobId: null,
  tid: null,
  lastRefresh: new Date()
}

const maxRenderTimeSeconds = 50

async function convertAudio(performanceId:number, uri: string) {

  let created = await prisma.recording.create({
    data: {
      name:uri,
      uri: uri + ".mp3",
      performanceId,
      label: "rawMix",
      format: "mp3"
    }
  })
}

function jobCompleted(performanceId:number, uri:string) {
  const url = configuration.address.wendy + "/" + uri
  const status = "satisfied"
  axios.patch(`${configuration.address.friends}/performance/${performanceId}`, {
    status,
    url
  })
  .then(function patched(response:any) {
    log.state(`Patched performance ${performanceId} with values`, {status, url})
  })
  .catch(function badResponse(resp:any) {
    log.error("Error on patch to friends")
    log.state(resp)
  })
}

async function run(taskDoc:Pri.RenderTask, job:Pri.Job):Promise<string|Pri.Recording> {
    markStarted(job)
    const { inputFile, outFile } = job 
    const hiResUri = recUri(taskDoc.name) + ".aiff"
    
    let mp3Path
    try {
        const absOutPath = path.resolve("./" + hiResUri)
        mp3Path = await task.main(inputFile, absOutPath)
    } catch (e) {
        log.error("Caught an error running the sc task")
        log.state(e)
        return "Failed to render audio for composition"
    }
    
    prisma.recording.create({
      data: {
        uri: hiResUri,
        performanceId: taskDoc.performanceId,
        label: "deterministic",
        format: "aiff"
      }
    })

    let created = await prisma.recording.create({
      data: {
        uri: taskDoc.name + ".mp3",
        performanceId: taskDoc.performanceId,
        label: "deterministic",
        format: "mp3"
      }
    })

    console.log("Completed async render at mp3Path " + mp3Path)
    return created
}

async function getNextJob() {
  let jobDoc 
  try {
    jobDoc = await prisma.job.findFirst({
      where: {
        status: "pending"
      },
      orderBy: {
        createdAt: "desc"
      }
    })
  } catch (findJobError) {
    console.log("Error while looking for an unstarted job")
    console.log(findJobError)
  }

  return jobDoc
}

async function markStarted(job:Pri.Job) {
  return await prisma.job.update({
    where: {
      id: job.id
    },
    data: { status: "started" }
  })
  .then(function ready(jobDoc) {

  })
}

async function markFailed(job:Pri.Job) {
  return prisma.job.update({
    where: {
      id: job.id
    },
    data: { status: "failed" }
  })
}

async function markSatisfied(job:Pri.Job) {
  return prisma.job.update({
    where: {
      id: job.id
    },
    data: { status: "satisfied" }
  })
}


async function cron(interval:number = 1000) {
  let tid:any

  function refresh() {
    spawnState.lastRefresh = new Date()
    spawnState.tid = null
    tick()
  }
  
  async function tick():Promise<any> {
      if (spawnState.jobId) {

        let now = new Date() 
        let runTime = now.getTime() - spawnState.lastRefresh.getTime() / 1000
        if (runTime > maxRenderTimeSeconds) {
          let badJob = await prisma.job.findFirst({
            where: { id: spawnState.jobId }
          })
          markFailed(badJob)
          .then(() => console.log("Marked job " + badJob.id + " as failed"))
          .catch(function pErr(err) { console.log("Unexpected prisma error", err) })
          .finally(refresh)
        }
      }
      if (spawnState.tid) {
        return spawnState.tid = setTimeout(refresh, interval)
      } 

      let next = await getNextJob()
      if (!next) {
        return spawnState.tid = setTimeout(refresh, interval)
      }

      spawnState.tid = null
      let taskDoc:Pri.RenderTask
      try {
        taskDoc = await prisma.renderTask.findFirst({
          where: { jobId: next.id }
        })
      } catch (prismaError) {
        log.error("Unexpected prisma connection error")
        log.state(prismaError)
        return spawnState.tid = setTimeout(refresh, interval)
      }

      spawnState.jobId = next.id

      run(taskDoc, next)
      .then(async function completedRender(result:string|Pri.Recording) {
          if (typeof result == 'string') return Promise.reject(result)
          
          return markSatisfied(next).then(function notify(jobDoc) {
              return jobCompleted(taskDoc.performanceId, result.uri)
              
          })
      })
      .catch(function renderError(err) {
          console.log("Error while rendering")
          console.log(err)
          markFailed(next)
          .then(() => console.log("Marked job " + next.id + " as failed"))
          .catch(function pErr(err) { console.log("Unexpected prisma error", err) })
      })
      .finally(() => { spawnState.jobId = null; tick() })
  }

  tick()

}


cron()