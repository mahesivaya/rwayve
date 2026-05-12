import { API_BASE } from "@/config/env";
export async function apiFetch(path, options = {}) {
    const { auth = true, headers, ...rest } = options;
    const token = localStorage.getItem("token");
    const response = await fetch(`${API_BASE}${path}`, {
        ...rest,
        headers: {
            "Content-Type": "application/json",
            ...(auth && token
                ? {
                    Authorization: `Bearer ${token}`,
                }
                : {}),
            ...headers,
        },
    });
    // Global error handling
    if (response.status === 401) {
        console.error("Unauthorized");
        localStorage.removeItem("token");
        window.location.href = "/login";
        throw new Error("Unauthorized");
    }
    if (!response.ok) {
        let message = "Request failed";
        try {
            const data = await response.json();
            message = data.error || message;
        }
        catch { }
        throw new Error(message);
    }
    return response;
}
