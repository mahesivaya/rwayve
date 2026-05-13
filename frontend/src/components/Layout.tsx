import { Link, Outlet, useNavigate, useLocation } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import { lazy, Suspense, useEffect, useMemo, useRef, useState } from "react";
import { SearchContext } from "../search/SearchContext";
import "./Layout.css";

// Lazy-loaded so the split pane doesn't bloat the initial bundle and only
// pays for what the user actually opens in the secondary tab.
const HomeView = lazy(() => import("../home/Home"));
const EmailsView = lazy(() => import("../emails/Emails"));
const ChatView = lazy(() => import("../chat/Chat"));
const CallView = lazy(() => import("../call/Call"));
const SchedulerView = lazy(() => import("../scheduler/Scheduler"));
const DriveView = lazy(() => import("../drive/DriveBox"));
const NotesView = lazy(() => import("../notes/Notes"));
const AIChatView = lazy(() => import("../aichat/AIChat"));

type AppKey = "home" | "emails" | "chat" | "call" | "scheduler" | "drive" | "notes" | "aichat";

const SPLIT_APPS = [
  { key: "home" as AppKey, label: "Home", path: "/", icon: "🏠", Comp: HomeView },
  { key: "emails" as AppKey, label: "Emails", path: "/emails", icon: "📧", Comp: EmailsView },
  { key: "chat" as AppKey, label: "Chat", path: "/chat", icon: "💬", Comp: ChatView },
  { key: "call" as AppKey, label: "Call", path: "/call", icon: "📞", Comp: CallView },
  { key: "scheduler" as AppKey, label: "Scheduler", path: "/scheduler", icon: "📅", Comp: SchedulerView },
  { key: "drive" as AppKey, label: "Files", path: "/drive", icon: "📁", Comp: DriveView },
  { key: "notes" as AppKey, label: "Notes", path: "/notes", icon: "📝", Comp: NotesView },
  { key: "aichat" as AppKey, label: "AI Chat", path: "/aichat", icon: "✨", Comp: AIChatView },
];

const SEARCH_LABELS: Record<string, string> = {
  "/home": "home",
  "/emails": "all emails",
  "/email-files": "email files",
  "/chat": "users and messages",
  "/call": "calls",
  "/scheduler": "meetings",
  "/drive": "files",
  "/notes": "notes",
  "/aichat": "AI chat",
  "/profile": "profile",
  "/settings": "settings",
};

export default function Layout() {
  const { user, logout } = useAuth();
  const navigate = useNavigate();
  const location = useLocation();

  // Max 2 panes: the URL-driven main pane (left) plus one optional split.
  const [splitOpen, setSplitOpen] = useState(false);
  // Right pane starts empty — the user picks an app from the top header,
  // and the click is intercepted to fill this pane instead of navigating.
  const [splitView, setSplitView] = useState<AppKey | null>(null);
  // When split is open, this decides which pane the next header-link click
  // affects. "left" → normal URL navigation (Outlet updates). "right" →
  // preventDefault and set splitView. Default to "right" so opening split
  // and clicking an app feels like adding a second view.
  const [splitTarget, setSplitTarget] = useState<"left" | "right">("right");
  const [searchQuery, setSearchQuery] = useState("");

  // Profile dropdown (replaces the standalone Logout button).
  const [menuOpen, setMenuOpen] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  // Close the dropdown on any click outside its container.
  useEffect(() => {
    if (!menuOpen) return;
    const onDocClick = (e: { target: any }) => {
      if (menuRef.current && !menuRef.current.contains(e.target)) {
        setMenuOpen(false);
      }
    };
    document.addEventListener("mousedown", onDocClick);
    return () => document.removeEventListener("mousedown", onDocClick);
  }, [menuOpen]);

  if (!user) return null;

  const splitApp = SPLIT_APPS.find((a) => a.key === splitView) ?? null;
  const SplitComp = splitApp?.Comp ?? null;
  const splitLabel = splitApp?.label ?? null;
  const searchLabel = SEARCH_LABELS[location.pathname] ?? "this page";
  const normalizedSearchQuery = searchQuery.trim().toLowerCase();
  const searchValue = useMemo(
    () => ({ searchQuery, normalizedSearchQuery, setSearchQuery }),
    [searchQuery, normalizedSearchQuery]
  );

  // When the split is open, header link clicks target the right pane instead
  // of navigating the URL. When closed, the link behaves normally.
  const navItem = (path: string, app: AppKey, label: string) => {
    const isLeftActive = location.pathname === path;
    const isRightActive = splitOpen && splitView === app;
    return (
      <Link
        to={path}
        className={[
          isLeftActive ? "active" : "",
          isRightActive ? "active-split" : "",
        ].filter(Boolean).join(" ")}
        onClick={(e: { preventDefault: () => void }) => {
          // Split open + target=right → load into right pane (no URL change).
          // Split open + target=left  → fall through to normal Link nav so
          // the URL updates and Outlet re-renders the left pane.
          // Split closed → normal Link nav.
          if (splitOpen && splitTarget === "right") {
            e.preventDefault();
            setSplitView(app);
          }
        }}
      >
        {label}
      </Link>
    );
  };

  const splitToggleButton = (
    <button
      type="button"
      className={`split-toggle-btn ${splitOpen ? "active" : ""}`}
      onClick={() => {
        setSplitOpen((s) => !s);
        if (splitOpen) setSplitView(null);
      }}
      title={splitOpen ? "Close split view" : "Open split view"}
      aria-label={splitOpen ? "Close split view" : "Open split view"}
    >
      <span className="split-btn-icon" aria-hidden="true">
        <span className="split-btn-pane" />
        <span className="split-btn-divider">◫</span>
        <span className="split-btn-pane" />
      </span>
    </button>
  );

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
          {navItem("/notes", "notes", "Notes")}
          {navItem("/aichat", "aichat", "AI Chat")}
        </div>

        <div className="actions">
          {splitOpen && (
            <div className="split-target" role="group" aria-label="Header click target">
              <button
                type="button"
                className={`split-target-btn ${splitTarget === "left" ? "active" : ""}`}
                onClick={() => setSplitTarget("left")}
                title="Next click loads into the LEFT pane (URL)"
              >
                ← Left
              </button>
              <button
                type="button"
                className={`split-target-btn ${splitTarget === "right" ? "active" : ""}`}
                onClick={() => setSplitTarget("right")}
                title="Next click loads into the RIGHT pane"
              >
                Right →
              </button>
            </div>
          )}
          {splitOpen && splitView && (
            <span className="split-hint" title={`Right pane: ${splitLabel}`}>
              ↗ {splitLabel}
            </span>
          )}

          {splitToggleButton}

          <div className="profile-menu" ref={menuRef}>
            <button
              className="profile-trigger"
              onClick={() => setMenuOpen((o: boolean) => !o)}
              aria-haspopup="true"
              aria-expanded={menuOpen}
              title={user.email}
            >
              <span className="profile-avatar">
                {(user.email?.[0] ?? "?").toUpperCase()}
              </span>
              <span className="profile-trigger-caret">▾</span>
            </button>

            {menuOpen && (
              <div className="profile-dropdown" role="menu">
                <div className="profile-dropdown-header">
                  <div className="profile-dropdown-name">{user.email}</div>
                </div>

                <button
                  className="profile-dropdown-item"
                  onClick={() => {
                    setMenuOpen(false);
                    navigate("/profile");
                  }}
                >
                  <span className="profile-dropdown-icon">👤</span>
                  My Profile
                </button>

                <button
                  className="profile-dropdown-item"
                  onClick={() => {
                    setMenuOpen(false);
                    navigate("/settings");
                  }}
                >
                  <span className="profile-dropdown-icon">⚙️</span>
                  Settings & Privacy
                </button>

                <div className="profile-dropdown-divider" />

                <button
                  className="profile-dropdown-item profile-dropdown-logout"
                  onClick={() => {
                    setMenuOpen(false);
                    logout();
                    navigate("/login");
                  }}
                >
                  <span className="profile-dropdown-icon">⏻</span>
                  Log out
                </button>
              </div>
            )}
          </div>
        </div>
      </div>

      <SearchContext.Provider value={searchValue}>
      <div className="global-search-row">
        <div className="global-search-box">
          <span className="global-search-icon" aria-hidden="true">⌕</span>
          <input
            type="search"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder={`Search ${searchLabel}`}
            aria-label={`Search ${searchLabel}`}
          />
          {searchQuery && (
            <button
              type="button"
              className="global-search-clear"
              onClick={() => setSearchQuery("")}
              title="Clear search"
              aria-label="Clear search"
            >
              ×
            </button>
          )}
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
          <Link to="/notes">📝</Link>
          <Link to="/aichat">✨</Link>

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
            <span className="split-btn-icon" aria-hidden="true">
              <span className="split-btn-pane" />
              <span className="split-btn-divider">◫</span>
              <span className="split-btn-pane" />
            </span>
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
              <div className="split-pane-toolbar">
                <span className="split-pane-title">
                  {splitLabel ? `${splitLabel} tab` : "Split tab"}
                </span>
                <button
                  className="split-close-btn"
                  onClick={() => {
                    setSplitOpen(false);
                    setSplitView(null);
                  }}
                  title="Close extra tab"
                  aria-label="Close extra tab"
                >
                  ✕ Close tab
                </button>
              </div>

              <div className="split-pane-body">
                {SplitComp ? (
                  <Suspense fallback={<div className="split-loading">Loading…</div>}>
                    <SplitComp />
                  </Suspense>
                ) : (
                  <div className="split-empty">
                    <div className="split-empty-icon">◫</div>
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
      </SearchContext.Provider>
    </div>
  );
}
