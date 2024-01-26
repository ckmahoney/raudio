import { Response } from 'express'

export type JobStatus = "pending" | "failed" | "satisfied"
type Resource = "job-wendy-render-raw" 
export type CreateResponse<R extends Resource> = { created: {resource: R, id: number}}
export type UpdateResponse<R extends Resource> = { updated: {resource: R, id: number}}
export type Id<R extends Resource> = number
export type AsyncError<T> = T | {error: string, code?:number}
export type Error = {error: string}
export type Res<V> = Response<AsyncError<V>>
export type NoId<V> = V & { id?: any; userId?: any  }
