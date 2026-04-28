import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import { useEffect, useState } from "react";
import SendEmail from "./SendEmail";
import { decryptMessage } from "../crypto/crypto";
import { loadPrivateKey } from "../crypto/keyStore";
import { useNavigate, useSearchParams } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import { apiFetch } from "../api";
export default function Emails() {
    const [searchParams] = useSearchParams();
    const navigate = useNavigate();
    const { login } = useAuth();
    const [emails, setEmails] = useState([]);
    const [loadingMore, setLoadingMore] = useState(false);
    const [hasMore, setHasMore] = useState(true);
    const [accounts, setAccounts] = useState([]);
    const [activeAccount, setActiveAccount] = useState(null);
    const [selected, setSelected] = useState(null);
    const [privateKey, setPrivateKey] = useState(null);
    const [showCompose, setShowCompose] = useState(false);
    const [minimized, setMinimized] = useState(false);
    const API_BASE = import.meta.env.VITE_API_URL;
    // 🔐 Load private key
    // 🔐 Load private key
    useEffect(() => {
        const initKey = async () => {
            try {
                const key = await loadPrivateKey();
                if (key)
                    setPrivateKey(key);
            }
            catch (err) {
                console.error("❌ Failed to load private key:", err);
            }
        };
        initKey();
    }, []);
    // 📧 Fetch accounts (production-safe)
    const fetchAccounts = async () => {
        try {
            const res = await apiFetch("/api/accounts");
            const data = await res.json();
            setAccounts(data);
        }
        catch (err) {
            console.error(err);
        }
    };
    useEffect(() => {
        const params = new URLSearchParams(window.location.search);
        if (params.get("connected") === "true") {
            console.log("🔄 Refreshing accounts after OAuth");
            fetchAccounts();
            // clean URL
            window.history.replaceState({}, document.title, "/emails");
        }
    }, []);
    useEffect(() => {
        fetchAccounts();
    }, []);
    useEffect(() => {
        const urlParams = new URLSearchParams(window.location.search);
        if (urlParams.get("connected") === "true") {
            window.history.replaceState({}, document.title, "/emails");
            fetchAccounts(); // 🔥 refresh
        }
    }, []);
    // 📥 Fetch emails
    useEffect(() => {
        const fetchEmails = async () => {
            const token = localStorage.getItem("token");
            let url = "http://localhost:8080/api/emails";
            if (activeAccount !== null) {
                url += `?account_id=${activeAccount}`;
            }
            const res = await fetch(url, {
                headers: { Authorization: `Bearer ${token}` },
            });
            const data = await res.json();
            setEmails(data);
            setHasMore(data.length === 50); // pagination check
        };
        fetchEmails();
    }, [activeAccount]);
    const loadMore = async () => {
        if (emails.length === 0 || !hasMore)
            return;
        setLoadingMore(true);
        const token = localStorage.getItem("token");
        const last = emails[emails.length - 1];
        const before = Math.floor(new Date(last.created_at).getTime() / 1000);
        const before_id = last.id;
        let url = `http://localhost:8080/api/emails?before=${before}&before_id=${before_id}`;
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
    // Connect to gmail
    const connectGmail = () => {
        const token = localStorage.getItem("token");
        if (!token) {
            alert("Login required ❌");
            return;
        }
        window.location.href =
            `http://localhost:8080/gmail/login?token=${token}`;
    };
    // 🔓 Open email
    const openEmail = async (email) => {
        let bodyText = email.body;
        try {
            if (privateKey && email.body?.startsWith("WAYVE_SECURE_V1")) {
                let raw = email.body.replace("WAYVE_SECURE_V1", "").trim();
                const payload = JSON.parse(raw);
                const decrypted = await decryptMessage(new Uint8Array(payload.data), new Uint8Array(payload.key), new Uint8Array(payload.iv), privateKey);
                bodyText = decrypted;
            }
        }
        catch (err) {
            console.error("Decrypt failed", err);
            bodyText = "❌ Unable to decrypt";
        }
        setSelected({ ...email, body: bodyText });
    };
    return (_jsxs("div", { style: { display: "flex", height: "100%" }, children: [_jsxs("div", { style: { width: "35%", borderRight: "1px solid #ddd" }, children: [_jsxs("div", { style: { padding: 10 }, children: [_jsx("button", { onClick: () => setShowCompose(true), style: {
                                    width: "100%",
                                    background: "#007bff",
                                    color: "white",
                                    padding: "10px",
                                    borderRadius: 6,
                                    border: "none",
                                    marginBottom: 10
                                }, children: "+ Compose" }), _jsx("button", { onClick: connectGmail, style: {
                                    width: "100%",
                                    background: "#f5f5f5",
                                    padding: "10px",
                                    borderRadius: 6,
                                    border: "1px solid #ddd"
                                }, children: "\u2795 Add Account" })] }), _jsxs("div", { style: { padding: 10, display: "flex", flexDirection: "column" }, children: [_jsx("button", { onClick: () => {
                                    setActiveAccount(null);
                                    setEmails([]);
                                    setHasMore(true);
                                }, style: {
                                    marginBottom: 5, // 🔥 vertical spacing
                                    textAlign: "left",
                                    background: activeAccount === null ? "#ddd" : "white"
                                }, children: "All" }), accounts.map((acc) => (_jsx("button", { onClick: () => {
                                    setActiveAccount(acc.id);
                                    setEmails([]);
                                    setHasMore(true);
                                }, style: {
                                    marginBottom: 5, // 🔥 vertical spacing
                                    textAlign: "left",
                                    background: activeAccount === acc.id ? "#ddd" : "white"
                                }, children: acc.email }, acc.id)))] }), _jsxs("div", { style: { overflowY: "auto", height: "80%" }, children: [emails.map((email) => (_jsxs("div", { style: { padding: 10, cursor: "pointer" }, onClick: () => openEmail(email), children: [_jsx("strong", { children: email.sender }), _jsx("div", { children: email.subject }), email.body?.startsWith("WAYVE_SECURE_V1") && (_jsx("span", { children: "\uD83D\uDD10" }))] }, `${email.account_id}-${email.gmail_id || email.id}-${email.created_at}`))), hasMore && (_jsx("button", { onClick: loadMore, disabled: loadingMore, children: loadingMore ? "Loading..." : "Load More" }))] })] }), _jsx("div", { style: { flex: 1, padding: 20 }, children: selected ? (_jsxs(_Fragment, { children: [_jsx("h2", { children: selected.subject }), selected.body?.startsWith("WAYVE_SECURE_V1") ? (_jsx("p", { children: selected.body })) : (_jsx("div", { dangerouslySetInnerHTML: { __html: selected.body } }))] })) : (_jsx("p", { children: "Select an email" })) }), showCompose && (_jsxs("div", { style: {
                    position: "fixed",
                    bottom: 20,
                    right: 20,
                    width: 400,
                    height: 500,
                    background: "white",
                    boxShadow: "0 4px 20px rgba(0,0,0,0.2)",
                    borderRadius: 8,
                    display: "flex",
                    flexDirection: "column",
                    zIndex: 1000
                }, children: [_jsxs("div", { style: {
                            background: "#007bff",
                            color: "white",
                            padding: "10px",
                            borderTopLeftRadius: 8,
                            borderTopRightRadius: 8,
                            display: "flex",
                            justifyContent: "space-between",
                            alignItems: "center"
                        }, children: [_jsx("span", { children: "New Message" }), _jsx("button", { onClick: () => setShowCompose(false), style: {
                                    background: "transparent",
                                    border: "none",
                                    color: "white",
                                    fontSize: 16,
                                    cursor: "pointer"
                                }, children: "\u2715" })] }), _jsx("div", { style: {
                            flex: 1,
                            overflow: "auto",
                            padding: 10
                        }, children: _jsx(SendEmail, {}) })] }))] }));
}
