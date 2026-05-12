import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { logger } from "../utils/logger";
import { apiFetch } from "../api/client";
import { useState, useEffect } from "react";
import { buildEncryptedBody } from "./encryptEmail";
export default function SendEmail({ accountId, onClose, onSent }) {
    const [to, setTo] = useState("");
    const [subject, setSubject] = useState("");
    const [body, setBody] = useState("");
    const [status, setStatus] = useState("");
    const [loading, setLoading] = useState(false);
    useEffect(() => {
        if (!status)
            return;
        const timer = setTimeout(() => setStatus(""), 3000);
        return () => clearTimeout(timer);
    }, [status]);
    const sendEmail = async () => {
        if (!to || !subject || !body) {
            setStatus("Please fill all fields ⚠️");
            return;
        }
        const token = localStorage.getItem("token");
        if (!token) {
            setStatus("You must login first ❌");
            return;
        }
        setLoading(true);
        setStatus("");
        try {
            logger.warn("📨 BEFORE ENCRYPT:", body);
            const finalBody = await buildEncryptedBody(to, body, token);
            logger.warn("🔐 AFTER ENCRYPT:", finalBody);
            // 🔥 2. Send email
            const res = await apiFetch(`/api/send`, {
                method: "POST",
                body: JSON.stringify({
                    account_id: accountId,
                    to,
                    subject,
                    body: finalBody,
                }),
            });
            const text = await res.text();
            if (!res.ok) {
                throw new Error(text || "Failed to send");
            }
            setStatus("Email sent successfully ✅");
            setTo("");
            setSubject("");
            setBody("");
            onSent?.();
            setTimeout(() => onClose?.(), 800);
        }
        catch (err) {
            logger.error(err);
            setStatus(err.message || "Failed to send email ❌");
        }
        finally {
            setLoading(false);
        }
    };
    return (_jsxs("div", { style: {
            display: "flex",
            flexDirection: "column",
            gap: "10px"
        }, children: [_jsxs("div", { style: { display: "flex", justifyContent: "space-between", alignItems: "center" }, children: [_jsx("h3", { style: { margin: 0 }, children: "Compose Email" }), onClose && (_jsx("button", { onClick: onClose, style: {
                            background: "transparent",
                            border: "none",
                            fontSize: 18,
                            cursor: "pointer",
                            color: "#6b7280"
                        }, "aria-label": "Close", children: "\u2715" }))] }), _jsx("input", { placeholder: "To", value: to, onChange: (e) => setTo(e.target.value), style: {
                    padding: "8px",
                    borderRadius: 5,
                    border: "1px solid #ccc"
                } }), _jsx("input", { placeholder: "Subject", value: subject, onChange: (e) => setSubject(e.target.value), style: {
                    padding: "8px",
                    borderRadius: 5,
                    border: "1px solid #ccc"
                } }), _jsx("textarea", { placeholder: "Message", value: body, onChange: (e) => setBody(e.target.value), style: {
                    padding: "8px",
                    borderRadius: 5,
                    border: "1px solid #ccc",
                    minHeight: 120,
                    resize: "none"
                } }), _jsx("button", { onClick: sendEmail, disabled: loading, style: {
                    background: "#007bff",
                    color: "white",
                    padding: "10px",
                    borderRadius: 5,
                    border: "none",
                    cursor: "pointer"
                }, children: loading ? "Sending..." : "Send" }), status && (_jsx("div", { style: {
                    fontSize: 12,
                    color: status.includes("success") ? "green" : "red"
                }, children: status }))] }));
}
