export const logger = {
    log: (...args: any[]) => {
      if (import.meta.env.DEV) {
        console.log(`[LOG ${new Date().toISOString()}]`, ...args);
      }
    },
  
    error: (...args: any[]) => {
      if (import.meta.env.DEV) {
        console.error(`[ERROR ${new Date().toISOString()}]`, ...args);
      }
    },
      
    debug: (...args: any[]) => {
      if (import.meta.env.DEV) {
        console.debug(`[DEBUG ${new Date().toISOString()}]`, ...args);
      }
    }
  };