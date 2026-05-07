import { logger } from "./utils/logger";
const log = logger.scope("api");
const API_BASE = import.meta.env.VITE_API_URL;
export const apiFetch = async (endpoint, options = {}) => {
    const token = localStorage.getItem("token");
    const method = options.method || "GET";
    log.debug(`→ ${method} ${endpoint}`);
    const start = performance.now();
    let res;
    try {
        res = await fetch(`${API_BASE}${endpoint}`, {
            ...options,
            headers: {
                "Content-Type": "application/json",
                Authorization: `Bearer ${token}`,
                ...(options.headers || {}),
            },
        });
    }
    catch (err) {
        log.error(`✗ ${method} ${endpoint} (network)`, err);
        throw err;
    }
    const ms = Math.round(performance.now() - start);
    if (res.status === 401) {
        log.warn(`← ${method} ${endpoint} 401 in ${ms}ms — clearing token`);
        localStorage.removeItem("token");
        window.location.href = "/login";
        throw new Error("Unauthorized");
    }
    if (!res.ok) {
        log.warn(`← ${method} ${endpoint} ${res.status} in ${ms}ms`);
    }
    else {
        log.debug(`← ${method} ${endpoint} ${res.status} in ${ms}ms`);
    }
    return res;
};
