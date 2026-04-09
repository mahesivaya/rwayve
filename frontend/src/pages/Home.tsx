import { useAuth } from "../auth/AuthContext";
import { useNavigate } from "react-router-dom";

export default function Home() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  return (
    <div className="home">
      <h1>Welcome to Wayve 🚀</h1>

      {user ? (
        <>
          <p>You are logged in as <b>{user.email}</b></p>

          {/* 🔥 EMAILS */}
          <button onClick={() => navigate("/emails")}>
            Go to Emails
          </button>

          {/* 🔥 CHAT */}
          <button onClick={() => navigate("/chat")}>
            Go to Chat
          </button>

          {/* 🔥 SCHEDULER */}
          <button onClick={() => navigate("/scheduler")}>
            Go to Scheduler
          </button>

          {/* 🔥 DRIVE */}
          <button onClick={() => navigate("/drive")}>
            Go to Drive
          </button>

          {/* ✅ LOGOUT */}
          <button
            onClick={() => {
              logout();
              navigate("/");
            }}
          >
            Logout
          </button>
        </>
      ) : (
        <>
          <p>Your all-in-one platform for Email, Chat, and Scheduling.</p>

          {/* ✅ LOGIN */}
          <button onClick={() => navigate("/login")}>
            Login
          </button>

          {/* ✅ REGISTER (NEW) */}
          <button onClick={() => navigate("/register")}>
            Register
          </button>
        </>
      )}
    </div>
  );
}