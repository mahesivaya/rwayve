import { useAuth } from "../auth/AuthContext";
import { Link, Outlet, useNavigate } from "react-router-dom";

export default function Header() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  return (
    <div>
      {user ? (
        <>
          <span>{user.email}</span>
          <button onClick={() => {
    logout();
    navigate("/login");
  }}>Logout</button>
        </>
      ) : (
        <button>Login</button>
      )}
    </div>
  );
}