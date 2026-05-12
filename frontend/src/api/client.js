import { API_BASE } from "../config/env";
export async function apiFetch(path, options = {}) {
    const { auth = true, headers, ...rest } = options;
    const token = localStorage.getItem("token");
    const response = await fetch(`${API_BASE}${path}`, {
        ...rest,
        headers: {
            "Content-Type": "application/json",
            ...(auth && token
                ? { Authorization: `Bearer ${token}` }
                : {}),
            ...headers,
        },
    });
    if (response.status === 401) {
        const isChangePassword = path.includes("/profile/password");
        let bodyMessage = "";
        try {
            const data = await response.clone().json();
            bodyMessage = data?.error || data?.message || "";
        }
        catch { }
        if (isChangePassword && bodyMessage) {
            throw new Error(bodyMessage);
        }
        console.error("Unauthorized");
        localStorage.removeItem("token");
        if (import.meta.env.MODE !== "test") {
            window.location.href = "/login";
        }
        throw new Error("Unauthorized");
    }
    if (!response.ok) {
        let message = "Request failed";
        try {
            const data = await response.clone().json();
            message = data.error || data.message || message;
        }
        catch {
            // ignore json parse errors
        }
        throw new Error(message);
    }
    return response;
}
