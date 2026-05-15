import { logger } from "../utils/logger";
import { useEffect, useState } from "react";
import type { FormEvent } from "react";
import { login } from "../api/Auth";
import { useNavigate, useSearchParams } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import { homePathForAccount } from "../auth/accountHome";
import { API_BASE } from "../config";
import "./login.css";

export default function Login() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [params] = useSearchParams();
  const navigate = useNavigate();
  const { login: authLogin } = useAuth();

  // Surface OAuth-side errors that the backend redirected here with.
  useEffect(() => {
    if (params.get("error") === "email_exists") {
      setError("This email is already registered with a password. Please login.");
    }
  }, [params]);

  const handleLogin = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError("");

    try {
      const data = await login(email, password);

      if (!data || !data.token) {
        throw new Error("No token returned");
      }

      authLogin(data.token, data.account_type ?? "personal");

      navigate(homePathForAccount(data.account_type));
    } catch (err) {
      logger.error(err);
      setError("Login failed. Check your credentials.");
    }
  };

  const handleGoogle = () => {
    window.location.href = `${API_BASE}/gmail/login?mode=signup`;
  };

  return (
    <div className="login-page">
      <form className="login-card" onSubmit={handleLogin}>
        <h2>Welcome back 👋</h2>
        <p className="subtitle">Login to your Wayve account</p>

        <input
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          placeholder="Email"
          required
        />

        <input
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder="Password"
          required
        />

        <button type="submit">Login</button>

        <div className="auth-divider"><span>or</span></div>

        <button
          type="button"
          className="google-btn"
          onClick={handleGoogle}
        >
          Continue with Gmail
        </button>

        {error && <p className="error">{error}</p>}

        <p className="switch-auth">
          <span onClick={() => navigate("/forgot-password")}>
            Forgot password?
          </span>
        </p>

        <p className="switch-auth">
          Don’t have an account?{" "}
          <span onClick={() => navigate("/register")}>
            Register
          </span>
        </p>
      </form>
    </div>
  );
}
