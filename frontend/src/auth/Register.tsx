import { useState } from "react";
import { register } from "../api/Auth";
import { useAuth } from "../auth/AuthContext";
import { useNavigate } from "react-router-dom";

export default function Register() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");

  const [error, setError] = useState("");

  const { login } = useAuth(); // 🔥 important
  const navigate = useNavigate();

  const handleRegister = async (e: any) => {
    e.preventDefault();

    setError("");

    try {
      const data = await register(email, password, confirm);

      // 🔥 SAVE TOKEN
      localStorage.setItem("token", data.token);

      // 🔥 UPDATE GLOBAL STATE
      login(data.token);

      // 🔥 REDIRECT
      navigate("/emails");

    } catch (err: any) {
      setError(err.message);
    }
  };

  return (
    <form onSubmit={handleRegister}>
      <h2>Register</h2>

      <input
        placeholder="Email"
        value={email}
        onChange={(e) => setEmail(e.target.value)}
      />

      <input
        type="password"
        placeholder="Password"
        value={password}
        onChange={(e) => setPassword(e.target.value)}
      />

      <input
        type="password"
        placeholder="Confirm Password"
        value={confirm}
        onChange={(e) => setConfirm(e.target.value)}
      />

      <button type="submit">Register</button>

      {error && <p style={{ color: "red" }}>{error}</p>}
    </form>
  );
}