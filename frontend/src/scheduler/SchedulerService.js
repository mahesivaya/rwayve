import { apiFetch } from "../api/client";
const browserTz = () => Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC";
// ================= FETCH =================
export const getMeetings = async () => {
    const res = await apiFetch(`/api/meetings`);
    return res.json();
};
// ================= CREATE =================
export const createMeetingApi = async (data) => {
    const res = await apiFetch(`/api/meetings`, {
        method: "POST",
        body: JSON.stringify({
            ...data,
            participants: data.participants ?? [], // ✅ safety
            tz: browserTz(),
        }),
    });
    return res.json();
};
// ================= UPDATE =================
export const updateMeetingApi = async (id, data) => {
    const res = await apiFetch(`/api/meetings/${id}`, {
        method: "PUT",
        body: JSON.stringify({
            ...data,
            participants: data.participants ?? [], // safety
            tz: browserTz(),
        }),
    });
    return res.json(); // { message: "Meeting updated" }
};
// ================= DELETE =================
export const deleteMeetingApi = async (id) => {
    const res = await apiFetch(`/api/meetings/${id}`, {
        method: "DELETE"
    });
    return res.json(); // { message: "Meeting deleted" }
};
