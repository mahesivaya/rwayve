import { logger } from "../utils/logger";
const API_BASE = import.meta.env.VITE_API_URL;

import { useEffect, useState, useRef } from "react";
import { useAuth } from "../auth/AuthContext";

type User = {
  id: number;
  email: string;
};

type Message = {
  sender_id: number;
  receiver_id: number;
  content: string;
  status: "sent" | "delivered" | "read";
  created_at: string;
};

export default function Chat() {
  const { user } = useAuth();

  const [users, setUsers] = useState<User[]>([]);
  const [messages, setMessages] = useState<Message[]>([]);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);
  const [input, setInput] = useState("");

  const wsRef = useRef<WebSocket | null>(null);

  // =============================
  // 🔥 FORMAT TIME
  // =============================
  const formatTime = (dateStr: string) => {
    const d = new Date(dateStr);
    return d.toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  // =============================
  // 🔥 STATUS ICON
  // =============================
  const getStatusIcon = (status: string) => {
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
        const res = await fetch(`${API_BASE}/api/users/all`, {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        });

        const text = await res.text();

        try {
          const data = JSON.parse(text);

          // remove current user
          const filtered = data.filter((u: User) => u.id !== user?.id);

          setUsers(filtered);
        } catch {
          logger.error("Users error:", text);
        }

      } catch (err) {
        logger.error("Fetch users failed", err);
      }
    };

    if (user) fetchUsers();
  }, [user]);

  // =============================
  // 🔥 CONNECT WEBSOCKET
  // =============================
  useEffect(() => {
    if (!user) return;

    const ws = new WebSocket(`ws://localhost/ws/chat?user_id=${user.id}`);
    wsRef.current = ws;

    ws.onopen = () => {
      logger.log("✅ WS connected");
    };

    ws.onmessage = (event) => {
      const msg: Message = JSON.parse(event.data);

      // Echoes of our own sends are already shown optimistically — skip them
      // (they also tend to arrive with a missing/renamed timestamp, which
      // rendered as "Invalid Date").
      if (msg.sender_id === user.id) return;

      setMessages((prev) => [...prev, msg]);
    };

    ws.onclose = () => {
      logger.log("❌ WS disconnected");
    };

    return () => {
      ws.close();
    };
  }, [user]);

  // =============================
  // 🔥 LOAD MESSAGES
  // =============================
  const loadMessages = async (otherUser: User) => {
    if (!user) return;

    const token = localStorage.getItem("token");

    try {
      const res = await fetch(
        `${API_BASE}/api/messages?user1=${user.id}&user2=${otherUser.id}`,
        {
          headers: {
            Authorization: `Bearer ${token}`,
          },
        }
      );

      const data = await res.json();

      setMessages(data);
      setSelectedUser(otherUser);

    } catch (err) {
      logger.error("Failed to load messages", err);
    }
  };

  // =============================
  // 🔥 SEND MESSAGE
  // =============================
  const sendMessage = () => {
    if (!wsRef.current || !user || !selectedUser || !input.trim()) return;

    const now = new Date().toISOString();

    const message: Message = {
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
  return (
    <div style={{ display: "flex", height: "100%" }}>

      {/* LEFT USERS */}
      <div style={{ width: "30%", borderRight: "1px solid #ddd" }}>
        <h3 style={{ padding: 10 }}>Users</h3>

        {users.map((u) => (
          <div
            key={u.id}
            style={{
              padding: 10,
              cursor: "pointer",
              borderBottom: "1px solid #eee",
              background: selectedUser?.id === u.id ? "#f0f0f0" : "white",
            }}
            onClick={() => loadMessages(u)}
          >
            {u.email}
          </div>
        ))}
      </div>

      {/* RIGHT CHAT */}
      <div style={{ flex: 1, display: "flex", flexDirection: "column" }}>

        {/* MESSAGES */}
        <div style={{ flex: 1, padding: 10, overflowY: "auto" }}>
          {messages.map((msg, i) => (
            <div
              key={i}
              style={{
                textAlign:
                  msg.sender_id === user?.id ? "right" : "left",
                marginBottom: 10,
              }}
            >
              <div
                style={{
                  padding: "8px 12px",
                  background:
                    msg.sender_id === user?.id ? "#DCF8C6" : "#eee",
                  borderRadius: 8,
                  display: "inline-block",
                  maxWidth: "70%",
                }}
              >
                {msg.content}

                {/* 🔥 TIME + STATUS */}
                <div
                  style={{
                    fontSize: 10,
                    marginTop: 4,
                    opacity: 0.7,
                    textAlign: "right",
                  }}
                >
                  {formatTime(msg.created_at)}{" "}
                  {msg.sender_id === user?.id &&
                    getStatusIcon(msg.status)}
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* INPUT */}
        {selectedUser && (
          <div style={{ display: "flex", padding: 10 }}>
            <input
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && !e.shiftKey) {
                  e.preventDefault();
                  sendMessage();
                }
              }}
              style={{ flex: 1, marginRight: 10 }}
              placeholder="Type message..."
            />
            <button onClick={sendMessage}>Send</button>
          </div>
        )}
      </div>
    </div>
  );
}