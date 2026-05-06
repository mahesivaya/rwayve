import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useEffect, useState } from "react";
import "./profile.css";
const API_BASE = import.meta.env.VITE_API_URL;
const authHeaders = () => {
    const token = localStorage.getItem("token");
    return {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
    };
};
export default function Profile() {
    const [profile, setProfile] = useState(null);
    const [firstName, setFirstName] = useState("");
    const [lastName, setLastName] = useState("");
    const [saving, setSaving] = useState(false);
    const [status, setStatus] = useState(null);
    useEffect(() => {
        const load = async () => {
            const res = await fetch(`${API_BASE}/api/profile`, { headers: authHeaders() });
            if (!res.ok)
                return;
            const data = await res.json();
            setProfile(data);
            setFirstName(data.first_name ?? "");
            setLastName(data.last_name ?? "");
        };
        load();
    }, []);
    useEffect(() => {
        if (!status)
            return;
        const t = setTimeout(() => setStatus(null), 2000);
        return () => clearTimeout(t);
    }, [status]);
    const save = async () => {
        setSaving(true);
        try {
            const res = await fetch(`${API_BASE}/api/profile`, {
                method: "PUT",
                headers: authHeaders(),
                body: JSON.stringify({ first_name: firstName, last_name: lastName }),
            });
            if (!res.ok)
                throw new Error(await res.text());
            const data = await res.json();
            setProfile(data);
            setStatus("Saved ✓");
        }
        catch {
            setStatus("Save failed");
        }
        finally {
            setSaving(false);
        }
    };
    if (!profile) {
        return (_jsx("div", { className: "profile-page", children: _jsx("div", { className: "profile-loading", children: "Loading\u2026" }) }));
    }
    return (_jsx("div", { className: "profile-page", children: _jsxs("div", { className: "profile-card", children: [_jsx("h2", { className: "profile-title", children: "My Profile" }), _jsxs("div", { className: "profile-row", children: [_jsx("label", { children: "Username" }), _jsx("div", { className: "profile-readonly", children: profile.email })] }), _jsxs("div", { className: "profile-row", children: [_jsx("label", { htmlFor: "profile-first", children: "First name" }), _jsx("input", { id: "profile-first", value: firstName, onChange: (e) => setFirstName(e.target.value), placeholder: "First name" })] }), _jsxs("div", { className: "profile-row", children: [_jsx("label", { htmlFor: "profile-last", children: "Last name" }), _jsx("input", { id: "profile-last", value: lastName, onChange: (e) => setLastName(e.target.value), placeholder: "Last name" })] }), _jsxs("div", { className: "profile-actions", children: [_jsx("button", { className: "profile-save", onClick: save, disabled: saving, children: saving ? "Saving…" : "Save" }), status && _jsx("span", { className: "profile-status", children: status })] })] }) }));
}
