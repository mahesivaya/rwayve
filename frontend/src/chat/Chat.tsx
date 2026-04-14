import { useEffect, useRef, useState } from "react";
import { useAuth } from "../auth/AuthContext";
import "./chat.css";

type User = {
  id: number;
  email: string;
};

type Message = {
  message_id?: number;
  sender_id: number;
  receiver_id: number;
  content: string;
  status?: "sent" | "delivered" | "read";
  type?: string; // for WS events
};

export default function Chat() {
  const [users, setUsers] = useState<User[]>([]);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);

  const { user } = useAuth();
  const currentUserId = user?.id;

  const endRef = useRef<HTMLDivElement | null>(null);

  // ✅ Load users
  useEffect(() => {
    fetch("/api/users")
      .then(res => res.json())
      .then(data => {
        console.log("USERS API:", data);
        setUsers(data);
      })
      .catch(err => console.error("Users error:", err));
  }, []);

  // ✅ WebSocket connect
  useEffect(() => {
    if (!currentUserId) return;

    const ws = new WebSocket(
      `ws://${window.location.host}/ws/chat?user_id=${currentUserId}`
    );

    ws.onopen = () => console.log("✅ WS connected");

    ws.onmessage = (event) => {
      console.log("📥 RAW WS:", event.data);

      try {
        const msg: Message = JSON.parse(event.data);

        // 🔥 HANDLE STATUS UPDATE
        if (msg.type === "status_update") {
          setMessages(prev =>
            prev.map(m =>
              m.message_id === msg.message_id
                ? { ...m, status: msg.status }
                : m
            )
          );
          return;
        }

        // 🔥 PREVENT DUPLICATES
        setMessages(prev => {
          if (
            msg.message_id &&
            prev.some(m => m.message_id === msg.message_id)
          ) {
            return prev;
          }
          return [...prev, msg];
        });

      } catch (e) {
        console.error("❌ Parse error:", e);
      }
    };

    ws.onerror = (err) => console.error("WS error", err);

    setSocket(ws);
    return () => ws.close();
  }, [currentUserId]);

  // ✅ Clear chat when switching users
  useEffect(() => {
    setMessages([]);
  }, [selectedUser]);

  // ✅ Load chat history + mark as read
  useEffect(() => {
    if (!selectedUser || !currentUserId) return;

    fetch(`/api/messages?user1=${currentUserId}&user2=${selectedUser.id}`)
      .then(res => res.json())
      .then(data => {
        setMessages(data);

        // 🔥 SEND READ EVENT
        if (socket && socket.readyState === WebSocket.OPEN) {
          socket.send(JSON.stringify({
            sender_id: currentUserId,
            receiver_id: selectedUser.id,
            content: "",
            status: "read"
          }));
        } else {
          console.warn("⚠️ WS not ready, skipping read event");
        }
      })
      .catch(console.error);
  }, [selectedUser, currentUserId, socket]);

  // ✅ Auto scroll
  useEffect(() => {
    if (messages.length > 0) {
      endRef.current?.scrollIntoView({ behavior: "smooth" });
    }
  }, [messages]);

  // ✅ Send message
  const sendMessage = () => {
    if (!socket) return;
    if (!selectedUser) return alert("Select a user first");
    if (!input.trim()) return;
    if (!currentUserId) return;

    const msg: Message = {
      sender_id: currentUserId,
      receiver_id: selectedUser.id,
      content: input,
    };

    console.log("📤 Sending:", msg);

    socket.send(JSON.stringify(msg));
    setInput("");
  };

  // ✅ Filter messages
  const chatMessages = messages.filter(
    m =>
      selectedUser &&
      (
        (m.sender_id === currentUserId && m.receiver_id === selectedUser.id) ||
        (m.sender_id === selectedUser.id && m.receiver_id === currentUserId)
      )
  );

  return (
    <div className="chat-container">

      {/* 🧑 USER LIST */}
      <div className="user-list">
        <h3>Users</h3>

        {users
          .filter(u => u.id !== currentUserId)
          .map(u => (
            <div
              key={u.id}
              onClick={() => setSelectedUser(u)}
              className={`user-item ${
                selectedUser?.id === u.id ? "active" : ""
              }`}
            >
              💬 {u.email}
            </div>
          ))}
      </div>

      {/* 💬 CHAT AREA */}
      <div className="chat-area">
        <h3 className="chat-header">
          {selectedUser ? `Chat with ${selectedUser.email}` : "Select a user"}
        </h3>

        <div className="messages">
          {chatMessages.map((msg, i) => (
            <div
              key={msg.message_id || i}
              className={`message ${
                msg.sender_id === currentUserId ? "me" : ""
              }`}
            >
              <span
                className={`bubble ${
                  msg.sender_id === currentUserId ? "me" : "other"
                }`}
              >
                {msg.content}

                {/* ✅ STATUS TICKS */}
                {msg.sender_id === currentUserId && (
                  <span className="status">
                    {msg.status === "sent" && " ✔"}
                    {msg.status === "delivered" && " ✔✔"}
                    {msg.status === "read" && " ✔✔"}
                  </span>
                )}
              </span>
            </div>
          ))}
          <div ref={endRef} />
        </div>

        <div className="chat-input">
          <textarea
            value={input}
            onChange={e => setInput(e.target.value)}
            placeholder="Type message..."
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                sendMessage();
              }
            }}
          />

          <button onClick={sendMessage} disabled={!selectedUser}>
            Send
          </button>
        </div>
      </div>
    </div>
  );
}