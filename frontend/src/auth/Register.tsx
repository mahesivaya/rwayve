import { useState } from "react";
import { register } from "../api/Auth";

export default function Register() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirm, setConfirm] = useState("");

  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");

  const handleRegister = async (e: any) => {
    e.preventDefault();

    setError("");
    setSuccess("");

    try {
      await register(email, password, confirm);

      setSuccess("Account created successfully ✅");

      setEmail("");
      setPassword("");
      setConfirm("");

    } catch (err: any) {
      setError(err.message); // ✅ shows backend message
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

      {/* ✅ ERROR MESSAGE */}
      {error && (
        <p style={{ color: "red" }}>
          {error}
        </p>
      )}

      {/* ✅ SUCCESS MESSAGE */}
      {success && (
        <p style={{ color: "green" }}>
          {success}
        </p>
      )}
    </form>
  );
}