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

          {/* 🔥 BUTTON TO EMAILS */}
          <button onClick={() => navigate("/emails")}>
            Go to Emails
          </button>


          {/* ✅ LOGOUT BUTTON */}
          <button
            onClick={() => {
              logout();
              navigate("/login");
            }}
          >
            Logout
          </button>

          <p>Go to Emails, Chat, or Scheduler to start.</p>
        </>
      ) : (
        <>
          <p>Your all-in-one platform for Email, Chat, and Scheduling.</p>

          {/* ✅ LOGIN BUTTON */}
          <button onClick={() => navigate("/login")}>
            Login
          </button>
        </>
      )}
    </div>
  );
}