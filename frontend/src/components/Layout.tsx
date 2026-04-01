import { Link, Outlet, useNavigate } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";

export default function Layout() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  return (
    <div>
      {/* HEADER */}
      <div style={{ display: "flex", justifyContent: "space-between", padding: 10 }}>
        <div>Wayve</div>

        <div>
          <Link to="/">Home</Link> |{" "}
          <Link to="/emails">Emails</Link> |{" "}
          <Link to="/chat">Chat</Link> |{" "}
          <Link to="/scheduler">Scheduler</Link>
        </div>

        <div>
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

      {/* 🔥 THIS RENDERS CHILD ROUTES */}
      <Outlet />
    </div>
  );
}