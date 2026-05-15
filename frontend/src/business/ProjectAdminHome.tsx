import { FormEvent, useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import {
  createAdminOrganization,
  listAdminOrganizations,
  type AdminOrganization,
} from "../api/admin";
import { useAuth } from "../auth/useAuth";
import "./projectAdmin.css";

export default function ProjectAdminHome() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const [businessName, setBusinessName] = useState("");
  const [adminUsername, setAdminUsername] = useState("");
  const [adminEmail, setAdminEmail] = useState("");
  const [adminPassword, setAdminPassword] = useState("");
  const [businesses, setBusinesses] = useState<AdminOrganization[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");

  useEffect(() => {
    let alive = true;

    listAdminOrganizations()
      .then((items) => {
        if (alive) setBusinesses(items);
      })
      .catch((err) => {
        if (alive) {
          setError(err instanceof Error ? err.message : "Failed to load businesses");
        }
      })
      .finally(() => {
        if (alive) setLoading(false);
      });

    return () => {
      alive = false;
    };
  }, []);

  const createBusiness = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setError("");
    setSuccess("");
    setSaving(true);

    try {
      const created = await createAdminOrganization({
        name: businessName,
        adminUsername,
        adminEmail,
        adminPassword,
      });
      setBusinesses((prev) => {
        const exists = prev.some((item) => item.id === created.id);
        return exists
          ? prev.map((item) => (item.id === created.id ? created : item))
          : [...prev, created].sort((a, b) => a.name.localeCompare(b.name));
      });
      setBusinessName("");
      setAdminUsername("");
      setAdminEmail("");
      setAdminPassword("");
      setSuccess(
        `Created business ${created.name}` +
          (created.admin ? ` with admin ${created.admin.email}` : "")
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create business");
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="project-admin-home">
      <div className="project-admin-header">
        <div>
          <h1>Project Admin Home</h1>
          <p>{user?.email}</p>
        </div>
        <div className="project-admin-header-actions">
          <button onClick={() => navigate("/business-home")}>Create accounts</button>
          <button
            className="danger"
            onClick={() => {
              logout();
              navigate("/login");
            }}
          >
            Logout
          </button>
        </div>
      </div>

      <section className="project-admin-panel">
        <div className="project-admin-section-header">
          <div>
            <h2>Create business</h2>
            <p>Add a new business organization and provision its business admin account.</p>
          </div>
        </div>

        <form className="project-admin-form" onSubmit={createBusiness}>
          <label>
            <span>Business name</span>
            <input
              value={businessName}
              onChange={(event) => setBusinessName(event.target.value)}
              placeholder="Enter business name"
              required
            />
          </label>
          <label>
            <span>Business admin username</span>
            <input
              value={adminUsername}
              onChange={(event) => setAdminUsername(event.target.value)}
              placeholder="Enter business admin username"
              required
            />
          </label>
          <label>
            <span>Business admin email</span>
            <input
              type="email"
              value={adminEmail}
              onChange={(event) => setAdminEmail(event.target.value)}
              placeholder="Enter business admin email"
              required
            />
          </label>
          <label>
            <span>Business admin password</span>
            <input
              type="password"
              value={adminPassword}
              onChange={(event) => setAdminPassword(event.target.value)}
              placeholder="At least 6 characters"
              minLength={6}
              required
            />
          </label>
          <button type="submit" disabled={saving}>
            {saving ? "Creating..." : "Create business"}
          </button>
        </form>

        {error && <div className="project-admin-error">{error}</div>}
        {success && <div className="project-admin-success">{success}</div>}
      </section>

      <section className="project-admin-panel">
        <div className="project-admin-section-header">
          <div>
            <h2>Business names</h2>
            <p>All businesses currently available in the project.</p>
          </div>
          <span>{businesses.length} total</span>
        </div>

        {loading ? (
          <div className="project-admin-empty">Loading businesses...</div>
        ) : businesses.length === 0 ? (
          <div className="project-admin-empty">No businesses created yet.</div>
        ) : (
          <div className="business-name-list">
            {businesses.map((business) => (
              <article key={business.id}>
                <strong>{business.name}</strong>
                <span>{business.user_count} users</span>
              </article>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
