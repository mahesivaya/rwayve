import { useState } from "react";
import type { FormEvent } from "react";
import { register } from "../api/Auth";
import { useAuth } from "../auth/AuthContext";
import { useNavigate } from "react-router-dom";
import "./login.css"; // ✅ reuse styles

export default function Register() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");
  const [error, setError] = useState("");

  const { login } = useAuth();
  const navigate = useNavigate();

  const handleRegister = async (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError("");

    // ✅ basic validation
    if (password !== confirm) {
      setError("Passwords do not match");
      return;
    }

    try {
      const data = await register(email, password, confirm);

      if (!data || !data.token) {
        throw new Error("No token returned from server");
      }

      localStorage.setItem("token", data.token);
      login(data.token);

      navigate("/emails");
    } catch (err: any) {
      setError(err.message || "Registration failed");
    }
  };

  return (
    <div className="login-page">
      <form className="login-card" onSubmit={handleRegister}>
        <h2>Create account 🚀</h2>
        <p className="subtitle">Join Wayve to get started</p>

        <input
          type="email"
          placeholder="Email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          required
        />

        <input
          type="password"
          placeholder="Password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          required
        />

        <input
          type="password"
          placeholder="Confirm Password"
          value={confirm}
          onChange={(e) => setConfirm(e.target.value)}
          required
        />

        <button type="submit">Register</button>

        {/* ✅ Error message */}
        {error && <p className="error">{error}</p>}

        {/* ✅ Switch to login */}
        <p className="switch-auth">
          Already have an account?{" "}
          <span onClick={() => navigate("/login")}>
            Login
          </span>
        </p>
      </form>
    </div>
  );
}