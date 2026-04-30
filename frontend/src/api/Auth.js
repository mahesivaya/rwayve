import { logger } from "../utils/logger";
export async function register(email, password, confirm) {
    const res = await fetch("http://localhost:8080/api/register", {
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
    // 🔥 IMPORTANT: read response
    const data = await res.json();
    // 🔥 handle backend errors
    if (!res.ok) {
        throw new Error(data.message || "Register failed");
    }
    return data; // ✅ THIS FIXES YOUR BUG
}
export async function login(email, password) {
    try {
        const res = await fetch("http://localhost:8080/api/login", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ email, password }),
        });
        const text = await res.text();
        if (!res.ok) {
            throw new Error(`Login failed: ${res.status} ${text}`);
        }
        return text ? JSON.parse(text) : {};
    }
    catch (err) {
        logger.error("login error:", err);
        throw err;
    }
}
