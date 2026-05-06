import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useEffect, useRef, useState } from "react";
import "./aichat.css";
const API_BASE = import.meta.env.VITE_API_URL;
const authHeaders = () => {
    const token = localStorage.getItem("token");
    return {
        "Content-Type": "application/json",
        Authorization: `Bearer ${token}`,
    };
};
export default function AIChat() {
    const [messages, setMessages] = useState([]);
    const [input, setInput] = useState("");
    const [busy, setBusy] = useState(false);
    const [error, setError] = useState(null);
    const scrollRef = useRef(null);
    // Pin the scroll to the bottom whenever new messages arrive.
    useEffect(() => {
        const el = scrollRef.current;
        if (el)
            el.scrollTop = el.scrollHeight;
    }, [messages, busy]);
    const send = async () => {
        const text = input.trim();
        if (!text || busy)
            return;
        setError(null);
        setInput("");
        const next = [...messages, { role: "user", content: text }];
        setMessages(next);
        setBusy(true);
        try {
            const res = await fetch(`${API_BASE}/api/ai/chat`, {
                method: "POST",
                headers: authHeaders(),
                body: JSON.stringify({ messages: next }),
            });
            if (!res.ok) {
                throw new Error((await res.text()) || `Error ${res.status}`);
            }
            const data = await res.json();
            const reply = (data.reply ?? "").trim();
            if (!reply) {
                throw new Error("Empty reply from model");
            }
            setMessages((prev) => [...prev, { role: "model", content: reply }]);
        }
        catch (err) {
            setError(err?.message ?? "Request failed");
        }
        finally {
            setBusy(false);
        }
    };
    const onKeyDown = (e) => {
        if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            send();
        }
    };
    const clear = () => {
        setMessages([]);
        setError(null);
    };
    return (_jsxs("div", { className: "ai-chat", children: [_jsxs("div", { className: "ai-chat-header", children: [_jsxs("div", { className: "ai-chat-title", children: [_jsx("span", { className: "ai-chat-icon", children: "\u2728" }), "AI Chat", _jsx("span", { className: "ai-chat-sub", children: "Gemini" })] }), messages.length > 0 && (_jsx("button", { className: "ai-chat-clear", onClick: clear, disabled: busy, children: "Clear" }))] }), _jsxs("div", { className: "ai-chat-messages", ref: scrollRef, children: [messages.length === 0 && (_jsxs("div", { className: "ai-chat-empty", children: [_jsx("div", { className: "ai-chat-empty-icon", children: "\u2728" }), _jsx("div", { className: "ai-chat-empty-title", children: "Ask anything" }), _jsx("div", { className: "ai-chat-empty-hint", children: "Type a message below to start chatting with Gemini." })] })), messages.map((m, i) => (_jsx("div", { className: `ai-msg ${m.role === "user" ? "ai-msg-user" : "ai-msg-model"}`, children: _jsx("div", { className: "ai-msg-bubble", children: m.content }) }, i))), busy && (_jsx("div", { className: "ai-msg ai-msg-model", children: _jsxs("div", { className: "ai-msg-bubble ai-msg-typing", children: [_jsx("span", {}), _jsx("span", {}), _jsx("span", {})] }) }))] }), error && _jsx("div", { className: "ai-chat-error", children: error }), _jsxs("div", { className: "ai-chat-input-row", children: [_jsx("textarea", { className: "ai-chat-input", placeholder: "Message AI\u2026  (Enter to send, Shift+Enter for newline)", value: input, onChange: (e) => setInput(e.target.value), onKeyDown: onKeyDown, rows: 2, disabled: busy }), _jsx("button", { className: "ai-chat-send", onClick: send, disabled: busy || !input.trim(), title: "Send", children: busy ? "…" : "Send" })] })] }));
}
