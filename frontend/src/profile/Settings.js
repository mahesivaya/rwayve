import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useEffect, useState } from "react";
import "./profile.css";
import { apiFetch } from "../api/client";
export default function Settings() {
    const [accounts, setAccounts] = useState([]);
    const [loaded, setLoaded] = useState(false);
    const fetchAccounts = async () => {
        try {
            const res = await apiFetch("/api/accounts");
            const data = await res.json();
            setAccounts(data);
        }
        finally {
            setLoaded(true);
        }
    };
    useEffect(() => {
        void fetchAccounts();
    }, []);
    const remove = async (id, email) => {
        if (!confirm(`Disconnect ${email}? Synced messages will be removed.`)) {
            return;
        }
        try {
            await apiFetch(`/api/accounts/${id}`, {
                method: "DELETE",
            });
            setAccounts((prev) => prev.filter((a) => a.id !== id));
        }
        catch {
            alert("Failed to remove account");
        }
    };
    return (_jsx("div", { className: "settings-page", children: _jsxs("div", { className: "settings-card", children: [_jsx("h2", { className: "settings-title", children: "Settings & Privacy" }), _jsx("div", { className: "settings-section-title", children: "Connected email accounts" }), !loaded ? (_jsx("div", { className: "settings-empty", children: "Loading\u2026" })) : accounts.length === 0 ? (_jsx("div", { className: "settings-empty", children: "No email accounts connected." })) : (_jsx("div", { className: "settings-list", children: accounts.map((acc) => (_jsxs("div", { className: "settings-account", children: [_jsx("span", { className: "settings-account-icon", children: "\uD83D\uDCE7" }), _jsx("span", { className: "settings-account-email", title: acc.email, children: acc.email }), _jsx("button", { className: "settings-account-delete", onClick: () => void remove(acc.id, acc.email), children: "Remove" })] }, acc.id))) }))] }) }));
}
