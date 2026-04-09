import { useState } from "react";
import type { FormEvent } from "react";
import { login } from "../api/Auth";
import { useNavigate } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import "./login.css";

export default function Login() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const navigate = useNavigate();
  const { login: authLogin } = useAuth();

  const handleLogin = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();

    try {
      const data = await login(email, password);

      if (!data || !data.token) {
        throw new Error("No token returned");
      }

      localStorage.setItem("token", data.token);
      authLogin(data.token);

      navigate("/emails");
    } catch (err) {
      console.error(err);
      alert("Login failed");
    }
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