import { Navigate, useNavigate, useParams } from "react-router-dom";
import { useAuth } from "../auth/useAuth";
import { homePathForUser } from "../auth/accountHome";
import "./businessHome.css";

const FEATURES = [
  { path: "/emails", title: "Mail", desc: "Send and manage business email." },
  { path: "/chat", title: "Team Chat", desc: "Channels and direct messages for the team." },
  { path: "/tasks", title: "Tasks", desc: "Track action items and workflows." },
  { path: "/scheduler", title: "Scheduler", desc: "Plan meetings and team schedules." },
  { path: "/drive", title: "Drive", desc: "Shared files for the business." },
  { path: "/notes", title: "Notes", desc: "Keep shared notes and references." },
];

// Per-business landing page, reached at /business/:slug. The page is business
// aware: it renders the current user's business name and tools. Only the
// business's own members may view it.
export default function BusinessHome() {
  const { slug } = useParams();
  const { user, logout } = useAuth();
  const navigate = useNavigate();

  // Once the slug is resolved, bounce anyone who isn't a member of this
  // business (a different business, or no business at all) back to their home.
  if (user) {
    const isBusinessUser =
      user.account_type === "business_admin" || user.organization_id != null;
    const slugMismatch =
      !!user.organization_slug && user.organization_slug !== slug;
    if (!isBusinessUser || slugMismatch) {
      return <Navigate to={homePathForUser(user)} replace />;
    }
  }

  const isBusinessAdmin = user?.account_type === "business_admin";
  const businessName = user?.organization_name ?? "Business";

  return (
    <div className="business-home">
      <header className="business-home-header">
        <div>
          <h1>{businessName} Home</h1>
          <p>{user?.email}</p>
        </div>
        <div className="business-home-actions">
          {isBusinessAdmin && (
            <button onClick={() => navigate("/business-home")}>
              Manage accounts
            </button>
          )}
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
      </header>

      <section className="business-home-welcome">
        <h2>Welcome to {businessName}</h2>
        <p>Your shared workspace. Jump into any tool below to get started.</p>
      </section>

      <div className="business-home-grid">
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
