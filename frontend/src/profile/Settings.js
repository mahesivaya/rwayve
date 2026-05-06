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
export default function Settings() {
    const [accounts, setAccounts] = useState([]);
    const [loaded, setLoaded] = useState(false);
    const fetchAccounts = async () => {
        const res = await fetch(`${API_BASE}/api/accounts`, { headers: authHeaders() });
        if (!res.ok) {
            setLoaded(true);
            return;
        }
        const data = await res.json();
        setAccounts(data);
        setLoaded(true);
    };
    useEffect(() => {
        fetchAccounts();
    }, []);
    const remove = async (id, email) => {
        if (!confirm(`Disconnect ${email}? Synced messages will be removed.`))
            return;
        const res = await fetch(`${API_BASE}/api/accounts/${id}`, {
            method: "DELETE",
            headers: authHeaders(),
        });
        if (res.ok) {
            setAccounts((prev) => prev.filter((a) => a.id !== id));
        }
        else {
            alert("Failed to remove account");
        }
    };
    return (_jsx("div", { className: "settings-page", children: _jsxs("div", { className: "settings-card", children: [_jsx("h2", { className: "settings-title", children: "Settings & Privacy" }), _jsx("div", { className: "settings-section-title", children: "Connected email accounts" }), !loaded ? (_jsx("div", { className: "settings-empty", children: "Loading\u2026" })) : accounts.length === 0 ? (_jsx("div", { className: "settings-empty", children: "No email accounts connected." })) : (_jsx("div", { className: "settings-list", children: accounts.map((acc) => (_jsxs("div", { className: "settings-account", children: [_jsx("span", { className: "settings-account-icon", children: "\uD83D\uDCE7" }), _jsx("span", { className: "settings-account-email", title: acc.email, children: acc.email }), _jsx("button", { className: "settings-account-delete", onClick: () => remove(acc.id, acc.email), children: "Remove" })] }, acc.id))) }))] }) }));
}
