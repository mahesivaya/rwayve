import { Link, Outlet, useNavigate } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import "./Layout.css";
export default function Layout() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  return (
    <div className="app">
      
      <div className="header">
        <div className="logo">Wayve</div>

        <div className="nav">
          <Link to="/">Home</Link>
          <Link to="/emails">Emails</Link>
          <Link to="/chat">Chat</Link>
          <Link to="/scheduler">Scheduler</Link>
        </div>

        <div className="actions">
          {user ? (
            <>
              <span style={{ marginRight: 10 }}>{user.email}</span>
              <button onClick={logout}>Logout</button>
            </>
          ) : (
            <>
              <button onClick={() => navigate("/login")}>
                Login
              </button>

              <button onClick={() => navigate("/register")}>
                Register
              </button>
            </>
          )}
        </div>
      </div>

      <div className="content">
        <Outlet />
      </div>
    </div>
  );
}