import { Link, Outlet, useNavigate, useLocation } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import "./layout.css";

export default function Layout() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();

  // 🚨 Extra safety (should already be protected by ProtectedRoute)
  if (!user) return null;

  return (
    <div className="app">
      {/* HEADER */}
      <div className="header">
        {/* ✅ Logo clickable */}
        <div className="logo" onClick={() => navigate("/")}>
          Wayve 🚀
        </div>

        {/* NAVIGATION */}
        <div className="nav">
          <Link
            to="/"
            className={location.pathname === "/" ? "active" : ""}
          >
            Home
          </Link>

          <Link
            to="/emails"
            className={location.pathname === "/emails" ? "active" : ""}
          >
            Emails
          </Link>

          <Link
            to="/chat"
            className={location.pathname === "/chat" ? "active" : ""}
          >
            Chat
          </Link>

          <Link
            to="/scheduler"
            className={location.pathname === "/scheduler" ? "active" : ""}
          >
            Scheduler
          </Link>

          <Link
            to="/drive"
            className={location.pathname === "/drive" ? "active" : ""}
          >
            Files
          </Link>
        </div>

        {/* USER ACTIONS */}
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

      {/* MAIN CONTENT */}
      <div className="content">
        <Outlet />
      </div>
    </div>
  );
}