import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
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
const NotesView = lazy(() => import("../notes/Notes"));
const SPLIT_APPS = [
    { key: "home", label: "Home", path: "/", icon: "🏠", Comp: HomeView },
    { key: "emails", label: "Emails", path: "/emails", icon: "📧", Comp: EmailsView },
    { key: "chat", label: "Chat", path: "/chat", icon: "💬", Comp: ChatView },
    { key: "call", label: "Call", path: "/call", icon: "📞", Comp: CallView },
    { key: "scheduler", label: "Scheduler", path: "/scheduler", icon: "📅", Comp: SchedulerView },
    { key: "drive", label: "Files", path: "/drive", icon: "📁", Comp: DriveView },
    { key: "notes", label: "Notes", path: "/notes", icon: "📝", Comp: NotesView },
];
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
    if (!user)
        return null;
    const splitApp = SPLIT_APPS.find((a) => a.key === splitView) ?? null;
    const SplitComp = splitApp?.Comp ?? null;
    const splitLabel = splitApp?.label ?? null;
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
    return (_jsxs("div", { className: "app", children: [_jsxs("div", { className: "header", children: [_jsx("div", { className: "logo", onClick: () => navigate("/"), children: "Wayve \uD83D\uDE80" }), _jsxs("div", { className: "nav", children: [navItem("/", "home", "Home"), navItem("/emails", "emails", "Emails"), navItem("/chat", "chat", "Chat"), navItem("/call", "call", "Call"), navItem("/scheduler", "scheduler", "Scheduler"), navItem("/drive", "drive", "Files"), navItem("/notes", "notes", "Notes")] }), _jsxs("div", { className: "actions", children: [splitOpen && (_jsxs("div", { className: "split-target", role: "group", "aria-label": "Header click target", children: [_jsx("button", { type: "button", className: `split-target-btn ${splitTarget === "left" ? "active" : ""}`, onClick: () => setSplitTarget("left"), title: "Next click loads into the LEFT pane (URL)", children: "\u2190 Left" }), _jsx("button", { type: "button", className: `split-target-btn ${splitTarget === "right" ? "active" : ""}`, onClick: () => setSplitTarget("right"), title: "Next click loads into the RIGHT pane", children: "Right \u2192" })] })), splitOpen && splitView && (_jsxs("span", { className: "split-hint", title: `Right pane: ${splitLabel}`, children: ["\u2197 ", splitLabel] })), _jsx("span", { className: "user-email", children: user.email }), _jsx("button", { className: "logout-btn", onClick: () => {
                                    logout();
                                    navigate("/login");
                                }, children: "Logout" })] })] }), _jsxs("div", { className: "body", children: [_jsxs("div", { className: "icon-sidebar", children: [_jsx(Link, { to: "/emails", children: "\uD83D\uDCE7" }), _jsx(Link, { to: "/chat", children: "\uD83D\uDCAC" }), _jsx(Link, { to: "/call", children: "\uD83D\uDCDE" }), _jsx(Link, { to: "/scheduler", children: "\uD83D\uDCC5" }), _jsx(Link, { to: "/drive", children: "\uD83D\uDCC1" }), _jsx(Link, { to: "/notes", children: "\uD83D\uDCDD" }), _jsx("div", { className: "icon-sidebar-spacer" }), _jsx("button", { className: `icon-split-btn ${splitOpen ? "active" : ""}`, onClick: () => {
                                    setSplitOpen((s) => !s);
                                    if (splitOpen)
                                        setSplitView(null);
                                }, title: splitOpen ? "Close split view" : "Open split view", "aria-label": splitOpen ? "Close split view" : "Open split view", children: "\u2AFC" })] }), _jsx("div", { className: "account-sidebar" }), _jsxs("div", { className: `content ${splitOpen ? "split" : ""}`, children: [_jsx("div", { className: "split-pane left", children: _jsx(Outlet, {}) }), splitOpen && (_jsxs("div", { className: "split-pane right", children: [_jsx("button", { className: "split-close-floating", onClick: () => {
                                            setSplitOpen(false);
                                            setSplitView(null);
                                        }, title: "Close split", "aria-label": "Close split", children: "\u2715" }), _jsx("div", { className: "split-pane-body", children: SplitComp ? (_jsx(Suspense, { fallback: _jsx("div", { className: "split-loading", children: "Loading\u2026" }), children: _jsx(SplitComp, {}) })) : (_jsxs("div", { className: "split-empty", children: [_jsx("div", { className: "split-empty-icon", children: "\u2AFC" }), _jsx("div", { className: "split-empty-title", children: "Split view ready" }), _jsx("div", { className: "split-empty-hint", children: "Pick an app from the top header to load it here." })] })) })] }))] })] })] }));
}
