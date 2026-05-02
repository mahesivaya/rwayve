import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import { useEffect, useRef, useState } from "react";
import "./emails.css";
import "./loadMore.css";
import SendEmail from "./SendEmail";
const API_BASE = import.meta.env.VITE_API_URL;
export default function Emails() {
    const [accounts, setAccounts] = useState([]);
    const [emails, setEmails] = useState([]);
    const [selectedEmail, setSelectedEmail] = useState(null);
    const [activeAccount, setActiveAccount] = useState(null);
    const [activeFolder, setActiveFolder] = useState("inbox");
    const [hasMore, setHasMore] = useState(true);
    const [loadingMore, setLoadingMore] = useState(false);
    const [composeOpen, setComposeOpen] = useState(false);
    const emailCache = useRef({});
    // ================= FETCH ACCOUNTS =================
    const fetchAccounts = async () => {
        const token = localStorage.getItem("token");
        const res = await fetch(`${API_BASE}/api/accounts`, {
            headers: { Authorization: `Bearer ${token}` },
        });
        const data = await res.json();
        setAccounts(data);
    };
    useEffect(() => {
        fetchAccounts();
    }, []);
    // ================= HANDLE OAUTH RETURN =================
    // After /oauth/callback redirects back with ?connected=true, refresh the
    // account list so the newly linked account shows up immediately. The 30s
    // sync worker will import its emails on the next tick.
    useEffect(() => {
        const params = new URLSearchParams(window.location.search);
        if (params.get("connected") === "true") {
            fetchAccounts();
            window.history.replaceState({}, "", "/emails");
        }
    }, []);
    // ================= ADD ACCOUNT =================
    const addAccount = () => {
        const token = localStorage.getItem("token");
        if (!token)
            return;
        window.location.href = `${API_BASE}/gmail/login?token=${encodeURIComponent(token)}`;
    };
    // ================= FETCH EMAILS =================
    useEffect(() => {
        const fetchEmails = async () => {
            const token = localStorage.getItem("token");
            let url = `${API_BASE}/api/emails?folder=${activeFolder}`;
            if (activeAccount !== null) {
                url += `&account_id=${activeAccount}`;
            }
            const res = await fetch(url, {
                headers: { Authorization: `Bearer ${token}` },
            });
            const data = await res.json();
            setEmails(data);
            setHasMore(data.length === 50);
            setSelectedEmail(null);
        };
        fetchEmails();
    }, [activeAccount, activeFolder]);
    // ================= LOAD MORE =================
    const loadMore = async () => {
        if (!hasMore || emails.length === 0)
            return;
        setLoadingMore(true);
        const token = localStorage.getItem("token");
        const last = emails[emails.length - 1];
        const before = Math.floor(new Date(last.created_at).getTime() / 1000);
        const before_id = last.id;
        let url = `${API_BASE}/api/emails?folder=${activeFolder}&before=${before}&before_id=${before_id}`;
        if (activeAccount !== null) {
            url += `&account_id=${activeAccount}`;
        }
        const res = await fetch(url, {
            headers: { Authorization: `Bearer ${token}` },
        });
        const data = await res.json();
        setEmails((prev) => [...prev, ...data]);
        setHasMore(data.length === 50);
        setLoadingMore(false);
    };
    // ================= OPEN EMAIL =================
    const openEmail = async (email) => {
        if (emailCache.current[email.id]) {
            setSelectedEmail(emailCache.current[email.id]);
            return;
        }
        const token = localStorage.getItem("token");
        const res = await fetch(`${API_BASE}/api/emails/${email.id}`, {
            headers: { Authorization: `Bearer ${token}` },
        });
        const data = await res.json();
        emailCache.current[email.id] = data;
        setSelectedEmail(data);
    };
    // ================= UI =================
    return (_jsxs("div", { className: "main", children: [_jsxs("div", { className: "sidebar", children: [_jsx("button", { className: "compose-btn", onClick: () => setComposeOpen(true), disabled: accounts.length === 0, title: accounts.length === 0 ? "Add an account first" : "Compose", children: "Compose" }), _jsx("div", { className: "mail-section-title", children: "Accounts" }), _jsx("button", { className: `filter-btn ${activeAccount === null ? "active" : ""}`, onClick: () => setActiveAccount(null), children: "\uD83C\uDF10 All Accounts" }), accounts.map((acc) => (_jsx("button", { className: `filter-btn ${activeAccount === acc.id ? "active" : ""}`, onClick: () => setActiveAccount(acc.id), children: acc.email }, acc.id))), _jsx("button", { className: "add-email-btn", onClick: addAccount, children: "\u2795 Add Account" }), _jsx("div", { className: "mail-section-title", children: "Folders" }), _jsxs("div", { className: "mail-filters", children: [_jsx("button", { className: `filter-btn ${activeFolder === "inbox" ? "active" : ""}`, onClick: () => setActiveFolder("inbox"), children: "\uD83D\uDCE5 Inbox" }), _jsx("button", { className: `filter-btn ${activeFolder === "sent" ? "active" : ""}`, onClick: () => setActiveFolder("sent"), children: "\uD83D\uDCE4 Sent" })] })] }), _jsxs("div", { className: "email-list", children: [emails.map((email) => (_jsxs("div", { className: `email-item ${selectedEmail?.id === email.id ? "active" : ""}`, onClick: () => openEmail(email), children: [_jsxs("div", { className: "email-top", children: [_jsx("span", { className: "email-sender", children: email.sender }), _jsx("span", { className: "email-time", children: new Date(email.created_at).toLocaleTimeString() })] }), _jsx("div", { className: "email-subject", children: email.subject }), _jsx("div", { className: "email-preview", children: email.preview || "" })] }, email.id))), hasMore && (_jsx("div", { className: "load-more-wrap", children: _jsx("button", { className: "load-more-btn", onClick: loadMore, disabled: loadingMore, children: loadingMore ? "Loading..." : "Load More" }) }))] }), composeOpen && accounts.length > 0 && (_jsx("div", { onClick: () => setComposeOpen(false), style: {
                    position: "fixed",
                    inset: 0,
                    background: "rgba(0,0,0,0.4)",
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    zIndex: 1000,
                }, children: _jsx("div", { onClick: (e) => e.stopPropagation(), style: {
                        background: "#fff",
                        padding: 20,
                        borderRadius: 8,
                        width: 480,
                        maxWidth: "90vw",
                        boxShadow: "0 10px 30px rgba(0,0,0,0.2)",
                    }, children: _jsx(SendEmail, { accountId: activeAccount ?? accounts[0].id, onClose: () => setComposeOpen(false) }) }) })), _jsx("div", { className: "email-detail", children: !selectedEmail ? (_jsx("p", { children: "Select an email" })) : (_jsxs(_Fragment, { children: [_jsx("h2", { children: selectedEmail.subject }), _jsxs("p", { children: [_jsx("b", { children: "From:" }), " ", selectedEmail.sender] }), _jsxs("p", { children: [_jsx("b", { children: "To:" }), " ", selectedEmail.receiver] }), _jsx("div", { className: "email-body", children: selectedEmail.body })] })) })] }));
}
