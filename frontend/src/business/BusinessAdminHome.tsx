import { FormEvent, useState } from "react";
import { useNavigate } from "react-router-dom";
import { createAdminUser, type AdminCreatedUser } from "../api/admin";
import { useAuth } from "../auth/AuthContext";
import "../home/home.css";
import "./businessAdmin.css";

export default function BusinessAdminHome() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [createdUsers, setCreatedUsers] = useState<AdminCreatedUser[]>([]);
  const [createError, setCreateError] = useState("");
  const [createSuccess, setCreateSuccess] = useState("");
  const [creating, setCreating] = useState(false);

  const createUser = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setCreateError("");
    setCreateSuccess("");
    setCreating(true);

    try {
      const created = await createAdminUser(username, email, password);
      setCreatedUsers((prev) => [created, ...prev]);
      setUsername("");
      setEmail("");
      setPassword("");
      setCreateSuccess(`Created user ${created.email}`);
    } catch (err) {
      setCreateError(err instanceof Error ? err.message : "Failed to create user");
    } finally {
      setCreating(false);
    }
  };

  return (
    <div className="business-admin-home">
      <div className="business-admin-header">
        <div>
          <h1>Business Admin page</h1>
          <p>{user?.email}</p>
        </div>
        <button
          onClick={() => {
            logout();
            navigate("/login");
          }}
        >
          Logout
        </button>
      </div>

      <section className="business-admin-create">
        <div className="business-admin-section-header">
          <div>
            <h2>Create user</h2>
            <p>Add a new personal account with username, email, and password.</p>
          </div>
        </div>

        <form className="business-admin-form" onSubmit={createUser}>
          <label>
            <span>Username</span>
            <input
              value={username}
              onChange={(event) => setUsername(event.target.value)}
              placeholder="Enter username"
              required
            />
          </label>

          <label>
            <span>Email</span>
            <input
              type="email"
              value={email}
              onChange={(event) => setEmail(event.target.value)}
              placeholder="Enter email"
              required
            />
          </label>

          <label>
            <span>Password</span>
            <input
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder="Enter password"
              required
            />
          </label>

          <button type="submit" disabled={creating}>
            {creating ? "Creating..." : "Create user"}
          </button>
        </form>

        {createError && <div className="business-admin-error">{createError}</div>}
        {createSuccess && <div className="business-admin-success">{createSuccess}</div>}

        {createdUsers.length > 0 && (
          <div className="business-admin-created-list">
            {createdUsers.map((created) => (
              <div key={created.id} className="business-admin-created-row">
                <strong>{created.username || created.email}</strong>
                <span>{created.email}</span>
              </div>
            ))}
          </div>
        )}
      </section>

      <div className="business-admin-grid">
        <article onClick={() => navigate("/emails")}>
          <h3>Mail</h3>
          <p>Manage business communication from the shared workspace.</p>
        </article>
        <article onClick={() => navigate("/chat")}>
          <h3>Team Chat</h3>
          <p>Create channels, manage members, and coordinate team work.</p>
        </article>
        <article onClick={() => navigate("/tasks")}>
          <h3>Tasks</h3>
          <p>Create and track action items for business workflows.</p>
        </article>
        <article onClick={() => navigate("/scheduler")}>
          <h3>Scheduler</h3>
          <p>Review meetings and plan team schedules.</p>
        </article>
      </div>
    </div>
  );
}
