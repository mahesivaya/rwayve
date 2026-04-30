const API_BASE = import.meta.env.VITE_API_URL;
const getAuthHeaders = () => {
    const token = localStorage.getItem("token");
    return {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
    };
};
// ================= FETCH =================
export const getMeetings = async () => {
    const res = await fetch(`${API_BASE}/api/meetings`, {
        headers: {
            Authorization: `Bearer ${localStorage.getItem("token")}`,
        },
    });
    if (!res.ok) {
        throw new Error(`Fetch failed: ${res.status}`);
    }
    return res.json();
};
// ================= CREATE =================
export const createMeetingApi = async (data) => {
    const res = await fetch(`${API_BASE}/api/meetings`, {
        method: "POST",
        headers: getAuthHeaders(),
        body: JSON.stringify({
            ...data,
            participants: data.participants ?? [], // ✅ safety
        }),
    });
    if (!res.ok) {
        const text = await res.text();
        throw new Error(text || "Create failed");
    }
    return res.json();
};
// ================= UPDATE =================
export const updateMeetingApi = async (id, data) => {
    const res = await fetch(`${API_BASE}/api/meetings/${id}`, {
        method: "PUT",
        headers: getAuthHeaders(),
        body: JSON.stringify({
            ...data,
            participants: data.participants ?? [], // safety
        }),
    });
    if (!res.ok) {
        const text = await res.text();
        throw new Error(text || "Update failed");
    }
    return res.json(); // { message: "Meeting updated" }
};
// ================= DELETE =================
export const deleteMeetingApi = async (id) => {
    const res = await fetch(`${API_BASE}/api/meetings/${id}`, {
        method: "DELETE",
        headers: getAuthHeaders(),
    });
    if (!res.ok) {
        const text = await res.text();
        throw new Error(text || "Delete failed");
    }
    return res.json(); // { message: "Meeting deleted" }
};
