import path from 'path';
import fs from 'fs';

function makeMessage(args:any[]):string {
  return args.map((x:any) => x instanceof Error ? x.toString() : JSON.stringify(x)).join(' ')
}

export function state(...args:any[]) {
    const LOGFILE = path.resolve('application.log')
    const logMessage = makeMessage(args);
    const currentDateTime = (new Date()).toUTCString()
  
    const formattedLogMessage = `[${currentDateTime}] STATE ${logMessage}\n`;
    console.log(logMessage)
    fs.appendFile(LOGFILE, formattedLogMessage, { flag: 'a+' }, (err:any) => {
      if (err) {
        console.error('Error writing to log file:', err);
      }
    });
  }

  export function debug(...args:any[]) {
    const LOGFILE = path.resolve('application.log')
    const logMessage = makeMessage(args);
    const currentDateTime = (new Date()).toUTCString()
  
    const formattedLogMessage = `[${currentDateTime}] DEBUG ${logMessage}\n`;
    console.log(logMessage)
    fs.appendFile(LOGFILE, formattedLogMessage, { flag: 'a+' }, (err:any) => {
      if (err) {
        console.error('Error writing to log file:', err);
      }
    });
  }


  export function error(...args:any[]) {
    const LOGFILE = path.resolve('application.log')
    const logMessage = makeMessage(args);
    const currentDateTime = (new Date()).toUTCString()
  
    const formattedLogMessage = `[${currentDateTime}] ERROR ${logMessage}\n`;
    console.log(logMessage)
    fs.appendFile(LOGFILE, formattedLogMessage, { flag: 'a+' }, (err:any) => {
      if (err) {
        console.error('Error writing to log file:', err);
      }
    });
  }