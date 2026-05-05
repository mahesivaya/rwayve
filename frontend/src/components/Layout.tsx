import { Link, Outlet, useNavigate, useLocation } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
// use emojis for now (avoids lucide issues)
import "./Layout.css";

export default function Layout() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();

  if (!user) return null;

  return (
    <div className="app">
      {/* 🔝 HEADER */}
      <div className="header">
        <div className="logo" onClick={() => navigate("/")}>
          Wayve 🚀
        </div>

        <div className="nav">
          <Link to="/" className={location.pathname === "/" ? "active" : ""}>Home</Link>
          <Link to="/emails" className={location.pathname === "/emails" ? "active" : ""}>Emails</Link>
          <Link to="/chat" className={location.pathname === "/chat" ? "active" : ""}>Chat</Link>
          <Link to="/call" className={location.pathname === "/call" ? "active" : ""}>Call</Link>
          <Link to="/scheduler" className={location.pathname === "/scheduler" ? "active" : ""}>Scheduler</Link>
          <Link to="/drive" className={location.pathname === "/drive" ? "active" : ""}>Files</Link>
        </div>

        <div className="actions">
          <span className="user-email">{user.email}</span>
          <button
            className="logout-btn"
            onClick={() => {
              logout();
              navigate("/login");
            }}
          >
            Logout
          </button>
        </div>
      </div>

      {/* 🔥 BODY */}
      <div className="body">
  {/* LEFT ICON BAR */}
  <div className="icon-sidebar">
    <Link to="/emails">📧</Link>
    <Link to="/chat">💬</Link>
    <Link to="/call">📞</Link>
    <Link to="/scheduler">📅</Link>
    <Link to="/drive">📁</Link>
  </div>

  {/* SECOND SIDEBAR (EMAIL ACCOUNTS) */}
  <div className="account-sidebar">
    {/* your existing accounts UI */}
  </div>

  {/* MAIN CONTENT */}
  <div className="content">
    <Outlet />
  </div>
</div>
    </div>
  );
}