import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
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
const SPLIT_APPS = [
    { key: "home", label: "Home", path: "/", icon: "🏠", Comp: HomeView },
    { key: "emails", label: "Emails", path: "/emails", icon: "📧", Comp: EmailsView },
    { key: "chat", label: "Chat", path: "/chat", icon: "💬", Comp: ChatView },
    { key: "call", label: "Call", path: "/call", icon: "📞", Comp: CallView },
    { key: "scheduler", label: "Scheduler", path: "/scheduler", icon: "📅", Comp: SchedulerView },
    { key: "drive", label: "Files", path: "/drive", icon: "📁", Comp: DriveView },
    { key: "notes", label: "Notes", path: "/notes", icon: "📝", Comp: NotesView },
    { key: "aichat", label: "AI Chat", path: "/aichat", icon: "✨", Comp: AIChatView },
];
const SEARCH_LABELS = {
    "/home": "home",
    "/emails": "all emails",
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
    const [splitView, setSplitView] = useState(null);
    // When split is open, this decides which pane the next header-link click
    // affects. "left" → normal URL navigation (Outlet updates). "right" →
    // preventDefault and set splitView. Default to "right" so opening split
    // and clicking an app feels like adding a second view.
    const [splitTarget, setSplitTarget] = useState("right");
    const [searchQuery, setSearchQuery] = useState("");
    // Profile dropdown (replaces the standalone Logout button).
    const [menuOpen, setMenuOpen] = useState(false);
    const menuRef = useRef(null);
    // Close the dropdown on any click outside its container.
    useEffect(() => {
        if (!menuOpen)
            return;
        const onDocClick = (e) => {
            if (menuRef.current && !menuRef.current.contains(e.target)) {
                setMenuOpen(false);
            }
        };
        document.addEventListener("mousedown", onDocClick);
        return () => document.removeEventListener("mousedown", onDocClick);
    }, [menuOpen]);
    if (!user)
        return null;
    const splitApp = SPLIT_APPS.find((a) => a.key === splitView) ?? null;
    const SplitComp = splitApp?.Comp ?? null;
    const splitLabel = splitApp?.label ?? null;
    const searchLabel = SEARCH_LABELS[location.pathname] ?? "this page";
    const normalizedSearchQuery = searchQuery.trim().toLowerCase();
    const searchValue = useMemo(() => ({ searchQuery, normalizedSearchQuery, setSearchQuery }), [searchQuery, normalizedSearchQuery]);
    // When the split is open, header link clicks target the right pane instead
    // of navigating the URL. When closed, the link behaves normally.
    const navItem = (path, app, label) => {
        const isLeftActive = location.pathname === path;
        const isRightActive = splitOpen && splitView === app;
        return (_jsx(Link, { to: path, className: [
                isLeftActive ? "active" : "",
                isRightActive ? "active-split" : "",
            ].filter(Boolean).join(" "), onClick: (e) => {
                // Split open + target=right → load into right pane (no URL change).
                // Split open + target=left  → fall through to normal Link nav so
                // the URL updates and Outlet re-renders the left pane.
                // Split closed → normal Link nav.
                if (splitOpen && splitTarget === "right") {
                    e.preventDefault();
                    setSplitView(app);
                }
            }, children: label }));
    };
    const splitToggleButton = (_jsx("button", { type: "button", className: `split-toggle-btn ${splitOpen ? "active" : ""}`, onClick: () => {
            setSplitOpen((s) => !s);
            if (splitOpen)
                setSplitView(null);
        }, title: splitOpen ? "Close split view" : "Open split view", "aria-label": splitOpen ? "Close split view" : "Open split view", children: _jsxs("span", { className: "split-btn-icon", "aria-hidden": "true", children: [_jsx("span", { className: "split-btn-pane" }), _jsx("span", { className: "split-btn-divider", children: "\u25EB" }), _jsx("span", { className: "split-btn-pane" })] }) }));
    return (_jsxs("div", { className: "app", children: [_jsxs("div", { className: "header", children: [_jsx("div", { className: "logo", onClick: () => navigate("/"), children: "Wayve \uD83D\uDE80" }), _jsxs("div", { className: "nav", children: [navItem("/", "home", "Home"), navItem("/emails", "emails", "Emails"), navItem("/chat", "chat", "Chat"), navItem("/call", "call", "Call"), navItem("/scheduler", "scheduler", "Scheduler"), navItem("/drive", "drive", "Files"), navItem("/notes", "notes", "Notes"), navItem("/aichat", "aichat", "AI Chat")] }), _jsxs("div", { className: "actions", children: [splitOpen && (_jsxs("div", { className: "split-target", role: "group", "aria-label": "Header click target", children: [_jsx("button", { type: "button", className: `split-target-btn ${splitTarget === "left" ? "active" : ""}`, onClick: () => setSplitTarget("left"), title: "Next click loads into the LEFT pane (URL)", children: "\u2190 Left" }), _jsx("button", { type: "button", className: `split-target-btn ${splitTarget === "right" ? "active" : ""}`, onClick: () => setSplitTarget("right"), title: "Next click loads into the RIGHT pane", children: "Right \u2192" })] })), splitOpen && splitView && (_jsxs("span", { className: "split-hint", title: `Right pane: ${splitLabel}`, children: ["\u2197 ", splitLabel] })), splitToggleButton, _jsxs("div", { className: "profile-menu", ref: menuRef, children: [_jsxs("button", { className: "profile-trigger", onClick: () => setMenuOpen((o) => !o), "aria-haspopup": "true", "aria-expanded": menuOpen, title: user.email, children: [_jsx("span", { className: "profile-avatar", children: (user.email?.[0] ?? "?").toUpperCase() }), _jsx("span", { className: "profile-trigger-caret", children: "\u25BE" })] }), menuOpen && (_jsxs("div", { className: "profile-dropdown", role: "menu", children: [_jsx("div", { className: "profile-dropdown-header", children: _jsx("div", { className: "profile-dropdown-name", children: user.email }) }), _jsxs("button", { className: "profile-dropdown-item", onClick: () => {
                                                    setMenuOpen(false);
                                                    navigate("/profile");
                                                }, children: [_jsx("span", { className: "profile-dropdown-icon", children: "\uD83D\uDC64" }), "My Profile"] }), _jsxs("button", { className: "profile-dropdown-item", onClick: () => {
                                                    setMenuOpen(false);
                                                    navigate("/settings");
                                                }, children: [_jsx("span", { className: "profile-dropdown-icon", children: "\u2699\uFE0F" }), "Settings & Privacy"] }), _jsx("div", { className: "profile-dropdown-divider" }), _jsxs("button", { className: "profile-dropdown-item profile-dropdown-logout", onClick: () => {
                                                    setMenuOpen(false);
                                                    logout();
                                                    navigate("/login");
                                                }, children: [_jsx("span", { className: "profile-dropdown-icon", children: "\u23FB" }), "Log out"] })] }))] })] })] }), _jsxs(SearchContext.Provider, { value: searchValue, children: [_jsx("div", { className: "global-search-row", children: _jsxs("div", { className: "global-search-box", children: [_jsx("span", { className: "global-search-icon", "aria-hidden": "true", children: "\u2315" }), _jsx("input", { type: "search", value: searchQuery, onChange: (e) => setSearchQuery(e.target.value), placeholder: `Search ${searchLabel}`, "aria-label": `Search ${searchLabel}` }), searchQuery && (_jsx("button", { type: "button", className: "global-search-clear", onClick: () => setSearchQuery(""), title: "Clear search", "aria-label": "Clear search", children: "\u00D7" }))] }) }), _jsxs("div", { className: "body", children: [_jsxs("div", { className: "icon-sidebar", children: [_jsx(Link, { to: "/emails", children: "\uD83D\uDCE7" }), _jsx(Link, { to: "/chat", children: "\uD83D\uDCAC" }), _jsx(Link, { to: "/call", children: "\uD83D\uDCDE" }), _jsx(Link, { to: "/scheduler", children: "\uD83D\uDCC5" }), _jsx(Link, { to: "/drive", children: "\uD83D\uDCC1" }), _jsx(Link, { to: "/notes", children: "\uD83D\uDCDD" }), _jsx(Link, { to: "/aichat", children: "\u2728" }), _jsx("div", { className: "icon-sidebar-spacer" }), _jsx("button", { className: `icon-split-btn ${splitOpen ? "active" : ""}`, onClick: () => {
                                            setSplitOpen((s) => !s);
                                            if (splitOpen)
                                                setSplitView(null);
                                        }, title: splitOpen ? "Close split view" : "Open split view", "aria-label": splitOpen ? "Close split view" : "Open split view", children: _jsxs("span", { className: "split-btn-icon", "aria-hidden": "true", children: [_jsx("span", { className: "split-btn-pane" }), _jsx("span", { className: "split-btn-divider", children: "\u25EB" }), _jsx("span", { className: "split-btn-pane" })] }) })] }), _jsx("div", { className: "account-sidebar" }), _jsxs("div", { className: `content ${splitOpen ? "split" : ""}`, children: [_jsx("div", { className: "split-pane left", children: _jsx(Outlet, {}) }), splitOpen && (_jsxs("div", { className: "split-pane right", children: [_jsxs("div", { className: "split-pane-toolbar", children: [_jsx("span", { className: "split-pane-title", children: splitLabel ? `${splitLabel} tab` : "Split tab" }), _jsx("button", { className: "split-close-btn", onClick: () => {
                                                            setSplitOpen(false);
                                                            setSplitView(null);
                                                        }, title: "Close extra tab", "aria-label": "Close extra tab", children: "\u2715 Close tab" })] }), _jsx("div", { className: "split-pane-body", children: SplitComp ? (_jsx(Suspense, { fallback: _jsx("div", { className: "split-loading", children: "Loading\u2026" }), children: _jsx(SplitComp, {}) })) : (_jsxs("div", { className: "split-empty", children: [_jsx("div", { className: "split-empty-icon", children: "\u25EB" }), _jsx("div", { className: "split-empty-title", children: "Split view ready" }), _jsx("div", { className: "split-empty-hint", children: "Pick an app from the top header to load it here." })] })) })] }))] })] })] })] }));
}
