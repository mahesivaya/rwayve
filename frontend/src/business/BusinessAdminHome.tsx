import { FormEvent, useState } from "react";
import { useNavigate } from "react-router-dom";
import { createAdminUser, type AdminCreatedUser } from "../api/admin";
import { normalizeAccountType } from "../auth/accountHome";
import { useAuth } from "../auth/useAuth";
import "../home/home.css";
import "./businessAdmin.css";

// Mirrors the backend slugify(): lowercase, ASCII-alphanumeric only.
const slugify = (value: string) => value.toLowerCase().replace(/[^a-z0-9]/g, "");

export default function BusinessAdminHome() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const [handle, setHandle] = useState("");
  const [password, setPassword] = useState("");
  const [accountType, setAccountType] = useState("personal");
  const [organizationName, setOrganizationName] = useState("");
  const [createdUsers, setCreatedUsers] = useState<AdminCreatedUser[]>([]);
  const [createError, setCreateError] = useState("");
  const [createSuccess, setCreateSuccess] = useState("");
  const [creating, setCreating] = useState(false);

  const currentAccountType = normalizeAccountType(user?.account_type);
  const isProjectAdmin = currentAccountType === "project_admin";
  const pageTitle = isProjectAdmin ? "Complete Project Admin page" : "Business Admin page";

  // Email domain the new account will land on — the business domain, or
  // wayve.com for plain personal/project accounts.
  const emailDomain = isProjectAdmin
    ? accountType === "business_admin" && organizationName
      ? `${slugify(organizationName)}.com`
      : "wayve.com"
    : user?.organization_slug
      ? `${user.organization_slug}.com`
      : "your-business.com";

  const createUser = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setCreateError("");
    setCreateSuccess("");
    setCreating(true);

    try {
      const created = await createAdminUser(
        handle,
        password,
        isProjectAdmin ? accountType : "business",
        organizationName
      );
      setCreatedUsers((prev) => [created, ...prev]);
      setHandle("");
      setPassword("");
      setAccountType("personal");
      setOrganizationName("");
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
          <h1>{pageTitle}</h1>
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
            <h2>Create account</h2>
            <p>
              {isProjectAdmin
                ? "Create project admin, business admin, or personal accounts."
                : "Add a new account inside your business. Enter a handle — the email is generated automatically."}
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

          {isProjectAdmin && (
            <>
              <label>
                <span>Account type</span>
                <select
                  value={accountType}
                  onChange={(event) => setAccountType(event.target.value)}
                >
                  <option value="personal">Personal</option>
                  <option value="business_admin">Business admin</option>
                  <option value="project_admin">Project admin</option>
                </select>
              </label>

              {accountType === "business_admin" && (
                <label>
                  <span>Organization</span>
                  <input
                    value={organizationName}
                    onChange={(event) => setOrganizationName(event.target.value)}
                    placeholder="Organization name"
                    required
                  />
                </label>
              )}
            </>
          )}

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
