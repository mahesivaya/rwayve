import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useEffect, useState, useRef } from "react";
import { useAuth } from "../auth/AuthContext";
export default function Chat() {
    const { user } = useAuth();
    const [users, setUsers] = useState([]);
    const [messages, setMessages] = useState([]);
    const [selectedUser, setSelectedUser] = useState(null);
    const [input, setInput] = useState("");
    const wsRef = useRef(null);
    // =============================
    // 🔥 FORMAT TIME
    // =============================
    const formatTime = (dateStr) => {
        const d = new Date(dateStr);
        return d.toLocaleTimeString([], {
            hour: "2-digit",
            minute: "2-digit",
        });
    };
    // =============================
    // 🔥 STATUS ICON
    // =============================
    const getStatusIcon = (status) => {
        switch (status) {
            case "sent":
                return "✓";
            case "delivered":
                return "✓✓";
            case "read":
                return "👁";
            default:
                return "";
        }
    };
    // =============================
    // 🔥 FETCH USERS
    // =============================
    useEffect(() => {
        const fetchUsers = async () => {
            const token = localStorage.getItem("token");
            try {
                const res = await fetch("http://localhost:8080/api/users/all", {
                    headers: {
                        Authorization: `Bearer ${token}`,
                    },
                });
                const text = await res.text();
                try {
                    const data = JSON.parse(text);
                    // remove current user
                    const filtered = data.filter((u) => u.id !== user?.id);
                    setUsers(filtered);
                }
                catch {
                    console.error("Users error:", text);
                }
            }
            catch (err) {
                console.error("Fetch users failed", err);
            }
        };
        if (user)
            fetchUsers();
    }, [user]);
    // =============================
    // 🔥 CONNECT WEBSOCKET
    // =============================
    useEffect(() => {
        if (!user)
            return;
        const ws = new WebSocket(`ws://localhost/ws/chat?user_id=${user.id}`);
        wsRef.current = ws;
        ws.onopen = () => {
            console.log("✅ WS connected");
        };
        ws.onmessage = (event) => {
            const msg = JSON.parse(event.data);
            setMessages((prev) => [...prev, msg]);
        };
        ws.onclose = () => {
            console.log("❌ WS disconnected");
        };
        return () => {
            ws.close();
        };
    }, [user]);
    // =============================
    // 🔥 LOAD MESSAGES
    // =============================
    const loadMessages = async (otherUser) => {
        if (!user)
            return;
        const token = localStorage.getItem("token");
        try {
            const res = await fetch(`http://localhost:8080/api/messages?user1=${user.id}&user2=${otherUser.id}`, {
                headers: {
                    Authorization: `Bearer ${token}`,
                },
            });
            const data = await res.json();
            setMessages(data);
            setSelectedUser(otherUser);
        }
        catch (err) {
            console.error("Failed to load messages", err);
        }
    };
    // =============================
    // 🔥 SEND MESSAGE
    // =============================
    const sendMessage = () => {
        if (!wsRef.current || !user || !selectedUser || !input.trim())
            return;
        const now = new Date().toISOString();
        const message = {
            sender_id: user.id,
            receiver_id: selectedUser.id,
            content: input,
            status: "sent", // initial status
            created_at: now,
        };
        wsRef.current.send(JSON.stringify(message));
        setMessages((prev) => [...prev, message]);
        setInput("");
    };
    // =============================
    // UI
    // =============================
    return (_jsxs("div", { style: { display: "flex", height: "100%" }, children: [_jsxs("div", { style: { width: "30%", borderRight: "1px solid #ddd" }, children: [_jsx("h3", { style: { padding: 10 }, children: "Users" }), users.map((u) => (_jsx("div", { style: {
                            padding: 10,
                            cursor: "pointer",
                            borderBottom: "1px solid #eee",
                            background: selectedUser?.id === u.id ? "#f0f0f0" : "white",
                        }, onClick: () => loadMessages(u), children: u.email }, u.id)))] }), _jsxs("div", { style: { flex: 1, display: "flex", flexDirection: "column" }, children: [_jsx("div", { style: { flex: 1, padding: 10, overflowY: "auto" }, children: messages.map((msg, i) => (_jsx("div", { style: {
                                textAlign: msg.sender_id === user?.id ? "right" : "left",
                                marginBottom: 10,
                            }, children: _jsxs("div", { style: {
                                    padding: "8px 12px",
                                    background: msg.sender_id === user?.id ? "#DCF8C6" : "#eee",
                                    borderRadius: 8,
                                    display: "inline-block",
                                    maxWidth: "70%",
                                }, children: [msg.content, _jsxs("div", { style: {
                                            fontSize: 10,
                                            marginTop: 4,
                                            opacity: 0.7,
                                            textAlign: "right",
                                        }, children: [formatTime(msg.created_at), " ", msg.sender_id === user?.id &&
                                                getStatusIcon(msg.status)] })] }) }, i))) }), selectedUser && (_jsxs("div", { style: { display: "flex", padding: 10 }, children: [_jsx("input", { value: input, onChange: (e) => setInput(e.target.value), style: { flex: 1, marginRight: 10 }, placeholder: "Type message..." }), _jsx("button", { onClick: sendMessage, children: "Send" })] }))] })] }));
}
