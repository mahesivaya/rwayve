import { Link, Outlet, useNavigate } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import "./layout.css";

export default function Layout() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  return (
    <div className="app">
      {/* HEADER */}
      <div className="header">
        <div className="logo">Wayve</div>

        <div className="nav">
          <Link to="/">Home</Link>
          <Link to="/emails">Emails</Link>
          <Link to="/chat">Chat</Link>
          <Link to="/scheduler">Scheduler</Link>
        </div>

        <div className="actions">
          {user && (
            <>
              <span>{user.email}</span>
              <button
                onClick={() => {
                  logout();
                  navigate("/login");
                }}
              >
                Logout
              </button>
            </>
          )}
        </div>
      </div>

      {/* ✅ IMPORTANT WRAPPER */}
      <div className="content">
        <Outlet />
      </div>
    </div>
  );
}