const LEVEL_ORDER = {
    debug: 10,
    info: 20,
    warn: 30,
    error: 40,
};
const isDev = (() => {
    try {
        return Boolean(import.meta.env?.DEV);
    }
    catch {
        return false;
    }
})();
const minLevel = isDev ? "debug" : "warn";
// Ring buffer so devtools can inspect the last N log lines — useful when
// chasing a bug and you want to see what preceded the failure.
const BUFFER_LIMIT = 500;
const buffer = [];
function pushBuffer(entry) {
    buffer.push(entry);
    if (buffer.length > BUFFER_LIMIT)
        buffer.shift();
}
function emit(level, scope, args) {
    if (LEVEL_ORDER[level] < LEVEL_ORDER[minLevel])
        return;
    const ts = new Date().toISOString();
    pushBuffer({ ts, level, scope, args });
    const tag = scope ? `[${level.toUpperCase()} ${ts} ${scope}]` : `[${level.toUpperCase()} ${ts}]`;
    const fn = level === "error" ? console.error
        : level === "warn" ? console.warn
            : level === "debug" ? console.debug
                : console.log;
    fn(tag, ...args);
}
function make(scope) {
    return {
        debug: (...args) => emit("debug", scope, args),
        log: (...args) => emit("info", scope, args),
        info: (...args) => emit("info", scope, args),
        warn: (...args) => emit("warn", scope, args),
        error: (...args) => emit("error", scope, args),
        event: (name, data) => emit("info", scope, data === undefined ? [`• ${name}`] : [`• ${name}`, data]),
        time: async (label, fn) => {
            const start = performance.now();
            try {
                const out = await fn();
                const ms = Math.round(performance.now() - start);
                emit("debug", scope, [`⏱ ${label} ok in ${ms}ms`]);
                return out;
            }
            catch (err) {
                const ms = Math.round(performance.now() - start);
                emit("warn", scope, [`⏱ ${label} failed in ${ms}ms`, err]);
                throw err;
            }
        },
        scope: (name) => make(scope ? `${scope}.${name}` : name),
    };
}
export const logger = make();
/** Snapshot of the most recent log entries. Newest last. */
export const getRecentLogs = () => buffer.slice();
if (isDev && typeof window !== "undefined") {
    window.__log = {
        recent: getRecentLogs,
        logger,
    };
}
