import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
import { useAuth } from "../auth/AuthContext";
import { useNavigate } from "react-router-dom";
import "./home.css";
export default function Home() {
    const { user, logout } = useAuth();
    const navigate = useNavigate();
    if (!user) {
        return (_jsxs("div", { className: "home", children: [_jsx("h1", { children: "Welcome to Wayve \uD83D\uDE80" }), _jsx("p", { children: "Your all-in-one platform for Email, Chat, and Scheduling." }), _jsxs("div", { className: "auth-buttons", children: [_jsx("button", { onClick: () => navigate("/login"), children: "Login" }), _jsx("button", { onClick: () => navigate("/register"), children: "Register" })] })] }));
    }
    return (_jsxs("div", { className: "dashboard", children: [_jsx("div", { className: "dashboard-header", children: _jsxs("h2", { children: ["Welcome, ", user.email, " \uD83D\uDC4B"] }) }), _jsxs("div", { className: "dashboard-grid", children: [_jsxs("div", { className: "card", onClick: () => navigate("/emails"), children: [_jsx("h3", { children: "\uD83D\uDCE7 Emails" }), _jsx("p", { children: "View and send emails" })] }), _jsxs("div", { className: "card", onClick: () => navigate("/chat"), children: [_jsx("h3", { children: "\uD83D\uDCAC Chat" }), _jsx("p", { children: "Real-time messaging" })] }), _jsxs("div", { className: "card", onClick: () => navigate("/call"), children: [_jsx("h3", { children: " \uD83D\uDCDE  \uD83C\uDFA5 Call" }), _jsx("p", { children: "Real-time calling" })] }), _jsxs("div", { className: "card", onClick: () => navigate("/scheduler"), children: [_jsx("h3", { children: "\uD83D\uDCC5 Scheduler" }), _jsx("p", { children: "Manage your meetings" })] }), _jsxs("div", { className: "card", onClick: () => navigate("/drive"), children: [_jsx("h3", { children: "\uD83D\uDCC1 Drive" }), _jsx("p", { children: "Store and manage files" })] })] })] }));
}
