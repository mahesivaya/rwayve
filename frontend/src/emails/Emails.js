import { jsx as _jsx, jsxs as _jsxs, Fragment as _Fragment } from "react/jsx-runtime";
import { useEffect, useRef, useState } from "react";
import "./emails.css";
import "./loadMore.css";
import SendEmail from "./SendEmail";
import { API_BASE } from "../config/env";
import { apiFetch } from "../api/client";
import { decryptMessage } from "../crypto/crypto";
import { loadPrivateKey } from "../crypto/keyStore";
import { useAuth } from "../auth/AuthContext";
import { useGlobalSearch } from "../search/SearchContext";
const WAYVE_SECURE_PREFIX = "WAYVE_SECURE_V1";
function normalizeEmailBody(body) {
    if (!/[<&][a-zA-Z#/!]/.test(body)) {
        return body;
    }
    const doc = new DOMParser().parseFromString(body, "text/html");
    return doc.body.textContent || body;
}
function parseWayveEncryptedBody(body) {
    const trimmed = normalizeEmailBody(body).trim();
    if (!trimmed.startsWith(WAYVE_SECURE_PREFIX)) {
        return null;
    }
    const jsonStart = trimmed.indexOf("{");
    if (jsonStart === -1) {
        throw new Error("Encrypted Wayve email is missing its payload");
    }
    const jsonEnd = trimmed.lastIndexOf("}");
    if (jsonEnd < jsonStart) {
        throw new Error("Encrypted Wayve email payload is incomplete");
    }
    const parsed = JSON.parse(trimmed.slice(jsonStart, jsonEnd + 1));
    if (parsed?.type !== "wayve_encrypted" ||
        !Array.isArray(parsed.data) ||
        !Array.isArray(parsed.key) ||
        !Array.isArray(parsed.iv)) {
        throw new Error("Encrypted Wayve email payload is invalid");
    }
    return parsed;
}
function emailBodyErrorMessage(err) {
    const message = err instanceof Error ? err.message : "";
    if (message.includes("private key") ||
        message.includes("decrypt") ||
        message.includes("operation failed")) {
        return "Unable to decrypt this fully encrypted email on this device. Sign out and back in to refresh your Wayve encryption key, then ask the sender to resend it.";
    }
    return "Failed to load email body. Try again.";
}
async function decryptWayveBodyIfNeeded(body, userId) {
    const encrypted = parseWayveEncryptedBody(body);
    if (!encrypted) {
        return body;
    }
    const privateKeys = [];
    const scopedPrivateKey = await loadPrivateKey(userId);
    if (scopedPrivateKey) {
        privateKeys.push(scopedPrivateKey);
    }
    if (userId) {
        const legacyPrivateKey = await loadPrivateKey();
        if (legacyPrivateKey && legacyPrivateKey !== scopedPrivateKey) {
            privateKeys.push(legacyPrivateKey);
        }
    }
    if (privateKeys.length === 0) {
        throw new Error("This device does not have your Wayve private key");
    }
    let lastError = null;
    for (const privateKey of privateKeys) {
        try {
            return await decryptMessage(new Uint8Array(encrypted.data), new Uint8Array(encrypted.key), new Uint8Array(encrypted.iv), privateKey);
        }
        catch (err) {
            lastError = err;
        }
    }
    throw lastError || new Error("Unable to decrypt Wayve email");
}
export default function Emails() {
    const { user } = useAuth();
    const { normalizedSearchQuery } = useGlobalSearch();
    const [accounts, setAccounts] = useState([]);
    const [emails, setEmails] = useState([]);
    const [selectedEmail, setSelectedEmail] = useState(null);
    const [activeAccount, setActiveAccount] = useState(null);
    const [activeFolder, setActiveFolder] = useState("inbox");
    const [hasMore, setHasMore] = useState(true);
    const [loadingMore, setLoadingMore] = useState(false);
    const [refreshTick, setRefreshTick] = useState(0);
    const [composeOpen, setComposeOpen] = useState(false);
    const emailCache = useRef({});
    // ================= NARROW MODE (split-pane / small viewport) =================
    // When the container is narrow (e.g. rendered inside the split view), we
    // collapse the 3-pane layout to a stacked one: show the list OR the detail,
    // not both. The threshold is the container width — independent of viewport
    // size, so this also responds correctly to a resized split.
    const mainRef = useRef(null);
    const [isNarrow, setIsNarrow] = useState(false);
    useEffect(() => {
        const el = mainRef.current;
        if (!el)
            return;
        const ro = new ResizeObserver((entries) => {
            for (const entry of entries) {
                setIsNarrow(entry.contentRect.width < 800);
            }
        });
        ro.observe(el);
        return () => ro.disconnect();
    }, []);
    const showList = !isNarrow || selectedEmail === null;
    const showDetail = !isNarrow || selectedEmail !== null;
    // ================= FETCH ACCOUNTS =================
    const fetchAccounts = async () => {
        try {
            const res = await apiFetch("api/accounts");
            const data = await res.json();
            setAccounts(data);
        }
        catch (err) {
            console.error(err);
        }
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
            setRefreshTick((tick) => tick + 1);
            window.history.replaceState({}, "", "/emails");
            let attempts = 0;
            const poll = window.setInterval(() => {
                attempts += 1;
                setRefreshTick((tick) => tick + 1);
                if (attempts >= 12) {
                    window.clearInterval(poll);
                }
            }, 2000);
            return () => window.clearInterval(poll);
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
            let url = `api/emails?folder=${activeFolder}`;
            if (activeAccount !== null) {
                url += `&account_id=${activeAccount}`;
            }
            if (normalizedSearchQuery) {
                url += `&q=${encodeURIComponent(normalizedSearchQuery)}`;
            }
            const res = await apiFetch(url);
            const data = await res.json();
            const hasMorePage = res.headers.get("x-has-more") === "true";
            setEmails(data);
            setHasMore(hasMorePage || data.length === 50);
            setSelectedEmail(null);
        };
        void fetchEmails();
    }, [activeAccount, activeFolder, refreshTick, normalizedSearchQuery]);
    // ================= LOAD MORE =================
    const loadMore = async () => {
        if (!hasMore || emails.length === 0)
            return;
        setLoadingMore(true);
        try {
            const last = emails[emails.length - 1];
            const before = Math.floor(new Date(last.created_at).getTime() / 1000);
            const before_id = last.id;
            let url = `/api/emails?folder=${activeFolder}&before=${before}&before_id=${before_id}`;
            if (activeAccount !== null) {
                url += `&account_id=${activeAccount}`;
            }
            if (normalizedSearchQuery) {
                url += `&q=${encodeURIComponent(normalizedSearchQuery)}`;
            }
            const res = await apiFetch(url);
            const data = await res.json();
            const hasMorePage = res.headers.get("x-has-more") === "true";
            setEmails((prev) => [...prev, ...data]);
            setHasMore(hasMorePage);
        }
        finally {
            setLoadingMore(false);
        }
    };
    // ================= OPEN EMAIL =================
    const openEmail = async (email) => {
        if (emailCache.current[email.id]) {
            setSelectedEmail(emailCache.current[email.id]);
            return;
        }
        let data;
        try {
            // 1) Show metadata immediately. Body may be empty if body_worker hasn't
            //    fetched it yet — render the placeholder via bodyLoading.
            const res = await apiFetch(`/api/emails/${email.id}`);
            data = await res.json();
        }
        catch (err) {
            console.error("Email detail load failed", err);
            setSelectedEmail({
                ...email,
                body: "",
                _bodyError: "Failed to load email body. Try again.",
            });
            return;
        }
        if (data.body) {
            try {
                const decryptedBody = await decryptWayveBodyIfNeeded(data.body, user?.id);
                const decryptedData = { ...data, body: decryptedBody };
                emailCache.current[email.id] = decryptedData;
                setSelectedEmail(decryptedData);
                return;
            }
            catch (err) {
                console.error("Wayve email decrypt failed", err);
                setSelectedEmail({
                    ...data,
                    body: "",
                    _bodyError: emailBodyErrorMessage(err),
                });
                return;
            }
        }
        setSelectedEmail({ ...data, _bodyLoading: true });
        // 2) If body wasn't ready, hit the on-demand endpoint. Backend triggers a
        //    Gmail fetch, encrypts, persists, and returns the body.
        if (!data.body) {
            try {
                const bodyRes = await apiFetch(`/api/emails/${email.id}/body`);
                const { body } = await bodyRes.json();
                const decryptedBody = await decryptWayveBodyIfNeeded(body || "", user?.id);
                const merged = { ...data, body: decryptedBody, _bodyLoading: false };
                emailCache.current[email.id] = merged;
                // Only update if user hasn't navigated away to a different email.
                setSelectedEmail((cur) => (cur && cur.id === email.id ? merged : cur));
                return;
            }
            catch (err) {
                console.error("Wayve email body load/decrypt failed", err);
                setSelectedEmail((cur) => cur && cur.id === email.id
                    ? {
                        ...cur,
                        _bodyLoading: false,
                        _bodyError: emailBodyErrorMessage(err),
                    }
                    : cur);
            }
            return;
        }
        emailCache.current[email.id] = data;
    };
    // ================= UI =================
    return (_jsxs("div", { ref: mainRef, className: `main ${isNarrow ? "narrow" : ""}`, children: [_jsxs("div", { className: "sidebar", children: [_jsx("button", { className: "compose-btn", onClick: () => setComposeOpen(true), disabled: accounts.length === 0, title: accounts.length === 0 ? "Add an account first" : "Compose", children: "Compose" }), _jsx("div", { className: "mail-section-title", children: "Accounts" }), _jsx("button", { className: `filter-btn ${activeAccount === null ? "active" : ""}`, onClick: () => setActiveAccount(null), children: "\uD83C\uDF10 All Accounts" }), accounts.map((acc) => (_jsx("button", { className: `filter-btn ${activeAccount === acc.id ? "active" : ""}`, onClick: () => setActiveAccount(acc.id), children: acc.email }, acc.id))), _jsx("button", { className: "add-email-btn", onClick: addAccount, children: "\u2795 Add Account" }), _jsx("div", { className: "mail-section-title", children: "Folders" }), _jsxs("div", { className: "mail-filters", children: [_jsx("button", { className: `filter-btn ${activeFolder === "inbox" ? "active" : ""}`, onClick: () => setActiveFolder("inbox"), children: "\uD83D\uDCE5 Inbox" }), _jsx("button", { className: `filter-btn ${activeFolder === "sent" ? "active" : ""}`, onClick: () => setActiveFolder("sent"), children: "\uD83D\uDCE4 Sent" })] })] }), showList && (_jsxs("div", { className: "email-list", children: [emails.map((email) => (_jsxs("div", { className: `email-item ${selectedEmail?.id === email.id ? "active" : ""}`, onClick: () => openEmail(email), children: [_jsxs("div", { className: "email-top", children: [_jsx("span", { className: "email-sender", children: email.sender }), _jsx("span", { className: "email-time", children: new Date(email.created_at).toLocaleTimeString() })] }), _jsx("div", { className: "email-subject", children: email.subject }), _jsx("div", { className: "email-preview", children: email.preview || "" })] }, email.id))), hasMore && (_jsx("div", { className: "load-more-wrap", children: _jsx("button", { className: "load-more-btn", onClick: loadMore, disabled: loadingMore, children: loadingMore ? "Loading..." : "Show more emails" }) }))] })), composeOpen && accounts.length > 0 && (_jsx("div", { onClick: () => setComposeOpen(false), style: {
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
                    }, children: _jsx(SendEmail, { accountId: activeAccount ?? accounts[0].id, onClose: () => setComposeOpen(false) }) }) })), showDetail && (_jsxs("div", { className: "email-detail", children: [isNarrow && selectedEmail && (_jsx("button", { className: "email-detail-back", onClick: () => setSelectedEmail(null), title: "Close email", "aria-label": "Close email", children: "\u2715 Close email" })), !selectedEmail ? (_jsx("p", { children: "Select an email" })) : (_jsxs(_Fragment, { children: [_jsx("h2", { children: selectedEmail.subject }), _jsxs("p", { children: [_jsx("b", { children: "From:" }), " ", selectedEmail.sender] }), _jsxs("p", { children: [_jsx("b", { children: "To:" }), " ", selectedEmail.receiver] }), _jsx("div", { className: "email-body", children: selectedEmail._bodyLoading ? (_jsxs("div", { className: "email-body-loading", children: [_jsx("span", { className: "spinner", "aria-hidden": "true" }), _jsx("span", { children: "Loading email \u2026" })] })) : selectedEmail._bodyError ? (_jsx("p", { className: "email-body-error", children: typeof selectedEmail._bodyError === "string"
                                        ? selectedEmail._bodyError
                                        : "Failed to load email body. Try again." })) : (selectedEmail.body) })] }))] }))] }));
}
