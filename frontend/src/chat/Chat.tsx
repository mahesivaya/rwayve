import { useEffect, useState } from "react";

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

  const currentUserId = 1; // 🔥 replace later with logged-in user

  // ✅ Load users
  useEffect(() => {
    fetch("/api/accounts")
      .then(res => res.json())
      .then(data => setUsers(data))
      .catch(err => console.error("Users error:", err));
  }, []);

  // ✅ WebSocket connect
  useEffect(() => {
    const ws = new WebSocket("ws://localhost/ws/chat");

    ws.onopen = () => console.log("✅ WS connected");

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        setMessages(prev => [...prev, msg]);
      } catch {
        console.log("Non-JSON message:", event.data);
      }
    };

    ws.onerror = (err) => console.error("WS error", err);

    setSocket(ws);

    return () => ws.close();
  }, []);

  // ✅ Send message
  const sendMessage = () => {
    if (!socket) {
      console.log("❌ No socket");
      return;
    }

    if (!selectedUser) {
      alert("Select a user first");
      return;
    }

    if (!input.trim()) return;

    const msg = {
      sender_id: currentUserId,
      receiver_id: selectedUser.id,
      content: input,
    };

    console.log("Sending:", msg);

    // ✅ send to backend
    socket.send(JSON.stringify(msg));

    // ✅ show instantly in UI (IMPORTANT)
    setMessages(prev => [...prev, msg]);

    setInput("");
  };

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
      <div style={{ flex: 1, padding: "20px" }}>
        <h3>
          Chat {selectedUser ? `with ${selectedUser.email}` : ""}
        </h3>

        <div style={{ height: 300, overflowY: "auto", border: "1px solid #ccc", marginBottom: "10px" }}>
          {messages.map((msg, i) => (
            <div key={i}>
              <b>{msg.sender_id === currentUserId ? "You" : "Them"}:</b> {msg.content}
            </div>
          ))}
        </div>

        <input
          value={input}
          onChange={e => setInput(e.target.value)}
          placeholder="Type message..."
          style={{ marginRight: "10px" }}
        />

        <button onClick={sendMessage}>Send</button>
      </div>

    </div>
  );
}