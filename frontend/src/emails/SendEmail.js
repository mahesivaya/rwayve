import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { logger } from "../utils/logger";
import { useState, useEffect } from "react";
import { encryptMessage } from "../crypto/crypto";
export default function SendEmail() {
    const [accountId] = useState(1);
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
            // 🔥 1. Check if receiver is Wayve user
            const checkRes = await fetch(`http://localhost:8080/api/users?email=${to}`, {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });
            let finalBody = body;
            if (checkRes.ok) {
                const users = await checkRes.json();
                // 👉 FIX: handle array response
                const user = Array.isArray(users) ? users[0] : users;
                logger.log("USER RESPONSE:", user);
                if (user && user.public_key) {
                    const parsedKey = typeof user.public_key === "string"
                        ? JSON.parse(user.public_key)
                        : user.public_key;
                    const publicKey = await crypto.subtle.importKey("spki", new Uint8Array(parsedKey), { name: "RSA-OAEP", hash: "SHA-256" }, true, ["encrypt"]);
                    const { encryptedMessage, encryptedKey, iv } = await encryptMessage(body, publicKey);
                    finalBody =
                        "WAYVE_SECURE_V1\n" +
                            JSON.stringify({
                                type: "wayve_encrypted",
                                data: Array.from(new Uint8Array(encryptedMessage)),
                                key: Array.from(new Uint8Array(encryptedKey)),
                                iv: Array.from(iv),
                            });
                }
            }
            logger.log("FINAL BODY:", finalBody);
            // 🔥 2. Send email
            const res = await fetch("http://localhost:8080/api/send", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: `Bearer ${token}`,
                },
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
        }
        catch (err) {
            logger.error(err);
            setStatus(err.message || "Failed to send email ❌");
        }
        setLoading(false);
    };
    return (_jsxs("div", { style: {
            display: "flex",
            flexDirection: "column",
            gap: "10px"
        }, children: [_jsx("h3", { style: { marginBottom: 5 }, children: "Compose Email" }), _jsx("input", { placeholder: "To", value: to, onChange: (e) => setTo(e.target.value), style: {
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
