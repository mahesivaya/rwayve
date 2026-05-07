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
function emit(level, scope, args) {
    if (LEVEL_ORDER[level] < LEVEL_ORDER[minLevel])
        return;
    const ts = new Date().toISOString();
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
        scope: (name) => make(scope ? `${scope}.${name}` : name),
    };
}
export const logger = make();
