import { Link, Outlet, useNavigate, useLocation } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import { lazy, Suspense, useState } from "react";
import "./Layout.css";

// Lazy-loaded so the split pane doesn't bloat the initial bundle and only
// pays for what the user actually opens in the secondary tab.
const HomeView = lazy(() => import("../home/Home"));
const EmailsView = lazy(() => import("../emails/Emails"));
const ChatView = lazy(() => import("../chat/Chat"));
const CallView = lazy(() => import("../call/Call"));
const SchedulerView = lazy(() => import("../scheduler/Scheduler"));
const DriveView = lazy(() => import("../drive/DriveBox"));

type AppKey = "home" | "emails" | "chat" | "call" | "scheduler" | "drive";

const SPLIT_APPS = [
  { key: "home" as AppKey, label: "Home", path: "/", icon: "🏠", Comp: HomeView },
  { key: "emails" as AppKey, label: "Emails", path: "/emails", icon: "📧", Comp: EmailsView },
  { key: "chat" as AppKey, label: "Chat", path: "/chat", icon: "💬", Comp: ChatView },
  { key: "call" as AppKey, label: "Call", path: "/call", icon: "📞", Comp: CallView },
  { key: "scheduler" as AppKey, label: "Scheduler", path: "/scheduler", icon: "📅", Comp: SchedulerView },
  { key: "drive" as AppKey, label: "Files", path: "/drive", icon: "📁", Comp: DriveView },
];

export default function Layout() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();

  // Max 2 panes: the URL-driven main pane (left) plus one optional split.
  const [splitOpen, setSplitOpen] = useState(false);
  // Right pane starts empty — the user picks an app from the top header,
  // and the click is intercepted to fill this pane instead of navigating.
  const [splitView, setSplitView] = useState<AppKey | null>(null);

  if (!user) return null;

  const splitApp = SPLIT_APPS.find((a) => a.key === splitView) ?? null;
  const SplitComp = splitApp?.Comp ?? null;
  const splitLabel = splitApp?.label ?? null;

  // When the split is open, header link clicks target the right pane instead
  // of navigating the URL. When closed, the link behaves normally.
  const navItem = (path: string, app: AppKey, label: string) => {
    const isMain = !splitOpen && location.pathname === path;
    const isSplit = splitOpen && splitView === app;
    return (
      <Link
        to={path}
        className={[
          isMain ? "active" : "",
          isSplit ? "active-split" : "",
        ].filter(Boolean).join(" ")}
        onClick={(e: { preventDefault: () => void }) => {
          if (splitOpen) {
            e.preventDefault();
            setSplitView(app);
          }
        }}
      >
        {label}
      </Link>
    );
  };

  return (
    <div className="app">
      {/* 🔝 HEADER */}
      <div className="header">
        <div className="logo" onClick={() => navigate("/")}>Wayve 🚀</div>

        <div className="nav">
          {navItem("/", "home", "Home")}
          {navItem("/emails", "emails", "Emails")}
          {navItem("/chat", "chat", "Chat")}
          {navItem("/call", "call", "Call")}
          {navItem("/scheduler", "scheduler", "Scheduler")}
          {navItem("/drive", "drive", "Files")}
        </div>

        <div className="actions">
          {splitOpen && (
            <span className="split-hint">
              {splitView ? `↗ Split: ${splitLabel}` : "↗ Split open — pick an app"}
            </span>
          )}
          <span className="user-email">{user.email}</span>
          <button
            className="logout-btn"
            onClick={() => {
              logout();
              navigate("/login");
            }}
          >
            Logout
          </button>
        </div>
      </div>

      {/* 🔥 BODY */}
      <div className="body">
        {/* LEFT ICON BAR */}
        <div className="icon-sidebar">
          <Link to="/emails">📧</Link>
          <Link to="/chat">💬</Link>
          <Link to="/call">📞</Link>
          <Link to="/scheduler">📅</Link>
          <Link to="/drive">📁</Link>

          <div className="icon-sidebar-spacer" />

          <button
            className={`icon-split-btn ${splitOpen ? "active" : ""}`}
            onClick={() => {
              setSplitOpen((s) => !s);
              if (splitOpen) setSplitView(null);
            }}
            title={splitOpen ? "Close split view" : "Open split view"}
            aria-label={splitOpen ? "Close split view" : "Open split view"}
          >
            ⫼
          </button>
        </div>

        {/* SECOND SIDEBAR (EMAIL ACCOUNTS) */}
        <div className="account-sidebar"></div>

        {/* MAIN CONTENT */}
        <div className={`content ${splitOpen ? "split" : ""}`}>
          <div className="split-pane left">
            <Outlet />
          </div>

          {splitOpen && (
            <div className="split-pane right">
              <button
                className="split-close-floating"
                onClick={() => {
                  setSplitOpen(false);
                  setSplitView(null);
                }}
                title="Close split"
                aria-label="Close split"
              >
                ✕
              </button>

              <div className="split-pane-body">
                {SplitComp ? (
                  <Suspense fallback={<div className="split-loading">Loading…</div>}>
                    <SplitComp />
                  </Suspense>
                ) : (
                  <div className="split-empty">
                    <div className="split-empty-icon">⫼</div>
                    <div className="split-empty-title">Split view ready</div>
                    <div className="split-empty-hint">
                      Pick an app from the top header to load it here.
                    </div>
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
