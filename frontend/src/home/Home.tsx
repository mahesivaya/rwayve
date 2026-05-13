import { useAuth } from "../auth/AuthContext";
import { useNavigate } from "react-router-dom";
import { useGlobalSearch } from "../search/SearchContext";
import "./home.css";

export default function Home() {
  const { user, logout } = useAuth();
  const { normalizedSearchQuery } = useGlobalSearch();
  const navigate = useNavigate();

  const cards = [
    { path: "/emails", title: "📧 Emails", description: "View and send emails" },
    { path: "/chat", title: "💬 Chat", description: "Real-time messaging" },
    { path: "/call", title: " 📞  🎥 Call", description: "Real-time calling" },
    { path: "/scheduler", title: "📅 Scheduler", description: "Manage your meetings" },
    { path: "/drive", title: "📁 Drive", description: "Store and manage files" },
    { path: "/notes", title: "📝 Notes", description: "Store and manage notes" },
    { path: "/aichat", title: "✨ AI Chat", description: "Chat with AI" },
  ];

  const visibleCards = normalizedSearchQuery
    ? cards.filter((card) =>
        [card.title, card.description]
          .join(" ")
          .toLowerCase()
          .includes(normalizedSearchQuery)
      )
    : cards;

  if (!user) {
    return (
      <div className="home">
        <h1>Welcome to Wayve 🚀</h1>
        <p>Your all-in-one platform for Email, Chat, and Scheduling.</p>

        <div className="auth-buttons">
          <button onClick={() => navigate("/")}>Login</button>
          <button onClick={() => navigate("/login")}>Login</button>
          <button onClick={() => navigate("/register")}>Register</button>
        </div>
      </div>
    );
  }

  return (
    <div className="dashboard">
      {/* HEADER */}
      <div className="dashboard-header">
        <h2>Welcome, {user.email} 👋</h2>
      </div>

      {/* GRID */}
      <div className="dashboard-grid">
        {visibleCards.map((card) => (
          <div key={card.path} className="card" onClick={() => navigate(card.path)}>
            <h3>{card.title}</h3>
            <p>{card.description}</p>
          </div>
        ))}
      </div>
    </div>
  );
}
