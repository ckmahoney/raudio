export const APP_VERSION = "wendy-0.9.0"

type Locations = {
    faceplate: string
    friends: string
    tinPan: string 
    wendy: string
}
export let address:Locations = {
    faceplate: process.env.FACEPLATE_URL,
    friends: process.env.FRIENDS_URL,
    tinPan: process.env.TIN_PAN_URL,
    wendy: process.env.WENDY_URL,
}
