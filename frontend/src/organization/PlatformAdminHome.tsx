import { FormEvent, useEffect, useState } from "react";
import {
  createAdminOrganization,
  listAdminOrganizations,
  type AdminOrganization,
} from "../api/admin";
import { useAuth } from "../auth/useAuth";
import { slugify } from "../auth/accountHome";
import "./platformAdmin.css";

export default function PlatformAdminHome() {
  const { user } = useAuth();
  const [organizationName, setOrganizationName] = useState("");
  const [adminHandle, setAdminHandle] = useState("");
  const [adminPassword, setAdminPassword] = useState("");
  const [organizations, setOrganizations] = useState<AdminOrganization[]>([]);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");

  useEffect(() => {
    let alive = true;

    listAdminOrganizations()
      .then((items) => {
        if (alive) setOrganizations(items);
      })
      .catch((err) => {
        if (alive) {
          setError(err instanceof Error ? err.message : "Failed to load organizations");
        }
      })
      .finally(() => {
        if (alive) setLoading(false);
      });

    return () => {
      alive = false;
    };
  }, []);

  const createOrganization = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setError("");
    setSuccess("");
    setCreating(true);

    const adminEmail = `${slugify(adminHandle)}@${slugify(organizationName)}.com`;

    try {
      const created = await createAdminOrganization({
        name: organizationName,
        adminUsername: adminHandle,
        adminEmail,
        adminPassword,
      });
      setOrganizations((prev) => {
        const exists = prev.some((item) => item.id === created.id);
        return exists
          ? prev.map((item) => (item.id === created.id ? created : item))
          : [...prev, created].sort((a, b) => a.name.localeCompare(b.name));
      });
      setOrganizationName("");
      setAdminHandle("");
      setAdminPassword("");
      setSuccess(
        `Created organization ${created.name}` +
          (created.admin ? ` with admin ${created.admin.email}` : "")
      );
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create organization");
    } finally {
      setCreating(false);
    }
  };

  return (
    <div className="platform-admin-home">
      <div className="platform-admin-header">
        <div>
          <h1>Platform Admin Home</h1>
          <p>{user?.email}</p>
        </div>
      </div>

      <section className="platform-admin-panel">
        <div className="platform-admin-section-header">
          <div>
            <h2>Create organization</h2>
            <p>Add a new organization and provision its primary administrator account.</p>
          </div>
        </div>

        <form className="platform-admin-form" onSubmit={createOrganization}>
          <label>
            <span>Organization name</span>
            <input
              value={organizationName}
              onChange={(event) => setOrganizationName(event.target.value)}
              placeholder="Enter organization name"
              required
            />
          </label>
          <label>
            <span>Organization admin handle</span>
            <input
              value={adminHandle}
              onChange={(event) => setAdminHandle(event.target.value)}
              placeholder="e.g. john"
              required
            />
          </label>
          {adminHandle && organizationName && (
            <p className="platform-admin-hint">
              Login email will be{" "}
              <strong>
                {slugify(adminHandle)}@{slugify(organizationName)}.com
              </strong>
            </p>
          )}
          <label>
            <span>Organization admin password</span>
            <input
              type="password"
              value={adminPassword}
              onChange={(event) => setAdminPassword(event.target.value)}
              placeholder="At least 6 characters"
              minLength={6}
              required
            />
          </label>
          <button type="submit" disabled={creating}>
            {creating ? "Creating..." : "Create Organization"}
          </button>
        </form>

        {error && <div className="platform-admin-error">{error}</div>}
        {success && <div className="platform-admin-success">{success}</div>}
      </section>

      <section className="platform-admin-panel">
        <div className="platform-admin-section-header">
          <div>
            <h2>Organization names</h2>
            <p>All organizations currently available on the platform.</p>
          </div>
          <span>{organizations.length} total</span>
        </div>

        {loading ? (
          <div className="platform-admin-empty">Loading organizations...</div>
        ) : organizations.length === 0 ? (
          <div className="platform-admin-empty">No organizations created yet.</div>
        ) : (
          <div className="organization-name-list">
            {organizations.map((org) => (
              <article key={org.id}>
                <strong>{org.name}</strong>
                <span>
                  {org.slug ? `${org.slug}.com · ` : ""}
                  {org.user_count} users
                  {org.admin && (
                    <>
                      <br /><small style={{ color: '#6b7280' }}>Admin: {org.admin.email}</small>
                    </>
                  )}
                </span>
              </article>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
