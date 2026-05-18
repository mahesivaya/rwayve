import { FormEvent, useEffect, useState } from "react";
import {
  createAdminOrganization,
  listAdminOrganizations,
  type AdminOrganization,
} from "../api/admin";
import { useAuth } from "../auth/useAuth";
import { slugify } from "../auth/accountHome";
import "./projectAdmin.css";

export default function PlatformAdminHome() {
  const { user } = useAuth();
  const [businessName, setBusinessName] = useState("");
  const [adminHandle, setAdminHandle] = useState("");
  const [adminPassword, setAdminPassword] = useState("");
  const [businesses, setBusinesses] = useState<AdminOrganization[]>([]);
  const [loading, setLoading] = useState(true);
  const [creating, setCreating] = useState(false);
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
    setCreating(true);

    try {
      const created = await createAdminOrganization({
        name: businessName,
        adminHandle,
        adminPassword,
      });
      setBusinesses((prev) => {
        const exists = prev.some((item) => item.id === created.id);
        return exists
          ? prev.map((item) => (item.id === created.id ? created : item))
          : [...prev, created].sort((a, b) => a.name.localeCompare(b.name));
      });
      setBusinessName("");
      setAdminHandle("");
      setAdminPassword("");
      setSuccess(
        `Created business ${created.name}` +
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
            <p>Add a new business organization and provision its business admin account.</p>
          </div>
        </div>

        <form className="platform-admin-form" onSubmit={createBusiness}>
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
            <span>Business admin handle</span>
            <input
              value={adminHandle}
              onChange={(event) => setAdminHandle(event.target.value)}
              placeholder="e.g. john"
              required
            />
          </label>
          {adminHandle && businessName && (
            <p className="platform-admin-hint">
              Login email will be{" "}
              <strong>
                {slugify(adminHandle)}@{slugify(businessName)}.com
              </strong>
            </p>
          )}
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
          <button type="submit" disabled={creating}>
            {creating ? "Creating..." : "Create organization"}
          </button>
        </form>

        {error && <div className="platform-admin-error">{error}</div>}
        {success && <div className="platform-admin-success">{success}</div>}
      </section>

      <section className="platform-admin-panel">
        <div className="platform-admin-section-header">
          <div>
            <h2>Business names</h2>
            <p>All businesses currently available in the project.</p>
          </div>
          <span>{businesses.length} total</span>
        </div>

        {loading ? (
          <div className="platform-admin-empty">Loading businesses...</div>
        ) : businesses.length === 0 ? (
          <div className="platform-admin-empty">No businesses created yet.</div>
        ) : (
          <div className="business-name-list">
            {businesses.map((business) => (
              <article key={business.id}>
                <strong>{business.name}</strong>
                <span>
                  {business.slug ? `${business.slug}.com · ` : ""}
                  {business.user_count} users
                </span>
              </article>
            ))}
          </div>
        )}
      </section>
    </div>
  );
}
