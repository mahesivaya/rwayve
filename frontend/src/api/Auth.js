import { logger } from "../utils/logger";
const log = logger.scope("auth");
const API_BASE = import.meta.env.VITE_API_URL;
export async function register(email, password, confirm) {
    log.info("register attempt", { email });
    const res = await fetch(`${API_BASE}/api/register`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            email,
            password,
            confirm_password: confirm,
        }),
    });
    const data = await res.json();
    if (!res.ok) {
        log.warn("register rejected", { email, status: res.status, message: data?.message });
        throw new Error(data.message || "Register failed");
    }
    log.info("register ok", { email });
    return data;
}
export async function login(email, password) {
    log.info("login attempt", { email });
    try {
        const res = await fetch(`${API_BASE}/api/login`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ email, password }),
        });
        const text = await res.text();
        if (!res.ok) {
            log.warn("login rejected", { email, status: res.status });
            throw new Error(`Login failed: ${res.status} ${text}`);
        }
        log.info("login ok", { email });
        return text ? JSON.parse(text) : {};
    }
    catch (err) {
        log.error("login error", err);
        throw err;
    }
}
