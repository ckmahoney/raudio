const { describe, expect, it } = require('@jest/globals')

const fs = require('fs')
const path = require("path")
function headers() {
    return {
        "Accept": "application/json",
        "Content-Type": "application/json",
    }
}

describe("Rendering Performance", () => {
    describe("Create Job", () => {
        let jobId  = -1;

        it("endpoint returns a created job response on success", async () => {
            return await fetch("http://localhost:5000/render/", {
                method: "POST",
                headers: headers(),
                body: fs.readFileSync(path.resolve('./test/data/test-composition.json'))
            })
            .then(async function checkResponse(res) {
                expect(res.status).toBe(201)
                return await res.json()
            })
            .then(function checkResponse(response) {
                expect(typeof response).toBe("object")
                expect(typeof response.created).toBe("object")
                expect(response.created.resource).toBe("job-wendy-render-raw")
                expect(typeof response.created.id).toBe("number")
                jobId = response.created.id
            })
        })

        it("patch endpoint updates job status to failed", async () => {
            let body = {status: "failed"}
            return await fetch("http://localhost:5000/job/"+jobId, {
                method: "PATCH",
                headers: headers(),
                body: JSON.stringify(body)
            })
            .then(async function checkResponse(res) {
                expect(res.status).toBe(200)
                return await res.json()
            })
            .then(function checkResponse(response) {
                expect(typeof response).toBe("object")
                expect(typeof response.updated).toBe("object")
                expect(response.updated.resource).toBe("job-wendy-render-raw")
                expect(typeof response.updated.id).toBe("number")
                expect(jobId).toBe(response.updated.id)
                return response.updated
            })
            .then(function compareGet(updated) {
                let actual = fetch("http://localhost:5000/job/"+jobId, {
                    headers: headers(),
                })
                .then(async function gotJobDoc(response) {
                    return await response.json()
                })
                .then(function lookDoc(doc) {
                    expect(body.status).toBe(doc.status)
                })
            })
        })

        it("patch endpoint updates job status to satisfied", async () => {
            let body = {status: "satisfied"}
            return await fetch("http://localhost:5000/job/"+jobId, {
                method: "PATCH",
                headers: headers(),
                body: JSON.stringify(body)
            })
            .then(async function checkResponse(res) {
                expect(res.status).toBe(200)
                return await res.json()
            })
            .then(function checkResponse(response) {
                expect(typeof response).toBe("object")
                expect(typeof response.updated).toBe("object")
                expect(response.updated.resource).toBe("job-wendy-render-raw")
                expect(typeof response.updated.id).toBe("number")
                expect(jobId).toBe(response.updated.id)
            })
            .then(function compareGet(updated) {
                let actual = fetch("http://localhost:5000/job/"+jobId, {
                    headers: headers(),
                })
                .then(async function gotJobDoc(response) {
                    return await response.json()
                })
                .then(function lookDoc(doc) {
                    expect(body.status).toBe(doc.status)
                })
            })
        })

        it("delete endpoint returns a boolean result", async () => {
            return await fetch("http://localhost:5000/job/"+jobId, {
                headers: headers(),
                method: "DELETE",
            })
            .then(async function checkResponse(res) {
                expect(res.status).toBe(204)
            })
        })

    })


    // describe("Process Job", () => {
    //     it("checks for pending jobs", () => {
        
    //     })
    //     it("calls a child process to write audio", () => {
        
    //     })
    //     it("updates the record in the job queue db on job complete", () => {

    //     })
    
    //     it("sends an http request to friends with job payload", () => {
    
    //     })
    // })
})