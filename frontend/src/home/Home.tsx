import { useAuth } from "../auth/AuthContext";
import { useNavigate } from "react-router-dom";
import "./home.css";

export default function Home() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  if (!user) {
    return (
      <div className="home">
        <h1>Welcome to Wayve 🚀</h1>
        <p>Your all-in-one platform for Email, Chat, and Scheduling.</p>

        <div className="auth-buttons">
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
        <div className="card" onClick={() => navigate("/emails")}>
          <h3>📧 Emails</h3>
          <p>View and send emails</p>
        </div>

        <div className="card" onClick={() => navigate("/chat")}>
          <h3>💬 Chat</h3>
          <p>Real-time messaging</p>
        </div>

        <div className="card" onClick={() => navigate("/call")}>
          <h3> 📞  🎥 Call</h3>
          <p>Real-time calling</p>
        </div>

        <div className="card" onClick={() => navigate("/scheduler")}>
          <h3>📅 Scheduler</h3>
          <p>Manage your meetings</p>
        </div>

        <div className="card" onClick={() => navigate("/drive")}>
          <h3>📁 Drive</h3>
          <p>Store and manage files</p>
        </div>
      </div>
    </div>
  );
}