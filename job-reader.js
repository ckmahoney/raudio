"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const client_1 = require("../node_modules/.prisma/client");
const path_1 = __importDefault(require("path"));
const axios_1 = __importDefault(require("axios"));
const log = __importStar(require("./log"));
const configuration = __importStar(require("./conf"));
const task = __importStar(require("./sc-task"));
const prisma = new client_1.PrismaClient();
const OUT_DIR = "./out";
function recUri(name) {
    return "/out/" + name;
}
const spawnState = {
    jobId: null,
    tid: null,
    lastRefresh: new Date()
};
const maxRenderTimeSeconds = 50;
async function convertAudio(performanceId, uri) {
    let created = await prisma.recording.create({
        data: {
            name: uri,
            uri: uri + ".mp3",
            performanceId,
            label: "rawMix",
            format: "mp3"
        }
    });
}
function jobCompleted(performanceId, uri) {
    const url = configuration.address.wendy + "/" + uri;
    const status = "satisfied";
    axios_1.default.patch(`${configuration.address.friends}/performance/${performanceId}`, {
        status,
        url
    })
        .then(function patched(response) {
        log.state(`Patched performance ${performanceId} with values`, { status, url });
    })
        .catch(function badResponse(resp) {
        log.error("Error on patch to friends");
        log.state(resp);
    });
}
async function run(taskDoc, job) {
    markStarted(job);
    const { inputFile, outFile } = job;
    const hiResUri = recUri(taskDoc.name) + ".aiff";
    let mp3Path;
    try {
        const absOutPath = path_1.default.resolve("./" + hiResUri);
        mp3Path = await task.main(inputFile, absOutPath);
    }
    catch (e) {
        log.error("Caught an error running the sc task");
        log.state(e);
        return "Failed to render audio for composition";
    }
    prisma.recording.create({
        data: {
            uri: hiResUri,
            performanceId: taskDoc.performanceId,
            label: "deterministic",
            format: "aiff"
        }
    });
    let created = await prisma.recording.create({
        data: {
            uri: taskDoc.name + ".mp3",
            performanceId: taskDoc.performanceId,
            label: "deterministic",
            format: "mp3"
        }
    });
    console.log("Completed async render at mp3Path " + mp3Path);
    return created;
}
async function getNextJob() {
    let jobDoc;
    try {
        jobDoc = await prisma.job.findFirst({
            where: {
                status: "pending"
            },
            orderBy: {
                createdAt: "desc"
            }
        });
    }
    catch (findJobError) {
        console.log("Error while looking for an unstarted job");
        console.log(findJobError);
    }
    return jobDoc;
}
async function markStarted(job) {
    return await prisma.job.update({
        where: {
            id: job.id
        },
        data: { status: "started" }
    })
        .then(function ready(jobDoc) {
    });
}
async function markFailed(job) {
    return prisma.job.update({
        where: {
            id: job.id
        },
        data: { status: "failed" }
    });
}
async function markSatisfied(job) {
    return prisma.job.update({
        where: {
            id: job.id
        },
        data: { status: "satisfied" }
    });
}
async function cron(interval = 1000) {
    let tid;
    function refresh() {
        spawnState.lastRefresh = new Date();
        spawnState.tid = null;
        tick();
    }
    async function tick() {
        if (spawnState.jobId) {
            let now = new Date();
            let runTime = now.getTime() - spawnState.lastRefresh.getTime() / 1000;
            if (runTime > maxRenderTimeSeconds) {
                let badJob = await prisma.job.findFirst({
                    where: { id: spawnState.jobId }
                });
                markFailed(badJob)
                    .then(() => console.log("Marked job " + badJob.id + " as failed"))
                    .catch(function pErr(err) { console.log("Unexpected prisma error", err); })
                    .finally(refresh);
            }
        }
        if (spawnState.tid) {
            return spawnState.tid = setTimeout(refresh, interval);
        }
        let next = await getNextJob();
        if (!next) {
            return spawnState.tid = setTimeout(refresh, interval);
        }
        spawnState.tid = null;
        let taskDoc;
        try {
            taskDoc = await prisma.renderTask.findFirst({
                where: { jobId: next.id }
            });
        }
        catch (prismaError) {
            log.error("Unexpected prisma connection error");
            log.state(prismaError);
            return spawnState.tid = setTimeout(refresh, interval);
        }
        spawnState.jobId = next.id;
        run(taskDoc, next)
            .then(async function completedRender(result) {
            if (typeof result == 'string')
                return Promise.reject(result);
            return markSatisfied(next).then(function notify(jobDoc) {
                const url = configuration.address.wendy + "/" + result.uri;
                const status = "satisfied";
                console.log("Notification ready to listen at " + url);
                return axios_1.default.patch(`${configuration.address.friends}/performance/${taskDoc.performanceId}`, {
                    status,
                    url
                })
                    .then(function patched(response) {
                    log.state(`Patched performance ${taskDoc.performanceId} with values`, { status, url });
                })
                    .catch(function badResponse(resp) {
                    log.error("Error on patch to friends");
                    log.state(resp);
                });
            });
        })
            .catch(function renderError(err) {
            console.log("Error while rendering");
            console.log(err);
            markFailed(next)
                .then(() => console.log("Marked job " + next.id + " as failed"))
                .catch(function pErr(err) { console.log("Unexpected prisma error", err); });
        })
            .finally(() => { spawnState.jobId = null; tick(); });
    }
    tick();
}
cron();
//# sourceMappingURL=job-reader.js.map