import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import { useEffect, useRef, useState } from "react";
import "./emails.css";
const API_BASE = import.meta.env.VITE_API_URL;
export default function Emails() {
    const [accounts, setAccounts] = useState([]);
    const [emails, setEmails] = useState([]);
    const [selectedEmail, setSelectedEmail] = useState(null);
    const [activeAccount, setActiveAccount] = useState(null);
    const [activeFolder, setActiveFolder] = useState("inbox");
    const [hasMore, setHasMore] = useState(true);
    const [loadingMore, setLoadingMore] = useState(false);
    const emailCache = useRef({});
    // ================= FETCH ACCOUNTS =================
    useEffect(() => {
        const fetchAccounts = async () => {
            const token = localStorage.getItem("token");
            const res = await fetch(`${API_BASE}/api/accounts`, {
                headers: { Authorization: `Bearer ${token}` },
            });
            const data = await res.json();
            setAccounts(data);
        };
        fetchAccounts();
    }, []);
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
    return (_jsxs("div", { className: "main", children: [_jsxs("div", { className: "sidebar", children: [_jsx("button", { className: "compose-btn", children: "Compose" }), _jsx("button", { className: "add-email-btn", onClick: () => setActiveAccount(null), children: "\uD83C\uDF10 All" }), _jsx("div", { className: "mail-section-title", children: "Accounts" }), accounts.map((acc) => (_jsx("button", { className: `filter-btn ${activeAccount === acc.id ? "active" : ""}`, onClick: () => setActiveAccount(acc.id), children: acc.email }, acc.id))), _jsx("div", { className: "mail-section-title", children: "Folders" }), _jsxs("div", { className: "mail-filters", children: [_jsx("button", { className: `filter-btn ${activeFolder === "inbox" ? "active" : ""}`, onClick: () => setActiveFolder("inbox"), children: "\uD83D\uDCE5 Inbox" }), _jsx("button", { className: `filter-btn ${activeFolder === "sent" ? "active" : ""}`, onClick: () => setActiveFolder("sent"), children: "\uD83D\uDCE4 Sent" })] })] }), _jsxs("div", { className: "email-list", children: [emails.map((email) => (_jsxs("div", { className: `email-item ${selectedEmail?.id === email.id ? "active" : ""}`, onClick: () => openEmail(email), children: [_jsxs("div", { className: "email-top", children: [_jsx("span", { className: "email-sender", children: email.sender }), _jsx("span", { className: "email-time", children: new Date(email.created_at).toLocaleTimeString() })] }), _jsx("div", { className: "email-subject", children: email.subject }), _jsx("div", { className: "email-preview", children: email.preview || "" })] }, email.id))), hasMore && (_jsx("button", { className: "add-email-btn", onClick: loadMore, children: loadingMore ? "Loading..." : "Load More" }))] }), _jsx("div", { className: "email-detail", children: !selectedEmail ? (_jsx("p", { children: "Select an email" })) : (_jsxs(_Fragment, { children: [_jsx("h2", { children: selectedEmail.subject }), _jsxs("p", { children: [_jsx("b", { children: "From:" }), " ", selectedEmail.sender] }), _jsxs("p", { children: [_jsx("b", { children: "To:" }), " ", selectedEmail.receiver] }), _jsx("div", { className: "email-body", children: selectedEmail.body })] })) })] }));
}
