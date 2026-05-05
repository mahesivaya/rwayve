import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { Link, Outlet, useNavigate, useLocation } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import { lazy, Suspense, useState } from "react";
import "./Layout.css";

const HomeView = lazy(() => import("../home/Home"));
const EmailsView = lazy(() => import("../emails/Emails"));
const ChatView = lazy(() => import("../chat/Chat"));
const CallView = lazy(() => import("../call/Call"));
const SchedulerView = lazy(() => import("../scheduler/Scheduler"));
const DriveView = lazy(() => import("../drive/DriveBox"));

const SPLIT_APPS = [
    { key: "home", label: "Home", path: "/", icon: "🏠", Comp: HomeView },
    { key: "emails", label: "Emails", path: "/emails", icon: "📧", Comp: EmailsView },
    { key: "chat", label: "Chat", path: "/chat", icon: "💬", Comp: ChatView },
    { key: "call", label: "Call", path: "/call", icon: "📞", Comp: CallView },
    { key: "scheduler", label: "Scheduler", path: "/scheduler", icon: "📅", Comp: SchedulerView },
    { key: "drive", label: "Files", path: "/drive", icon: "📁", Comp: DriveView },
];

export default function Layout() {
    const { user, logout } = useAuth();
    const navigate = useNavigate();
    const location = useLocation();

    const [splitOpen, setSplitOpen] = useState(false);
    const [splitView, setSplitView] = useState(null);

    if (!user) return null;

    const splitApp = SPLIT_APPS.find((a) => a.key === splitView) ?? null;
    const SplitComp = splitApp?.Comp ?? null;
    const splitLabel = splitApp?.label ?? null;

    const navItem = (path, app, label) => {
        const isMain = !splitOpen && location.pathname === path;
        const isSplit = splitOpen && splitView === app;
        return _jsx(Link, {
            to: path,
            className: [isMain ? "active" : "", isSplit ? "active-split" : ""].filter(Boolean).join(" "),
            onClick: (e) => {
                if (splitOpen) {
                    e.preventDefault();
                    setSplitView(app);
                }
            },
            children: label,
        }, path);
    };

    return (_jsxs("div", { className: "app", children: [
        _jsxs("div", { className: "header", children: [
            _jsx("div", { className: "logo", onClick: () => navigate("/"), children: "Wayve 🚀" }),
            _jsxs("div", { className: "nav", children: [
                navItem("/", "home", "Home"),
                navItem("/emails", "emails", "Emails"),
                navItem("/chat", "chat", "Chat"),
                navItem("/call", "call", "Call"),
                navItem("/scheduler", "scheduler", "Scheduler"),
                navItem("/drive", "drive", "Files"),
            ] }),
            _jsxs("div", { className: "actions", children: [
                splitOpen && _jsx("span", {
                    className: "split-hint",
                    children: splitView ? `↗ Split: ${splitLabel}` : "↗ Split open — pick an app",
                }),
                _jsx("span", { className: "user-email", children: user.email }),
                _jsx("button", {
                    className: "logout-btn",
                    onClick: () => { logout(); navigate("/login"); },
                    children: "Logout",
                }),
            ] }),
        ] }),
        _jsxs("div", { className: "body", children: [
            _jsxs("div", { className: "icon-sidebar", children: [
                _jsx(Link, { to: "/emails", children: "📧" }),
                _jsx(Link, { to: "/chat", children: "💬" }),
                _jsx(Link, { to: "/call", children: "📞" }),
                _jsx(Link, { to: "/scheduler", children: "📅" }),
                _jsx(Link, { to: "/drive", children: "📁" }),
                _jsx("div", { className: "icon-sidebar-spacer" }),
                _jsx("button", {
                    className: `icon-split-btn ${splitOpen ? "active" : ""}`,
                    onClick: () => {
                        setSplitOpen((s) => !s);
                        if (splitOpen) setSplitView(null);
                    },
                    title: splitOpen ? "Close split view" : "Open split view",
                    "aria-label": splitOpen ? "Close split view" : "Open split view",
                    children: "⫼",
                }),
            ] }),
            _jsx("div", { className: "account-sidebar" }),
            _jsxs("div", { className: `content ${splitOpen ? "split" : ""}`, children: [
                _jsx("div", { className: "split-pane left", children: _jsx(Outlet, {}) }),
                splitOpen && _jsxs("div", { className: "split-pane right", children: [
                    _jsx("button", {
                        className: "split-close-floating",
                        onClick: () => { setSplitOpen(false); setSplitView(null); },
                        title: "Close split",
                        "aria-label": "Close split",
                        children: "✕",
                    }),
                    _jsx("div", { className: "split-pane-body", children:
                        SplitComp
                            ? _jsx(Suspense, { fallback: _jsx("div", { className: "split-loading", children: "Loading…" }), children: _jsx(SplitComp, {}) })
                            : _jsxs("div", { className: "split-empty", children: [
                                _jsx("div", { className: "split-empty-icon", children: "⫼" }),
                                _jsx("div", { className: "split-empty-title", children: "Split view ready" }),
                                _jsx("div", { className: "split-empty-hint", children: "Pick an app from the top header to load it here." }),
                            ] }),
                    }),
                ] }),
            ] }),
        ] }),
    ] }));
}
