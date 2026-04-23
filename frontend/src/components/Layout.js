import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { Link, Outlet, useNavigate, useLocation } from "react-router-dom";
import { useAuth } from "../auth/AuthContext";
import "./layout.css";
export default function Layout() {
    const { user, logout } = useAuth();
    const navigate = useNavigate();
    const location = useLocation();
    // 🚨 Extra safety (should already be protected by ProtectedRoute)
    if (!user)
        return null;
    return (_jsxs("div", { className: "app", children: [_jsxs("div", { className: "header", children: [_jsx("div", { className: "logo", onClick: () => navigate("/"), children: "Wayve \uD83D\uDE80" }), _jsxs("div", { className: "nav", children: [_jsx(Link, { to: "/", className: location.pathname === "/" ? "active" : "", children: "Home" }), _jsx(Link, { to: "/emails", className: location.pathname === "/emails" ? "active" : "", children: "Emails" }), _jsx(Link, { to: "/chat", className: location.pathname === "/chat" ? "active" : "", children: "Chat" }), _jsx(Link, { to: "/call", className: location.pathname === "/call" ? "active" : "", children: "Call" }), _jsx(Link, { to: "/scheduler", className: location.pathname === "/scheduler" ? "active" : "", children: "Scheduler" }), _jsx(Link, { to: "/drive", className: location.pathname === "/drive" ? "active" : "", children: "Files" })] }), _jsxs("div", { className: "actions", children: [_jsx("span", { className: "user-email", children: user.email }), _jsx("button", { className: "logout-btn", onClick: () => {
                                    logout();
                                    navigate("/login");
                                }, children: "Logout" })] })] }), _jsx("div", { className: "content", children: _jsx(Outlet, {}) })] }));
}
