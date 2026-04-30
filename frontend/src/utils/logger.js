export const logger = {
    log: (...args) => {
        if (import.meta.env.DEV) {
            console.log(`[LOG ${new Date().toISOString()}]`, ...args);
        }
    },
    error: (...args) => {
        if (import.meta.env.DEV) {
            console.error(`[ERROR ${new Date().toISOString()}]`, ...args);
        }
    },
    debug: (...args) => {
        if (import.meta.env.DEV) {
            console.debug(`[DEBUG ${new Date().toISOString()}]`, ...args);
        }
    }
};
