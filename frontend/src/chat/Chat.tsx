import { useEffect, useRef, useState } from "react";
import { useAuth } from "../auth/AuthContext";

type User = {
  id: number;
  email: string;
};

type Message = {
  sender_id: number;
  receiver_id: number;
  content: string;
};

export default function Chat() {
  const [users, setUsers] = useState<User[]>([]);
  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [selectedUser, setSelectedUser] = useState<User | null>(null);

  // const currentUserId = 1;
  const { user } = useAuth();
  const currentUserId = user?.id;

  const endRef = useRef<HTMLDivElement | null>(null);

  // ✅ Load users
  useEffect(() => {
    fetch("/api/users")
      .then(res => res.json())
      .then(setUsers)
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
      console.log("📩 WS RAW:", event.data);
      try {
        const msg = JSON.parse(event.data);
        console.log("✅ Parsed:", msg);
        setMessages(prev => [...prev, msg]);
      } catch {
        console.log("Non-JSON:", event.data);
      }
    };

    ws.onerror = (err) => console.error("WS error", err);

    setSocket(ws);
    return () => ws.close();
  }, [currentUserId]);

  // ✅ Load chat history when user selected
  useEffect(() => {
    if (!selectedUser) return;

    fetch(`/api/messages?user1=${currentUserId}&user2=${selectedUser.id}`)
      .then(res => res.json())
      .then(setMessages)
      .catch(console.error);
  }, [selectedUser]);

  // ✅ Auto scroll
  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages]);

  // ✅ Send message
  const sendMessage = () => {
    if (!socket) return;
    if (!selectedUser) return alert("Select a user first");
    if (!input.trim()) return;

    const msg = {
      sender_id: currentUserId,
      receiver_id: selectedUser.id,
      content: input,
    };

    socket.send(JSON.stringify(msg));

    // instant UI
    // setMessages(prev => [...prev, msg]);
    setInput("");
  };

  // ✅ Filter messages for current chat
  const chatMessages = messages.filter(
    m =>
      selectedUser &&
      (
        (m.sender_id === currentUserId && m.receiver_id === selectedUser.id) ||
        (m.sender_id === selectedUser.id && m.receiver_id === currentUserId)
      )
  );

  return (
    <div style={{ display: "flex", height: "100vh" }}>

      {/* 🧑 USER LIST */}
      <div style={{ width: "250px", borderRight: "1px solid #ccc" }}>
        <h3>Users</h3>

        {users.map(user => (
          <div
            key={user.id}
            onClick={() => setSelectedUser(user)}
            style={{
              padding: "10px",
              cursor: "pointer",
              background: selectedUser?.id === user.id ? "#eee" : "white"
            }}
          >
            📧 {user.email}
          </div>
        ))}
      </div>

      {/* 💬 CHAT AREA */}
      <div style={{ flex: 1, padding: "20px", display: "flex", flexDirection: "column" }}>
        <h3>
          Chat {selectedUser ? `with ${selectedUser.email}` : ""}
        </h3>

        {/* messages */}
        <div style={{
          flex: 1,
          overflowY: "auto",
          border: "1px solid #ccc",
          marginBottom: "10px",
          padding: "10px"
        }}>
          {chatMessages.map((msg, i) => (
            <div key={i} style={{
              textAlign: msg.sender_id === currentUserId ? "right" : "left",
              marginBottom: "8px"
            }}>
              <span style={{
                background: msg.sender_id === currentUserId ? "#d1e7ff" : "#eee",
                padding: "6px 10px",
                borderRadius: "10px",
                display: "inline-block"
              }}>
                {msg.content}
              </span>
            </div>
          ))}

          <div ref={endRef} />
        </div>

        {/* input */}
        <div style={{ display: "flex" }}>
          <textarea
            value={input}
            onChange={e => setInput(e.target.value)}
            placeholder="Type message..."
            style={{ flex: 1, marginRight: "10px", height: "60px" }}
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