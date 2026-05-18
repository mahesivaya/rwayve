import { FormEvent, useState } from "react";
import { useNavigate } from "react-router-dom";
import { createAdminUser, type AdminCreatedUser } from "../api/admin";
import { slugify, getEmailDomain } from "../auth/accountHome";
import { useAuth } from "../auth/useAuth";
import "../home/home.css";
import "./businessAdmin.css";

// Business admins create accounts for users inside their own business. The
// new account is always a "business" account on the business email domain;
// provisioning businesses + business admins is the project admin's job.
export default function BusinessAdminHome() {
  const { user } = useAuth();
  const navigate = useNavigate();
  const [handle, setHandle] = useState("");
  const [password, setPassword] = useState("");
  const [createdUsers, setCreatedUsers] = useState<AdminCreatedUser[]>([]);
  const [createError, setCreateError] = useState("");
  const [createSuccess, setCreateSuccess] = useState("");
  const [creating, setCreating] = useState(false);

  // New accounts land on the business domain, e.g. john@<business-slug>.com.
  const emailDomain = getEmailDomain(user?.organization_slug || "your-business");

  const createUser = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setCreateError("");
    setCreateSuccess("");
    setCreating(true);

    try {
      const created = await createAdminUser(handle, password);
      setCreatedUsers((prev) => [created, ...prev]);
      setHandle("");
      setPassword("");
      setCreateSuccess(`Created account ${created.email}`);
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
      </div>

      <section className="business-admin-create">
        <div className="business-admin-section-header">
          <div>
            <h2>Create account</h2>
            <p>
              Add a new account inside your business. Enter a handle — the
              email is generated automatically.
            </p>
          </div>
        </div>

        <form className="business-admin-form" onSubmit={createUser}>
          <label>
            <span>Handle</span>
            <input
              value={handle}
              onChange={(event) => setHandle(event.target.value)}
              placeholder="e.g. john"
              required
            />
          </label>

          {handle && (
            <p className="business-admin-hint">
              Login email will be{" "}
              <strong>
                {slugify(handle)}@{emailDomain}
              </strong>
            </p>
          )}

          <label>
            <span>Password</span>
            <input
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder="At least 6 characters"
              minLength={6}
              required
            />
          </label>

          <button type="submit" disabled={creating}>
            {creating ? "Creating..." : "Create account"}
          </button>
        </form>

        {createError && <div className="business-admin-error">{createError}</div>}
        {createSuccess && <div className="business-admin-success">{createSuccess}</div>}

        {createdUsers.length > 0 && (
          <div className="business-admin-created-list">
            {createdUsers.map((created) => (
              <div key={created.id} className="business-admin-created-row">
                <strong>{created.username || created.email}</strong>
                <span>{created.email} · {created.account_type}</span>
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
