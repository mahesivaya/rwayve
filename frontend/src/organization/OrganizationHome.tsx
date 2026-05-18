import { Navigate, useNavigate, useParams } from "react-router-dom";
import { useAuth } from "../auth/useAuth";
import { homePathForUser } from "../auth/accountHome";
import "./organizationHome.css";

const FEATURES = [
  { path: "/emails", title: "Mail", desc: "Send and manage organization email." },
  { path: "/chat", title: "Team Chat", desc: "Channels and direct messages for the team." },
  { path: "/tasks", title: "Tasks", desc: "Track action items and workflows." },
  { path: "/scheduler", title: "Scheduler", desc: "Plan meetings and organization schedules." },
  { path: "/drive", title: "Drive", desc: "Shared files for the organization." },
  { path: "/notes", title: "Notes", desc: "Keep shared notes and references." },
];

// Per-organization landing page, reached at /organization/:slug. The page is
// organization aware: it renders the current user's organization name and
// tools. Only the organization's own members may view it.
export default function OrganizationHome() {
  const { slug } = useParams();
  const { user } = useAuth();
  const navigate = useNavigate();

  // Once the slug is resolved, bounce anyone who isn't a member of this
  // organization (a different organization, or none at all) back to their home.
  if (user) {
    const isOrganizationUser =
      user.account_type === "organization_admin" || user.organization_id != null;
    const slugMismatch =
      !!user.organization_slug && user.organization_slug !== slug;
    if (!isOrganizationUser || slugMismatch) {
      return <Navigate to={homePathForUser(user)} replace />;
    }
  }

  const isOrganizationAdmin = user?.account_type === "organization_admin";
  const organizationName = user?.organization_name ?? "Organization";

  return (
    <div className="organization-home">
      <header className="organization-home-header">
        <div>
          <h1>{organizationName} Home</h1>
          <p>{user?.email}</p>
        </div>
        <div className="organization-home-actions">
          {isOrganizationAdmin && (
            <button onClick={() => navigate("/organization-home")}>
              Manage accounts
            </button>
          )}
        </div>
      </header>

      <section className="organization-home-welcome">
        <h2>Welcome to {organizationName}</h2>
        <p>Your shared workspace. Jump into any tool below to get started.</p>
      </section>

      <div className="organization-home-grid">
        {FEATURES.map((feature) => (
          <article key={feature.path} onClick={() => navigate(feature.path)}>
            <h3>{feature.title}</h3>
            <p>{feature.desc}</p>
          </article>
        ))}
      </div>
    </div>
  );
}
